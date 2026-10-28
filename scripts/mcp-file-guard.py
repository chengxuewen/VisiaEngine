#!/usr/bin/env python3
"""Validate the hand-maintained MCP host file (codebuddy, generation disabled).

When a host's MCP file is hand-maintained (e.g. codebuddy root `.mcp.json` while
its generation is disabled per D39/A), it is not covered by mcp-links drift
assertions. This layer keeps it honest: valid JSON, expected servers, and the
launcher script referenced by `args` exists. Check-only (no writes).

Exit codes: 0 clean, 1 findings (broken hand-maintained file), 2 guard.
"""

import json
import os
import sys

import agent_common as ac

PREFIX = "GATE-MCP-FILE"
REQUIRED = ("local-codegraph", "local-github")


def _validate(path):
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
    for name in REQUIRED:
        if name not in servers:
            problems.append("missing server %s" % name)
    for name, entry in servers.items():
        for arg in entry.get("args") or []:
            if isinstance(arg, str) and "mcp-launch" in arg:
                full = arg if os.path.isabs(arg) else os.path.join(ac.ROOT, arg)
                if not os.path.exists(full):
                    problems.append("%s launcher not found: %s" % (name, arg))
    return problems


def run(command, targets, cfg, dry_run):
    hub = cfg.get("mcp_file_guard")
    if not hub:
        print("%s ✗ no mcp_file_guard configured" % PREFIX)
        return ac.EXIT_GUARD
    path = os.path.join(ac.ROOT, hub)
    if command == "list":
        print("%-10s mcp-file -> %s" % ("hub", hub))
        return ac.EXIT_OK
    problems = _validate(path)
    if command == "check":
        if problems:
            print(" [ FIX ] mcp-file %s:" % hub)
            for problem in problems:
                print("        %s" % problem)
            print("%s ✗ hand-maintained MCP file is broken" % PREFIX)
            return ac.EXIT_FINDINGS
        print("%s ✓ %s valid (servers + launcher)" % (PREFIX, hub))
    return ac.EXIT_OK


if __name__ == "__main__":
    print(__doc__.strip())
    sys.exit(0)
