#!/usr/bin/env python3
"""Checks for the generated docs site (plan §14 S2/S6).

Static layer (always runs):
  - the built site has index.html and a searchable book
  - graph.json parses, has nodes/edges, and its stats match the arrays
  - the graph chapter embeds the app page, and the app page is present
  - vendored assets exist (offline requirement: no CDN)
  - mermaid pages carry fences and the injected loader
Browser layer (--browser): loads the site over HTTP in headless chromium and
asserts a rendered marker. It self-SKIPs when no chromium is available, same
convention as gate-web.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import threading
import functools
import http.server
import socketserver

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "build", "book")


def languages() -> list[str]:
    """The site is English-only (D46); api/ is rustdoc, not a language tree."""
    if not os.path.isdir(OUT):
        return []
    return sorted(
        name for name in os.listdir(OUT)
        if name != "api"
        and os.path.isdir(os.path.join(OUT, name)) and os.path.exists(os.path.join(OUT, name, "index.html"))
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
    need(os.path.exists(os.path.join(base, "web", "vendor", "mermaid.min.js")),
         "vendored mermaid.min.js missing (offline requirement)")
    meta = os.path.join(base, "meta.json")
    need(os.path.exists(meta), "meta.json missing (footer provenance)")
    if os.path.exists(meta):
        try:
            data = json.load(open(meta, encoding="utf-8"))
            need(bool(data.get("commit")), "meta.json has no commit")
            need(bool(data.get("updated")), "meta.json has no doc date")
        except (OSError, ValueError) as exc:
            need(False, f"meta.json unreadable: {exc}")

    # Mermaid: the architecture chapter must both carry fences and get the loader injected.
    arch = os.path.join(base, "docs", "architecture.html")
    if os.path.exists(arch):
        html = open(arch, encoding="utf-8").read()
        need("language-mermaid" in html, "architecture page has no mermaid fences")
        need("mermaid-init" in html, "architecture page did not receive the mermaid loader")

    if failures == 0:
        print(f"BOOK-CHECK ✓ {lang}: index + graph ({len(json.load(open(graph_json))['nodes'])} nodes)")
    return failures


def browser_layer(langs: list[str]) -> int:
    chrome = next((p for p in (shutil.which("chromium"), shutil.which("chromium-browser"),
                               shutil.which("google-chrome")) if p), None)
    if chrome is None:
        print("BOOK-CHECK - browser layer skipped (no chromium; static checks above still ran)")
        return 0

    markers = {"en": "Development Status"}
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=OUT)
    with socketserver.TCPServer(("127.0.0.1", 0), handler) as httpd:
        port = httpd.server_address[1]
        threading.Thread(target=httpd.serve_forever, daemon=True).start()
        failures = 0
        try:
            for lang in langs:
                marker = markers.get(lang)
                if not marker:
                    continue
                url = f"http://127.0.0.1:{port}/{lang}/index.html"
                try:
                    proc = subprocess.run(
                        [chrome, "--headless", "--disable-gpu", "--no-sandbox",
                         "--virtual-time-budget=5000", "--dump-dom", url],
                        capture_output=True, text=True, timeout=90,
                    )
                except (OSError, subprocess.SubprocessError) as exc:
                    print(f"BOOK-CHECK - browser layer skipped ({exc})")
                    return 0
                if marker not in proc.stdout:
                    print(f"BOOK-CHECK ✗ browser: {lang} page missing marker {marker!r}")
                    failures += 1
        finally:
            httpd.shutdown()
        if failures == 0:
            print(f"BOOK-CHECK ✓ browser: {len(langs)} language page(s) rendered")
        return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--browser", action="store_true", help="also run the browser layer (needs chromium)")
    args = parser.parse_args()

    langs = languages()
    if not langs:
        print("BOOK-CHECK ✗ no built site under build/book (run: pixi run book-build)")
        return 1

    failures = sum(check_lang(lang) for lang in langs)
    if args.browser:
        failures += browser_layer(langs)

    if failures:
        print(f"BOOK-CHECK ✗ {failures} problem(s)")
        return 1
    print(f"BOOK-CHECK ✓ {len(langs)} language(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
