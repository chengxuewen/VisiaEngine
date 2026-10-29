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
            http*|mailto:*|\#*|*\ *|build/*|target/*|www/*) continue ;;
        esac
        target=${dest%%#*}
        target=${target%%\?*}
        [ -n "$target" ] || continue
        [ -e "$dir/$target" ] || { echo "GATE-DOCS ✗ ④broken link: $f → $dest"; fail=1; }
    done < <(grep -oE '\]\([^)]+\)' "$f" 2>/dev/null | sed -E 's/^\]\(//; s/\)$//')
done < <(ls README.md docs/*.md docs/sdd/*.md docs/reference/*.md book/web/*.md llms.txt 2>/dev/null)

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

# ⑦ README wall-entry count == wall.json entries (P3: the lock survives its
# carrier -- gallery cards -> wall manifest; the drift lesson of 2026-10-08
# (40≠43, no owner) is why this half stays a machine gate. Entry/ctest truth
# lives in cmake-smoke; manifest/registry equality in ⑧.)
if [ -f examples/wasm/wall.json ]; then
    mc=$(grep -c '"id"' examples/wasm/wall.json)
    rg=$(grep -oE 'example wall \([0-9]+ entries' README.md | grep -oE '[0-9]+' | head -1 || true)
    [ "${rg:-x}" = "$mc" ] || { echo "GATE-DOCS ✗ ⑦README wall '${rg:-missing}' ≠ wall.json '$mc'"; fail=1; }
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

# ⑩ llms.txt numbers == live ground truth (V-A 2026-10-15: the generated file drifted to
# 41/44 unnoticed because nothing locked it; this net reuses the existing censuses). ABI:
# llms == README 'C ABI 49 entry points' == header signature names (③'s hs). Cards: llms ==
# README 'example registry (44 entries' == example-file set (①'s real_files -- NOT wall.json:
# registry 44 vs wall 43 are different ledgers, the E811 double file).
la=$(grep -oE 'C ABI surface \([0-9]+ entries' llms.txt | grep -oE '[0-9]+' || true)
ra=$(grep -oE 'C ABI [0-9]+ entry points' README.md | grep -oE '[0-9]+' | head -1 || true)
ca=$(echo "$hs" | wc -l)
[ "${la:-x}" = "$ca" ] && [ "${ra:-y}" = "$ca" ] || { echo "GATE-DOCS ✗ ⑩ABI: llms='${la:-missing}' README='${ra:-missing}' census='$ca'"; fail=1; }
lc=$(grep -oE '[0-9]+ example cards' llms.txt | grep -oE '[0-9]+' || true)
rc=$(grep -oE 'example registry \([0-9]+ entries' README.md | grep -oE '[0-9]+' | head -1 || true)
cc=$(echo "$real_files" | wc -l)
[ "${lc:-x}" = "$cc" ] && [ "${rc:-y}" = "$cc" ] || { echo "GATE-DOCS ✗ ⑩cards: llms='${lc:-missing}' README='${rc:-missing}' registry='$cc'"; fail=1; }

# 11) gitlink three-way alignment (C23 round 2026-10-15: the recurring "M PolyOrch" class).
# pin == submodule checkout HEAD == published. Offline-first: remote-tracking refs carry the
# published verdict when fresh; stale tracking falls back to ONE timeboxed ls-remote; neither
# proving publication = red (that is exactly the fresh-clone hazard this pin must never ship).
pin=$(git ls-files -s PolyOrch | awk '{print $2}')
ibranch=$(git -C PolyOrch symbolic-ref --quiet --short HEAD || echo "DETACHED")
ihead=$(git -C PolyOrch rev-parse HEAD 2>/dev/null)
[ "$ibranch" != "DETACHED" ] || { echo "GATE-DOCS x 11) submodule HEAD detached (C23 anchor rule)"; fail=1; }
if [ -n "$pin" ] && [ "$pin" != "$ihead" ]; then echo "GATE-DOCS x 11) pin ${pin:0:7} != submodule HEAD ${ihead:0:7} -- M PolyOrch drift, bump or reset"; fail=1; fi
if [ -n "$pin" ] && ! git -C PolyOrch branch -r --contains "$pin" 2>/dev/null | grep -q .; then
  pub=$(GIT_TERMINAL_PROMPT=0 timeout 15 git -C PolyOrch ls-remote origin 2>/dev/null | awk -v s="$pin" '$1==s{print $2; exit}' || true)
  if [ -n "$pub" ]; then echo "  11) pin published via ${pub##*/} (local tracking stale -- fetch to refresh)"; else
    echo "GATE-DOCS x 11) pin ${pin:0:7} on no remote ref (fresh-clone hazard) -- push the submodule before pinning, or rewind the pin"; fail=1
  fi
fi

# 12) CLI front door (visiaengine.sh / .bat) — the human entry documented in AGENTS.md and
# docs/env.md. Three things must hold, otherwise "docs say ./visiaengine.sh X" is a lie:
#   a) the wrapper parses and its own contract is intact (-h -> 0, no args -> 2);
#   b) every task name it mentions (in the docs AND in its own USAGE text) exists in pixi.toml;
#   c) pixi.toml is parsed directly, NOT via `pixi task list`: whether `pixi` resolves inside a
#      task depends on the machine-local O5 PATH fix, which a CI gate must not assume.
if [ -f visiaengine.sh ]; then
    bash -n visiaengine.sh || { echo "GATE-DOCS ✗ 12)syntax error in visiaengine.sh"; fail=1; }

    hout=$(./visiaengine.sh -h 2>&1); hrc=$?
    { [ "$hrc" = 0 ] && grep -q 'usage:' <<< "$hout"; } \
        || { echo "GATE-DOCS ✗ 12)-h must exit 0 and print usage (got $hrc)"; fail=1; }
    ./visiaengine.sh >/dev/null 2>&1; nrc=$?
    [ "$nrc" = 2 ] || { echo "GATE-DOCS ✗ 12)no-argument invocation must exit 2 (got $nrc)"; fail=1; }

    cli_tasks=$(awk '
        /^\[/ { in_tasks = ($0 ~ /^\[(feature\.[a-z0-9_-]+\.)?tasks\]$/); next }
        in_tasks && /^[A-Za-z0-9_-]+[ \t]*=/ { name=$0; sub(/[ \t]*=.*/, "", name); print name }
    ' pixi.toml | sort -u)
    tn=$(printf '%s\n' "$cli_tasks" | grep -c . || true)
    [ "${tn:-0}" -ge 60 ] \
        || { echo "GATE-DOCS ✗ 12)pixi.toml task parse looks broken (${tn:-0} names, expected >= 60)"; fail=1; }

    cli_refs=$(grep -ohE 'visiaengine\.(sh|bat)[[:space:]]+(-e[[:space:]]+[^[:space:]]+[[:space:]]+)?[^[:space:]]+' \
            visiaengine.sh visiaengine.bat AGENTS.md docs/env.md README.md 2>/dev/null \
        | sed -E 's/.*visiaengine\.(sh|bat)[[:space:]]+//; s/^-e[[:space:]]+[^[:space:]]+[[:space:]]+//' \
        | sort -u | grep -vE '^(-|<|>)' | grep -v '[<>]' || true)
    if [ -z "$cli_refs" ]; then
        echo "GATE-DOCS ✗ 12)no ./visiaengine.sh <task> reference found (docs and USAGE both empty?)"; fail=1
    fi
    # `pixi run <name>` also accepts executables from the environment PATH (cmake, cargo, ...),
    # so the allowlist below is part of the contract, not an escape hatch: a name that is
    # neither a task nor one of these is almost certainly a typo in the docs.
    cli_exes="cmake cargo node npm python3 ctest rustc rustfmt"
    while IFS= read -r name; do
        [ -n "$name" ] || continue
        if grep -qxF "$name" <<< "$cli_tasks" || grep -qwF "$name" <<< "$cli_exes"; then
            continue
        fi
        echo "GATE-DOCS ✗ 12)'$name' referenced by the CLI docs or USAGE is neither a pixi.toml task nor a known env executable"
        fail=1
    done <<< "$cli_refs"
fi

# ⑬ assembled book (S4): when build/book-src exists, every SUMMARY entry must resolve and
# the graph chapter must still embed the app page (guards the S2 path-collision regression).
# A bare clone has no build/book-src -> SKIP, same convention as cmake-smoke/gate-pack.
book_note="book: no build yet"
if [ -f build/book-src/SUMMARY.md ]; then
    book_chapters=0
    while IFS= read -r bdest; do
        book_chapters=$((book_chapters + 1))
        [ -e "build/book-src/$bdest" ] || { echo "GATE-DOCS ✗ ⑬SUMMARY points at a missing file: $bdest"; fail=1; }
    done < <(grep -oE '^- \[[^]]*\]\([^)]+\)' build/book-src/SUMMARY.md | sed -E 's/.*\]\(([^)]+)\)$/\1/')
    if [ -f build/book-src/web/graph.md ]; then
        grep -q 'src="graph-view.html"' build/book-src/web/graph.md \
            || { echo "GATE-DOCS ✗ ⑬graph chapter no longer embeds graph-view.html"; fail=1; }
    fi
    book_note="book: ${book_chapters} chapters"
else
    echo " [ SKIP ] ⑬ no build/book-src (run: pixi run book-build)"
fi

if [ "$fail" = 0 ]; then
    echo "GATE-DOCS ✓ (E $(echo "$files" | wc -l) files three-way / README=${rn}↔spec=${sn} / header signatures $(echo "$hs" | wc -l) names / links alive / wall $(grep -c '"id"' examples/wasm/wall.json 2>/dev/null || echo 0) entries / packages ${pk} / gitlink aligned / cli ${tn:-0} tasks / ${book_note})"
fi
exit $fail
