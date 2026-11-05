#!/usr/bin/env python3
"""skill-lint: executable checks for the skill tree.

Adopted from the same-lineage remote_control_host repo (.refinfo, read-only).

Checks every <skills>/*/SKILL.md:
  1. SKILL.md exists
  2. frontmatter with non-empty name / description
  3. name == directory name
  4. description <= 1024 chars
  5. SKILL.md <= 800 lines
  6. every @path reference resolves inside the skill dir
  7. registry (when present) mentions every skill directory
  8. status: inoperative => description contains INOPERATIVE
  9. boundary map (TRIGGERS.md) names every skill, and every owner it names exists

Check 9 is the anti-rot pair for the one-intent-one-owner map: a skill added without a
row is invisible to routing review, and a row pointing at a deleted skill is a lie agents
will follow. The map itself cannot be generated ("which skill owns this intent" is a
judgement), so it is hand-written and machine-checked from both sides instead.

Division of labor for check 7 (say it out loud so a red line names ONE judge):
the authoritative inventory is the gate-agents SKILLS-INDEX segment, rebuilt and
compared by scripts/skills-index.py. This script's registry pass is a cheap
independent re-check of the same fact from the other side (directory walk ->
registry text), so a generator that silently drops a skill still reddens here.
When both complain, gate-agents is the first judge.

Exit codes: 0 clean, 1 findings, 3 internal error.
Stdlib only; Python 3.8 compatible.
"""

import argparse
import os
import re
import shutil
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MAX_DESC = 1024
MAX_LINES = 800
REF = re.compile(
    r"@((?:\./)?[A-Za-z0-9_][A-Za-z0-9_./-]*\.(?:md|py|sh|json|toml|yaml|yml|cjs|mjs|js|ts|txt))"
)
OWNER_TOKEN = re.compile(r"`([a-z][a-z0-9-]{2,})`")
# Names that legitimately have no directory HERE because the host ships them (verified
# against the live skill panel, not guessed). Adding to this set is a claim about a host
# built-in -- a row in TRIGGERS.md must say so, and skill-lint keeps checking every other
# owner token against the real tree.
HOST_SIDE = {"create-skill", "create-subagent", "qoder-ide-helper", "canvas", "vercel-deploy"}


def parse_frontmatter(text):
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return None
    end = None
    for index in range(1, min(len(lines), 60)):
        if lines[index].strip() == "---":
            end = index
            break
    if end is None:
        return None
    data = {}
    key = None
    for line in lines[1:end]:
        match = re.match(r"^([A-Za-z_][A-Za-z0-9_-]*):\s*(.*)$", line)
        if match:
            key = match.group(1)
            value = match.group(2).strip()
            if value in (">", "|", ">-", "|-", ">+", "|+"):
                value = ""
            data[key] = value
        elif key and line.startswith((" ", "\t")):
            data[key] = (data.get(key, "") + " " + line.strip()).strip()
    return data


def check_skill(skill_dir):
    name = os.path.basename(skill_dir)
    findings = []
    path = os.path.join(skill_dir, "SKILL.md")
    if not os.path.isfile(path):
        return [(name, "missing SKILL.md")]
    with open(path, encoding="utf-8", errors="ignore") as handle:
        text = handle.read()
    front = parse_frontmatter(text)
    if front is None:
        return [(name, "missing frontmatter")]
    if not front.get("name"):
        findings.append((name, "frontmatter name empty"))
    if not front.get("description"):
        findings.append((name, "frontmatter description empty"))
    if front.get("name") and front["name"] != name:
        findings.append((name, "name != directory (%s)" % front["name"]))
    desc = front.get("description", "")
    if len(desc) > MAX_DESC:
        findings.append((name, "description %d > %d chars" % (len(desc), MAX_DESC)))
    line_count = text.count("\n") + 1
    if line_count > MAX_LINES:
        findings.append((name, "SKILL.md %d > %d lines" % (line_count, MAX_LINES)))
    for match in REF.finditer(text):
        ref = match.group(1)
        candidate = ref[2:] if ref.startswith("./") else ref
        if not os.path.exists(os.path.join(skill_dir, candidate)):
            findings.append((name, "unresolved @ref %s" % ref))
    if front.get("status", "").strip() == "inoperative":
        if "INOPERATIVE" not in desc.upper():
            findings.append((name, "inoperative but description lacks INOPERATIVE"))
    return findings


def skill_names(skills_dir):
    return sorted(
        entry
        for entry in os.listdir(skills_dir)
        if os.path.isdir(os.path.join(skills_dir, entry))
    )


