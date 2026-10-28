#!/usr/bin/env python3
"""Unified agent-config CLI: one entry for every generated host layer.

Delegates to the two existing single-source CLIs (no logic is reimplemented):
  sync    -> agent-links.py sync    + skills-links.py sync
  check   -> agent-links.py check   + skills-links.py check   (exit = max)
  status  -> both checks, output shown, ALWAYS exit 0 (read-only)

`visiaengine.sh agents <sub>` works automatically (pure passthrough).
The old tasks (agent-sync / gate-agents / gate-skills) stay as aliases; `ci` keeps
the two gates separate so the CONSTRAINTS gate list does not change.

Extra arguments are forwarded to both delegates (e.g. `--copy` for Windows
skills links).

Exit codes: 0 clean / 1 findings / 2 guard (mirrors the delegated CLIs).
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
AGENT_LINKS = os.path.join(ROOT, "scripts", "agent-links.py")
SKILLS_LINKS = os.path.join(ROOT, "scripts", "skills-links.py")

SUBCOMMANDS = ("sync", "check", "status")


def _run(script, command, extra):
    cmd = [sys.executable, script, command] + list(extra)
    return subprocess.run(cmd, cwd=ROOT).returncode


def main(argv):
    if len(argv) < 2 or argv[1] in ("-h", "--help"):
        print(__doc__.strip())
        return 0
    command = argv[1]
    if command not in SUBCOMMANDS:
        print(__doc__.strip())
        return 0
    extra = argv[2:]
    if command == "sync":
        return max(_run(AGENT_LINKS, "sync", extra), _run(SKILLS_LINKS, "sync", extra))
    if command == "check":
        return max(_run(AGENT_LINKS, "check", extra), _run(SKILLS_LINKS, "check", extra))
    # status: read-only, always exit 0; show both reports and label the worst code.
    print("== agent-links ==")
    rc1 = _run(AGENT_LINKS, "check", extra)
    print("== skills-links ==")
    rc2 = _run(SKILLS_LINKS, "check", extra)
    worst = max(rc1, rc2)
    label = {0: "clean", 1: "findings", 2: "guard"}.get(worst, "error")
    print("== agents status: %s (agent-links=%d, skills-links=%d) ==" % (label, rc1, rc2))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
