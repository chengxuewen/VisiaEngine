#!/usr/bin/env python3
"""Translation tooling for the docs book (S5).

Subcommands:
  pot     regenerate book/po/messages.pot from the translation scope only
          (the 7 human-facing pages -- the full book has ~2950 msgids, the scope
          has ~850; see plan §10)
  status  per-file translated / fuzzy / untranslated counts

The xgettext backend comes from mdbook-i18n-helpers, installed repo-locally into
.tools/ by `pixi run book-i18n-setup` (cargo-binstall, pinned).
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BOOK_DIR = os.path.join(ROOT, "book")
PO_DIR = os.path.join(BOOK_DIR, "po")
TOOLS = os.path.join(ROOT, ".tools", "bin")

# Translation scope (plan §10-2): what a newcomer reads, plus the reference index.
SCOPE = [
    "README.md",
    "docs/architecture.md",
    "docs/env.md",
    "docs/tutorials.md",
    "docs/whitepaper.md",
    "docs/gap-closure-roadmap.md",
    "docs/reference/README.md",
]


def tools_path() -> str:
    return TOOLS + os.pathsep + os.environ.get("PATH", "")


def cmd_pot(_args) -> int:
    if not os.path.exists(os.path.join(TOOLS, "mdbook-xgettext")):
        print("book-i18n: mdbook-xgettext missing -- run: pixi run book-i18n-setup")
        return 127

    stage = os.path.join(ROOT, "build", "i18n-src")
    shutil.rmtree(stage, ignore_errors=True)
    lines = ["# Summary", ""]
    for rel in SCOPE:
        src = os.path.join(ROOT, rel)
        if not os.path.exists(src):
            print(f"book-i18n: scope file missing: {rel}")
            return 1
        dst = os.path.join(stage, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copy2(src, dst)
        title = rel
        with open(src, encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("# "):
                    title = line[2:].strip()
                    break
        lines.append(f"- [{title}]({rel})")
    with open(os.path.join(stage, "SUMMARY.md"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")

    os.makedirs(PO_DIR, exist_ok=True)
    env = dict(os.environ, PATH=tools_path(),
               MDBOOK_BOOK__SRC=stage,
               MDBOOK_OUTPUT='{"xgettext":{"pot-file":"messages.pot"}}')
    proc = subprocess.run(["mdbook", "build", BOOK_DIR, "-d", PO_DIR], cwd=ROOT, env=env)
    if proc.returncode != 0:
        return proc.returncode
    pot = os.path.join(PO_DIR, "messages.pot")
    with open(pot, encoding="utf-8") as handle:
        print(f"book-i18n: {sum(1 for l in handle if l.startswith('msgid '))} msgids -> book/po/messages.pot")
    return 0


def parse_po(path: str) -> list[dict]:
    """Minimal PO parser: returns [{msgid, msgstr, fuzzy, refs}]."""
    entries, cur = [], {"msgid": None, "msgstr": "", "fuzzy": False, "refs": []}
    field = None
    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.rstrip("\n")
            if not line.strip():
                if cur["msgid"] is not None:
                    entries.append(cur)
                cur = {"msgid": None, "msgstr": "", "fuzzy": False, "refs": []}
                field = None
                continue
            if line.startswith("#,") and "fuzzy" in line:
                cur["fuzzy"] = True
            elif line.startswith("#:"):
                cur["refs"].extend(line[2:].split())
            elif line.startswith("msgid "):
                field = "msgid"
                cur["msgid"] = line[6:].strip().strip('"')
            elif line.startswith("msgstr "):
                field = "msgstr"
                cur["msgstr"] = line[7:].strip().strip('"')
            elif line.startswith('"'):
                text = line.strip().strip('"')
                if field:
                    cur[field] += text
    if cur["msgid"] is not None:
        entries.append(cur)
    return entries


def cmd_status(_args) -> int:
    po = os.path.join(PO_DIR, "zh-CN.po")
    if not os.path.exists(po):
        print("book-i18n: book/po/zh-CN.po missing")
        return 1
    by_file: dict[str, list[int]] = {}
    for entry in parse_po(po):
        if not entry["msgid"]:
            continue
        ref = entry["refs"][0].split(":")[0] if entry["refs"] else "?"
        ref = ref.split("build/i18n-src/", 1)[-1]  # xgettext records the staged path
        bucket = by_file.setdefault(ref, [0, 0, 0])  # translated, fuzzy, missing
        if entry["fuzzy"]:
            bucket[1] += 1
        elif entry["msgstr"]:
            bucket[0] += 1
        else:
            bucket[2] += 1
    total = [0, 0, 0]
    for ref in sorted(by_file):
        t, f, m = by_file[ref]
        print(f"  {ref:38s} ok={t:4d} fuzzy={f:3d} todo={m:4d}")
        total = [total[0] + t, total[1] + f, total[2] + m]
    denom = sum(total) or 1
    print(f"book-i18n: {total[0]}/{denom} translated ({100 * total[0] // denom}%), "
          f"{total[1]} fuzzy, {total[2]} untranslated")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("pot").set_defaults(func=cmd_pot)
    sub.add_parser("status").set_defaults(func=cmd_status)
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
