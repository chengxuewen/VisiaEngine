# Code Edit Safety

> **Target audience**: AI agents editing VisiaEngine source code.
> **Violation of these rules causes token waste from repeated fix cycles.**

## Tool Selection

| Change Size | Tool | Reason |
|-------------|------|--------|
| Rewrite entire function/file | `write` | Guarantees brace balance, no stale lines |
| ≤20 line single-location edit | `edit` | Minimal diff, safe for small changes |
| Structural pattern replacement | `ast_grep_replace` | Syntax-aware, preserves matching |
| Complex multi-file refactor | Delegate to subagent | Isolated context, verify independently |

## Forbidden Patterns

| Anti-Pattern | Why |
|--------------|-----|
| `sed` for code modification | Quote escaping errors, regex silent failures |
| Multiple sequential `edit` calls without re-reading | Line numbers drift, stale hash IDs |
| Deleting a line by replacing with empty `lines: []` and assuming brace count is still correct | May leave unbalanced braces |
| Appending `}` to "fix" an unclosed delimiter without counting braces first | Masks root cause, may create double-close |

## Verify Immediately

After EVERY code change (edit, write, or ast_grep_replace):

```
Rust:   cargo check -p <crate>       (5-15s)
TS/TSX: npx tsc --noEmit             (3-5s)
YAML:   docker compose config --quiet (compose files)
Shell:  bash -n <script>             (shell scripts)
```

### Batch edits array must be verified per-operation

