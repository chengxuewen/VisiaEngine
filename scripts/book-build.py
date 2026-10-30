#!/usr/bin/env python3
"""Assemble the mdBook source tree and build the docs site (plan §14, S1).

docs/** + book/toc.toml  -->  build/book-src/ (SUMMARY generated)
                          -->  mdbook build -d www/book/<lang>
                          -->  rustdoc (target/doc) merged into www/book/api if present

docs/ stays the single source of truth; build/ and www/ are fully derived.
Unlisted markdown files are appended under "Appendix" with a WARN (plan §9-1).
"""

from __future__ import annotations

import argparse
import re
import json
import os
import shutil
import subprocess
import sys
import tomllib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BOOK_DIR = os.path.join(ROOT, "book")
SRC_DIR = os.path.join(ROOT, "build", "book-src")
OUT_DIR = os.path.join(ROOT, "www", "book")


def first_heading(path: str) -> str:
    """Title = first `# ` line of the file, else the file name."""
    try:
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("# "):
                    return line[2:].strip()
    except OSError:
        pass
    return os.path.splitext(os.path.basename(path))[0]


def doc_files() -> list[str]:
    """All tracked-style markdown under docs/ (posix relpaths)."""
    found = []
    docs = os.path.join(ROOT, "docs")
    for base, _dirs, names in os.walk(docs):
        for name in sorted(names):
            if name.endswith(".md"):
                found.append(os.path.relpath(os.path.join(base, name), ROOT))
    return sorted(found)


def export_graph(src_dir: str) -> int:
    """Export the codegraph index into the book source tree (S2/S8)."""
    graph_db = os.path.join(ROOT, ".codegraph", "codegraph.db")
    if not os.path.exists(graph_db):
        print("BOOK - no .codegraph yet (run: pixi run mcp index) -- graph page will be empty")
        return 0
    return subprocess.run(
        ["node", "scripts/export-graph.mjs", os.path.join(src_dir, "graph.json")],
        cwd=ROOT, stdin=subprocess.DEVNULL,
    ).returncode


