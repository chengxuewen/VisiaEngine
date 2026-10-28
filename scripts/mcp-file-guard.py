#!/usr/bin/env python3
"""Validate the generated root `.mcp.json` (CodeBuddy project-scope MCP file).

The file is a generation product of `mcp-links.py` (single source
`.agents/mcp.json`), but it is also the file a human is most likely to hand-edit
while debugging a panel error -- so it gets its own shape gate:

  * valid JSON with an `mcpServers` object
  * server names == the single source's names (no drift, no leftovers)
  * `type` present and one of stdio / sse / streamableHttp (the IDE rejects the
    rest, and omitting it makes the panel refuse the whole file)
  * stdio entries: `command` is a STRING (an array crashes the panel with a JS
    `ir.replace is not a function`) and `args` is a list of strings
  * any referenced `mcp-launch` script actually exists on disk
  * env values are strings in the `${VAR}` form (opencode's `{env:VAR}` is not
    expanded by CodeBuddy and would be sent as a literal token)

Check-only (no writes). Exit codes: 0 clean, 1 findings, 2 guard.
"""

import json
import os
import sys

import agent_common as ac

PREFIX = "GATE-MCP-FILE"
TYPES = ("stdio", "sse", "streamableHttp")


def _server_names(path):
    with open(path) as handle:
        return set(json.load(handle).get("mcpServers", {}))


def _validate(path, expected):
    problems = []
    if not os.path.exists(path):
        return ["missing %s" % path]
    try:
        with open(path) as handle:
            doc = json.load(handle)
    except Exception as exc:  # noqa: BLE001 - report any parse failure
        return ["invalid JSON: %s" % exc]
    servers = doc.get("mcpServers")
    if not isinstance(servers, dict):
        return ["no mcpServers object"]
    if set(servers) != expected:
        problems.append(
            "server names != single source (file=%s, source=%s)"
            % (sorted(servers), sorted(expected))
        )
    for name, entry in servers.items():
        if not isinstance(entry, dict):
            problems.append("%s: entry is not an object" % name)
            continue
        kind = entry.get("type")
        if kind not in TYPES:
            problems.append("%s: type must be one of %s (got %r)" % (name, list(TYPES), kind))
        if kind == "stdio" or "command" in entry:
            command = entry.get("command")
            if not isinstance(command, str) or not command:
                problems.append("%s: command must be a non-empty STRING (got %r)" % (name, type(command).__name__))
            args = entry.get("args")
            if args is not None and (not isinstance(args, list) or not all(isinstance(a, str) for a in args)):
                problems.append("%s: args must be a list of strings" % name)
        env = entry.get("env")
        if env is not None:
            if not isinstance(env, dict):
                problems.append("%s: env must be an object" % name)
            else:
                for key, value in env.items():
                    if not isinstance(value, str):
                        problems.append("%s: env[%s] is not a string" % (name, key))
                    elif value.startswith("{env:"):
                        problems.append("%s: env[%s] uses opencode's {env:VAR}; use ${VAR}" % (name, key))
        for arg in entry.get("args") or []:
            if isinstance(arg, str) and "mcp-launch" in arg:
                full = arg if os.path.isabs(arg) else os.path.join(ac.ROOT, arg)
                if not os.path.exists(full):
                    problems.append("%s: launcher not found: %s" % (name, arg))
    return problems


def run(command, targets, cfg, dry_run):
    hub = cfg.get("mcp_file_guard")
    source = cfg.get("mcp_source")
    if not hub or not source:
        print("%s ✗ mcp_file_guard / mcp_source not configured" % PREFIX)
        return ac.EXIT_GUARD
    if command == "list":
        print("%-10s mcp-file -> %s (vs %s)" % ("hub", hub, source))
        return ac.EXIT_OK
    expected = _server_names(os.path.join(ac.ROOT, source))
    problems = _validate(os.path.join(ac.ROOT, hub), expected)
    if command == "check":
        if problems:
            print(" [ FIX ] mcp-file %s:" % hub)
            for problem in problems:
                print("        %s" % problem)
            print("%s ✗ broken MCP file (shape or drift)" % PREFIX)
            return ac.EXIT_FINDINGS
        print("%s ✓ %s valid (%d server(s), shapes ok, matches single source)" % (PREFIX, hub, len(expected)))
    return ac.EXIT_OK


if __name__ == "__main__":
    print(__doc__.strip())
    sys.exit(0)
