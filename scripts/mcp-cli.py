#!/usr/bin/env python3
"""MCP lifecycle CLI: install / index / sync / status.

Single entry point for the local MCP layer. Run it through pixi so `node`
resolves (the MCP npm packages are launched with the pixi-provided node):

    pixi run mcp -h
    pixi run mcp status

Design notes (see .omo/plans/2026-10-28-mcp-lifecycle-cli.md):
- Subcommands: `setup` (install at pin), `index` (init on first run, full rebuild
  after), `sync` (incremental), `status` (read-only).
- `status` is read-only and always exits 0; missing `node` exits 127 with a hint.
- `setup` is the ONLY explicit installer (`--save-exact`); the launcher still
  installs when a binary is missing (panel first run / clean clone), but the
  launcher no longer reinstalls on pin drift -- it fails with a hint instead.
- Pins live in scripts/mcp-servers.json (single source); package.json /
  package-lock.json / node_modules are gitignored local echoes.
"""

import argparse
import json
import os
import re
import shutil
import tempfile
import subprocess
import sys

import agent_common as ac

PREFIX = "MCP"
SPECS = os.path.join("scripts", "mcp-servers.json")
NODE_HINT = "node not found on PATH -- run via: pixi run mcp <subcommand>"
NPM_HINT = "npm not found on PATH -- run via: pixi run mcp setup"


def _specs():
    with open(os.path.join(ac.ROOT, SPECS)) as handle:
        return json.load(handle)


def _installed_version(pkg):
    """Version of `pkg` currently in ./node_modules, or None when absent."""
    parts = pkg.split("/") if pkg.startswith("@") else [pkg]
    manifest = os.path.join(ac.ROOT, "node_modules", *parts, "package.json")
    if not os.path.exists(manifest):
        return None
    try:
        with open(manifest) as handle:
            return json.load(handle).get("version")
    except (OSError, ValueError):
        return None


def _bin_path(bin_name):
    name = bin_name + ".cmd" if os.name == "nt" else bin_name
    return os.path.join(ac.ROOT, "node_modules", ".bin", name)


def _run_codegraph_status(node):
    """Pass `codegraph status` through; uninitialized is NOT an error (exit 0)."""
    binary = _bin_path("codegraph")
    if not os.path.exists(binary):
        print("%s codegraph binary missing -- run: pixi run mcp setup" % PREFIX)
        return
    proc = subprocess.run(
        [node, binary, "status"],
        cwd=ac.ROOT,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
    )
    sys.stdout.write(proc.stdout)
    sys.stderr.write(proc.stderr)


def _install(pkg, version, npm):
    """npm install -D --save-exact <pkg>@<version>; npm's stdout goes to stderr.

    Keep the package manager's chatter off stdout: for an MCP server, stdout is
    the JSON-RPC channel, and this code path is shared with the launcher.
    """
    print("%s installing %s@%s ..." % (PREFIX, pkg, version))
    proc = subprocess.run(
        [npm, "install", "-D", "--save-exact", "%s@%s" % (pkg, version)],
        cwd=ac.ROOT,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=None,  # npm's progress already goes to stderr; inherit it
        text=True,
    )
    if proc.stdout:
        sys.stderr.write(proc.stdout)
    return proc.returncode == 0


def _unblock(pkg):
    """Move stale npm temp dirs aside so npm's rename cannot hit ENOTEMPTY.

    When a version switch is interrupted, npm leaves a non-empty staging dir
    (`.<name>-<hash>`) inside the package scope; the next install tries to rename
    into that exact path and fails with ENOTEMPTY forever. The staging dirs are
    disposable, but bulk-deleting them is both slow and unnecessary -- renaming
    them out of node_modules is enough to unblock the install.
    """
    parts = pkg.split("/") if pkg.startswith("@") else [pkg]
    scope_dir = os.path.join(ac.ROOT, "node_modules", parts[0])
    if not os.path.isdir(scope_dir):
        return 0
    name = parts[-1]
    stale_root = tempfile.mkdtemp(prefix="mcp-cli-stale-")
    moved = 0
    for entry in os.listdir(scope_dir):
        if not entry.startswith("." + name + "-"):
            continue
        try:
            os.rename(os.path.join(scope_dir, entry), os.path.join(stale_root, entry))
            moved += 1
        except OSError:
            continue
    return moved


