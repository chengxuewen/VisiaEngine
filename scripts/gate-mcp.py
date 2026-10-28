#!/usr/bin/env python3
"""Pre-CI probe that every declared MCP server can at least start.

For each entry in .agents/mcp.json, run `pixi run mcp-<name>` with empty stdin
and a timeout; classify the outcome:
  green : clean start then EOF (exit 0)
  yellow: missing credential (server refuses to start without a token) -> skip,
          NOT a failure (review #19: don't let CI's absent token cause a false red)
  red   : cannot start at all (bad pin / missing binary / broken install) -> CI fails

Exit: 0 if no red, 1 if any red.
"""

import json
import os
import re
import shutil
import subprocess
import sys

import agent_common as ac

PREFIX = "GATE-MCP"
CFG = os.path.join(ac.ROOT, ".agents", "mcp.json")
TOKEN_HINT = re.compile(r"environment variable is not set|unauthorized|401|token|GITHUB_PERSONAL_ACCESS_TOKEN", re.I)
TIMEOUT = 300


def _host_path():
    """PATH as a GUI MCP host would inherit it: system dirs + ~/.local/bin, but NOT
    ~/.pixi/bin (which only our manual `export` adds). If pixi does not resolve here
    the GUI host fails -- this is the O5 regression the probe must catch."""
    pixi_dir = os.path.realpath(os.path.join(os.path.expanduser("~"), ".pixi", "bin"))
    dirs = []
    for d in os.environ.get("PATH", "").split(os.pathsep):
        if d and os.path.realpath(d) != pixi_dir:
            dirs.append(d)
    return os.pathsep.join(dirs)


def main():
    with open(CFG) as handle:
        servers = json.load(handle).get("mcpServers", {})
    if not servers:
        print("%s ✓ no servers declared" % PREFIX)
        return 0
    host_path = _host_path()
    env = dict(os.environ, PATH=host_path)
    red = 0
    yellow = 0
    if shutil.which("pixi", path=host_path) is None:
        print(" [ RED  ] pixi not on the GUI-inherited PATH (fix: bootstrap / O5 symlink)")
        red += 1
    for name in servers:
        # Invoke the launcher directly (node is in the pixi env) instead of nesting
        # `pixi run` -- a nested pixi is not on PATH inside a task (review #18).
        cmd = ["node", "scripts/mcp-serve.mjs", "--from", "scripts/mcp-servers.json", "--name", name]
        print(" [PROBE] %-14s %s" % (name, " ".join(cmd)))
        try:
            proc = subprocess.run(cmd, env=env, stdin=subprocess.DEVNULL, capture_output=True,
                                  text=True, timeout=TIMEOUT)
        except subprocess.TimeoutExpired:
            print(" [ RED  ] %-14s timeout after %ds" % (name, TIMEOUT))
            red += 1
            continue
        if proc.returncode == 0:
            print(" [ OK   ] %-14s starts" % name)
            continue
        tail = (proc.stderr or proc.stdout).strip().splitlines()
        if TOKEN_HINT.search(proc.stderr) or TOKEN_HINT.search(proc.stdout or ""):
            print(" [ SKIP ] %-14s missing credential (expected in CI)" % name)
            yellow += 1
            continue
        print(" [ RED  ] %-14s exit=%d" % (name, proc.returncode))
        if tail:
            print("        %s" % tail[-1])
        red += 1
    if red:
        print("%s ✗ %d red" % (PREFIX, red))
        return 1
    print("%s ✓ (%d green, %d skipped-credential)" % (PREFIX, len(servers) - yellow, yellow))
    return 0


if __name__ == "__main__":
    sys.exit(main())
