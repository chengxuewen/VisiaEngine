#!/usr/bin/env python3
"""Check / repair the per-host MCP activation layer.

Single source: <repo>/.agents/mcp.json (+ optional .agents/mcp.local.json for
credentials, gitignored). Targets: scripts/agent-targets.json.

Two host shapes (the command body is identical across hosts -- only the
envelope differs):
  - "mcpServers": write {"mcpServers": <merged>} to the host file
    (.mcp.json at the repo root for codebuddy, .cursor/mcp.json for cursor).
    The mcpServers key shape is the canonical one, so transfer is a verbatim copy
    of the merged map (only `command` is wrapped in the repo-relative launcher).
  - "named": opencode's opencode.json uses a named-object mcp{} (type/enabled/
    command/environment). Convert each canonical server to that shape and MERGE
    into opencode.json's existing mcp{} -- other keys (websearch, disabled
    local-playwright/postgres/openspace) are preserved (non-destructive).

Exit codes: 0 clean, 1 findings, 2 guard violation (nothing written).
"""

import json
import os
import re
import sys

import agent_common as ac

PREFIX = "GATE-MCP-LINKS"


def _merged_servers(cfg):
    src = os.path.join(ac.ROOT, cfg["mcp_source"])
    with open(src) as handle:
        canonical = json.load(handle).get("mcpServers", {})
    local = os.path.join(ac.ROOT, cfg["mcp_local"])
    if os.path.exists(local):
        with open(local) as handle:
            override = json.load(handle).get("mcpServers", {})
        for name, entry in override.items():
            base = dict(canonical.get(name, {}))
            base.update(entry)  # local wins (credentials)
            canonical[name] = base
    return canonical


# POSIX wrapper is referenced RELATIVE to the repo root: `mcp_file` is now the
# commit-tracked `.mcp.json`, which must stay machine-independent, and a
# project-scope MCP host spawns with cwd = workspace root. mcp-launch.sh resolves
# the repo root from BASH_SOURCE, so it does not depend on anything else.
LAUNCHER_SH = os.path.join("scripts", "mcp-launch.sh")
# Windows keeps the absolute path (escape hatch: a repo-relative .bat is not
# reliably executable through cmd); Windows devs get a local-only diff.
LAUNCHER_BAT = os.path.join(ac.ROOT, "scripts", "mcp-launch.bat")


def _host_entry(entry, target):
    """Canonical entry (`command` string + `args` list) -> host file entry.

    The MCP `mcpServers` schema requires `command` to be a STRING; a
    `command: ["bash", ...]` array makes CodeBuddy's panel die with a JS
    TypeError (`ir.replace is not a function`). A launcher host replaces the
    `pixi run <task>` prefix with the repo-relative wrapper and keeps the task
    plus any trailing args.
    """
    item = dict(entry)
    if not target.get("mcp_launcher"):
        return item
    command = item.get("command")
    args = list(item.get("args") or [])
    # Forward EVERYTHING after `run` verbatim: the canonical form may carry pixi flags
    # before the task name (`run -e serena mcp-serena`), and the launcher itself already
    # passes its argv straight through (`exec pixi run --manifest-path … "$@"`).
    if command == "pixi" and args[:1] == ["run"] and len(args) >= 2:
        tail = args[1:]
        if os.name == "nt":
            item["command"], item["args"] = LAUNCHER_BAT, tail
        else:
            item["command"], item["args"] = "bash", [LAUNCHER_SH] + tail
    return item


def _expected_mcp_file(merged, target):
    out = {}
    for name, entry in merged.items():
        out[name] = _host_entry(entry, target)
    return json.dumps({"mcpServers": out}, indent=2, sort_keys=True) + "\n"


_ENV_REF = re.compile(r"^\$\{([A-Z_][A-Z0-9_]*)\}$")


def _opencode_value(value):
    """Canonical `${VAR}` (MCP/codebuddy/claude form) -> opencode `{env:VAR}`."""
    if isinstance(value, str):
        match = _ENV_REF.match(value)
        if match:
            return "{env:%s}" % match.group(1)
    return value


