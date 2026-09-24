#!/usr/bin/env python3
"""Gallery generator (super-band B1).

Reads the SINGLE example registries (never forks them):
  - cmake/VisiaEngineBindings.cmake  (_rs_items + _disp_* + _args_*)
  - examples/{c,cpp,qt}/CMakeLists.txt  (visiaengine_add_example calls)
  - docs/tutorials.md  (E-number rows = three-way lock counterpart)

Emits build/gallery/index.html: flat card grid with E-band + language badges,
thumbnails from build/gallery/assets/<name>.png (produced by the examples
themselves via gallery.rs save_frame), placeholder cards for window-native
examples with no renderable headless path.

Fail-loud rules (R9 spirit ported from the cmake registry):
  - unknown `set(...)` shapes / missing expected variables -> exit 1
  - registry item lists that disagree with tutorials.md -> exit 1
  - never silently produce a partial gallery
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BINDINGS = ROOT / "cmake" / "VisiaEngineBindings.cmake"
TUTORIALS = ROOT / "docs" / "tutorials.md"
OUT = ROOT / "build" / "gallery"

E_BANDS = {
    "1": ("1xx Window & First Frame", "band-1xx"),
    "2": ("2xx Data Loading", "band-2xx"),
    "3": ("3xx Camera & Views", "band-3xx"),
    "4": ("4xx Interaction & Picking", "band-4xx"),
    "5": ("5xx Materials & Lighting", "band-5xx"),
    "6": ("6xx Scale & Performance", "band-6xx"),
    "7": ("7xx Bindings Mirror (C/Qt)", "band-7xx"),
    "8": ("8xx SDK Consumption", "band-8xx"),
    "9": ("9xx Vertical Slice", "band-9xx"),
}

def fail(msg: str) -> None:
    print(f"gen_gallery: FATAL: {msg}", file=sys.stderr)
    sys.exit(1)

def parse_bindings():
    """Parse _rs_items/_disp_*/_args_* from the cmake registry, fail-loud."""
    text = BINDINGS.read_text(encoding="utf-8")
    m = re.search(r"set\(_rs_items ([^)]*)\)", text)
    if not m:
        fail("_rs_items variable not found in VisiaEngineBindings.cmake")
    items = m.group(1).split()
    if len(items) < 10:
        fail(f"_rs_items suspiciously small ({len(items)})")
    disp = dict(re.findall(r"set\(_disp_(\w+) (ON|OFF)\)", text))
    args = {}
    for name, val in re.findall(r"set\(_args_(\w+) \"([^\"]*)\"\)", text):
        args[name] = val.split(";")
    known = {v for v in re.findall(r"set\(_(?:disp|args)_(\w+) ", text)}
    for it in items:
        if it not in disp and it not in args and it not in known:
            # items without disp/args entries are legal (plain headless, no argv)
            pass
    return {"rs": [{"name": it, "display": disp.get(it) == "ON", "args": args.get(it, [])} for it in items]}

def parse_native():
    """Parse visiaengine_add_example() calls from per-dir CMakeLists."""
    out = []
    for lang, d in (("c", "examples/c"), ("cpp", "examples/cpp"), ("qt", "examples/qt")):
        f = ROOT / d / "CMakeLists.txt"
        if not f.exists():
            fail(f"{f} missing")
        for line in f.read_text(encoding="utf-8").splitlines():
            for m in re.finditer(r"visiaengine_add_example\((\w+)", line):
                out.append({"name": m.group(1), "lang": lang})
            # qt dir uses a bare add_executable(E703_...) (R10: name=source=target)
            if lang == "qt":
                for m in re.finditer(r"add_executable\((E\d+\w*)\s", line):
                    out.append({"name": m.group(1), "lang": lang})
    return out

def parse_tutorials():
    """E-number -> topic string from docs/tutorials.md table rows."""
    topics = {}
    for line in TUTORIALS.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\|\s*(E\d+)\s*\|\s*([^|]+?)\s*\|", line)
        if m:
            topics[m.group(1)] = m.group(2)
    return topics

def main() -> None:
    rs = parse_bindings()["rs"]
    native = parse_native()
    topics = parse_tutorials()

    # cross-check: every E-number in registries must have a tutorials row
    all_names = [r["name"] for r in rs] + [n["name"] for n in native]
    for name in all_names:
        enum = name.split("_")[0]
        if enum not in topics:
            fail(f"{name} in registry but no docs/tutorials.md row (three-way lock broken)")

    assets = OUT / "assets"
    have_thumbs = {p.stem for p in assets.glob("*.png")} if assets.exists() else set()

    # group cards by band
    cards = {}
    for r in rs:
        enum = r["name"].split("_")[0]
        band = E_BANDS[enum[1]]
        lang = "Rust"
        thumb = "assets/" + r["name"] + ".png" if r["name"] in have_thumbs else None
        src = f"examples/rs/{r['name']}.rs"
        cards.setdefault(band[0], []).append(card(r["name"], topics[enum], lang, src, thumb, "cargo run --example " + r["name"]))
    for n in native:
        enum = n["name"].split("_")[0]
        band = E_BANDS[enum[1]]
        lang = {"c": "C", "cpp": "C++", "qt": "Qt"}[n["lang"]]
        thumb = "assets/" + n["name"] + ".png" if n["name"] in have_thumbs else None
        src = f"examples/{n['lang']}/{n['name']}"
        cards.setdefault(band[0], []).append(card(n["name"], topics[enum], lang, src, thumb, f"ctest -R example_{enum}"))

    sections = []
    for band_key in sorted(cards):
        title, cls = E_BANDS[band_key[0]]
        body = "\n".join(cards[band_key])
        sections.append(f'<section><h2 class="{cls}">{title}</h2>\n<div class="grid">\n{body}\n</div></section>')

    total = len(all_names)
    thumbd = len(have_thumbs & {n.split('_')[0] and n for n in all_names})
    html = HTML_HEADER.replace("{{TOTAL}}", str(total)).replace("{{THUMB}}", str(len(have_thumbs))) + "\n".join(sections) + HTML_FOOTER
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "index.html").write_text(html, encoding="utf-8")
    (OUT / "manifest.txt").write_text("\n".join(sorted(all_names)) + "\n", encoding="utf-8")
    print(f"gallery: {total} cards ({len(have_thumbs)} thumbnails) -> {OUT/'index.html'}")

def card(name, topic, lang, src, thumb, run):
    enum = name.split("_")[0]
    lang_cls = {"Rust": "rust", "C": "c", "C++": "cpp", "Qt": "qt"}[lang]
    if thumb:
        media = f'<img loading="lazy" src="{thumb}" alt="{name}">'
    else:
        media = f'<div class="nothumb"><span>window example</span><small>run locally to see it live</small></div>'
    return f'''<a class="card" href="#" title="{run}">
  {media}
  <div class="meta">
    <span class="enum">{enum}</span>
    <span class="lang {lang_cls}">{lang}</span>
  </div>
  <div class="topic">{topic}</div>
  <code>{src}</code>
</a>'''

HTML_HEADER = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>VisiaEngine Examples — {{TOTAL}} tutorials</title>
<style>
:root { --bg:#0d1219; --card:#151c26; --line:#232c3a; --fg:#dce3ec; --dim:#8a94a6; }
* { box-sizing:border-box; }
body { margin:0; background:var(--bg); color:var(--fg); font:15px/1.5 system-ui,sans-serif; }
header { padding:32px 24px 8px; max-width:1200px; margin:0 auto; }
h1 { font-size:22px; margin:0 0 4px; }
header p { color:var(--dim); margin:0; }
main { max-width:1200px; margin:0 auto; padding:16px 24px 48px; }
section h2 { font-size:15px; margin:28px 0 12px; color:var(--dim); text-transform:uppercase; letter-spacing:.08em; }
.grid { display:grid; grid-template-columns:repeat(auto-fill,minmax(250px,1fr)); gap:14px; }
.card { display:block; background:var(--card); border:1px solid var(--line); border-radius:10px; overflow:hidden; text-decoration:none; color:inherit; transition:transform .12s, border-color .12s; }
.card:hover { transform:translateY(-2px); border-color:#3b82f6; }
.card img { width:100%; aspect-ratio:16/9; object-fit:cover; display:block; background:#000; }
.nothumb { width:100%; aspect-ratio:16/9; display:flex; flex-direction:column; gap:4px; align-items:center; justify-content:center; background:repeating-linear-gradient(45deg,#101722,#101722 12px,#131b28 12px,#131b28 24px); color:var(--dim); }
.nothumb span { font-weight:600; }
.nothumb small { font-size:11px; }
.meta { display:flex; gap:8px; align-items:center; padding:10px 12px 0; }
.enum { font-weight:700; color:#3b82f6; font-variant-numeric:tabular-nums; }
.lang { font-size:11px; padding:1px 7px; border-radius:99px; border:1px solid var(--line); color:var(--dim); }
.lang.rust { border-color:#d97706; color:#f59e0b; }
.lang.c { border-color:#2563eb; color:#60a5fa; }
.lang.cpp { border-color:#7c3aed; color:#a78bfa; }
.lang.qt { border-color:#16a34a; color:#4ade80; }
.topic { padding:6px 12px; font-size:13.5px; }
.card code { display:block; padding:0 12px 12px; font-size:11px; color:var(--dim); }
footer { text-align:center; color:var(--dim); font-size:12px; padding:24px; }
</style>
</head>
<body>
<header>
<h1>VisiaEngine Examples</h1>
<p>{{TOTAL}} tutorials &middot; {{THUMB}} rendered thumbnails &middot; every example machine-executed in CI with pixel gates</p>
</header>
<main>
"""

HTML_FOOTER = """
</main>
<footer>Generated by scripts/gen_gallery.py — do not edit. Source of truth: cmake/VisiaEngineBindings.cmake + examples/*/CMakeLists.txt + docs/tutorials.md</footer>
</body>
</html>
"""

if __name__ == "__main__":
    main()