def check_boundary_map(names, boundary_path):
    """Both rot directions of the trigger boundary map."""
    if not os.path.isfile(boundary_path):
        return [("TRIGGERS.md", "boundary map missing (one intent, one owner)")]
    with open(boundary_path, encoding="utf-8", errors="ignore") as handle:
        text = handle.read()
    findings = []
    for name in names:
        if "`%s`" % name not in text:
            findings.append((name, "not named in the TRIGGERS.md boundary map"))
    known = set(names) | HOST_SIDE
    for line in text.splitlines():
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if len(cells) < 2 or cells[1].startswith("---") or cells[0].startswith("Intent"):
            continue
        for token in OWNER_TOKEN.findall(cells[1]):
            if token not in known:
                findings.append(("TRIGGERS.md", "owner `%s` has no skill directory" % token))
    return findings


def lint(skills_dir, registry_path, boundary_path=None):
    if not os.path.isdir(skills_dir):
        return None, 0
    names = skill_names(skills_dir)
    findings = []
    for name in names:
        findings.extend(check_skill(os.path.join(skills_dir, name)))
    if os.path.isfile(registry_path):
        with open(registry_path, encoding="utf-8", errors="ignore") as handle:
            registry = handle.read()
        for name in names:
            if "`%s`" % name not in registry:
                findings.append((name, "not mentioned in SKILL.md registry"))
    if boundary_path is None:
        boundary_path = os.path.join(ROOT, ".agents", "skills", "TRIGGERS.md")
    findings.extend(check_boundary_map(names, boundary_path))
    return findings, len(names)


def self_test(skills_dir, registry_path, boundary_path):
    """C19 breakage probes: every predicate must be shown to bite, and each probe must
    name the defect it planted. Temp files only; the repo is never touched."""
    if not os.path.isdir(skills_dir):
        print("SKILL-LINT SKIP self-test (no skill tree)")
        return 0
    tmp = tempfile.mkdtemp(prefix="skill-lint-probe-")
    results = []
    try:
        probe_dir = os.path.join(tmp, "probe-skill")
        os.makedirs(probe_dir)
        with open(os.path.join(probe_dir, "SKILL.md"), "w", encoding="utf-8") as handle:
            handle.write("---\nname: other-name\ndescription: planted mismatch\n---\n\nbody\n")
        # A map that neither names the planted skill nor points at a real owner: one call
        # must report BOTH boundary rot directions.
        probe_map = os.path.join(tmp, "TRIGGERS.md")
        with open(probe_map, "w", encoding="utf-8") as handle:
            handle.write("| Intent | Owning skill | Hand-off |\n|---|---|---|\n")
            handle.write("| some intent | `ghost-skill-no-such-dir` | none |\n")
        findings, _ = lint(tmp, registry_path, probe_map)
        results.append(("name != directory", any("name != directory" in m for _n, m in findings)))
        results.append(("map coverage miss", any("not named in the TRIGGERS.md" in m for _n, m in findings)))
        results.append(("dangling owner", any("no skill directory" in m for _n, m in findings)))
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    failed = [label for label, bit in results if not bit]
    for label, bit in results:
        print("SKILL-LINT %s self-test: %s" % ("✓" if bit else "✗", label))
    if failed:
        print("SKILL-LINT ✗ %d probe(s) did not red" % len(failed))
        return 1
    return 0


def main(argv):
    parser = argparse.ArgumentParser()
    parser.add_argument("--skills", default=os.path.join(ROOT, ".agents", "skills"))
    parser.add_argument("--registry", default=os.path.join(ROOT, "SKILL.md"))
    parser.add_argument("--boundary", default=os.path.join(ROOT, ".agents", "skills", "TRIGGERS.md"))
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv[1:])

    if args.self_test:
        return self_test(args.skills, args.registry, args.boundary)

    findings, count = lint(args.skills, args.registry, args.boundary)
    if findings is None:
        print("SKILL-LINT SKIP (no %s)" % args.skills)
        return 0
    if findings:
        for name, message in findings:
            print("%-32s %s" % (name, message))
        print("SKILL-LINT ✗ %d finding(s) across %d skill(s)" % (len(findings), count))
        return 1
    print("SKILL-LINT ✓ %d skill(s) clean" % count)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except Exception as exc:  # noqa: BLE001 - exit-code contract, not a lint rule
        print("SKILL-LINT ✗ internal error: %s" % exc, file=sys.stderr)
        sys.exit(3)
