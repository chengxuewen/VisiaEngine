#!/usr/bin/env python3
"""Generate a compact SKILL DIRECTORY into the root AGENTS.md hub.

Single source: <repo>/.agents/skills/*/SKILL.md. Target: the root AGENTS.md hub.
The generated list lives inside a marked region
  <!-- BEGIN:skills-directory --> ... <!-- END:skills-directory -->
and gate-agents asserts the region matches; sync only rewrites inside the markers.
Hand-written hub content is never touched. Skills whose frontmatter has
`status: deprecated` are tagged as a deprecated alias (e.g. review-hardcode).

Exit codes: 0 clean, 1 findings, 2 guard violation.
"""

import os
import re
import sys

import agent_common as ac

PREFIX = "GATE-SKILLS-INDEX"
BEGIN = "<!-- BEGIN:skills-directory -->"
END = "<!-- END:skills-directory -->"
REGISTRY_BEGIN = "<!-- BEGIN:skill-registry -->"
REGISTRY_END = "<!-- END:skill-registry -->"

_FM = re.compile(r"^---\n(.*?)\n---\n", re.DOTALL)


def _frontmatter(text):
    m = _FM.match(text)
    if not m:
        return {}
    fm = {}
    for line in m.group(1).splitlines():
        if ":" in line:
            key, val = line.split(":", 1)
            fm[key.strip()] = val.strip().strip('"').strip("'")
    return fm


def _english_only(text):
    """The skill DIRECTORY is a summary for the English hub; the routing triggers
    live in the real SKILL.md description, so strip CJK trigger runs here to keep
    AGENTS.md English (C17/D38: triggers stay in SKILL.md, not in the hub index)."""
    text = re.sub(r"[\u4e00-\u9fff]+", "", text)
    text = re.sub(r"\s{2,}", " ", text).strip()
    return text.lstrip("-—:： ").strip()


def _shorten(desc, limit=140):
    desc = _english_only(desc.strip())
    if not desc:
        return ""
    # first sentence, then hard-cap
    if ". " in desc:
        desc = desc.split(". ", 1)[0] + "."
    if len(desc) > limit:
        desc = desc[: limit - 3].rstrip() + "..."
    return desc


def _skill_rows(skills_root):
    rows = []
    for name in sorted(os.listdir(skills_root)):
        skill_md = os.path.join(skills_root, name, "SKILL.md")
        if not os.path.isfile(skill_md):
            continue
        with open(skill_md) as handle:
            fm = _frontmatter(handle.read())
        label = "`%s`" % (fm.get("name") or name)
        if fm.get("status") == "deprecated":
            label += " (deprecated alias)"
        rows.append((label, _shorten(fm.get("description", ""), limit=45)))
    return rows


_LEGACY_HEADER = "| Skill | Type | Purpose |"


def _build_registry(rows):
    parts = ["", "| Skill | Purpose |", "|-------|---------|"]
    for label, purpose in rows:
        parts.append("| %s | %s |" % (label, purpose.replace("|", "\\|")))
    return "\n".join(parts).rstrip("\n") + "\n"


def _registry_region(text):
    if REGISTRY_BEGIN in text and REGISTRY_END in text:
        return text.split(REGISTRY_BEGIN, 1)[1].split(REGISTRY_END, 1)[0]
    return None


def _registry_rows(region):
    return sum(1 for line in region.splitlines() if line.startswith("| `"))


def _strip_legacy_table(text):
    """Drop the pre-generation hand-written table (header: Skill | Type | Purpose)."""
    out, skipping = [], False
    for line in text.splitlines():
        if line.strip() == _LEGACY_HEADER:
            skipping = True
            continue
        if skipping:
            if line.startswith("|"):
                continue
            skipping = False
        out.append(line)
    return "\n".join(out).rstrip("\n") + "\n"


def _registry(command, cfg, rows, dry_run):
    """Generate the human-facing registry in SKILL.md (same rows as the hub)."""
    name = cfg.get("skills_registry", "SKILL.md")
    path = os.path.join(ac.ROOT, name)
    if not os.path.exists(path):
        print("%s ✗ registry-missing %s" % (PREFIX, path))
        return ac.EXIT_GUARD
    text = open(path).read()
    if not ac.single_marker_ok(PREFIX, text, REGISTRY_BEGIN, REGISTRY_END):
        return ac.EXIT_GUARD
    block = _build_registry(rows)
    region = _registry_region(text)
    if region is None:
        if command == "check":
            print(" [ FIX ] registry %s (skill-registry region absent)" % name)
            return ac.EXIT_FINDINGS
        new_text = _strip_legacy_table(text).rstrip("\n") + "\n\n" + REGISTRY_BEGIN + "\n" + block + REGISTRY_END + "\n"
        action = "APPEND"
    else:
        now = _registry_rows(region)
        if now == len(rows) and region.strip() == block.strip():
            return ac.EXIT_OK
        if command == "check":
            why = "row count %d != %d" % (now, len(rows)) if now != len(rows) else "content drift"
            print(" [ FIX ] registry %s (%s)" % (name, why))
            return ac.EXIT_FINDINGS
        new_text = (text.split(REGISTRY_BEGIN, 1)[0] + REGISTRY_BEGIN + "\n" + block
                    + REGISTRY_END + text.split(REGISTRY_END, 1)[1])
        action = "REWRITE"
    if not dry_run:
        with open(path, "w") as handle:
            handle.write(new_text)
    print(" [ %-7s] registry %s (skill-registry)" % (action, name))
    return ac.EXIT_OK


