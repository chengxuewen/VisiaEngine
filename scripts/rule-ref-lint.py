#!/usr/bin/env python3
"""rule-ref-lint: every C/D/PIT citation must resolve to a local memory entry.

Adopted from the same-lineage remote_control_host repo (.refinfo, read-only).

Scan roots: .agents/rules/**/*.md, .agents/skills/**/SKILL.md, AGENTS.md, SKILL.md
Resolution: headings of .agents/memorys/{conventions,decisions,pitfalls}.md (## or ###)
Exempt:    skills with `status: inoperative` in frontmatter; scripts/rule-ref-allowlist.txt

The generated host layers (.trae/.codebuddy/.qoder) and the hub's generated
regions are NOT scanned: they mirror the sources, so a citation error there is
already a sync problem, and scanning copies would double every finding.

Exit codes: 0 clean, 1 findings, 3 internal error.
Stdlib only; Python 3.8 compatible.
"""

import argparse
import os
import re
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

MEMORY_FILES = {
    "conventions": ".agents/memorys/conventions.md",
    "decisions": ".agents/memorys/decisions.md",
    "pitfalls": ".agents/memorys/pitfalls.md",
}
HEADING = {
    "conventions": re.compile(r"^#{2,3}\s+(C\d+)\b"),
    "decisions": re.compile(r"^#{2,3}\s+(D\d+)\b"),
    "pitfalls": re.compile(r"^#{2,3}\s+(PIT-\d+)\b"),
}
REF = re.compile(r"(?<![A-Za-z0-9])(?:C\d+|D\d+|PIT-\d+)(?![A-Za-z0-9])")
# Append-only discipline: a heading whose ID is followed by an addendum/correction
# marker re-uses the parent ID ON PURPOSE (D46 Correction, C19 addendum). The
# duplicate scan must not treat those as a collision, or the gate trains everyone
# to ignore it -- the exact "20 repeated ids" failure mode PIT-88 records.
ADDENDUM = re.compile(r"(?i)\b(addendum|correction|amendment|superseded|note)\b")
SCAN_ROOTS = [".agents/rules", ".agents/skills"]
SCAN_FILES = ["AGENTS.md", "SKILL.md"]
ALLOWLIST = "scripts/rule-ref-allowlist.txt"
INOPERATIVE = re.compile(r"(?m)^status:\s*inoperative\b")


def local_ids():
    ids = set()
    for kind, rel in MEMORY_FILES.items():
        path = os.path.join(ROOT, rel)
        if not os.path.isfile(path):
            continue
        with open(path, encoding="utf-8", errors="ignore") as handle:
            for line in handle:
                match = HEADING[kind].match(line)
                if match:
                    ids.add(match.group(1))
    return ids


def duplicate_ids():
    """C/D/PIT ids repeated inside one memory file. Three independent namespaces:
    C10 and D10 may coexist, so the comparison is per file and per kind only.
    Addendum/correction headings (same ID + marker word) are legitimate re-uses."""
    findings = []
    for kind, rel in MEMORY_FILES.items():
        path = os.path.join(ROOT, rel)
        if not os.path.isfile(path):
            continue
        seen = {}
        with open(path, encoding="utf-8", errors="ignore") as handle:
            for lineno, line in enumerate(handle, 1):
                match = HEADING[kind].match(line)
                if not match:
                    continue
                token = match.group(1)
                if ADDENDUM.search(line):
                    continue
                if token in seen:
                    findings.append(
                        "%s:%d: duplicate %s (first at line %d)" % (rel, lineno, token, seen[token])
                    )
                else:
                    seen[token] = lineno
    return findings


def allow_ids():
    ids = set()
    path = os.path.join(ROOT, ALLOWLIST)
    if not os.path.isfile(path):
        return ids
    with open(path, encoding="utf-8", errors="ignore") as handle:
        for line in handle:
            token = line.split("#", 1)[0].strip()
            if token:
                ids.add(token.split()[0])
    return ids


def iter_files():
    for rel_root in SCAN_ROOTS:
        root = os.path.join(ROOT, rel_root)
        if not os.path.isdir(root):
            continue
        for dirpath, _dirnames, filenames in os.walk(root):
            for name in sorted(filenames):
                if not name.endswith(".md"):
                    continue
                if rel_root.endswith("skills") and name != "SKILL.md":
                    continue
                yield os.path.join(dirpath, name)
    for rel in SCAN_FILES:
        path = os.path.join(ROOT, rel)
        if os.path.isfile(path):
            yield path


def is_inoperative(path):
    if not path.endswith("SKILL.md"):
        return False
    with open(path, encoding="utf-8", errors="ignore") as handle:
        head = handle.read(4096)
    return INOPERATIVE.search(head) is not None


def scan(paths, local, allow):
    findings = []
    exempt = 0
    for path in sorted(set(paths)):
        if is_inoperative(path):
            exempt += 1
            continue
        with open(path, encoding="utf-8", errors="ignore") as handle:
            for lineno, line in enumerate(handle, 1):
                for match in REF.finditer(line):
                    ref = match.group(0)
                    if ref in local or ref in allow:
                        continue
                    findings.append("%s:%d: %s" % (os.path.relpath(path, ROOT), lineno, ref))
    return sorted(set(findings)), exempt


def main(argv):
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv[1:])

    local = local_ids()
    allow = allow_ids()

    duplicates = duplicate_ids()
    if duplicates:
        print("\n".join(duplicates))
        print("DUPLICATE x %d repeated id(s)" % len(duplicates))
        return 1
    print("DUPLICATE OK no repeated C/D/PIT id")

    if args.self_test:
        tmp = tempfile.mkdtemp(prefix="rule-ref-probe-")
        probe = os.path.join(tmp, "probe.md")
        try:
            with open(probe, "w", encoding="utf-8") as handle:
                handle.write("- planted PIT-999\n")
            findings, _ = scan([probe], local, allow)
        finally:
            try:
                os.remove(probe)
                os.rmdir(tmp)
            except OSError:
                pass
        if findings:
            print("RULE-REF ✓ self-test (planted PIT-999 red)")
            return 0
        print("RULE-REF ✗ self-test did not red")
        return 1

    findings, exempt = scan(iter_files(), local, allow)
    if findings:
        print("\n".join(findings))
        print(
            "RULE-REF ✗ %d unresolved citation(s); %d local id(s), %d allowlisted, %d inoperative file(s) exempt"
            % (len(findings), len(local), len(allow), exempt)
        )
        return 1
    print(
        "RULE-REF ✓ citations resolve (%d local, %d allowlisted, %d inoperative exempt)"
        % (len(local), len(allow), exempt)
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    except Exception as exc:  # noqa: BLE001 - exit-code contract, not a lint rule
        print("RULE-REF ✗ internal error: %s" % exc, file=sys.stderr)
        sys.exit(3)
