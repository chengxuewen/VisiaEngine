#!/usr/bin/env python3
"""Aggregate the volatile memory layer into the root AGENTS.md hub.

Single source: <repo>/.agents/memorys/*.md (status / conventions / decisions /
pitfalls). Target: the root AGENTS.md "PROJECT KNOWLEDGE BASE" hub that host
agents read every turn.

Discipline (adjudication 4 + review #21): hand-written hub content is NEVER
overwritten. The generated dump lives inside a marked region
  <!-- BEGIN:agents-memory --> ... <!-- END:agents-memory -->
and gate-agents asserts the region matches; sync only rewrites inside the
markers. If the markers are absent, the block is appended at EOF (non-destructive).

Exit codes: 0 clean, 1 findings, 2 guard violation.
"""

import os
import sys

import agent_common as ac

PREFIX = "GATE-MEMORY"
BEGIN = "<!-- BEGIN:agents-memory -->"
END = "<!-- END:agents-memory -->"


# Per-file digest windows (D47): pitfalls and status are append-only logs that
# would otherwise push the hub past the host's observed ~30k injection ceiling.
WINDOW = {"pitfalls": 15, "status": 6}


def _digest(path):
    """One bullet per `## ` entry (title only), so AGENTS.md stays a navigable
    index instead of a 300 KB full-text dump. Full text lives in the source file."""
    out = []
    with open(path) as handle:
        for line in handle:
            if line.startswith("## "):
                out.append("- " + line[3:].strip())
    return out


def _windowed(items, limit):
    """Newest `limit` bullets plus a pointer line for the rest (D47)."""
    if not limit or len(items) <= limit:
        return items
    omitted = len(items) - limit
    word = "entry" if omitted == 1 else "entries"
    return ["- ... %d older %s omitted (read the full file)" % (omitted, word)] + items[-limit:]


def _build_block(memorys_root):
    rels = sorted(
        rel for rel in ac.iter_rel_files(memorys_root, ".md")
    )
    parts = [
        "",
        "## AGENT MEMORY (generated digest -- full text in `.agents/memorys/*.md`, read on demand)",
        "",
        "This region is a NAVIGABLE DIGEST: one bullet per entry plus a pointer to the full",
        "file. Read the source file before acting on a rule you are unsure about.",
        "",
    ]
    for rel in rels:
        name = os.path.splitext(rel)[0]
        parts.append("### %s" % name)
        parts.append("- full text: `.agents/memorys/%s`" % rel)
        for item in _windowed(_digest(os.path.join(memorys_root, rel)), WINDOW.get(name)):
            parts.append(item)
        parts.append("")
    return "\n".join(parts).rstrip("\n") + "\n"


def _current_region(hub_text):
    if BEGIN in hub_text and END in hub_text:
        return hub_text.split(BEGIN, 1)[1].split(END, 1)[0]
    return None


def run(command, targets, cfg, dry_run):
    memorys_root = os.path.join(ac.ROOT, cfg["memorys_root"])
    hub_path = os.path.join(ac.ROOT, cfg["memory_hub"])
    if not os.path.isdir(memorys_root):
        print("%s ✗ canonical-empty %s" % (PREFIX, memorys_root))
        return ac.EXIT_GUARD
    if not os.path.exists(hub_path):
        print("%s ✗ hub-missing %s" % (PREFIX, hub_path))
        return ac.EXIT_GUARD

    block = _build_block(memorys_root)
    hub_text = open(hub_path).read()
    if not ac.single_marker_ok(PREFIX, hub_text, BEGIN, END):
        return ac.EXIT_GUARD
    region = _current_region(hub_text)

    if command == "list":
        print("%-10s memory -> %s" % ("hub", cfg["memory_hub"]))
        return ac.EXIT_OK

    if region is None:
        findings = True  # block not present yet
        if command == "check":
            print(" [ FIX ] hub %s (memory region absent)" % cfg["memory_hub"])
            print("%s ✗ hub needs repair (run: agent-links.py sync)" % PREFIX)
            return ac.EXIT_FINDINGS
    else:
        if region.strip() == block.strip():
            findings = False
        else:
            findings = True
            if command == "check":
                print(" [ FIX ] hub %s (memory region drift)" % cfg["memory_hub"])
                print("%s ✗ hub needs repair (run: agent-links.py sync)" % PREFIX)
                return ac.EXIT_FINDINGS

    if command == "check":
        print("%s ✓ memory region matches (%d file(s))" % (PREFIX, len(ac.iter_rel_files(memorys_root, ".md"))))
        return ac.EXIT_OK

    # sync / dry-run
    if region is None:
        new_text = hub_text.rstrip("\n") + "\n\n" + BEGIN + "\n" + block + END + "\n"
        print(" [ APPEND ] hub %s (memory region)" % cfg["memory_hub"])
    else:
        before = hub_text.split(BEGIN, 1)[0]
        after = hub_text.split(END, 1)[1] if (END in hub_text) else ""
        new_text = before + BEGIN + "\n" + block + END + after
        print(" [ REWRITE] hub %s (memory region)" % cfg["memory_hub"])
    if not dry_run:
        with open(hub_path, "w") as handle:
            handle.write(new_text)
    print("%s ✓ sync done" % PREFIX)
    return ac.EXIT_OK


if __name__ == "__main__":
    print(__doc__.strip())
    sys.exit(0)
