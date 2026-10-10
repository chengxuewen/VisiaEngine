# C/C++/Qt Example Pixel-Witness Census (2026-10-09)

Read-only census of the native-binding examples. One artifact: this file. No example,
script, cmake, config, or memory file was touched (task card 4 = A: the table landing
is the end; the follow-up band owns all implementation decisions).

## Witness predicate (verbatim — the only predicate used)

> "the file contains at least one numeric pixel-family comparison whose measured value
> is asserted against a bound (a range or an equality), in a code path that runs
> non-interactively"

Application method: every tracked file read end to end; the bound column cites exact
`file:line` for each qualifying comparison. Whole-frame `memcmp(...) == 0` byte-equal
canaries are frame equalities, not family comparisons — recorded on their rows but NOT
counted toward the bound totals. Non-pixel numeric checks (entity counts, return codes,
attribute values, progress monotonicity) never satisfy the predicate.

## Tracked set

Source files only. The three `CMakeLists.txt` files under these directories are build
files, NOT tracked files of this census (tracked-set rule of the task; they match no
`.c`/`.cpp` extension — see the exclusion command in Totals).

- `examples/c` — 12 files
- `examples/cpp` — 4 files
- `examples/qt` — 1 file
- total — 17 (commands in Totals below)

## Census table (one row per tracked file, 17 rows)

