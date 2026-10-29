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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--public", action="store_true",
                        help="exclude chapters marked visibility = \"internal\"")
    args = parser.parse_args()

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
    graph_db = os.path.join(ROOT, ".codegraph", "codegraph.db")
    if os.path.exists(graph_db):
        code = subprocess.run(
            ["node", "scripts/export-graph.mjs", os.path.join(SRC_DIR, "graph.json")],
            cwd=ROOT, stdin=subprocess.DEVNULL,
        ).returncode
        if code != 0:
            print("BOOK WARN graph export failed -- graph page will be empty", file=sys.stderr)
    else:
        print("BOOK - no .codegraph yet (run: pixi run mcp index) -- graph page will be empty")

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

    def emit(entry_src: str, entry_title: str, group: str | None) -> None:
        nonlocal current_group
        if group != current_group:
            lines.append(f"# {group}" if group else "")
            current_group = group
        rel = entry_src
        lines.append(f"- [{entry_title}]({rel})")
        covered.add(entry_src)

    for entry in chapters:
        title = entry.get("title") or first_heading(os.path.join(ROOT, entry["source"]))
        emit(entry["source"], title, entry.get("group"))

    import glob as globmod
    for group in toc.get("group", []):
        pattern = os.path.join(ROOT, group["glob"])
        for match in sorted(globmod.glob(pattern, recursive=True)):
            rel = os.path.relpath(match, ROOT)
            emit(rel, first_heading(match), group["title"])

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

    # 6) Build the book. -d is cwd-relative, so pass absolute paths.
    lang = os.environ.get("BOOK_LANG", "en")
    dest = os.path.join(OUT_DIR, lang)
    build_dir, env = BOOK_DIR, dict(os.environ)
    tools_bin = os.path.join(ROOT, ".tools", "bin")
    if os.path.isdir(tools_bin):
        env["PATH"] = tools_bin + os.pathsep + env.get("PATH", "")
    if lang != "en":
        # A translated build needs the gettext preprocessor (mdbook-i18n-helpers).
        # It runs from a generated book dir so the English build never depends on .tools/.
        if not os.path.exists(os.path.join(tools_bin, "mdbook-gettext")):
            print("BOOK ✗ translated builds need mdbook-gettext -- run: pixi run book-i18n-setup",
                  file=sys.stderr)
            return 1
        build_dir = os.path.join(ROOT, "build", f"book-{lang}")
        os.makedirs(build_dir, exist_ok=True)
        with open(os.path.join(build_dir, "book.toml"), "w", encoding="utf-8") as handle:
            handle.write(
                "[book]\n"
                'title = "VisiaEngine"\n'
                'src = "../book-src"\n'
                f'language = "{lang}"\n\n'
                "[output.html]\n"
                'git-repository-url = "https://gitee.com/chengxuewen/VisiaEngine"\n'
                'edit-url-template = "https://gitee.com/chengxuewen/VisiaEngine/edit/main/{path}"\n'
                'additional-js = ["web/mermaid-init.js"]\n\n'
                "[preprocessor.gettext]\n"
                f'po-dir = "{os.path.join(BOOK_DIR, "po")}"\n'
            )
        # additional-js paths resolve against the book root, so the assets must
        # exist next to the generated book.toml as well.
        shutil.copytree(os.path.join(BOOK_DIR, "web"),
                        os.path.join(build_dir, "web"), dirs_exist_ok=True)
    proc = subprocess.run(
        ["mdbook", "build", build_dir, "-d", dest],
        cwd=ROOT, stdin=subprocess.DEVNULL, env=env,
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

    n = len(covered)
    print(f"BOOK ✓ built {n} chapter(s) -> {os.path.relpath(dest, ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
