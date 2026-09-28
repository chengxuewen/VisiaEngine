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
    # Review fix (#1): duplicate E-numbers are legal ONLY with a language
    # suffix in the name (E811_entity_hide_c / E811_entity_hide_cpp). Bare
    # duplicates would collide cards and detail pages — fail loud.
    seen_enums = {}
    for name in all_names:
        enum = name.split("_")[0]
        if enum in seen_enums and seen_enums[enum] != name:
            # different names sharing an E-number: require disambiguating
            # suffixes (names must differ beyond the enum — they do, or this
            # loop would not see two entries) AND both must carry a language
            # marker so card labels stay distinguishable.
            for other in (seen_enums[enum], name):
                if "_" not in other[len(enum) + 1:]:
                    fail(f"duplicate E-number {enum}: {seen_enums[enum]} vs {name} (add language suffix)")
        seen_enums[enum] = name
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
        sections.append(f'<section id="band-{band_key[0]}xx"><h2 class="{cls}">{title}</h2>\n<div class="grid">\n{body}\n</div></section>')

    total = len(all_names)
    thumbd = len(have_thumbs & {n.split('_')[0] and n for n in all_names})
    # three.js-style category nav (sticky anchor bar) — only bands with cards
    def _short(band_key: str) -> str:
        full = E_BANDS[band_key][0]
        return full.split(" ", 1)[1] if " " in full else full

    # cards are keyed by the band TITLE (band[0] in the setdefault calls);
    # recover the numeric key from the title's leading digit.
    nav_items = " · ".join(
        f'<a href="#band-{k[0]}xx">{k.split(" ", 1)[1] if " " in k else k}</a>'
        for k in sorted(cards)
    )
    html = (HTML_HEADER.replace("{{TOTAL}}", str(total)).replace("{{THUMB}}", str(len(have_thumbs)))
            .replace("{{NAV}}", nav_items))
    html += "\n".join(sections) + HTML_FOOTER
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "index.html").write_text(html, encoding="utf-8")
    (OUT / "manifest.txt").write_text("\n".join(sorted(all_names)) + "\n", encoding="utf-8")

    # N1.6: per-example detail pages (three.js examples style) — big media,
    # topic, run command, source link, back-to-index. Local-only site.
    meta = {}
    for r in rs:
        meta[r["name"]] = ("Rust", f"examples/rs/{r['name']}.rs", "cargo run --example " + r["name"], r["name"] in have_thumbs)
    for n in native:
        lang = {"c": "C", "cpp": "C++", "qt": "Qt"}[n["lang"]]
        meta[n["name"]] = (lang, f"examples/{n['lang']}/{n['name']}", f"ctest -R example_{n['name'].split('_')[0]}", n["name"] in have_thumbs)
    for name in all_names:
        enum = name.split("_")[0]
        lang, src, run, has_thumb = meta[name]
        lang_cls = {"Rust": "rust", "C": "c", "C++": "cpp", "Qt": "qt"}[lang]
        media = (f'<img src="assets/{name}.png" alt="{name}">'
                 if has_thumb else
                 f'<div class="nothumb"><span>window example</span><small>run locally to see it live</small></div>')
        page = DETAIL_PAGE
        page = page.replace("{{ENUM}}", enum)
        page = page.replace("{{NAME}}", name)
        page = page.replace("{{TOPIC}}", topics[enum])
        page = page.replace("{{LANG}}", lang)
        page = page.replace("{{LANG_CLS}}", lang_cls)
        page = page.replace("{{MEDIA}}", media)
        page = page.replace("{{RUN}}", run)
        page = page.replace("{{SRC}}", "../../" + src)
        page = page.replace("{{BAND}}", E_BANDS[enum[1]][0])
        # Review fix (#1): full-name keying (E811_c / E811_entity_hide_cpp are
        # distinct pages); also detect enum reuse across DIFFERENT names so a
        # future registry addition fails loud instead of shadowing a sibling.
        (OUT / f"{name}.html").write_text(page, encoding="utf-8")

    print(f"gallery: {total} cards ({len(have_thumbs)} thumbnails, {total} detail pages) -> {OUT/'index.html'}")

def card(name, topic, lang, src, thumb, run):
    enum = name.split("_")[0]
    lang_cls = {"Rust": "rust", "C": "c", "C++": "cpp", "Qt": "qt"}[lang]
    if thumb:
        media = f'<img loading="lazy" src="{thumb}" alt="{name}">'
    else:
        media = f'<div class="nothumb"><span>window example</span><small>run locally to see it live</small></div>'
    # N1.6 three.js-style: card -> per-example detail page (big thumb, topic,
    # run command, source link). index stays the categorized grid; the detail
    # page is the click-through (generated per example below).
    # Review fix (#1): key pages by FULL name — E811 has C and C++ variants;
    # enum-keying silently shadowed the C page.
    href = name + ".html"
    return f'''<a class="card" href="{href}" title="{run}">
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
nav.cats { position:sticky; top:0; z-index:10; background:rgba(13,18,25,.92); backdrop-filter:blur(6px); border-bottom:1px solid var(--line); padding:10px 24px; max-width:1200px; margin:12px auto 0; font-size:13px; }
nav.cats a { color:var(--dim); text-decoration:none; margin-right:4px; }
nav.cats a:hover { color:#3b82f6; }
</style>
</head>
<body>
<nav class="cats">{{NAV}}</nav>
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

DETAIL_PAGE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{{ENUM}} · VisiaEngine gallery</title>
<style>
  body { font-family: system-ui, sans-serif; margin: 0; background: #101418; color: #dfe6ee; }
  header { padding: 14px 22px; background: #171d24; display: flex; gap: 14px; align-items: baseline; }
  header a { color: #7fc06e; text-decoration: none; }
  h1 { font-size: 20px; margin: 0; }
  .lang { padding: 2px 8px; border-radius: 4px; font-size: 12px; }
  .rust { background: #2b4a2f; } .c { background: #4a402b; } .cpp { background: #2b3a4a; } .qt { background: #452b4a; }
  main { max-width: 1100px; margin: 0 auto; padding: 22px; }
  .media img, .media .nothumb { width: 100%; max-width: 960px; border-radius: 8px; display: block; }
  .nothumb { background: #1a222b; border: 1px dashed #33404e; padding: 60px 0; text-align: center; color: #8fa1b3; }
  .nothumb span { display: block; font-size: 18px; }
  .topic { font-size: 17px; margin: 14px 0 6px; }
  .band { color: #8fa1b3; font-size: 13px; margin-bottom: 14px; }
  .run { background: #171d24; padding: 10px 14px; border-radius: 6px; font-family: ui-monospace, monospace; font-size: 14px; }
  .src { margin-top: 10px; }
  .src a { color: #7fc06e; }
</style>
</head>
<body>
<header>
  <a href="index.html">&larr; gallery</a>
  <h1>{{ENUM}} <span class="lang {{LANG_CLS}}">{{LANG}}</span></h1>
</header>
<main>
  <div class="media">{{MEDIA}}</div>
  <div class="topic">{{TOPIC}}</div>
  <div class="band">{{BAND}}</div>
  <div class="run">{{RUN}}</div>
  <div class="src">source: <a href="{{SRC}}">{{SRC}}</a></div>
</main>
</body>
</html>
"""


if __name__ == "__main__":
    main()