| File | Class | Numeric pixel bound(s) — file:line, exact form, what it guards | Families asserted | Bound(s) run without a display? |
|---|---|---|---|---|
| `examples/c/E701_demo_headless.c` | WITNESS | `:26` `lum <= 40 → FAIL` — lum = R+G+B of one centre pixel from readback (`:23`–`:25`); guards "centre pixel is lit geometry, not background" (asserts lum ≥ 41) | bright (centre-pixel luminance; not a family count) | yes — `visiaengine_create_headless` (`:13`), straight-line `main`, headless ctest registration; note: second readback `:39` exists but feeds no comparison; `:53` opacity range check is numeric, non-pixel |
| `examples/c/E702_demo_x11.c` | NO-witness | none — zero `readback` occurrences; renders N frames into an X11 window (`:57`–`:58`), Expose repaint only | — | n/a (no pixel assert); registry `display` label + `--frames;3` (examples/c/CMakeLists.txt:26) |
| `examples/c/E704_host_callback.c` | NO-witness | none — zero `readback`; asserts are event counters (`:41` progress ≥ 5 + terminal equality; `:52` error==1) — numeric, non-pixel; `:20`/`:25` prints FAIL without capturing rc (rc-blind, recorded) | — | n/a; headless registration (examples/c/CMakeLists.txt:16) |
| `examples/c/E801_sdl_window.c` | NO-witness | none — zero `readback`; render return codes only | — | n/a; registry `display` label + `--frames;3` (examples/c/CMakeLists.txt:20) |
| `examples/c/E810_attr_survey.c` | NO-witness | none — zero readback, zero render; `:40` guards names==opac==n (attribute counts, not pixels) | — | n/a (headless registration, no display needed) |
| `examples/c/E811_entity_hide_c.c` | NO-witness | none — `:25` `visiaengine_entity_count(ve) != 2 → FAIL`: a numeric equality against a bound, but the measure is the entity domain, not pixels (zero readback) | — | n/a (headless registration) |
| `examples/c/E813_section_clip.c` | WITNESS | `:64` `r0 < 3000 → FAIL` red baseline present (count ≥ 3000); `:73` `count_fam(img, 1) != 0 → FAIL` clipped side exactly 0 red; `:74` `count_fam(img, 0) < 3000 → FAIL` kept side green ≥ 3000; `:85` `g2 < 800 or g2 > 4000 → FAIL` corner green ∈ [800,4000] (classifier `:44`: red r>120∧g<80∧b<80 / green g>120∧r<100∧b<100; band record: base red=10816, corner green=1764) | red, green | yes — headless (`:50`), straight-line `main`, headless ctest registration |
| `examples/c/E814_labels_headless.c` | WITNESS | `:48` `white < 60 → fail` — white-family pixel count floor 60 (classifier `:46` r>120∧g>120∧b>120; band record: measured white=3917) | white | yes, with a caveat — headless (`:18`), straight-line `main`; BUT `fail()` rc is discarded at `:48` (missing `return`): FAIL prints yet the process still exits 0, so this bound is exit-code-blind |
| `examples/c/E815_fly_camera.c` | NO-witness | none — zero `readback`; fly domain rejection (`:30`–`:38`), progress/arrival loop (`:44`–`:57`), equality checks on return codes — numeric, non-pixel | — | n/a (headless registration) |
| `examples/c/E816_minimap_nav.c` | WITNESS | `:96` `gcorner < 900 → FAIL` corner top-view green floor 900 (in-file comment: measured 1596); `:97` `rcorner < 300 or rcorner > 1200 → FAIL` corner red ∈ [300,1200] (measured 756); `:98` `rmain < 3000 → FAIL` main-region red floor 3000 (classifiers `:52`–`:53`, `:59`) | green (corner), red (corner), red (main) | yes — headless (`:64`), straight-line `main`; byte-equal canaries `:86` and `:141` (`memcmp(img, ref, …) == 0`) are excluded from the bound count (whole-frame equality, not family) |
| `examples/c/E817_two_layer_map.c` | WITNESS | 9 census-bound statements at `:86` (empty frame: corner == 172800 ∧ road/poi/basemap == 0), `:94` (basemap ≥ 1000), `:95` (road/poi == 0 exclusivity), `:106` (road ≥ 5000 ∧ poi ≥ 500), `:107` (basemap == 0 exclusivity), `:117` (road ≥ 5800 ∧ poi ≥ 780), `:118` (basemap ≥ 15600), `:133` (road ≥ 3200 ∧ poi ≥ 400), `:134` (basemap ≥ 36000); in-file comments: measured 2026-10-08 road=9782 poi=1314 basemap=26030 / road=5343 poi=684 basemap=60322, thresholds = baseline −40% (classifiers `:36`–`:38`, census `:50`–`:60`) | basemap (exclusion bucket), road, poi, corner (water is counted in the census but never bound) | yes — headless 480×360 (`:76`), straight-line `main`; re-mount byte-equality `:140`/`:144` excluded (frame equality); mount-count equalities `:69` (== 4) / `:72` (== 9) are numeric, non-pixel |
| `examples/cpp/E802_offscreen.cpp` | WITNESS | `:34` `lum <= 40 → fail` — centre-pixel R+G+B (`:32`–`:33`); guards lit centre (asserts lum ≥ 41; same form as E701) | bright (centre-pixel luminance) | yes — headless (`:21`), straight-line `main`, headless ctest registration; `:29` VE_ERR_SIZE probe is protocol, non-pixel |
| `examples/cpp/E811_entity_hide_cpp.cpp` | NO-witness | none — `:22` `entity_count() != 2 → return 1`: numeric, non-pixel (zero readback) | — | n/a (headless registration) |
| `examples/cpp/E812_mesh_add.cpp` | NO-witness | none — `:40`/`:54`/`:59` `entity_count()` equalities (2/2/0), return-code ledger checks; zero readback → no pixel measure exists | — | n/a (headless registration) |
| `examples/cpp/template.cpp` | NO-witness | none — calls render (`:14`) and readback (`:16`) but compares no number against the buffer; the `lum>40` at `:13` is a comment pointing at E802, not code | — | n/a (skeleton; not ctest-registered) |
| `examples/c/template.c` | NO-witness | none — skeleton that CALLS render (`:33`) yet zero readback, no numeric pixel use | — | n/a (skeleton; not ctest-registered) |
| `examples/qt/E703_qt_viewer.cpp` | NO-witness | none — zero readback; painting happens through the Qt widget pump (`w.start(frames)`, `:34`); file prints frames/rc (`:40`) | — | n/a; registered `example_E703_qt_viewer --frames 6` with `LABELS example;native;display` + `SKIP_RETURN_CODE 77` (examples/qt/CMakeLists.txt:15–20) — frame lane runs only on the display sub-state track |

