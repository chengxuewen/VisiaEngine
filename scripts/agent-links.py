#!/usr/bin/env python3
"""Thin dispatcher over the per-layer agent single-source generators.

Layers: rules (rules-links), mcp (mcp-links), memory (memory-aggregate). Each
generator exposes run(command, targets, cfg, dry_run). This dispatcher shares
one parsed config (scripts/agent-targets.json) and one command across all
layers and holds NO per-layer translation logic -- that lives in each generator
(adjudication 2-B: "per-layer small generators + shared base", avoid one mega-script).

usage:
    python3 scripts/agent-links.py check|sync|dry-run|list
exit: 0 clean, 1 findings, 2 guard violation. Highest exit code wins.
"""

import importlib.util
import json
import os
import sys

import agent_common as ac

CONFIG = os.path.join(ac.ROOT, "scripts", "agent-targets.json")


def _load_layer(name):
    """Load a generator module by file path (filenames use hyphens, not importable)."""
    path = os.path.join(ac.ROOT, "scripts", name + ".py")
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


rules_links = _load_layer("rules-links")
mcp_links = _load_layer("mcp-links")
memory_aggregate = _load_layer("memory-aggregate")
skills_index = _load_layer("skills-index")
constraints_index = _load_layer("constraints-index")
LAYERS = [rules_links, mcp_links, memory_aggregate, skills_index, constraints_index]


def main(argv):
    if len(argv) < 2 or argv[1] not in ("check", "sync", "dry-run", "list"):
        print(__doc__.strip())
        return 0
    command = argv[1]
    dry_run = command == "dry-run"
    with open(CONFIG) as handle:
        cfg = json.load(handle)
    targets = cfg["targets"]
    worst = 0
    for mod in LAYERS:
        rc = mod.run(command, targets, cfg, dry_run)
        worst = max(worst, rc)
        if worst == ac.EXIT_GUARD:
            break  # guard violation: stop, nothing was written
    if command == "check":
        if worst == ac.EXIT_OK:
            print("GATE-AGENTS ✓ all layers clean")
        elif worst == ac.EXIT_GUARD:
            print("GATE-AGENTS ✗ guard violation -- nothing written")
        else:
            print("GATE-AGENTS ✗ findings -- run: pixi run agent-sync")
    return worst


if __name__ == "__main__":
    sys.exit(main(sys.argv))
