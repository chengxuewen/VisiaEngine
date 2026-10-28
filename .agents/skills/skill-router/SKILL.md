---
name: skill-router
description: "Analyze user intent and output a recommended skill list. Auto-activates when the user says '怎么做' or '用什么技能', or when intent is ambiguous. Use when the user asks 'what skill should I use', 'how to approach this', or when intent is ambiguous."
---

# skill-router — Skill Routing Analysis

> When user intent is vague or several skills compete, analyze and recommend the best skill combination.

## Trigger Conditions

- The user says "怎么做" / "用什么技能" / "帮我选择" (how-to / which skill / help me choose)
- User intent is ambiguous and needs clarifying before recommending
- Multiple skills could apply and a comparative choice is needed

## Analysis Flow

### Step 1: Intent Classification

Read the user message and classify it:

| Class | Keywords | Default recommendation |
|------|--------|---------|
| **Implement** | "添加"/"实现"/"创建"/"修改" | `/think-before-act` + `/incremental-implementation` |
| **Fix** | "修复"/"bug"/"错误"/"不工作" | `/test-harness` + `/think-before-act` |
| **Refactor** | "重构"/"优化"/"简化"/"清理" | `/think-before-act` + `/code-simplification` |
| **Design** | "设计"/"架构"/"方案"/"怎么做" | `/think-before-act` + `/openspec-propose` |
| **Adjudication** | "裁决逐项过"/"一个个来"/"不要一起列出"/"walk the options one by one" | `/adjudication-walkthrough` |
| **Test** | "测试"/"E2E"/"覆盖率" | `/test-harness`（Web 面再加 `/browser-testing`） |
| **Docs** | "文档"/"README"/"说明" | `/doc-audit` |
| **Security** | "安全"/"权限"/"认证" | `/security-hardening` |
| **Explore** | "调研"/"对比"/"有什么方案" | `/ecosystem-scan` + the host's own subagent mechanism |
| **Ambiguous** | no clear keyword | propose 2-3 possible directions and let the user choose |

### Step 2: Context Check

Before recommending, check the current state:

1. **Which tasks are in_progress?** — if tasks are already running, only recommend skills related to the current task
2. **Which skills did the user recently use?** — avoid re-recommending one just used
3. **Current project phase?** — new feature vs fix vs refactor; each phase gets different recommendations
4. **Tech-stack match?** — ensure the recommended skill fits the current stack (Rust/TS/Docker/Web)

### Step 3: Output the Recommendation

**Single-skill case** (intent clear):
```
Recommendation: /test-harness
Reason: two consecutive fix commits — reproduce the failure in a test before changing code
```

**Multi-skill case** (a combination is needed):
```
Recommended combination:
1. /think-before-act — investigate options first, avoid blind edits
2. /incremental-implementation — one thin slice at a time, tests with each slice
Reason: non-trivial multi-module change that needs test cover
```

**Ambiguous case** (needs clarifying):
```
Intent unclear; possible directions:
1. Implementing a new feature → /think-before-act + /incremental-implementation
2. Fixing a bug → /test-harness + /think-before-act
3. Refactoring → /code-simplification
What specifically do you want to do?
```

### Step 4: Execution Suggestion

After the recommendation, if the user confirms, invoke the skill directly:
```
Confirmed: loading the /think-before-act skill...
```

## Relation to context-engineering

- **context-engineering**: routes **language rules** to modules (which `.agents/rules/<lang>/` files apply to which files); it does NOT pick skills.
- **skill-router**: the single source for **task → skill** classification (this skill).

Boundary: if the question is "which rules apply to this file/language", use `context-engineering`; if it is "which skill should I use", use this skill.

## When NOT to Recommend

| Scenario | Don't recommend | Reason |
|------|--------|------|
| The user is mid urgent-fix | any skill | don't interrupt; discuss after the fix |
| A 1-line change | /think-before-act | overkill (using a sledgehammer to crack a nut) |
| The user explicitly named a skill | other skills | respect the user's choice |
| The same skill was just recommended | the same skill | anti-harassment |
