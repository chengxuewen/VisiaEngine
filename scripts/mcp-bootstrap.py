#!/usr/bin/env python3
"""Idempotent MCP bootstrap (pixi run mcp-setup).

Ensures the codegraph binary is installed at the pinned version (the launcher
installs on first run; this just triggers it deterministically and surfaces the
index state). graph.db incremental-update strategy is deferred (review #6) -- we
only ensure the binary is present and report whether an index dir exists.
"""

import os
import subprocess
import sys

import agent_common as ac

PREFIX = "MCP-SETUP"


def main():
    print("%s ensuring codegraph binary (pinned) ..." % PREFIX)
    try:
        proc = subprocess.run(
            ["node", "scripts/mcp-serve.mjs", "--from", "scripts/mcp-servers.json", "--name", "local-codegraph"],
            stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=300)
    except subprocess.TimeoutExpired:
        print("%s ✗ codegraph install/start timed out" % PREFIX)
        return 1
    if proc.returncode != 0:
        tail = proc.stderr.strip().splitlines()
        print("%s ✗ codegraph failed: %s" % (PREFIX, tail[-1] if tail else "exit %d" % proc.returncode))
        return 1
    print("%s ✓ codegraph ready" % PREFIX)
    for cand in (".codegraph",):
        if os.path.exists(os.path.join(ac.ROOT, cand)):
            print("%s ℹ %s present (index state managed by codegraph)" % (PREFIX, cand))
    print("%s ✓ done (index build strategy deferred: review #6)" % PREFIX)
    return 0


if __name__ == "__main__":
    sys.exit(main())