## Falsifiability of the predicate (three decisive reads)

The predicate must separate "number bound" from "pixel bound". Both directions, quoted:

- Bound without pixels → classifies as no-witness.
  `examples/c/E811_entity_hide_c.c:25`:
  `if (visiaengine_entity_count(ve) != 2) { puts("E811 FAIL <CJK message>"); return 1; }`
  A numeric equality against a bound (2) exists, but the measure is an entity count from
  a query port; the file never reads back pixels (readback occurrences = 0).
- Pixels without a bound → classifies as no-witness.
  `examples/cpp/template.cpp:16`:
  `if (eng.readback(buf.data(), buf.size()) != VE_OK) { puts("readback"); return 1; }`
  Pixel data is fetched, but no number is ever asserted against the buffer contents.
- Pixel family bound in a straight-line headless `main` → classifies as witness.
  `examples/c/E817_two_layer_map.c:94`:
  `if (c.basemap < 1000) return fail("basemap family missing");`
  `c.basemap` is a per-pixel-bucket count (census `:50`–`:60`), floored at 1000, executed
  unconditionally in `main(void)` with `visiaengine_create_headless` (`:76`).

## Totals (every number with its producing command)

Tracked set:

```bash
$ find examples/c examples/cpp examples/qt -type f \( -name "*.c" -o -name "*.cpp" \) | wc -l
17
$ find examples/c -type f \( -name "*.c" -o -name "*.cpp" \) | wc -l   # → 12
$ find examples/cpp -type f \( -name "*.c" -o -name "*.cpp" \) | wc -l # → 4
$ find examples/qt -type f \( -name "*.c" -o -name "*.cpp" \) | wc -l  # → 1
$ find examples/c examples/cpp examples/qt -type f ! \( -name "*.c" -o -name "*.cpp" \) | sort
examples/c/CMakeLists.txt
examples/cpp/CMakeLists.txt
examples/qt/CMakeLists.txt
```

Class split (row identity: the two class counts sum to the tracked count):

```bash
$ grep -c '| WIT[N]ESS |' docs/reference/c-example-witness-census-2026-10-09.md
6
$ grep -c '| N[O]-witness |' docs/reference/c-example-witness-census-2026-10-09.md
11
$ grep -c 'WIT[N]ESS\|N[O]-witness' docs/reference/c-example-witness-census-2026-10-09.md
17
```

The brackets are a deliberate self-exclusion device: the exact tokens `WIT`+`ESS`
(upper) plus its negated form appear ONLY in the 17 table rows, so a plain grep of the two
class tokens over this file counts rows; the commands above therefore do not match
their own lines.

Numeric pixel-family bound statements (6 witness files):

```bash
$ grep -c "lum <= 40" examples/c/E701_demo_headless.c                       # → 1
$ grep -cE "r0 < 3000|count_fam\(img, 1\) != 0|count_fam\(img, 0\) < 3000|g2 < 800 \|\| g2 > 4000" examples/c/E813_section_clip.c  # → 4
$ grep -c "white < 60" examples/c/E814_labels_headless.c                    # → 1
$ grep -cE "gcorner < 900|rcorner < 300 \|\| rcorner > 1200|rmain < 3000" examples/c/E816_minimap_nav.c  # → 3
$ grep -c "if (c\." examples/c/E817_two_layer_map.c                         # → 9   (all nine `if (c.<family> …)` bound statements in main)
$ grep -c "lum <= 40" examples/cpp/E802_offscreen.cpp                       # → 1
```

Sum = 1 + 4 + 1 + 3 + 9 + 1 = **19** bound statements.

Structural observation (not a second predicate): on this binding face pixel data is
reachable only through `readback`, so a file with zero occurrences cannot contain a
pixel comparison at all:

