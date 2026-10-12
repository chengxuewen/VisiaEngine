# Lesson Memory & Self-Correction Rule

> When AI encounters mistakes, new problems, or new lessons, **proactively** write the lessons into the project memory and rule files.

## Triggers (any one triggers)

| Trigger scenario | Action |
|----------|------|
| Compile/build fails after editing code | Record to `pitfalls.md` + update `edit-safety.md` |
| The same error type appears a 2nd time | **Immediately** add a rule to `edit-safety.md` or create a new rule file |
| Community solution already exists | Record to `pitfalls.md` (with community link) + update `decisions.md` |
| Tests pass but feature doesn't work | Record to `pitfalls.md` + update `testing.md` to add E2E requirement |
| Dependency version conflict | Record to `pitfalls.md` + update `edit-safety.md` |
| Patch/script breaks build artifacts | Record to `pitfalls.md` + update `edit-safety.md` |
| User points out an issue AI didn't find | Record to `pitfalls.md` + analyze root cause + add prevention rule |
| Root cause only located after >1 failed attempts | Record to `pitfalls.md` (with failed attempts + correct path) |
| User corrects approach/preference | Record to `conventions.md` or `decisions.md` |

## Write Targets

| File | Content to write | Format |
|------|---------|------|
| `.agents/memorys/pitfalls.md` | problem + cause + solution + verification + forbidden | five-part (PIT-{n}) |
| `.agents/memorys/decisions.md` | architecture/technical decision + rationale + reference | `## D{N}: title` |
| `.agents/memorys/conventions.md` | development constraint/user preference | C{n}: "constraint" + check command |
| `.agents/rules/common/edit-safety.md` | runnable check rule | `### N. rule name` + check command + blocking condition |
| `.agents/rules/common/testing.md` | test coverage requirement | test type + execution command + pass criteria |

## Write Timing

1. **Write immediately**: write at the moment the problem is found, don't wait until the session ends
2. **Verify after fix**: after writing, verify the rule is executable (has concrete commands, not empty talk)
3. **Deduplication check**: before writing, `grep` the target file to confirm it's not duplicated

## Rule Quality Standards

| Standard | Description |
|------|------|
| **Executable** | Contains concrete shell command or check step, not "be careful about X" |
| **Verifiable** | Has clear pass/fail criteria |
| **Has blocking** | Clearly states what to do if it fails |
| **Has precedent** | References an actually occurring error |

## Anti-Patterns (forbidden)

| Anti-pattern | Correct approach |
|--------|---------|
| "be careful later" | Write concrete check commands into the rule file |
| Record without prevention | Every pitfall must map to one runnable rule |
| Rule says "be careful when editing" | Write `grep -c '{' file && grep -c '}' file` |
| Wait for user reminder before recording | Record as soon as found (C9), don't wait for user to point it out |

## Capture Checklist (ask yourself on every trigger)

1. **What went wrong?** (symptom)
2. **Why did it go wrong?** (root cause)
3. **What's the right way?** (solution)
4. **How to prevent it?** (check command / constraint)
5. **Where does it go?** (per write target table)

## Trivial-Fix Escape Hatch

1-line fix / no branch / no side effects → 1-line capture, no full template needed.

## Pitfall Template (must be included when writing to pitfalls.md)

```markdown
## PIT-{n}: Title (date)
- **Symptom**: [symptom description]
- **Root cause**: [root cause analysis]
- **Solution**: [correct approach]
- **Verification**: [check command]
```

## Prevention Escalation (high-cost/frequent lessons)

- Repeats ≥2 times or takes >3 rounds to locate → add CI gate or pre-commit hook
- Low frequency/low cost → only record to pitfalls.md
