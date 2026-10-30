#!/usr/bin/env python3
"""Prose gate for the human-facing docs (plan §11).

Three checks, all driven by the chapter types declared in book/toc.toml:

  banned   -- adjective patterns that stand in for a checkable fact
              (docs/prose-banned.toml; only chapters marked polish = true)
  type     -- each chapter must do the job its type claims:
              how-to      >=1 shell fence
              tutorial    >=1 fence or numbered step
              explanation >=1 mermaid fence, path:line anchor or external link;
                          no "Step N" procedure headings
              reference   no "you should / you must" instructions
  anchors  -- every path:line reference must point at a real file

Exit codes follow the shared gate convention: 0 clean, 1 findings.
"""

from __future__ import annotations

import argparse
import glob
import os
import re
import shutil
import subprocess
import sys
import tomllib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TOC = os.path.join(ROOT, "book", "toc.toml")
BANNED = os.path.join(ROOT, "docs", "prose-banned.toml")

SHELL_FENCE = re.compile(r"^```(?:bash|sh|console)\b", re.M)
ANY_FENCE = re.compile(r"^```", re.M)
NUMBERED = re.compile(r"^\s*\d+\.\s", re.M)
MERMAID = re.compile(r"^```mermaid\b", re.M)
ANCHOR = re.compile(r"([\w][\w./-]*\.(?:rs|md|toml|cpp|h|hpp|py|sh|json|yml|txt)):(\d+)")
EXTERNAL = re.compile(r"\]\(https?://")
STEP_HEADING = re.compile(r"^\s*(?:step\s*\d+|\u6b65\u9aa4\s*\d+)", re.I | re.M)
IMPERATIVE = re.compile(r"\byou (?:should|must)\b|\u5efa\u8bae|\u8bf7\u6267\u884c", re.I)


def load_chapters():
    with open(TOC, "rb") as handle:
        toc = tomllib.load(handle)
    # Grouped chapters count too: the SDD contracts are type=reference and must not
    # drift into instructions ("you should ..."), which is exactly what this gate watches.
    return list(toc.get("chapter", []))


def load_banned():
    with open(BANNED, "rb") as handle:
        data = tomllib.load(handle)
    words = []
    for section in ("english", "chinese"):
        words.extend(data.get(section, {}).get("words", []))
    return words


def read(path):
    with open(path, encoding="utf-8") as handle:
        return handle.read()


def run_markdownlint() -> list[str]:
    """Mechanical markdown rules (MD001/MD009/MD025/MD040).

    Kept in this gate so `pixi run gate-prose` is the single prose entry point.
    docs/reference/** is excluded via .markdownlintignore (plan 14-4).
    """
    binary = shutil.which("markdownlint")
    if binary is None:
        return ["markdownlint not on PATH (expected from the book feature)"]
    targets = ["README.md"] + sorted(glob.glob("docs/*.md")) + sorted(glob.glob("docs/sdd/*.md"))
    proc = subprocess.run([binary] + targets, cwd=ROOT, capture_output=True, text=True)
    if proc.returncode == 0:
        return []
    return [line for line in proc.stdout.strip().splitlines() if line.strip()]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args()

    banned = load_banned()
    findings = []

    for chapter in load_chapters():
        source = chapter["source"]
        path = os.path.join(ROOT, source)
        if not os.path.exists(path):
            # Site-only chapters live under book/ and are assembled into the src tree
            # at build time (toc paths are relative to that tree, e.g. web/graph.md).
            path = os.path.join(ROOT, "book", source)
        if not os.path.exists(path):
            findings.append((source, "missing", "chapter file does not exist"))
            continue
        text = read(path)
        ctype = chapter.get("type", "explanation")

        # banned words -- prose only (polish-marked chapters), never contracts
        if chapter.get("polish"):
            lowered = text.lower()
            for word in banned:
                if word.lower() in lowered:
                    findings.append((source, "banned", f"adjective without a fact: {word!r}"))

        # per-type structural expectations
        if ctype == "how-to" and not SHELL_FENCE.search(text):
            findings.append((source, "type", "how-to without a shell fence"))
        if ctype == "tutorial" and not (ANY_FENCE.search(text) or NUMBERED.search(text)):
            findings.append((source, "type", "tutorial without steps or code"))
        if ctype == "explanation":
            if not (MERMAID.search(text) or ANCHOR.search(text) or EXTERNAL.search(text)):
                findings.append((source, "type", "explanation without a diagram, anchor or source link"))
            if STEP_HEADING.search(text):
                findings.append((source, "type", "explanation contains step-by-step instructions (move them to a how-to)"))
        if ctype == "reference" and IMPERATIVE.search(text):
            findings.append((source, "type", "reference reads like instructions (you should / must)"))

        # anchors must resolve
        for match in ANCHOR.finditer(text):
            target, line = match.group(1), int(match.group(2))
            candidate = os.path.join(ROOT, target)
            if not os.path.exists(candidate):
                findings.append((source, "anchor", f"{target}:{line} does not exist"))
                continue
            total = sum(1 for _ in open(candidate, encoding="utf-8", errors="ignore"))
            if line < 1 or line > total:
                findings.append((source, "anchor", f"{target}:{line} is past the end ({total} lines)"))

    for line in run_markdownlint():
        findings.append(("markdown", "markdownlint", line))

    if findings:
        for source, rule, detail in findings:
            print(f"GATE-PROSE \u2717 {source}: {rule}: {detail}")
        print(f"GATE-PROSE \u2717 {len(findings)} finding(s)")
        return 1

    if not args.quiet:
        print(f"GATE-PROSE \u2713 {len(load_chapters())} chapter(s): banned words, types, anchors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
