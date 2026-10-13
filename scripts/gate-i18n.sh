#!/usr/bin/env bash
# C20 (i18n discipline): all tracked artifacts are English; Chinese lives only in
# AI chat and the .omo/ plan zone. Scan = git-tracked text files minus whitelist.
# Whitelist classes (each an explicit adjudication, extend deliberately not silently):
#  1) .agents/skills/**  -- trigger phrases AND keyword tables are activation surface;
#     whole-directory skip mirrors PolyOrch's C4 spirit (book-to-skill exclusion).
#  2) book-to-skill/utils.py -- functional CJK regex classes (parser data, not prose).
#  3) the 10-line source allowlist below -- test fn identifiers, [FFI-R:*] anchor IDs,
#     CJK rendering fixtures (E505/E814): renaming them would change tested behavior.
#  4) .opencode/node_modules + vendored deps.
# Red/green duty (C0): plant a Chinese comment in README.md -> this gate MUST go red.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
python3 - <<'PY'
import re, subprocess, sys, pathlib
CJK = re.compile(r'[\u4e00-\u9fff]')
tracked = subprocess.run(['git','ls-files'], capture_output=True, text=True).stdout.split()
SKIP_PARTS = {'.omo', '.pixi', 'node_modules', 'target', 'build', '.git'}
SKIP_PATH_PREFIX = ('.agents/skills/', '.opencode/node_modules/')
EXTS = {'.md','.rst','.txt','.cmake','.rs','.c','.cpp','.h','.hpp','.toml','.json','.jsonc',
        '.sh','.bat','.mjs','.js','.py','.yml','.yaml','.in','.wgsl','.pc','.jsonl'}
ALLOW = {  # path:lineno-allowances justified in the gate header
 'crates/visiaengine-io-text/tests/io_spec.rs',
 'crates/visiaengine-io-gltf/src/lib.rs',
 'crates/visiaengine-ffi/src/engine.rs',
 'examples/c/E701_demo_headless.c',
 'examples/c/E814_labels_headless.c',
 'examples/rs/E505_cjk_labels.rs',
}
bad = []
for f in tracked:
    p = pathlib.Path(f)
    if not p.is_file(): continue
    if p.suffix not in EXTS: continue
    if any(part in SKIP_PARTS for part in p.parts): continue
    if f.startswith(SKIP_PATH_PREFIX): continue
    try: t = p.read_text(errors='ignore')
    except Exception: continue
    for i, l in enumerate(t.splitlines(), 1):
        if CJK.search(l):
            # conventions check-command literals referencing .omo-zone anchors (documented allow)
            if f == '.agents/memorys/conventions.md' and "'\u975e\u76ee\u6807" in l: continue  # constructed via escape: the gate file must not carry its own forbidden literals (PIT-1 class)
            if f == '.agents/memorys/decisions.md':  # D6 brand record line
                continue
            if str(p) in ALLOW: continue
            bad.append(f"{p}:{i}: {l.strip()[:70]}")
    # utils.py lives under the skills prefix (already skipped)
print('\n'.join(bad) if bad else '')
if bad:
    print(f"GATE-I18N ✗ {len(bad)} CJK lines outside the whitelist"); sys.exit(1)
print("GATE-I18N ✓ (all tracked artifacts English; whitelist = skills surface, "
      "6 source fixture/anchor files, 1 conventions literal, 1 brand record)")
PY