def _is_pixi(spec):
    return spec.get("kind") == "pixi"


def _env_bin(spec):
    """Absolute path of a pixi-kind probe binary inside its (isolated) environment."""
    name = spec["probe"][0]
    if os.name == "nt":
        name += ".exe"
    return os.path.join(ac.ROOT, ".pixi", "envs", spec["pixi_env"], "bin", name)


def _pixi_version(spec):
    """Version reported by the probe binary; None when the env or the binary is absent.

    Deliberately does NOT shell out to `pixi`: nested pixi is not on PATH inside a task
    (review #18), and this helper is also reused by the gate.
    """
    binary = _env_bin(spec)
    if not os.path.exists(binary):
        return None
    try:
        proc = subprocess.run(
            [binary] + list(spec["probe"][1:]),
            cwd=ac.ROOT,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=120,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if proc.returncode != 0:
        return None
    lines = (proc.stdout or proc.stderr).strip().splitlines()
    first = lines[0].strip() if lines else ""
    # Serena prints "Serena 2.1.0-<git describe>-dirty": keep only the semver part so the
    # pin comparison is stable across re-installs (the git suffix moves every build).
    match = re.search(r"\d+\.\d+\.\d+", first)
    return match.group(0) if match else first


def cmd_setup(args):
    specs = _specs()
    npm = shutil.which("npm")
    needs_npm = any(not _is_pixi(spec) for spec in specs.values())
    if needs_npm and npm is None:
        print("%s %s" % (PREFIX, NPM_HINT))
        return 127
    failures = 0
    for name, spec in specs.items():
        version = spec["version"]
        if _is_pixi(spec):
            reported = _pixi_version(spec)
            if reported is None:
                print(
                    "%s %-16s ✗ env '%s' not installed -- run: pixi install -e %s"
                    % (PREFIX, name, spec["pixi_env"], spec["pixi_env"])
                )
                failures += 1
            elif not reported.startswith(version):
                print("%s %-16s pinch drift: %s vs pin %s -- run: pixi install -e %s" % (PREFIX, name, reported, version, spec["pixi_env"]))
                failures += 1
            else:
                print("%s %-16s ok (pixi %s, %s)" % (PREFIX, name, spec["pixi_env"], reported))
            continue
        pkg = spec["pkg"]
        have = _installed_version(pkg)
        if have == version and os.path.exists(_bin_path(spec["bin"])):
            print("%s %-16s ok (%s@%s)" % (PREFIX, name, pkg, version))
            continue
        if have is not None:
            print("%s %-16s drift: installed %s, pin %s -- reinstalling" % (PREFIX, name, have, version))
        unblocked = _unblock(pkg)
        if unblocked:
            print("%s %-16s moved %d stale staging dir(s) aside" % (PREFIX, name, unblocked))
        if not _install(pkg, version, npm):
            print("%s %-16s ✗ npm install failed" % (PREFIX, name))
            failures += 1
            continue
        after = _installed_version(pkg)
        if after != version or not os.path.exists(_bin_path(spec["bin"])):
            print("%s %-16s ✗ still not at pin (installed %s)" % (PREFIX, name, after or "-"))
            failures += 1
        else:
            print("%s %-16s ✓ ready (%s@%s)" % (PREFIX, name, pkg, version))
    if failures:
        print("%s ✗ %d server(s) not ready" % (PREFIX, failures))
        return ac.EXIT_FINDINGS
    print("%s ✓ all servers ready" % PREFIX)
    return ac.EXIT_OK


def _codegraph(node, *path_args):
    """Run the pinned codegraph binary with inherited stdio (progress stays visible)."""
    binary = _bin_path("codegraph")
    if not os.path.exists(binary):
        print("%s codegraph binary missing -- run: pixi run mcp setup" % PREFIX)
        return 127
    return subprocess.run(
        [node, binary, *path_args],
        cwd=ac.ROOT,
        stdin=subprocess.DEVNULL,
    ).returncode


def cmd_index(args):
    """First run initializes .codegraph; later runs rebuild it in full."""
    node = shutil.which("node")
    if node is None:
        print("%s %s" % (PREFIX, NODE_HINT))
        return 127
    initialized = os.path.isdir(os.path.join(ac.ROOT, ".codegraph"))
    step = ["index"] if initialized else ["init", "-y"]
    print("%s %s .codegraph (codegraph %s)" % (PREFIX, "rebuilding" if initialized else "initializing", " ".join(step)))
    code = _codegraph(node, *step)
    if code != 0:
        print("%s ✗ codegraph %s failed (exit %d)" % (PREFIX, " ".join(step), code))
        return ac.EXIT_FINDINGS
    print()
    print("%s index done -- scale check:" % PREFIX)
    _run_codegraph_status(node)
    return ac.EXIT_OK


def cmd_sync(args):
    """Incremental sync of the existing index (watch mode normally covers this)."""
    node = shutil.which("node")
    if node is None:
        print("%s %s" % (PREFIX, NODE_HINT))
        return 127
    if not os.path.isdir(os.path.join(ac.ROOT, ".codegraph")):
        print("%s no .codegraph yet -- run: pixi run mcp index" % PREFIX)
        return ac.EXIT_FINDINGS
    code = _codegraph(node, "sync")
    if code != 0:
        print("%s ✗ codegraph sync failed (exit %d)" % (PREFIX, code))
        return ac.EXIT_FINDINGS
    print("%s ✓ synced" % PREFIX)
    return ac.EXIT_OK


def cmd_status(args):
    specs = _specs()
    node = shutil.which("node")
    print("%s servers declared in %s:" % (PREFIX, SPECS))
    for name, spec in specs.items():
        version = spec["version"]
        if _is_pixi(spec):
            reported = _pixi_version(spec)
            if reported is None:
                state = "missing"
            elif reported.startswith(version):
                state = "ok"
            else:
                state = "DRIFT"
            print(
                "  %-16s %-38s pin=%-8s reported=%-24s %s"
                % (name, "pixi:" + spec["pixi_env"], version, reported or "-", state)
            )
            continue
        have = _installed_version(spec["pkg"])
        if have is None:
            state = "missing"
        elif have != version:
            state = "DRIFT"
        else:
            state = "ok"
        print(
            "  %-16s %-38s pin=%-8s installed=%-8s %s"
            % (name, spec["pkg"], version, have or "-", state)
        )
    print()
    if node is None:
        print("%s %s" % (PREFIX, NODE_HINT))
        return 127
    _run_codegraph_status(node)
    return ac.EXIT_OK


def build_parser():
    parser = argparse.ArgumentParser(
        prog="mcp",
        description="MCP lifecycle management (local stdio servers, pinned via scripts/mcp-servers.json).",
    )
    sub = parser.add_subparsers(dest="command", metavar="<subcommand>")
    setup = sub.add_parser("setup", help="install every server at the pinned version (idempotent)")
    setup.set_defaults(func=cmd_setup)
    index = sub.add_parser("index", help="initialize .codegraph on first run, full rebuild afterwards")
    index.set_defaults(func=cmd_index)
    sync = sub.add_parser("sync", help="incrementally sync an existing index (watch mode covers the normal case)")
    sync.set_defaults(func=cmd_sync)
    status = sub.add_parser("status", help="show pin vs installed versions and the codegraph index state")
    status.set_defaults(func=cmd_status)
    return parser


def main(argv=None):
    parser = build_parser()
    args = parser.parse_args(argv)
    handler = getattr(args, "func", None)
    if handler is None:
        parser.print_help(sys.stderr)
        return 2
    return handler(args)


if __name__ == "__main__":
    sys.exit(main())
