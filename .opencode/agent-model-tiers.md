# VisiaEngine Model Tier System

> Five-tier model mapping: premium-max / premium / fast / vision / lite
> Dual-provider architecture: DeepSeek official direct + New API gateway aggregation
> The gateway host:port lives in local config (`.opencode/opencode.json` / env), not in this document; it is written below as `<internal-gateway-host:port>`.

## Architecture

VisiaEngine reaches large models through two independent paths:

| Provider | Access mode | Traits |
|--------|----------|------|
| **DeepSeek official** | Direct API (`api.deepseek.com`) | Low latency, high stability, model versions controlled officially |
| **New API gateway** | Alias proxy (`<internal-gateway-host:port>`) | Multi-vendor aggregation, automatic degradation, dynamic model switching |

Inside, the New API gateway aggregates vendors such as DeepSeek / Qwen / Kimi / Doubao / GLM / MiniMax,
implementing single-provider multi-model degradation. Changing the alias mapping on the gateway side switches
models globally with zero project-config changes.

The DeepSeek official direct path serves as the independent fallback when the New API gateway is unavailable,
forming a complete high-availability chain.

## The five model tiers

### Dual-provider comparison table

| Tier | Alias | Purpose | DeepSeek official | New API gateway | New API Fallback 1 | New API Fallback 2 |
|------|------|------|---------------|-------------|-------------------|-------------------|
| premium-max | deepest reasoning | most complex tasks | `deepseek-reasoner` | `deepseek-v4-pro-max` | `kimi-k2.6` | `minimax-m3` |
| premium | main workhorse | orchestration/build/planning | `deepseek-chat` | `deepseek-v4-pro` | `qwen3.7-max` | `glm-5.1` |
| fast | quick | execution/search/review | `deepseek-coder` | `deepseek-v4-flash` | `qwen3.6-flash` | `doubao-seed-2.0-lite` |
| vision | visual | multimodal analysis | `deepseek-vl2` | `doubao-seed-2.0-pro` | `qwen3.6-plus` | — |
| lite | lightweight | minimal/trivial tasks | `deepseek-chat` | `qwen3-32b` | `qwen3-8b` | — |

### New API alias mapping notes

The New API gateway references models by semantic aliases (`premium` / `fast` etc.); the OpenCode config
references aliases rather than concrete model names. Switching models means editing the alias mapping on
the gateway side only — no project config file changes.

Current gateway alias mapping:

```
premium-max       → deepseek-v4-pro-max        # flagship reasoning
premium-max-1     → kimi-k2.6                  # Fallback 1
premium-max-2     → minimax-m3                 # Fallback 2

premium           → deepseek-v4-pro            # primary reasoning
premium-1         → qwen3.7-max                # Fallback 1
premium-2         → glm-5.1                    # Fallback 2

fast              → deepseek-v4-flash          # fast reasoning
fast-1            → qwen3.6-flash              # Fallback 1
fast-2            → doubao-seed-2.0-lite       # Fallback 2

vision            → doubao-seed-2.0-pro        # visual primary
vision-1          → qwen3.6-plus               # Fallback 1
vision-2          → gemini-3.5-flash           # Fallback 2

lite              → qwen3-32b                  # lightweight tasks
lite-1            → qwen3-8b                   # Fallback
```

## Per-tier detailed notes

### premium-max: deepest reasoning

- **Purpose**: the most complex tasks, needing deep reasoning and multi-step thought
- **Use cases**: architecture design, system planning, complex refactors, technical review, security audit
- **DeepSeek official model**: `deepseek-reasoner` — focused on complex reasoning chains, long context
- **New API chain**: `deepseek-v4-pro-max` / `kimi-k2.6` / `minimax-m3` — flagship model chain
- **Temperature**: 0.2 (high determinism)
- **Reasoning Effort**: high
- **Notes**: highest latency, highest cost — use only for genuinely deep-reasoning scenarios

### premium: primary reasoning

- **Purpose**: daily complex tasks needing strong comprehension and generation
- **Use cases**: agent orchestration, build scripting, plan generation, code review, architecture consulting
- **DeepSeek official model**: `deepseek-chat` — DeepSeek's latest chat model, well-rounded capability
- **New API chain**: `deepseek-v4-pro` / `qwen3.7-max` / `glm-5.1` — primary reasoning chain
- **Temperature**: 0.2 (high determinism)
- **Notes**: the most-used tier in the project; balances capability against cost

### fast: quick execution

- **Purpose**: simple execution tasks demanding low latency and high throughput
- **Use cases**: code search, file exploration, small code edits, quick reviews, library search
- **DeepSeek official model**: `deepseek-coder` — coding-optimized model, fast response
- **New API chain**: `deepseek-v4-flash` / `qwen3.6-flash` / `doubao-seed-2.0-lite`
- **Temperature**: 0.0 (fully deterministic)
- **Notes**: the tier most latency-sensitive; no complex reasoning — for high-frequency small tasks

### vision: visual specialist

- **Purpose**: multimodal understanding, image/PDF content analysis
- **Use cases**: UI screenshot analysis, document scanning, diagram comprehension, visual review
- **DeepSeek official model**: `deepseek-vl2` — vision-language model
- **New API chain**: `doubao-seed-2.0-pro` / `qwen3.6-plus`
- **Temperature**: 0.1 (highly deterministic)
- **Notes**: use only when multimodal capability is required; pure-text tasks should use other tiers