def _expected_opencode_mcp(merged):
    out = {}
    for name, entry in merged.items():
        command = [entry.get("command", "")] + list(entry.get("args") or [])
        out[name] = {
            "type": "local",
            "enabled": True,
            "command": command,
        }
        env = entry.get("env")
        if env:
            out[name]["environment"] = {k: _opencode_value(v) for k, v in env.items()}
    return out


def run(command, targets, cfg, dry_run):
    merged = _merged_servers(cfg)
    if command == "list":
        for t in targets:
            if t.get("mcp_file"):
                print("%-10s mcp %s (%s)" % (t["label"], t.get("mcp_file"), t.get("mcp_shape")))
            elif t.get("opencode_file"):
                print("%-10s mcp %s (named)" % (t["label"], t.get("opencode_file")))
        return ac.EXIT_OK

    failures = 0
    if command == "check":
        for t in targets:
            if not t.get("enabled"):
                continue
            if t.get("mcp_file"):
                path = os.path.join(ac.ROOT, t["mcp_file"])
                want = _expected_mcp_file(merged, t)
                have = open(path).read() if os.path.exists(path) else ""
                if have.strip() != want.strip():
                    failures += 1
                    print(" [ FIX ] %-10s %s (mcpServers drift)" % (t["label"], t["mcp_file"]))
                else:
                    print(" [ OK  ] %-10s %s" % (t["label"], t["mcp_file"]))
            elif t.get("opencode_file"):
                path = os.path.join(ac.ROOT, t["opencode_file"])
                with open(path) as handle:
                    cur = json.load(handle).get("mcp", {})
                want = _expected_opencode_mcp(merged)
                if any(cur.get(n) != want[n] for n in want):
                    failures += 1
                    print(" [ FIX ] %-10s %s (named mcp drift)" % (t["label"], t["opencode_file"]))
                else:
                    print(" [ OK  ] %-10s %s" % (t["label"], t["opencode_file"]))
        if failures:
            print("%s ✗ %d target(s) need repair (run: agent-links.py sync)" % (PREFIX, failures))
            return ac.EXIT_FINDINGS
        print("%s ✓ (%d server(s) × %d target(s))" % (PREFIX, len(merged), sum(1 for t in targets if t.get("enabled"))))
        return ac.EXIT_OK

    # sync / dry-run
    for t in targets:
        if not t.get("enabled"):
            continue
        if t.get("mcp_file"):
            path = os.path.join(ac.ROOT, t["mcp_file"])
            want = _expected_mcp_file(merged, t)
            have = open(path).read() if os.path.exists(path) else ""
            if have.strip() == want.strip():
                print(" [ OK  ] %-10s %s" % (t["label"], t["mcp_file"]))
                continue
            print(" [ WRITE] %-10s %s" % (t["label"], t["mcp_file"]))
            if not dry_run:
                os.makedirs(os.path.dirname(path), exist_ok=True)
                with open(path, "w") as handle:
                    handle.write(want)
        elif t.get("opencode_file"):
            path = os.path.join(ac.ROOT, t["opencode_file"])
            with open(path) as handle:
                doc = json.load(handle)
            want = _expected_opencode_mcp(merged)
            cur = doc.setdefault("mcp", {})
            if all(cur.get(n) == want[n] for n in want):
                print(" [ OK  ] %-10s %s" % (t["label"], t["opencode_file"]))
                continue
            print(" [ MERGE] %-10s %s" % (t["label"], t["opencode_file"]))
            if not dry_run:
                backup = ac.backup_name(os.path.dirname(path), os.path.basename(path))
                os.makedirs(os.path.dirname(backup), exist_ok=True)
                import shutil
                shutil.copy2(path, backup)  # never lose the assembly point
                cur.update(want)
                with open(path, "w") as handle:
                    json.dump(doc, handle, indent=2, sort_keys=False)
                    handle.write("\n")
    print("%s ✓ sync done" % PREFIX)
    return ac.EXIT_OK


if __name__ == "__main__":
    print(__doc__.strip())
    sys.exit(0)