def write_meta(src_dir: str) -> None:
    """Provenance for the page footer (S8): commit, last doc edit, graph index time."""
    def git(*cmd: str) -> str:
        try:
            return subprocess.run(
                ["git", *cmd], cwd=ROOT, capture_output=True, text=True,
                stdin=subprocess.DEVNULL,
            ).stdout.strip()
        except OSError:
            return ""

    indexed = ""
    graph_json = os.path.join(src_dir, "graph.json")
    if os.path.exists(graph_json):
        try:
            with open(graph_json, encoding="utf-8") as handle:
                indexed = json.load(handle).get("generated_at", "")[:16].replace("T", " ") + " UTC"
        except (OSError, ValueError):
            indexed = ""
    meta = {
        "commit": git("rev-parse", "--short", "HEAD") or "uncommitted",
        "updated": git("log", "-1", "--format=%ad", "--date=short") or "",
        "indexed": indexed,
    }
    with open(os.path.join(src_dir, "meta.json"), "w", encoding="utf-8") as handle:
        json.dump(meta, handle, indent=2)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--public", action="store_true",
                        help="exclude chapters marked visibility = \"internal\"")
    parser.add_argument("--graph-only", action="store_true",
                        help="only re-export graph.json + meta.json into build/book-src (no mdbook build)")
    args = parser.parse_args()

    if args.graph_only:
        if not os.path.isdir(SRC_DIR):
            print("BOOK \u2717 no build/book-src yet -- run: pixi run book-build", file=sys.stderr)
            return 1
        code = export_graph(SRC_DIR)
        write_meta(SRC_DIR)
        return 0 if code == 0 else code

    with open(os.path.join(BOOK_DIR, "toc.toml"), "rb") as handle:
        toc = tomllib.load(handle)

    # 1) Fresh assembly dir (build/ is gitignored output territory).
    if os.path.isdir(SRC_DIR):
        shutil.rmtree(SRC_DIR)
    os.makedirs(SRC_DIR)

    # 2) Copy the whole docs/ tree (keeps sibling assets next to chapters).
    shutil.copytree(os.path.join(ROOT, "docs"), os.path.join(SRC_DIR, "docs"))
    shutil.copy2(os.path.join(ROOT, "README.md"), os.path.join(SRC_DIR, "README.md"))

    # 3) Optional site assets (graph page arrives in S2).
    web_src = os.path.join(BOOK_DIR, "web")
    if os.path.isdir(web_src):
        shutil.copytree(web_src, os.path.join(SRC_DIR, "web"))

    # 3b) Graph data for the interactive page (S2). Missing index is not fatal:
    #     the page renders an actionable hint instead (plan §3.2 tolerance).
    graph_code = export_graph(SRC_DIR)
    if graph_code != 0:
        print("BOOK WARN graph export failed -- graph page will be empty", file=sys.stderr)

    # 4) Collect the chapter list in toc order, expanding glob groups.
    hidden = {entry["source"] for entry in toc.get("chapter", [])
              if entry.get("visibility") == "internal"}
    chapters: list[dict] = []
    for entry in toc.get("chapter", []):
        if args.public and entry["source"] in hidden:
            continue
        chapters.append(entry)

    # A public build must not ship the internal pages at all (direct URL would still work).
    if args.public:
        for rel in sorted(hidden):
            baked = os.path.join(SRC_DIR, os.path.relpath(os.path.join(ROOT, rel), ROOT))
            if os.path.exists(baked):
                os.remove(baked)
        print(f"BOOK - public build: excluded {len(hidden)} internal page(s) from the artifact")

    covered: set[str] = set()
    lines: list[str] = ["# Summary", ""]
    current_group = None

    current_parent = None

    def emit(entry_src: str, entry_title: str, group: str | None, parent: str | None = None) -> None:
        nonlocal current_group, current_parent
        if parent:
            # nested under a draft parent row: mdBook's fold collapses the children
            if parent != current_parent:
                lines.append(f"- [{parent}]()")
                current_parent = parent
            lines.append(f"  - [{entry_title}]({entry_src})")
            covered.add(entry_src)
            return
        current_parent = None
        if group != current_group:
            lines.append(f"# {group}" if group else "")
            current_group = group
        rel = entry_src
        lines.append(f"- [{entry_title}]({rel})")
        covered.add(entry_src)

    for entry in chapters:
        title = entry.get("title") or first_heading(os.path.join(ROOT, entry["source"]))
        emit(entry["source"], title, entry.get("group"), entry.get("parent"))

    import glob as globmod
    for group in toc.get("group", []):
        pattern = os.path.join(ROOT, group["glob"])
        nested = group.get("nested", False)
        parent = group["title"] if nested else None
        if parent:
            current_parent = None
        for match in sorted(globmod.glob(pattern, recursive=True)):
            rel = os.path.relpath(match, ROOT)
            title = first_heading(match)
            if nested:
                # sidebar navigation wants the short name; the page keeps the full h1
                title = re.split(r"\s+[\u2014\u2013]\s+|:\s+", title, maxsplit=1)[0].strip()
                if len(title) > 48:
                    title = title[:48].rstrip(" ,;(") + "…"
                emit(rel, title, None, parent)
            else:
                emit(rel, title, group["title"])

    # 5) Unlisted markdown -> Appendix, with a visible WARN (plan §9-1).
    #    Pages excluded by --public are not "unlisted": they are deliberately hidden.
    appendix = [f for f in doc_files() if f not in covered and not (args.public and f in hidden)]
    if appendix:
        lines.append("# Appendix (unlisted in book/toc.toml)")
        for rel in appendix:
            print(f"BOOK WARN unlisted: {rel} -> appendix", file=sys.stderr)
            lines.append(f"- [{first_heading(os.path.join(ROOT, rel))}]({rel})")
        covered.update(appendix)

    with open(os.path.join(SRC_DIR, "SUMMARY.md"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")

    write_meta(SRC_DIR)

    # 6) Build the book (English-only since D46; browser-side translation covers
    #    other languages, so there is no gettext preprocessor and no .tools dependency).
    dest = os.path.join(OUT_DIR, "en")
    proc = subprocess.run(
        ["mdbook", "build", BOOK_DIR, "-d", dest],
        cwd=ROOT, stdin=subprocess.DEVNULL,
    )
    if proc.returncode != 0:
        print("BOOK ✗ mdbook build failed", file=sys.stderr)
        return proc.returncode

    # 7) rustdoc merge: only when the author already ran docs-gen (cargo doc).
    rustdoc = os.path.join(ROOT, "target", "doc")
    if os.path.isdir(rustdoc):
        shutil.copytree(rustdoc, os.path.join(OUT_DIR, "api"), dirs_exist_ok=True)
        print("BOOK ✓ rustdoc merged -> www/book/api/")
    else:
        print("BOOK - no target/doc yet (run: pixi run docs-gen) -- api/ skipped")

    # D46 follow-up: prune language trees left by the retired bilingual build and
    # keep a minimal root redirect, so book-check stays green on old checkouts and
    # the root URL still lands somewhere useful.
    for name in sorted(os.listdir(OUT_DIR)):
        stale = os.path.join(OUT_DIR, name)
        if name in ("en", "api"):
            continue
        if os.path.isdir(stale) and os.path.exists(os.path.join(stale, "index.html")):
            shutil.rmtree(stale)
            print(f"BOOK ✓ pruned stale language tree {name}/")
    stale_langs = os.path.join(OUT_DIR, "langs.json")
    if os.path.exists(stale_langs):
        os.remove(stale_langs)
    root_index = os.path.join(OUT_DIR, "index.html")
    with open(root_index, "w", encoding="utf-8") as handle:
        handle.write('<!doctype html>\n<meta http-equiv="refresh" content="0; url=./en/">\n'
                     '<p>The book lives at <a href="./en/">./en/</a>.</p>\n')

    n = len(covered)
    print(f"BOOK ✓ built {n} chapter(s) -> {os.path.relpath(dest, ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
