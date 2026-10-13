# VisiaEngine Conventions & Constraints

> This file is VisiaEngine-only. C2-C8 and C10-C14 are reserved slots (C1/C9 inherited from the predecessor project's general methodology, kept to avoid dangling cross-references; C15/C16/C17 active). New conventions take the smallest free slot. Full predecessor archive: `.refinfo` (read-only).

## C1: Architecture-decision comparison format

**Constraint**: any architecture discussion involving option choice must enumerate, per option:
- **Pros/cons**: the advantages and disadvantages of each option
- **Sources/references**: the existing systems / open-source projects / industry practices borrowed from
- **Impact**: what choosing that option implies for subsequent development
- **Recommendation**: an explicit recommendation with reasons

Listing options for the user to choose without the above analysis is forbidden.

## C9: Automatic lesson consolidation

**Constraint**: problems, lessons, and experience discovered during development must be proactively written into the corresponding memory documents within the same session turn.

| Situation | → File to update |
|------|-----------|
| bug found / pitfall hit | `pitfalls.md` (symptom + root cause + solution + verification — all required) |
| user corrects AI behavior | `conventions.md` (new constraint, C{n} numbering) |
| architecture/config decision | `decisions.md` (D{n} numbering + rationale + impact) |
| project state change | `status.md` (date, Phase, test counts) |
| coding pattern / anti-pattern | `rules/common/coding-style.md` |
| security-related lesson | `rules/common/security.md` |
| edit-tool usage lesson | `rules/common/edit-safety.md` |

**Principle**: don't wait for the user to ask. Once a consolidatable experience is recognized, update proactively. Better over-record than omit.

## C14: Subagent artifacts must be verified (orchestrator iron law)

**Constraint**: completion claims returned by a subagent are not trustworthy. The orchestrator must verify the actual artifacts before marking a task complete.

**Verification checklist**:
- files claimed created → `cat`/`ls` to confirm existence + content completeness
- config claimed modified → `grep` the key fields
- commands claimed runnable → actually execute them
- tests claimed passing → re-run them

**Failure handling**: verification fails → resume the SAME subagent session carrying the concrete defects for repair (continue via task_id); do not take over the editing yourself.

## C15: Interface sentinels and parameter semantics must first be checked against the existing clause-ownership table

**Constraint**: (1) before a new interface sets a "failure/invalid sentinel", grep the clause library for existing value ownership (this case: `add_mesh` used 0 as the failure sentinel and collided with CAPI-01's black-and-white text — "entity pose slot0gen0=0 is legal = deliberate ownership" — only re-owning the return code + out-parameter catalog put it right); the same value domain meaning the opposite in two places = a guaranteed landmine. (2) one parameter surface serves no two masters: CI semantics (fast in-and-out `--frames`) and interactive semantics (resident window) must hang on two separate consumers (the run step's zero args vs the registry argv — ctest's sole judge); "one parameter set with two claimants" is a design alarm. (3) check method: before a new interface goes RED, write a "value-ownership cross-reference" passage into the clause draft (who owns the interpretation of 0/-1/MAX).
**Check command**: `grep -n 'sentinel\|MAX\| 0=' docs/sdd/capi.md | head` (no position collision in the value-ownership table before a new clause enters the library).

## C16: Decision-presentation format = plain words + diagram + directory + one-at-a-time

**Constraint**: any option/defect/impact analysis that needs user adjudication is presented as a four-piece set: (1) plain-language root cause (lead with the "one-line version"); (2) a schematic (data/flow/timeline ASCII — draw it whenever drawable); (3) the affected **directory structure / file manifest** itemized (including landing line positions); (4) with multiple topics, walk them one at a time (one card per adjudication); one-shot big lists are forbidden. The jargon-stacked version gets bounced back (this session's "speak plainly" corrections ×6, hard evidence). Recommendation reasons are restated using C1's four columns (pros-cons / source / impact / recommendation).

## C17: English-only for all persisted artifacts (2026-09-20 user ruling)

**Constraint**: Every artifact written to the repository MUST be in English — code comments (`//`, `///`, `/* */`, `#`), SDD contract clauses, `docs/`, `.agents/memorys/`, `.agents/rules/`, `.agents/skills/`, AGENTS.md, README, commit messages, example/demo source, config comments. The ONLY exception: live AI↔user chat replies (Chinese allowed when the user writes Chinese).

**Scope**: Prospective only (user chose A). The ~7,300 pre-existing Chinese lines (measured 2026-09-20) stay as-is; translate opportunistically when touching a file, no dedicated migration wave.

**Why**: repo is public-facing SDK distribution; mixed-language corpus is a liability for external contributors and for grep/CI gates that anchor on comment text.

**Check command**:
```bash
# new/changed .rs comments must be ASCII (run on staged diff; 0 matches expected)
git diff --cached -U0 -- '*.rs' | grep -P '^\+.*[\x{4e00}-\x{9fff}]' || echo OK
```

## C18: Shared-code boundary = the example's teaching claim (2026-10-08 band V, user rulings C2=A/C3=B)

**Constraint**: when pulling code duplicated across examples into shared components, pull only the parts that are **verbatim identical and carry no claim** (the device/surface/context boot sequences,
argv parsing). The code by which an example teaches its one lesson (present-path selection, mouse/keyboard semantics, pixel predicates) **must not be eaten by sharing** —
`render_view_rects` is ⑤b's course text; drag in E402/E403 is picking/box-selection, not orbiting. Criterion: if deleting some code leaves the example's tutorials.md
claim undemonstrated, that code is course text, not duplication.

**Check command**: `grep -n '非目标\|留本地' .omo/plans/viewer-shared-band-2026-10-08.md | head`;
the migration batch's gates (determinism pre-check → sha/timeline → interaction tri-state) live in `rules/common/testing.md`, section "Batch-migration gates".

## C19: Self-check/prune/gate-step predicates must be token-level, and a breakage probe must run before commissioning (2026-10-08 band V, hard evidence)

**Constraint**: predicates like "is this symbol still used" or "is this string still there", judged by **text line**, get polluted by comments and string literals.
Hard evidence: batch 12's import pruning only stripped `use` lines, so `Window` survived on a comment mention — what caught it was this repo's
`lint` (`cargo clippy --workspace --all-targets -- -D warnings`), **not the script**; and at the time my commit described the script as stronger than its implementation
(claiming "comment mentions don't count as usage"). Two failures of the same family: (1) predicate too coarse (2) self-description exceeding the implementation.

Correct predicate order: strip `//` comments → strip string literals → strip `use` lines → then query on word boundaries.
Before any new self-check/gate goes live, run a **breakage probe** (rename/delete the object under test; the gate must go red and the report must point at it alone).

**Check command**:
```bash
# (1) predicate self-proof: plant a breakage probe in the check itself (a hit >=1 makes a real gate; in-repo it must be 0)
printf 's = re.sub(r"use [^\\n]+", "", s)\n' > /tmp/c19-probe.py
grep -c 're\\.sub(r\"use ' /tmp/c19-probe.py                       # expect 1: the predicate can hit the crude form
grep -rn 're\\.sub(r\"use ' scripts examples/rs/src crates bindings 2>/dev/null | wc -l   # expect 0
rm -f /tmp/c19-probe.py
# (2) fallback advantage (this repo's real judge): the lint segment is -D warnings; a prune escapee is always caught by it
grep -n '^lint = ' pixi.toml
```
**Note (honest, 2026-10-08)**: the band's codemod (`/tmp/migrate_viewer.py`, session-scoped
and NOT in the repo -- plans and one-off scripts are deliberately not tracked) still carries
the crude form at lines 207/211: it strips `use` lines only, no comment/string stripping.
The token-level fix was implemented in the batch 13/14 one-off invocations, whose corrected
predicates are likewise untracked. Therefore the durable guarantee is ② -- `lint` with
`-D warnings` -- which is what actually caught `Window` in batch 12. ① exists so the next
person who writes a prune step can prove their predicate bites before trusting it.


### C19 addendum (2026-10-08, adopted after lesson-review)

Adopted ruling: **`lint` is the durable gate for prune-type steps; no new CI segment.**
The discipline that was missing is "do not claim the script checks more than it does".

- Durable gate (verified in-repo): `pixi.toml` L76
  `lint = "cargo clippy --workspace --all-targets -- -D warnings"` — an unused import that
  survives a crude prune becomes an error here. This is what caught `Window` (batch 12).
- Self-test for any new prune/gate predicate (no /tmp dependency, run it in one line):
  `printf 's = re.sub(r"use x","",s)\n' > /tmp/p; grep -c 're\.sub(r"use ' /tmp/p; rm /tmp/p`
  — must print **1**; if it prints 0 the predicate is decorative and must be rewritten
  before it is written into any rule or script.
- Pointers added where they will be seen: `rules/common/testing.md` (Batch-migration
  gates) and `rules/rust/coding-style.md` (Runnable Check section) both now name `lint -D
  warnings` as the true judge of import-pruning steps.

## C20: All tracked artifacts are English; Chinese lives in AI chat and the .omo plan zone (2026-10-12, user adjudication A)

The whole repo migrated to English artifacts (E1-E5, PolyOrch-C4-shaped but V-written:
skills/ is a directory-level whitelist -- trigger phrases and keyword tables are activation
surface, and PolyOrch's quoted-phrase heuristic would flag 1k+ lines of V skill bodies;
.opencode node_modules and book-to-skill parser regexes are vendored/functional). The
brand record line in decisions.md D6 and the `.omo`-anchoring conventions check literal are
the named residuals. Test fixtures that ARE the feature under test (E505 CJK glyphs,
E814 UTF-8 path, io_spec Chinese test-function identifiers) are functional data, not prose.

```bash
bash scripts/gate-i18n.sh   # red/green proof duty per C0 was performed 2026-10-12
```

New commits: subject+body in English from E1 onward (the pre-E1 Chinese commit history is
history, not policy; searchable via the whitepaper-zh-v0.1.0 tag and parent commits).

## C21: Semicolon is the CMake list separator -- multi-value table entries need their own delimiter (2026-10-12, caught live as CMP0002 target collision)

A `set(T "a=b;c;d")` entry is ONE string at `set()` but a 3-element LIST the moment
`foreach(x ${T})` iterates it -- each `;` splits. Table entries that bundle a value plus
its dependency list must use a non-list separator (`@` here) inside the entry and
`string(REPLACE "@" ";" ...)` after extraction. Symptom when violated: `add_custom_target`
dies with "another target with the same name already exists" on the SECOND entry, naming a
fragment (e.g. `visiaengine-render-build`) as the collision -- the fragment was a dependency
name iterated as a crate.

```bash
# any table file: entries must not carry raw ';' inside quoted values
grep -nE '"[^"]*;[^"]*=' cmake/VisiaEnginePolyOrchCrates.cmake && echo FAIL || echo PASS
```

## C22: "Directory taxonomy is single-axis; names follow their axis, never a cross-axis copy" (2026-10-12, js->wasm + capi->ffi rounds)

`crates/` = the rust set (implementation axis); `bindings/` = the consumer face
(consumer-language/artifact axis); `examples/` = teaching examples. A directory
must not embed the other axis (`bindings/js/rust/` and `bindings/c/visiaengine-capi`
were both the implementation axis living under the consumer axis -- both gone).
Target names likewise: rust crates carry rust-ecosystem suffixes (`-ffi`,
`-wasm`), CMake real names carry the consumer word (`visiaengine-c/-cpp/-qt`),
aliases carry the stable contract (`::c/::cpp/::qt`), IDE viewers carry the
content word (`-sources`). Aggregates (`-all`/`-<verb>`) exist ONLY to
collect a MULTI-button surface -- an aggregate over a single crate's verbs
is a convenience bundle, not a collection (wasm-all deleted 2026-10-12:
the rlibs aggregates stand on 9 buttons; one-crate lanes have no bundle).
Corpus: iceoryx2
(iceoryx2-ffi crate <-> iceoryx2-c package) and zenoh (ffi crate <-> zenoh-c repo)
both keep the two axes lexically distinct; bevy keeps crates flat.

```bash
# no implementation-language layer under the consumer tree; no rust crates under bindings/
test -z "$(find bindings -maxdepth 2 -type d \( -name rust -o -name '*-capi' -o -name '*-wasm' \) 2>/dev/null)" && echo PASS || echo FAIL
# crates/ is exactly the rust set (no consumer-face dirs)
ls crates/ | grep -vE '^(visiaengine-[a-z0-9-]+|\.gitignore)$' | grep -qE 'probe|demo|include' && echo FAIL || echo PASS
# an aggregate target may only exist where its family has >1 sibling buttons
# (wasm-all precedent: bundling one crate's verbs is not aggregation)
grep -rn "add_custom_target(visiaengine-wasm-all" cmake/ && echo FAIL || echo PASS
```