```bash
$ for f in $(find examples/c examples/cpp examples/qt -type f \( -name "*.c" -o -name "*.cpp" \) | sort); do printf "%-42s %s\n" "$f" "$(grep -c readback $f)"; done
examples/c/E701_demo_headless.c            2
examples/c/E702_demo_x11.c                 0
examples/c/E704_host_callback.c            0
examples/c/E801_sdl_window.c               0
examples/c/E810_attr_survey.c              0
examples/c/E811_entity_hide_c.c            0
examples/c/E813_section_clip.c             4
examples/c/E814_labels_headless.c          2
examples/c/E815_fly_camera.c               0
examples/c/E816_minimap_nav.c              6
examples/c/E817_two_layer_map.c            1
examples/c/template.c                      0
examples/cpp/E802_offscreen.cpp            3
examples/cpp/E811_entity_hide_cpp.cpp      0
examples/cpp/E812_mesh_add.cpp             0
examples/cpp/template.cpp                  2
examples/qt/E703_qt_viewer.cpp             0
```

7 of 17 files read back pixels: the 6 witnesses plus one skeleton
(`examples/cpp/template.cpp`, readback=2, asserts no bound) -- readback is necessary for a
witness on this surface, not sufficient.

## Rust-side precedents (the model a later band would follow)

- `crates/visiaengine-render-wgpu/tests/gltf_scene.rs:133`–`:134` —
  `assert!(red > 500)` / `assert!(green > 1200)`; in-file comment `:132` records
  baseline red=884 green=1931, thresholds = baseline − 40%.
- `crates/visiaengine-render-wgpu/tests/geo_pipeline.rs:230` — `fn golden_geo_window_fit()`.
- `crates/visiaengine-render-wgpu/tests/pick_highlight.rs:222` — `fn golden_pick_window_mirror()`.

Pattern shared by all: headless render → colour-family count → bound at measured −40%,
with the probe-measured value recorded in a comment.

## What this implies (record only — no implementation proposed here)

- 11 of 17 tracked files carry no pixel bound (command above: 17 − 6 = 11 rows).
- Of those 11, 8 have a visible output claim that no machine check witnesses today:
  E702, E704, E801, E815, E812, template.c, template.cpp call the render entry point
  (1/1/3/2/1/1/1 occurrences per the render-census loop below) and E703 presents through
  the Qt widget pump (`w.start(frames)`) on the display-only lane. A regression that blanks
  or distorts their frames would stay green.
- 3 files (E810, E811 C, E811 cpp) never touch pixels and make no visual claim —
  absence looks correct there, not a gap.
- 2 skeleton files, already counted in the 8 above; whether a bound belongs in the
  teaching template is a design question for the later band.
- E814 has a bound but the `fail()` return is discarded at `:48`, so it cannot redden
  ctest — one row in this census is exit-code-blind even though it holds a witness-class row.
- The two rc-blind event checks E704 `:20`/`:25` are recorded on the row above under
  the same honest-heading (they are non-pixel anyway).

```bash
# render-occurrence loop behind the implication counts
$ for f in examples/c/E702_demo_x11.c examples/c/E704_host_callback.c examples/c/E801_sdl_window.c \
    examples/c/E810_attr_survey.c examples/c/E811_entity_hide_c.c examples/c/E815_fly_camera.c \
    examples/c/template.c examples/cpp/E811_entity_hide_cpp.cpp examples/cpp/E812_mesh_add.cpp \
    examples/cpp/template.cpp examples/qt/E703_qt_viewer.cpp; do \
    printf "%-40s render=%s\n" "$f" "$(grep -cE 'visiaengine_render|\.render\(' $f)"; done
# (also measured earlier with the vis= column: outputs in-conversation on 2026-10-09)
```

Decision ownership: per the task's card 4 = A, the landing of this table ends this
task; which of the 6 candidate files get witnesses, and how, belongs to a later band.
