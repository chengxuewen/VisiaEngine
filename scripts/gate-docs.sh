#!/usr/bin/env bash
# gate-docs (batch 5 T3 / [E3D:D1·D3]; B5 grew it to six checks) — zero documentation drift:
#  ① E three-way agreement: example filename = header-comment E### = docs/tutorials.md (reserved gaps = index-line only);
#  ② README clause count == spec-trace live report (docs only write verifiable numbers);
#  ③ visiaengine.h prototype name set == Rust extern "C" fn name set (symbol count is guarded by gate-abi's nm);
#  ④ relative links in README/docs/llms.txt resolve (http/mailto/anchor/build exempt).
# Pure grep/diff <1s; mismatch = red + exit 1.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
fail=0
EXDIRS=(examples/rs examples/c examples/cpp examples/qt)   # S1: rs joins the table, the two emptied legacy dirs are gone; the capi row exits at S2

# ① E three-way
files=$(for d in "${EXDIRS[@]}"; do ls "$d" 2>/dev/null; done | grep -oE '^E[0-9]{3}' | sort -u)
headers=$(for d in "${EXDIRS[@]}"; do for x in "$d"/*.rs "$d"/*.c "$d"/*.cpp; do [ -f "$x" ] && head -1 "$x" | grep -oE '^(/\*+|//+) ?!? ?E[0-9]{3}' | grep -oE 'E[0-9]{3}'; done; done 2>/dev/null | sort -u)
real_files=$(for d in "${EXDIRS[@]}"; do ls "$d" 2>/dev/null; done | grep -E '^E[0-9]{3}_' | sort -u)
idx_files=$(grep -oE 'E[0-9]{3}_[A-Za-z0-9_]+\.(rs|cpp|c)' docs/tutorials.md | sort -u)
index=$(grep -oE '^\| E[0-9]{3}' docs/tutorials.md | grep -oE 'E[0-9]{3}' | sort -u)
[ "$files" = "$headers" ] || { echo "GATE-DOCS ✗ ①filename set ≠ header set:"; diff <(echo "$files") <(echo "$headers"); fail=1; }
[ "$idx_files" = "$real_files" ] || { echo "GATE-DOCS ✗ ①index path set ≠ disk reality:"; diff <(echo "$idx_files") <(echo "$real_files"); fail=1; }
for e in $index; do
    grep -q "^${e}" <<< "$files" \
        || { echo "GATE-DOCS ✗ ①index $e has no file on disk"; fail=1; }
done

# ② README clause count
rn=$(grep -oE '\*\*[0-9]+\*\* SDD' README.md | grep -oE '[0-9]+' | head -1 || true)
sn=$(bash scripts/spec-trace.sh 2>/dev/null | grep -oE '[0-9]+ pairs aligned' | grep -oE '^[0-9]+' || true)
[ "${rn:-x}" = "${sn:-y}" ] || { echo "GATE-DOCS ✗ ②README '${rn:-missing}' ≠ spec-trace '${sn:-missing}'"; fail=1; }

# ③ header signature name set
hs=$(grep -oE 'visiaengine_[a-z0-9_]+\(' bindings/c/include/visiaengine.h | sed 's/(//' | sort -u)
rs=$(grep -rhoE 'fn visiaengine_[a-z0-9_]+' crates/visiaengine-ffi/src/ | sed 's/^fn //' | sort -u)
[ "$hs" = "$rs" ] || { echo "GATE-DOCS ✗ ③.h prototype set ≠ Rust extern set:"; diff <(echo "$hs") <(echo "$rs"); fail=1; }

# ④ relative links — extended this band from .md-only to every markdown link
# destination in README.md + docs/**/*.md + llms.txt. Skips: http(s)/mailto, pure-#
# anchors, destinations containing spaces (prose artifacts, not valid targets), and
# build//target/ paths (generated trees absent in a bare clone — the gallery/
# docs-gen bands own those outputs).
while IFS= read -r f; do
    dir=$(dirname "$f")
    while IFS= read -r dest; do
        case "$dest" in
            http*|mailto:*|\#*|*\ *|build/*|target/*) continue ;;
        esac
        target=${dest%%#*}
        target=${target%%\?*}
        [ -n "$target" ] || continue
        [ -e "$dir/$target" ] || { echo "GATE-DOCS ✗ ④broken link: $f → $dest"; fail=1; }
    done < <(grep -oE '\]\([^)]+\)' "$f" 2>/dev/null | sed -E 's/^\]\(//; s/\)$//')
done < <(ls README.md docs/*.md docs/sdd/*.md docs/reference/*.md llms.txt 2>/dev/null)

# ⑤ root-facade purity (T-A 2026-10-10 ruling 1-B+: assembly-layer positioning) — soft line
# cap 120 + exemption marker + structural hard locks (compile rules banned from the root;
# polyorch_* assembly calls and umbrella ALIAS whitelisted). The original 60-line lock was
# relaxed during the PolyOrch full refactor: line count is a proxy metric, not the disease;
# the disease = unreadable root file / compile rules in the root — those two stay hard.
if [ -f CMakeLists.txt ]; then
    rl=$(wc -l < CMakeLists.txt)
    exempt=$(grep -q '# assembly-growth' CMakeLists.txt && echo 1 || echo 0)
    if [ "$rl" -gt 120 ]; then
        if [ "$exempt" = 1 ]; then
            echo "GATE-DOCS ⑤root CMakeLists at ${rl} lines exceeds 120 (assembly-growth exemption on record, the manual promise gave its reason)"
        else
            echo "GATE-DOCS ✗ ⑤root CMakeLists at ${rl} lines exceeds 120: sink into a cmake/ module, or add a '# assembly-growth: <reason>' exemption marker to the root file (with explanation)"
            fail=1
        fi
    fi
    # structural hard lock: real compile rules banned from the root (blacklist), assembly statements pass. add_library passes only in ALIAS form.
    bad=$(grep -nE 'add_executable\(|add_library\(' CMakeLists.txt | grep -vE 'add_library\([^)]*ALIAS' | grep -vE '^\s*#|polyorch_' || true)
    if [ -n "$bad" ]; then
        echo "GATE-DOCS ✗ ⑤root facade carries compile rules (assembly layer must not load them; ALIAS umbrellas excepted): $bad"; fail=1
    fi
fi

# ⑧ workspace package count == README (ungated numbers drift: the clause count needed a lock,
# then ctest/gallery counts, now this). Value comes from cargo metadata, never hand-typed.
pk=$(cargo metadata --no-deps --format-version 1 2>/dev/null | \
     python3 -c 'import json,sys;print(len(json.load(sys.stdin)["packages"]))' 2>/dev/null || true)
rpkg=$(grep -oE 'workspace has [0-9]+ packages' README.md | grep -oE '[0-9]+' | head -1 || true)
if [ -z "${pk:-}" ]; then
    echo "GATE-DOCS ✗ ⑧cargo metadata read failed (asserting against nothing = untrustworthy)"; fail=1
elif [ "${rpkg:-x}" != "$pk" ]; then
    echo "GATE-DOCS ✗ ⑧README package count '${rpkg:-missing}' ≠ cargo '$pk'"; fail=1
fi

# ⑨ pixel gates must not hard-code the clear colour: read it out of the frame instead.
# Measured lesson (PIT-45): a one-triple-wrong literal made a coverage assertion count the
# whole frame — constant-true for an entire band, and it passed CI the whole time.
bad=$(grep -rnE 'let clear: \[u8; 3\] = \[[0-9]' crates examples --include='*.rs' 2>/dev/null || true)
if [ -n "$bad" ]; then echo "GATE-DOCS ✗ ⑨hand-written clear-colour literal (switch to self-calibration + a frequency guard):"; echo "$bad"; fail=1; fi

# ⑥ gallery manifest lock (B5 "one artifact, four consumers": source+prose+test+card —
# registries are the single accounting point; gallery is DERIVED, never hand-edited)
# the manifest is produced by `pixi run gallery` (inside build/, gitignored); absent = red (reminder to run the generator first).
GM=build/gallery/manifest.txt
if [ ! -f "$GM" ]; then
    echo "GATE-DOCS ✗ ⑥gallery manifest absent (run pixi run gallery first)"; fail=1
else
    gm_names=$(grep -oE '^E[0-9]{3}' "$GM" | sort -u)
    reg_names=$(for d in examples/rs examples/c examples/cpp examples/qt; do ls "$d" 2>/dev/null; done | grep -oE '^E[0-9]{3}' | sort -u)
    [ "$gm_names" = "$reg_names" ] || { echo "GATE-DOCS ✗ ⑥gallery manifest ≠ disk registry set:"; diff <(echo "$reg_names") <(echo "$gm_names"); fail=1; }
fi

# ⑦ README gallery card count == manifest live count (2026-10-08: 40≠43 drifted with no
# owner; the lock arrived with the installed-tree band; the entry-count half lives in
# cmake-smoke — it is the one holding the ctest truth, each lock lives in the tool with the data).
if [ -f "$GM" ]; then
    mc=$(grep -c '^E' "$GM")
    rg=$(grep -oE 'Example gallery \([0-9]+ cards' README.md | grep -oE '[0-9]+' | head -1 || true)
    [ "${rg:-x}" = "$mc" ] || { echo "GATE-DOCS ✗ ⑦README gallery '${rg:-missing}' ≠ manifest '$mc'"; fail=1; }
fi

# ⑧ wall manifest lock (T-H11): wall.json number set == registry numbers;
# every kind=live entry has its html file beside the manifest. The wall is
# hand-listed (like the gallery manifest was DERIVED) -- this lock is the drift net.
if [ -f examples/wasm/wall.json ]; then
    python3 - <<'WALLPY' || fail=1
import json, re, pathlib, sys
w = json.load(open("examples/wasm/wall.json"))
ids = [e["id"] for e in w["entries"]]
wnum = {i[:4] for i in ids}
reg = set()
for d in ("examples/rs", "examples/c", "examples/cpp", "examples/qt"):
    p = pathlib.Path(d)
    if p.is_dir():
        reg |= {m.group(0) for f in p.iterdir() if (m := re.match(r"E\d{3}", f.name))}
bad = []
if wnum != reg:
    bad.append(f"number-set drift: wall {sorted(wnum-reg)} extra / {sorted(reg-wnum)} missing")
for e in w["entries"]:
    if e["kind"] == "live" and not (pathlib.Path("examples/wasm") / (e["id"] + ".html")).is_file():
        bad.append(f"live entry without html: {e['id']}")
dup = {i for i in ids if ids.count(i) > 1}
if dup: bad.append(f"duplicate ids: {sorted(dup)}")
print("WALLPY ✗ " + "; ".join(bad) if bad else f"w11 ok: {len(ids)} entries == registry {len(reg)} numbers")
sys.exit(1 if bad else 0)
WALLPY
fi

if [ "$fail" = 0 ]; then
    echo "GATE-DOCS ✓ (E $(echo "$files" | wc -l) files three-way / README=${rn}↔spec=${sn} / header signatures $(echo "$hs" | wc -l) names / links alive / gallery $(grep -c '^E[0-9]' "$GM" 2>/dev/null || echo 0) cards / packages ${pk})"
fi
exit $fail
