#!/usr/bin/env python3
"""Serve the built docs site on the LAN and rebuild it when sources change (S8).

Foreground by design (plan §12-3): Ctrl-C stops it, no PID file, no supervisor.
`docs/`, `book/` and the codegraph database are polled; a change triggers a full
rebuild of every built language (mdbook build is a couple of seconds here).

Why not `mdbook serve`: it owns its own --dest-dir and would wipe the sibling
language trees, the landing page and api/ that live next to them.
"""

from __future__ import annotations

import functools
import http.server
import os
import socket
import subprocess
import sys
import threading
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "build", "book")
POLL_SECONDS = 3.0


def built_langs() -> list[str]:
    if not os.path.isdir(OUT):
        return []
    return sorted(
        name for name in os.listdir(OUT)
        if os.path.isdir(os.path.join(OUT, name))
        and os.path.exists(os.path.join(OUT, name, "index.html"))
    )


def signature() -> tuple:
    """Cheap change detector: mtimes of the doc/book trees plus the graph database."""
    parts = []
    for base in (os.path.join(ROOT, "docs"), os.path.join(ROOT, "book")):
        for walk_root, _dirs, files in os.walk(base):
            for name in sorted(files):
                path = os.path.join(walk_root, name)
                try:
                    parts.append((path, os.stat(path).st_mtime_ns))
                except OSError:
                    pass
    db = os.path.join(ROOT, ".codegraph", "codegraph.db")
    if os.path.exists(db):
        parts.append((db, os.stat(db).st_mtime_ns))
    return tuple(parts)


def rebuild() -> None:
    started = time.time()
    proc = subprocess.run(
        [sys.executable, os.path.join("scripts", "book-build.py")],
        cwd=ROOT, stdin=subprocess.DEVNULL,
        capture_output=True, text=True,
    )
    if proc.returncode != 0:
        tail = (proc.stderr or proc.stdout).strip().splitlines()
        print(f"[book-serve] rebuild failed: {tail[-1] if tail else 'unknown'}")
    else:
        print(f"[book-serve] rebuilt in {time.time() - started:.1f}s")
    # graph.json / meta.json were rewritten inside the source tree; the English build
    # already copied them, the loop above covered every configured language.


def watch(stop: threading.Event) -> None:
    seen = signature()
    while not stop.wait(POLL_SECONDS):
        current = signature()
        if current != seen:
            seen = current
            print("[book-serve] change detected -> rebuilding")
            rebuild()
            seen = signature()   # a rebuild writes build/ but not docs/book; re-arm anyway


def lan_ip() -> str:
    try:
        with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as probe:
            probe.connect(("8.8.8.8", 80))
            return probe.getsockname()[0]
    except OSError:
        return "127.0.0.1"


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *_args):  # keep the terminal readable while rebuilding
        pass


def main() -> int:
    if not built_langs():
        print("book-serve: nothing built yet -- run: pixi run book-build")
        return 1

    host = os.environ.get("BOOK_HOST", "0.0.0.0")
    port = int(os.environ.get("BOOK_PORT", "3000"))
    handler = functools.partial(QuietHandler, directory=OUT)
    with http.server.ThreadingHTTPServer((host, port), handler) as httpd:
        print(f"book-serve: http://127.0.0.1:{port}/   (LAN: http://{lan_ip()}:{port}/)")
        print(f"book-serve: serving {os.path.relpath(OUT, ROOT)}; "
              f"watching docs/ + book/ every {POLL_SECONDS:.0f}s; Ctrl-C to stop")
        proxy = os.environ.get("http_proxy") or os.environ.get("HTTP_PROXY")
        bypass = os.environ.get("no_proxy") or os.environ.get("NO_PROXY") or ""
        if proxy and "127.0.0.1" not in bypass:
            print(f"book-serve: WARNING http_proxy={proxy} is set without 127.0.0.1 in "
                  f"no_proxy -- browsers/curl may fetch the proxy page instead. "
                  f"Add no_proxy=127.0.0.1,localhost")

        stop = threading.Event()
        threading.Thread(target=watch, args=(stop,), daemon=True).start()
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\nbook-serve: stopped")
        finally:
            stop.set()
    return 0


if __name__ == "__main__":
    sys.exit(main())
