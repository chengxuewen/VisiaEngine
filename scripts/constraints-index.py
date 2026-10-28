#!/usr/bin/env python3
"""Generate the CI-gate list into CONSTRAINTS.md from pixi.toml.

Single source of truth: `pixi.toml` `[tasks] ci` depends-on. Target: the root
CONSTRAINTS.md marker region:
  <!-- BEGIN:ci-gates --> ... <!-- END:ci-gates -->
The list is DERIVED (never hand-edited); gate-agents asserts it matches
pixi.toml. Hand-written content (the anti-gaming rules) is never touched.

Exit codes: 0 clean, 1 findings, 2 guard violation.
"""

import os
import re
import sys

import agent_common as ac

PREFIX = "GATE-CONSTRAINTS"
BEGIN = "<!-- BEGIN:ci-gates -->"
END = "<!-- END:ci-gates -->"


def _ci_gates(pixi_path):
    with open(pixi_path) as handle:
        text = handle.read()
    m = re.search(
        r"^\s*ci\s*=\s*\{.*?depends-on\s*=\s*\[(.*?)\]",
        text,
        re.DOTALL | re.MULTILINE,
    )
    if not m:
        return None
    return re.findall(r'"([^"]+)"', m.group(1))


def _build_block(gates):
    parts = [
        "",
        "## CI gates (generated from `pixi.toml` -> `[tasks] ci` depends-on; edit pixi.toml, not this block)",
        "",
        "`pixi run ci` runs, in order:",
        "",
    ]
    for i, gate in enumerate(gates, 1):
        parts.append("%d. `%s`" % (i, gate))
    return "\n".join(parts).rstrip("\n") + "\n"


def _current_region(text):
    if BEGIN in text and END in text:
        return text.split(BEGIN, 1)[1].split(END, 1)[0]
    return None


def run(command, targets, cfg, dry_run):
    pixi_path = os.path.join(ac.ROOT, "pixi.toml")
    hub_path = os.path.join(ac.ROOT, cfg["constraints_hub"])
    gates = _ci_gates(pixi_path)
    if gates is None:
        print("%s ✗ no ci depends-on found in %s" % (PREFIX, pixi_path))
        return ac.EXIT_GUARD
    if not os.path.exists(hub_path):
        print("%s ✗ hub-missing %s" % (PREFIX, hub_path))
        return ac.EXIT_GUARD

    block = _build_block(gates)
    hub_text = open(hub_path).read()
    if not ac.single_marker_ok(PREFIX, hub_text, BEGIN, END):
        return ac.EXIT_GUARD
    region = _current_region(hub_text)

    if command == "list":
        print("%-10s constraints -> %s (%d gates)" % ("hub", cfg["constraints_hub"], len(gates)))
        return ac.EXIT_OK

    if region is None:
        if command == "check":
            print(" [ FIX ] hub %s (ci-gates region absent)" % cfg["constraints_hub"])
            print("%s ✗ hub needs repair (run: agent-links.py sync)" % PREFIX)
            return ac.EXIT_FINDINGS
    elif region.strip() != block.strip():
        if command == "check":
            print(" [ FIX ] hub %s (ci-gates drift)" % cfg["constraints_hub"])
            print("%s ✗ hub needs repair (run: agent-links.py sync)" % PREFIX)
            return ac.EXIT_FINDINGS

    if command == "check":
        print("%s ✓ ci-gates match (%d gates)" % (PREFIX, len(gates)))
        return ac.EXIT_OK

    # sync / dry-run
    if region is None:
        new_text = hub_text.rstrip("\n") + "\n\n" + BEGIN + "\n" + block + END + "\n"
        print(" [ APPEND ] hub %s (ci-gates)" % cfg["constraints_hub"])
    else:
        before = hub_text.split(BEGIN, 1)[0]
        after = hub_text.split(END, 1)[1] if (END in hub_text) else ""
        new_text = before + BEGIN + "\n" + block + END + after
        print(" [ REWRITE] hub %s (ci-gates)" % cfg["constraints_hub"])
    if not dry_run:
        with open(hub_path, "w") as handle:
            handle.write(new_text)
    print("%s ✓ sync done" % PREFIX)
    return ac.EXIT_OK


if __name__ == "__main__":
    print(__doc__.strip())
    sys.exit(0)