Multiple replace operations referencing adjacent areas can easily misalign boundary line numbers/content (one operation may overwrite another's reserved region). Run the corresponding format validation immediately after each edit call; if corruption is found, re-read the file to restore — do not layer fixes on top.

If verification fails, STOP. Do NOT apply another edit on top. Instead:
1. `git diff` to see what changed
2. If the change is wrong, `git checkout -- <file>` to revert
3. Re-apply the fix correctly

## Brace Safety Checklist

Before marking any multi-line edit complete, verify:
- [ ] Every `{` has a matching `}` at the same indent level
- [ ] Every `(` has a matching `)`
- [ ] Every `[` has a matching `]`
- [ ] No duplicate function definitions or closing braces
- [ ] `cargo check` / `tsc --noEmit` passes

## When to Delegate

Delegate to a `deep` category subagent when:
- The change touches 3+ files
- The change requires understanding cross-module dependencies
- You've failed the same edit 2+ times

The subagent gets a clean context, reads the files fresh, and applies all changes atomically.

## Architectural Decision Gate (NON-NEGOTIABLE)

Before implementing ANY architectural change (protocol, data flow, transport mode, API contract):
- **ALWAYS ask the user first** using the `question` tool with explicit options
- **NEVER fall back to an alternative architecture** without user approval
- **NEVER silently switch** from the agreed architecture (e.g., SFU → P2P) even if it seems "easier"
- **NEVER implement a workaround** that changes the system's design without explicit user consent

If the agreed approach fails, report the failure and ask: "Approach X failed because of Y. Suggest switching to Z. Proceed?"

## Test Execution Constraint (NON-NEGOTIABLE)

After claiming tests are written or features are working:
- **ALWAYS run the tests** against the live system. Writing test files without executing them is a violation.
- **ALWAYS report actual test output** — pass/fail counts, error messages. Never claim "tests pass" without evidence.
- **E2E tests MUST run against the actual running service**, not mocked endpoints.
- If tests fail, fix them in the same turn. Do not defer to "later".

## Verification Honesty (NON-NEGOTIABLE)

- **NEVER claim a feature works based on a partial test.** A Python WS test passing does NOT mean the browser flow works.
- **ALWAYS verify at the actual user-facing layer.** If the feature is browser-based, test in the browser. If it's API-based, test with curl.
- **ALWAYS report exactly what was tested and what was NOT tested.** Example: "Python WS test passed. Browser flow NOT yet verified."
- **NEVER present a component test as end-to-end proof.** Each layer must be verified independently.
- **If you cannot verify at the user-facing layer, say so explicitly.** Do not imply success.

## Feature Flag Discipline

- **ALL required features MUST be in `default` features** in Cargo.toml — never require manual `--features` for core functionality
- **Build commands in docs MUST include all features** — never document `cargo build` without required features
- **Before running server, ALWAYS verify**: `cargo build -p <app-crate>` (with defaults) produces working binary
- **If a feature is optional, it must be explicitly opt-out** (disable with `--no-default-features`)

## Self-Verification Requirement (NON-NEGOTIABLE)

- **ALWAYS verify browser-based features yourself using Playwright MCP tools** (`local-playwright_browser_navigate`, `local-playwright_browser_evaluate`, etc.)
- **NEVER ask the user to test what you can test yourself.** If Playwright is available, use it.
- **After fixing a browser bug, ALWAYS re-test in the browser** before reporting the fix.
- **Report the actual browser console output** as evidence of verification.

## User Confirmation Before Edit (NON-NEGOTIABLE)

- **NEVER start editing files without explicit user approval.** Describing a plan ≠ approval to execute.
- **When user asks 'what can be done' or 'is it possible to...', they are asking a question, not giving an instruction to edit.** Answer the question. Do NOT edit files.
- **Before editing, present the plan AND use the `question` tool to confirm.** Wait for affirmative response before touching files.
- **Silence / 'continue' / timeout ≠ approval.** Only explicit 'yes' / 'do it' / 'execute' counts.

## Process Management (shell)

- **NEVER use `pgrep -f` / `pkill -f` with a pattern that matches your own shell command line** (e.g. `pgrep -f "<my-binary>"` from a bash tool whose command string contains that literal) — it kills the shell itself, hanging the tool. Use `pgrep -x <exact-process-name>` (matches process name only, e.g. `<my-binary>`), or exclude own PID.
- **Killing + relaunching in one shell command** can kill the just-started process (SIGTERM/SIGHUP to the process group on tool timeout). Launch with `setsid nohup ... < /dev/null & disown` and verify with `pgrep -x` in a separate call.
- **Port-in-use on relaunch** (e.g. `Failed to bind 0.0.0.0:9801`) almost always means the old process survived the kill — verify with `ss -tlnp | grep <port>` and kill by PID.
- **Container-recreated services lose apt-installed tools** (gdb etc.) — install debug tools in the Dockerfile dev target, not per-container.

**Source**: debugging round (2026-08-04: pgrep -f self-kill, container rebuild loses gdb)

## Network Tooling (bash)

- **curl to localhost must use --noproxy**: when the bash environment has `http_proxy` set, `curl http://127.0.0.1:5173` routes through the proxy → timeout/hang (manifests as "Vite unresponsive"). Use `curl --noproxy "*" http://127.0.0.1:PORT/`.
- **Container tcpdump filtering beware of NAT**: packets from host to container subnet (172.18.0.2) have their source IP rewritten to the gateway (172.18.0.1) — `not host 172.18.0.1` filters out the local browser/application traffic too. Distinguish by **source port** (host fixed port vs browser random port), not by source IP.

**Source**: debugging round (2026-08-04)

## Git Recovery Operations

- **Batch `git restore <paths>` to recover staged deletions may not fully write back all directories** — `git ls-files` (index) has files but the disk (worktree) is empty, and grep returns nothing for that directory. Root cause: `restore` is incomplete for staged-deletion paths. **Prefer `git checkout HEAD -- <paths>`** (forces write-back from HEAD to the worktree).
- **Verification must be exhaustive, not sampled**: after recovering/deleting N directories, check each one with `for d in ...; do echo "[$d] index=$(git ls-files $d/ | wc -l) worktree=$(ls $d/ 2>/dev/null | wc -l)"; done` — index and worktree counts must all match. Checking only some directories = missed (precedent: 10 directories recovered but only 7 actually written back, 3 empty on disk went undetected).

**Source**: 2026-08-06 .agents trim recovery round

### 8. Multi-line edit replacement must verify line uniqueness

**Rule**: after a multi-line edit replacement on .py/.rs files, if the replacement contains duplicate patterns (identical lines), grep must verify uniqueness:

```bash
grep -c "duplicate-pattern" <file>    # expect 1; >1 = edit inserted duplicates
```

**Precedent**: the 2026-08-10 session had three edit-tool anomalies — (1) the replacement lost leading lines (main.rs config-path indentation corrupted but syntax-valid, compiled with stale logic); (2) duplicated dispatch-line insertion (identical lines 287/288 in the old project's CLI script → the restart/run-host ran the container rebuild twice). **Fix**: (1) switched to python precise string replacement (read file → replace → write back); (2) deleted the duplicate lines then verified with grep -c.

**Blocking condition**: committing after a multi-line edit without verifying uniqueness/line count.

### 9. Grep current state before consecutive edits in the same area

**Rule**: when making consecutive edits to the same file/function/area, run `grep -c "<anchor line content>" <file>` before each edit to confirm uniqueness;

**Extension (2026-09-14, third judgment)**: anchor text for cross-step python batch-edit scripts **must be read from disk by the script at run time** (grep/sed live state); hand-typing it from the previous round's draft memory is forbidden — 3 script asserts in this session crashed precisely for this reason (rustfmt reflow had changed a comment AUTO→PIXI, a D8 heading variant, a smoke-bullet indentation layer); the assert-first pattern has proven zero-loss, but one crash = one wasted round. For "existing content + insert" patterns (adding logging or changing a signature before old code), prefer python precise string replacement (read → replace → write back) over edit's lines array, which can match multiple times.

**Precedent**: the 2026-08-11 debugging round — the edit tool inserted duplicates three times (stop() function signature ×2, main declaration ×2, residual log line), each only exposed at build time, wasting 3 rounds. The fix unified on python replace (assert count==1).

**Blocking condition**: making a 2nd consecutive edit to the same function without grep verification; committing with duplicate insertions still present.

### 10. Python batch replacement scripts must write per-block or verify upfront

**Rule**: for multi-block python replacement scripts (assert → replace → write pattern), **write to disk immediately after each block's replace**, or **verify all asserts upfront then do the replacements in one pass**; forbidden: "replace everything then write once at the end" (any assert failure → the entire write is lost, hit twice per precedent).

**Verification**: after the script runs, `grep -c "<key replacement content>" <file>` confirms each block took effect; before re-running after failure, check which blocks already wrote.

**Blocking condition**: multi-block script writing everything at the end; re-running without confirming intermediate state after an assert failure.
**Blocking condition**: multi-block script writing everything at the end; re-running without confirming intermediate state after an assert failure.

### 11. Large markdown appends use heredoc, not edit tool JSON

**Rule**: when **appending** new entries to large markdown such as `.agents/memorys/*.md` (containing quotes/backticks/long Chinese text), prefer `cat >> file <<'EOF'` heredoc;
**forbidden: using the edit tool for long-content appends** — edit's JSON payload repeatedly fails to parse with complex quotes/backticks/excessively long content
(hit 3 times this session: "unsupported op undefined" ×2 + JSON parse error ×1, each wasting a round).

**Verification**: after appending, `grep -c "<key heading>" <file>` confirms it took effect + `wc -l` shows the line count increased.

**Blocking condition**: using the edit tool to append long markdown/memory content and failing to switch to heredoc afterward.

### 12. Forbidden: running cargo fmt indiscriminately — workspace format drift

**Rule**: forbidden to run `cargo fmt` / `pixi run cargo fmt` without arguments or with `-- <path>` — cargo fmt's `--` parameter is **NOT path filtering**; it formats the entire workspace; and this workspace has rustfmt version drift (historical files' format disagrees with current rustfmt expectations; a full fmt produces a 112-file / 3000+ line diff). For single-file formatting use `rustfmt --edition 2024 <file>` (the rustfmt CLI supports single files), or maintain style manually.

**Precedent**: 2026-08-12 `cargo fmt -- <file>.rs` (cargo fmt's `--` argument is not path filtering) accidentally formatted 112 workspace files (3103 insertions); functional changes had to be re-applied only after `git checkout -- .` restored the tree, wasting 3+ rounds.

**Verification**: after any fmt operation, `git diff --stat | wc -l` must == the expected file count (usually 1); `git status --short` shows no unexpected files.

**Blocking condition**: attempting single-file formatting with cargo fmt; not verifying the diff scope after fmt.

### 13. Batch edit hits hash mismatch → full re-read then retry; forbidden to splice partial tags from error output (2026-08-17)

**Rule**: after a batch edit reports "hash mismatch", **fully re-read the target file before retrying**; forbidden to splice the updated partial LINE#IDs from the error output directly into the second call (unchanged lines still carry old tags → fails again, wasting 2 rounds). For global config (`~/.config/opencode/*.jsonc`) or other files that opencode may rewrite during runtime, **re-read on the spot before editing** (tags read early in the session go stale).

**Precedent**: 2026-08-17 omo.jsonc batch edit — first attempt had 8-line mismatches → retrying with the error-hint's updated tags still failed → succeeded only after a full re-read of lines 167-347 (3 rounds vs 2 rounds).

**Verification**: after an edit, `python3 -m json.tool <file>` (json) or `grep -c '"reasoningEffort": "low"' .omo/omo.jsonc` (jsonc target field) confirms it took effect.

**Blocking condition**: retrying by splicing partial tags from error output without a re-read; no syntax validation after a JSON replacement.

### 14. Repo-level renames/batch edits are agent-concurrency no-go zones

**Rule**: while subagents are running (a background task hasn't received its completion notification), **forbidden** to execute repo-level renames, cross-file batch replacements, or directory-level `git checkout/restore`. Subagents may: (1) have later commits overwrite/restore the working tree (git checkout undoing "contamination" also wipes the orchestrator's uncommitted changes — proven: a 10-file rename was erased wholesale); (2) keep coding on stale content, producing conflict merges.

**Verification**: after a rename/batch replacement, `grep -rc "<old pattern>" <scope>` must be 0 + binary-level verification (`readelf --dyn-syms` symbol names); before redoing, confirm `git log` is static + no background tasks.

**Blocking condition**: performing repo-level replacement while subagents are incomplete; committing without dual symbol/content verification after batch replacement.

### 15. Bash script cleanup pkill/pgrep + set -e — must use || true; timeout needs -k

**Rule**: (1) in `set -euo pipefail` scripts, **cleanup-type `pkill`/`pgrep` must be suffixed with `|| true`** — pkill **exits 1 when no process matches** → set -e **exits the script instantly** (the trap cleanup swallows the scene — observers see "hung 200s+" but it was actually a fast-fail; proven: branding E2E segment 4 died before start); (2) wrap long commands with **`timeout -k 5 <sec>`** (TERM alone goes only to the direct child and may be ignored — some process managers swallow TERM — without -k the child-process chain can't be killed); (3) the first diagnostic for "hung vs instant-fail" = **`bash -x scripts/x.sh`, check the last line** (`+ pkill` then straight to trap cleanup = instant fail; last line is the target command = genuinely hung); (4) cleanup-type pkill uses `-x` exact names (forbidden: `-f` matching your own command line — see the Process Management rule above; hit again this session, wasting a 200s timeout round).

**Verification**: `grep -nE "pkill|pgrep" scripts/*.sh` — confirm `|| true` on every cleanup-type call; `grep -rn "timeout [0-9]" scripts/` to confirm long commands carry -k.

**Blocking condition**: pkill/pgrep in scripts without `|| true`; timeout wrapping long commands without -k; diagnosing a "hang" by guessing deadlock instead of locating with bash -x.

**Source**: 2026-08-21 app-branding E2E debugging round

### 16. pkill/pgrep -f pattern containing own cmdline literal = self-kill, hangs the shell (second occurrence, 2026-09-01)

**Rule**: when cleaning up browsers/child processes, forbidden to use `pkill -f <string>` when that string appears in the current bash command line (paths like chrome-linux64/chrome easily end up in your own cmdline). Use `ps -eo pid,comm` (comm is the exact column) + kill by PID, or the bracket-grep trick `[c]hrome`.
**Precedent**: the previous project hit it first; in the 2026-09-01 debugging round `pkill -f "ms-playwright/chromium-1234/chrome-linux64/chrome"` (pattern string in the own command line) → the shell silently hung through two 60s/260s tool windows.
**Blocking condition**: any `pkill -f`/`pgrep -f` pattern string overlapping as a substring with the current command line.

### 17. After a heredoc python block comes a command separator — the following git commit is not short-circuited by an assert failure

**Rule**: in a combination like `python3 - <<'EOF' ... EOF` newline `git add ... && git commit`, the **python assert failure does NOT stop the later git chain** (newline = separator, not `&&`). Consequence: the edit is intercepted but the commit still runs, and the action claimed in the commit message diverges from the actual artifact (precedent: a docs(memory) commit claimed "AGENTS synced" while AGENTS was untouched; fixed by a follow-up commit). For multi-step "edit → commit" flows, either chain everything with `&&`, or re-run `git status --short` before committing to verify the staged surface.
**Precedent**: the 2026-09-03 pixi-environment commit round (anchor-line memory imprecise → assert intercepted → message/artifact divergence).
**Verification**: before committing, `git diff --cached --stat` must match the message's claimed file list item by item.
**Scope extension (2026-09-14)**: the same measure-first discipline covers **declared numbers in plan documents and READMEs** (Momus caught "six segments" that were actually 9; gate-docs ② turning the README clause count into a machine assertion is this rule's gate-ification). Any verifiable number entering a document: run it once first and take the true value.
**Blocking condition**: a message containing "N files synced"-style plural claims without verifying the cached diff item by item.

### 18. Insert-style rewrites must consume the old lines + alternation assertions must use grep -E (2026-09-16: quadruple hit + false green, both proven)

**Rule**: (1) When edit/replace does "replace + expand lines", ALL lines of the old block must be inside the `pos..end` range — anchoring only the first line leaves zombie lines (seen four times this session: double gate-line / double terminal-state line / vanished if-line / double anchor); right after the change run `sort FILE | uniq -d` (excluding fence noise) + an old-name `git grep` zero-out assertion. (2) A shell assertion `grep 'a|b|c'` without `-E` = BRE literal — the alternation silently fails and a "=0 zero-out gate" is permanently true, a false green; prove a predicate can hit a positive case first (`grep -E 'a|b'` on a file known to contain a must be non-empty). (3) The "if X errors → remedy" phrasing = a conditional-branch cliffhanger: when the remedy is certainly harmless, switch to "pre-set at birth"; when uncertain, keep the condition but test it on the spot.

**Verification**: `bash -n scripts/*.sh`; run each zero-out grep once with a positive control first.
**Blocking condition**: committing after batch rewrites without running the duplicate-line/residue assertions; a zero-out gate without positive self-proof.

### 19. Gate commands must not end in a pipe (the tail program eats rc) — shipped-broken, proven 2026-09-16

**Rule**: the exit code of `cmd | tail -1 && next` is **tail's** (always 0) — after a red gate is swallowed by the pipe, commit/chaining proceeds as usual (this case: cmake-smoke's syntax was broken yet after `| tail -1` it still ran `&& git add`; the broken script entered the repo and took a follow-up entry to square the books). Discipline: (1) gate assertions use bare commands (`cmd > log 2>&1; rc=$?`; display with `tail log` as a separate statement); (2) when chaining is mandatory use `set -o pipefail` or `${PIPESTATUS[0]}`; (3) before judging green from `grep -cE '<word-list>'`, prove the word list covers every failure shape of that step (same blind spot as `Diff in` missing from the fmt assertion vocabulary — fmt failure prints neither ✗ nor error).
**Precedent**: the 2026-09-16 B1d band (two files in the fmt segment were never rustfmt-ed; the assertion vocabulary lacked `Diff in` → a false-green "FAILED=0" entered the repo; closed at 4dc7f4f).
**Verification**: `bash -c 'false | tail -1 && echo BAD'` must print BAD — the counter-example on record; eyeball every gate call site with `grep -nE '\| *(tail|head).*&&'`.
**Blocking condition**: a newly added gate command ends in a pipe without pipefail.

### 20. rustfmt only eats the .ts/.rs whitelist — non-Rust input rewrites and corrupts files (proven 2026-09-17)

**Rule**: the command list for batch rustfmt must **not** contain non-.rs files like Cargo.toml/*.md/*.json — rustfmt parses them with Rust lexing and rewrites/corrupts the original file (this case: Cargo.toml's Chinese comments were treated as tokens, erroring and being rewritten; restored via git checkout).
**Precedent**: point-cloud band M2, `rustfmt --edition 2024 … Cargo.toml` mis-fed, corrupting the workspace once.
**Verification**: rustfmt invocation lines checked with `grep -nE 'rustfmt.*\.(toml|md|json|toml)'`, or the script asserts `${f##*.}` == rs.
**Blocking condition**: any rustfmt command whose file arguments include non-.rs.

### 21. team_create inline long JSON specs fail parsing easily — use a declarative config file (2026-09-28)
**Rule**: passing `team_create` an inline JSON object with a long prompt repeatedly produced the false error `requires exactly one of teamName or inline_spec` (it was actually a payload parse failure). **Correct path**: write `~/.omo/teams/<name>/config.json` (short prompt; detailed instructions are delivered via team_send_message), then `team_create({ teamName })`. Member instructions are always posted after creation, never stuffed into the spec.
**Verification**: `python3 -m json.tool ~/.omo/teams/<name>/config.json` — pass JSON validation first.
**Blocking condition**: forcing an inline spec over 10 lines; spec JSON not pre-validated.

### 22. Writing IR construction code from memory = ghost-API triple — grep the real signatures before writing (2026-09-28)
**Rule**: writing a generator without verifying the render IR's real signatures (PointMark/StrokeSeg color is actually `[f32;3]`, not a 4-element array; ShadowSetup's field group proj/view_rot/eye/light_dir/size/bias; DrawCommand has no Viewport variant; multiview goes through `render_to_pixels_rects`) → 3 rounds of compile errors + a whole-file rewrite. **Mandatory before writing**: `grep -n "impl <Type>" crates/**/contract.rs` to get the real signatures; copy construction shapes from existing test files (strokes/shadows/clip/multiview each have verified frame constructions); do not invent new generator modules (YAGNI).
**Verification**: construction code passing `cargo check` on the first try = copied correctly; 2 consecutive compile errors = stop and go grep.
**Blocking condition**: writing more than 30 new lines of IR construction without reading the corresponding type definitions first.

### 23. Root of pixel-predicate false greens: the predicate matches a string that is always present — positive self-proof must be a rename probe (2026-09-28)
**Rule**: the first version of the gate-abi hpp mirror gate `grep -q "$sym"` was permanently green (the C symbol name, as a forwarding-call string, is always in the hpp); switching to the method-name shape still had a false green (a semantic rename: destroy→destroy_now). **Discipline**: every new gate at commissioning must undergo a **breakage probe** — renaming/destroying the object under test must turn the gate red, with the report precisely naming the broken object (no collateral false alarms); after restoration it must turn green. Even when the direction is right, verify the report is right too.
**Verification**: run the manual rename probe once (proven this session: fly_state→fly_state_TMP went red and reported only fly_state).
**Blocking condition**: a new gate entering the repo without the breakage probe.

### 24. Self-proof samples must take lines from the real file — hand-made echo samples conjure indentation that doesn't exist (2026-09-28, N2.3)
**Rule**: when verifying whether a grep/predicate can hit, take the test line from the real file with `sed -n 'Np'`; hand-made echo is forbidden — when the N2.3 mirror-gate predicate was tightened, the hand-made sample carried indentation while the real file's `[[nodiscard]]` method sat at column zero; the single-line test MATCHed but the real run NO-MATCHed, and a round was wasted finding the root cause.
**Verification**: self-proof commands take the form `sed -n '165p' file | grep -qE ...` (line number from a real anchor).
**Blocking condition**: drawing conclusions from hand-made echo samples while debugging a predicate.

### 25. Semantic refactors must first check "time domain vs normalized" — sampler endpoint assertions are free domain guards (2026-09-28, N2.2)
**Rule**: for APIs with keyframe/time-domain parameters, the consumer example's trial run first writes a **domain assertion** (endpoint values equal verbatim) — E305's first run passed normalized t=1.0 into the 0..5.0 time-domain sampler and the endpoint assertion caught it on the spot (left=[0,-4,3] vs right=[0,0,0.5]), with zero leakage into the window surface. **The sampler's doc must state the domain** ("KEYFRAME TIME DOMAIN, not normalized").
**Verification**: the consumer example's prove() must contain an endpoint-identity assert.
**Blocking condition**: shipping an example for a new sampler / domain-parameter API without endpoint assertions.

### 26. A batch script's self-description must match its implementation; after a crash inspect the disk before resuming — don't assume which files were changed (2026-10-08, band V)

**Rule**: (1) In any codemod/batch script, every comment claiming "it checks X" must point to a specific line in the implementation; if you can't point to it, the check was never done —
the comment is a fake gate. Proven: I claimed that when pruning imports "names mentioned in comments don't count as used", but the implementation only stripped
`use` lines, so `Window` survived — it was caught by `clippy -- -D warnings`, not by the script. (2) Script errors/assert-death must happen **before writing to disk**,
and before resuming, first run `git diff --stat` + grep feature strings to confirm which files changed and which didn't; forbidden: "probably only the previous one got changed".
(3) Structural rewrites use tuple/AST parsing rather than literal matching; shapes the parser can't handle (nested tuple patterns `Some((g, mat))`,
non-`App` naming like `TileApp`) must be **explicitly rejected with the reason printed** — never silently skipped or position-guessed.

**Verification**: `pixi run lint` (-D warnings) as the real judge for pruning-type steps; the file count in `git diff --stat` must equal what the script printed.
**Blocking condition**: script comments describing checks the implementation doesn't have; or re-running a batch job after a crash without inspecting the disk.

### 27. Long markdown/script writes: the outer heredoc must not embed the same delimiter name; self-check with `bash -n` immediately on mistakes (2026-10-08 lesson-review, proven)

**Rule**: (1) If long lesson-review / memory-type content contains code fences, and inside those fences a **line-alone delimiter** such as `<<'PY'` or `PY` appears,
the outer heredoc is **closed prematurely** and the remaining text keeps being executed as commands by bash — proven once in this session
(python reported "EOF while scanning triple-quoted string literal", and the trailing lines were misread by the shell).
Correct approach: use an **outer delimiter that is unique and never appears in the content** (e.g. `<<'PYPART1'`), or **split into multiple commands** each writing one segment;
for large markdown blocks prefer plain `cat >> f <<'EOF'` (pure text, no nesting); use python only to change a middle segment.
(2) When a single command contains `f-string`/format strings, the placeholder count and the actual arguments must be **checked on the spot** (this session's `%-24s…%d`
had 4 placeholders but 6 arguments were passed → "not all arguments converted"; the red was in **my probe**, not in the object under test).
(3) Regex **group names must be ASCII only**: non-ASCII/sign-bearing group names like `(?P=j>\s*)` make re throw
`bad character in group name` directly; and any script must be **run once standalone** before being handed to the batch (that exception happened before touching disk = zero loss,
the toolchain cousin of #26's "assert before write").

**Verification**: `bash -n <the script you changed>`; after a heredoc, `git diff --stat` lists only the expected files;
`grep -c "<new-section feature string>" <target file>` == 1.
**Blocking condition**: embedding a heredoc inside a heredoc with the same delimiter in one command; or using a format string in a gate assertion without self-proving it first.

### 28. Re-anchor every check that greps a translatable literal, in the same commit as the translation (2026-10-12 E2 round)

**Rule**: a gate/check whose grep pattern embeds prose from a file owned by a translation/
refactor batch dies silently when that batch lands ("sentinel"/"stale artifact" replaced Chinese anchors):
the check still RUNS, greps NOTHING, and passes vacuously (PIT-43 class). Sweep order when a
batch edits prose: (1) grep the repo for literals that referenced the old text;
(2) re-anchor pattern AND target in one commit; (3) the gate file itself must not contain the
forbidden literals it scans (PIT-1 self-match -- build patterns from escapes, e.g.
"'\u975e\u76ee\u6807").

**Verification**: for each re-anchored check, run it once against the new anchor and expect
the documented pass; run `bash scripts/gate-i18n.sh` after editing any gate that scans CJK.

**Blocking condition**: landing a prose-translation batch in the same commit as its dependent
gates without re-running each gate verbatim; leaving a gate file carrying its own scanned literals.

### 29. Directory moves re-derive every CARGO_MANIFEST_DIR-relative depth (2026-10-12 T-H3.5)

**Rule**: when a crate moves to a shallower/deeper directory, `git grep` for the
crate NAME is not enough -- fixture/asset paths built from
`env!("CARGO_MANIFEST_DIR")` + `../` chains break by DEPTH, a class invisible to
name greps. After any crate move: grep `'CARGO_MANIFEST_DIR'` in the moved crate,
recompute each `../` chain from the new home, then run the crate's test button
button (the compile error surfaces only there).

**Verification**: `grep -rn 'CARGO_MANIFEST_DIR' crates/<moved>/` -- every
adjacent `../` count must equal the new depth; `cmake --build <tree> --target
<crate>-test` green.

**Blocking condition**: committing a crate move without the moved crate's
test-button leg executed.


### 30. Supplier logic changes need the build tree's INTERNAL caches flushed (2026-10-13)

**Rule**: after changing supplier logic that caches configure-time probes
by path/manifest key (CACHE INTERNAL), wipe the stale keyed entries in
every build tree before testing the change -- an empty cached value
short-circuits before the new code runs, and the fix reads as a no-op.

**Verification**: `grep -c polyorch_meta_srcs cmake-build/CMakeCache.txt`
(entries = stale keys); File API sources query on a mounted target.

**Blocking condition**: judging a supplier-mount fix by a single
reconfigure over a warm tree.

**Source**: PIT-53 (2026-10-13 ide_mount [[example]] round).