### lite: lightweight

- **Purpose**: minimal tasks, chasing extreme cost-efficiency
- **Use cases**: simple Q&A, log summarization, content formatting, metadata generation
- **DeepSeek official model**: `deepseek-chat` (lightweight calls)
- **New API chain**: `qwen3-32b` / `qwen3-8b`
- **Temperature**: 0.2
- **Notes**: limited capability — for highly deterministic tasks needing no creativity

## Agent tier assignment

### Agents

| Agent | Tier | Role |
|-------|------|------|
| oracle | premium-max | architecture consulting, technical decisions |
| sisyphus | premium | main coordination agent |
| hephaestus | premium | build management |
| prometheus | premium-max | plan generation |
| atlas | premium | implementation execution |
| librarian | fast | library/docs search |
| explore | fast | code exploration |
| metis | fast | metrics and data analysis |
| momus | premium | review and critique |
| sisyphus-junior | fast | simple execution |
| multimodal-looker | vision | visual analysis |

### Categories (task() dispatch)

| Category | Tier | Role |
|------|------|------|
| visual-engineering | premium | visualization engineering |
| ultrabrain | premium-max | ultra-deep-thinking tasks |
| deep | premium | deep analysis |
| unspecified-high | premium | high-complexity unclassified |
| artistry | fast | creative / document writing |
| quick | fast | simple tasks |
| unspecified-low | fast | low-complexity unclassified |
| writing | fast | documentation writing |

## Selection guide

### Which tier for what

| Task type | Recommended tier | Backup |
|----------|----------|------|
| architecture design / system review | premium-max | premium |
| agent orchestration / planning | premium | fast (simple steps) |
| writing code / refactoring | premium | fast (small changes) |
| file search / grep | fast | — |
| bug debugging | premium | premium-max (hard-to-reproduce bugs) |
| UI screenshot analysis | vision | premium (no vision need) |
| log summary / formatting | lite | fast |
| code review | premium | fast (simple formatting review) |
| security audit | premium-max | premium |
| simple Q&A | lite | fast |

### Quick judgment

1. **Needs deep thinking?** → premium-max
2. **Needs synthesis and coding?** → premium
3. **Simple execution or search?** → fast
4. **Needs to see images/PDFs?** → vision
5. **Doable in a few sentences?** → lite

## Provider selection guide

### When to use DeepSeek official

- When the New API gateway is unreachable (network isolation / VPN down)
- When lowest latency is needed (direct saves one hop vs the gateway)
- When DeepSeek official has released a newer model the gateway hasn't synced
- When DeepSeek-official-exclusive features are needed (e.g. `deepseek-reasoner`'s detailed reasoning traces)

### When to use the New API gateway

- Daily development (the default path)
- When multi-vendor automatic degradation is wanted (a timed-out model switches to the next)
- When capabilities of non-DeepSeek models are needed (e.g. Doubao's vision, Kimi's long context)
- When the gateway already holds the optimal model mapping — no need to care about the concrete vendor

### Preference order

```
New API gateway → automatic degradation → DeepSeek official direct
```

OpenCode's `runtime_fallback` mechanism handles the logic above automatically:
1. First try the New API gateway alias (e.g. `new-api/premium`)
2. On failure try the New API fallback models (`new-api/premium-1`, `new-api/premium-2`)
3. After all fail, switch the config manually to a DeepSeek official model ID

## Fallback chain

### Agent-level fallback

Each agent defines a 3-level fallback in `oh-my-openagent.jsonc`:

```jsonc
"oracle": {
  "model": "new-api/premium-max",       // primary
  "fallback_models": [
    "new-api/premium-max-1",            // Fallback 1
    "new-api/premium-max-2"             // Fallback 2
  ]
}
```

### Runtime fallback

The `runtime_fallback` global config handles network-level errors:

| Setting | Value | Meaning |
|--------|-----|------|
| retry_on_errors | 402, 429, 500, 502, 503, 504 | HTTP status codes that trigger the fallback |
| max_fallback_attempts | 2 | at most 2 degradation attempts |
| cooldown_seconds | 60 | 60-second cooldown after failure |
| timeout_seconds | 60 | per-request timeout |

### Fallback data flow

```
user request
  │
  ▼
Agent primary model (new-api/premium)
  │  ├─ success → return result
  │  └─ failure → retry (max 2)
  │       │
  │       ▼
  │   Fallback 1 (new-api/premium-1)
  │     ├─ success → return result
  │     └─ failure → retry
  │          │
  │          ▼
  │      Fallback 2 (new-api/premium-2)
  │        ├─ success → return result
  │        └─ all failed → gateway unavailable, switch to DeepSeek official
  │             │
  │             ▼
  │         DeepSeek official (deepseek-chat / deepseek-reasoner)
  │           └─ success/failure → return result or report error
```

## Adding a new model

1. Confirm the model's availability on DeepSeek official and get the API endpoint
2. Register the new model channel and alias mapping on the New API gateway
3. Update this document's mapping tables
4. The OMO config needs no changes (it references aliases)

## Notes

- DeepSeek official model IDs may change with official updates; `api.deepseek.com` docs are the authority
- The New API alias mapping is maintained by the gateway admin; project members need not care about the concrete mapping
- Switching to DeepSeek official direct requires the API Key configured in the environment
- Visual tasks should prefer New API (Doubao's vision beats DeepSeek official)
- Low-cost tasks on the lite tier save measurably in token spend
