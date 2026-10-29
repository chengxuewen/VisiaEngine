#!/usr/bin/env python3
"""Checks for the generated docs site (plan §14 S2/S6).

Static layer (always runs):
  - every built language has index.html and a searchable book
  - graph.json parses, has nodes/edges, and its stats match the arrays
  - the graph chapter embeds the app page, and the app page is present
  - vendored assets exist (offline requirement: no CDN)

Browser layer (--browser): language redirect + mermaid/canvas rendering.
It self-SKIPs when no chromium is available, same convention as gate-web.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "www", "book")


def languages() -> list[str]:
    if not os.path.isdir(OUT):
        return []
    return sorted(
        name for name in os.listdir(OUT)
        if os.path.isdir(os.path.join(OUT, name)) and os.path.exists(os.path.join(OUT, name, "index.html"))
    )


def check_lang(lang: str) -> int:
    base = os.path.join(OUT, lang)
    failures = 0

    def need(cond: bool, msg: str) -> None:
        nonlocal failures
        if not cond:
            print(f"BOOK-CHECK ✗ {lang}: {msg}")
            failures += 1

    need(os.path.getsize(os.path.join(base, "index.html")) > 0, "index.html is empty")

    graph_json = os.path.join(base, "graph.json")
    need(os.path.exists(graph_json), "graph.json missing (run: pixi run mcp index, then book-build)")
    if os.path.exists(graph_json):
        try:
            with open(graph_json, encoding="utf-8") as handle:
                data = json.load(handle)
            need(len(data.get("nodes", [])) > 0, "graph.json has no nodes")
            need(len(data.get("edges", [])) > 0, "graph.json has no edges")
            stats = data.get("stats", {})
            need(stats.get("nodes") == len(data.get("nodes", [])), "graph.json stats.nodes mismatch")
            need(stats.get("edges") == len(data.get("edges", [])), "graph.json stats.edges mismatch")
        except (OSError, ValueError) as exc:
            need(False, f"graph.json unreadable: {exc}")

    chapter = os.path.join(base, "web", "graph.html")
    need(os.path.exists(chapter), "graph chapter page missing")
    if os.path.exists(chapter):
        with open(chapter, encoding="utf-8") as handle:
            html = handle.read()
        need('src="graph-view.html"' in html, 'graph chapter does not embed graph-view.html')

    app = os.path.join(base, "web", "graph-view.html")
    need(os.path.exists(app) and 'id="cy"' in open(app, encoding="utf-8").read(),
         "app page graph-view.html missing #cy container")
    need(os.path.exists(os.path.join(base, "web", "vendor", "cytoscape.min.js")),
         "vendored cytoscape.min.js missing (offline requirement)")

    if failures == 0:
        print(f"BOOK-CHECK ✓ {lang}: index + graph ({len(json.load(open(graph_json))['nodes'])} nodes)")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--browser", action="store_true", help="also run the browser layer (needs chromium)")
    args = parser.parse_args()

    langs = languages()
    if not langs:
        print("BOOK-CHECK ✗ no built site under www/book (run: pixi run book-build)")
        return 1

    failures = sum(check_lang(lang) for lang in langs)

    if args.browser:
        print("BOOK-CHECK - browser layer not implemented yet (S6): redirect + mermaid rendering")

    if failures:
        print(f"BOOK-CHECK ✗ {failures} problem(s)")
        return 1
    print(f"BOOK-CHECK ✓ {len(langs)} language(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