def _build_block(skills_root):
    rows = _skill_rows(skills_root)
    parts = ["", "## SKILL DIRECTORY (generated -- edit .agents/skills/*, not this block)", ""]
    parts.append("| Skill | Purpose |")
    parts.append("|-------|---------|")
    for label, purpose in rows:
        parts.append("| %s | %s |" % (label, purpose.replace("|", "\\|")))
    return "\n".join(parts).rstrip("\n") + "\n"


def _current_region(hub_text):
    if BEGIN in hub_text and END in hub_text:
        return hub_text.split(BEGIN, 1)[1].split(END, 1)[0]
    return None


MEMORY_BEGIN = "<!-- BEGIN:agents-memory -->"


def _without_own_region(hub_text):
    if BEGIN not in hub_text or END not in hub_text:
        return hub_text
    before, rest = hub_text.split(BEGIN, 1)
    _own, after = rest.split(END, 1)
    return before + after.lstrip("\n")


def _place(hub_text, block):
    """Insert the skills region BEFORE the memory region (D47/B4).

    Hosts truncate long instruction files from the tail, so the skill directory
    must not sit at EOF. Idempotent: an already-correct region rewrites to itself.
    """
    text = _without_own_region(hub_text)
    section = BEGIN + "\n" + block + END + "\n"
    if MEMORY_BEGIN in text:
        head, tail = text.split(MEMORY_BEGIN, 1)
        return head + section + "\n" + MEMORY_BEGIN + tail
    return text.rstrip("\n") + "\n\n" + section


def run(command, targets, cfg, dry_run):
    skills_root = os.path.join(ac.ROOT, cfg["skills_root"])
    hub_path = os.path.join(ac.ROOT, cfg["memory_hub"])
    if not os.path.isdir(skills_root):
        print("%s ✗ canonical-empty %s" % (PREFIX, skills_root))
        return ac.EXIT_GUARD
    if not os.path.exists(hub_path):
        print("%s ✗ hub-missing %s" % (PREFIX, hub_path))
        return ac.EXIT_GUARD

    block = _build_block(skills_root)
    hub_text = open(hub_path).read()
    if not ac.single_marker_ok(PREFIX, hub_text, BEGIN, END):
        return ac.EXIT_GUARD
    region = _current_region(hub_text)
    rows = _skill_rows(skills_root)
    count = len(rows)

    if command == "list":
        print("%-10s skills -> %s" % ("hub", cfg["memory_hub"]))
        return ac.EXIT_OK

    memory_pos = hub_text.find(MEMORY_BEGIN)
    own_pos = hub_text.find(BEGIN)
    misplaced = memory_pos != -1 and own_pos != -1 and own_pos > memory_pos

    if region is None:
        if command == "check":
            print(" [ FIX ] hub %s (skills directory absent)" % cfg["memory_hub"])
            print("%s ✗ hub needs repair (run: agent-links.py sync)" % PREFIX)
            return ac.EXIT_FINDINGS
    elif region.strip() != block.strip() or misplaced:
        if command == "check":
            reason = "drift" if region.strip() != block.strip() else "out of order (must precede the memory region)"
            print(" [ FIX ] hub %s (skills directory %s)" % (cfg["memory_hub"], reason))
            print("%s ✗ hub needs repair (run: agent-links.py sync)" % PREFIX)
            return ac.EXIT_FINDINGS

    if command == "check":
        reg_rc = _registry(command, cfg, rows, dry_run)
        if reg_rc != ac.EXIT_OK:
            return reg_rc
        print("%s ✓ skills directory matches (%d skill(s), before memory)" % (PREFIX, count))
        return ac.EXIT_OK

    # sync / dry-run
    new_text = _place(hub_text, block)
    print(" [ PLACE ] hub %s (skills directory before memory)" % cfg["memory_hub"])
    if not dry_run:
        with open(hub_path, "w") as handle:
            handle.write(new_text)
    reg_rc = _registry(command, cfg, rows, dry_run)
    print("%s ✓ sync done" % PREFIX)
    return reg_rc


if __name__ == "__main__":
    print(__doc__.strip())
    sys.exit(0)
