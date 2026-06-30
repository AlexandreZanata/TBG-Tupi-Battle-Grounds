# AGENTS.md — Universal Entry Point for Coding Agents

> **Read this first** in any new agent session (Cursor, Claude Code, Codex, Windsurf, etc.).
> This repository is an **Agent Harness** — rules and tooling for AI-assisted projects. It is **not** an application scaffold.

**Language:** 100% English — code, comments, docs, commits, and all agent output.

---

## What this repo is

| Is | Is not |
|----|--------|
| Modular rule library for coding agents | Ready-to-run app template |
| Installable harness for any project | Domain-specific product code |
| OWASP 2025 + Agentic 2026 security rules | Replacement for your project glossary |

When rules conflict with existing code, **rules prevail** — unless the user explicitly overrides for a task.

---

## Rules path (resolve first)

```bash
pip install -r harness/requirements.txt   # once per machine
./harness/rules-path.sh
```

| Context | Config file | `rules_dir` |
|---------|-------------|-------------|
| This harness repo | `harness/harness.config.yaml` | `rules/` |
| After `install.sh` in another project | `agent-harness/harness.config.yaml` | `agent-rules/` |

Paths are relative to **project root**. Never hardcode `rules/` if `rules-path.sh` returns `agent-rules/`.

---

## Always load (base context)

Read these files at session start or before non-trivial work:

1. `{rules_dir}/AGENT-CORE-PRINCIPLES.md` — architecture contract
2. `{rules_dir}/00-core/size-and-complexity-limits.md` — **80 lines/function, 200 lines/file, cyclomatic ≤10**
3. `{rules_dir}/04-testing/contract-first-tests.md` — **read before ANY test** (unit, integration, E2E)
4. `{rules_dir}/09-ai-agent-specific/token-economy.md` — load less, execute better
5. `{rules_dir}/09-ai-agent-specific/anti-hallucination.md` — verify before assert

Cursor users: `.cursor/rules/*.mdc` applies automatically (`alwaysApply`), including Ponytail YAGNI rules.

### Ponytail (YAGNI / minimal implementation)

Static rules inspired by [Ponytail](https://github.com/DietrichGebert/ponytail) (MIT) — no plugin or proxy. Always on in Cursor via `.cursor/rules/ponytail.mdc`.

```bash
./harness/resolve-rules.sh yagni minimal ponytail
```

Harness security and TDD rules **override** Ponytail when they conflict. Attribution: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

---

## Conditional load (task-specific)

Load **2–6 files only** — not the entire rule tree.

```bash
./harness/resolve-rules.sh <keywords from task>
# Installed project: ./agent-harness/resolve-rules.sh <keywords>
```

| Task | Example keywords |
|------|------------------|
| API endpoint | `api endpoint auth validation contract` |
| Security review | `owasp security authz bola agentic` |
| Domain feature | `domain layer state event` |
| Bug fix | `bugfix regression error` |
| Performance | `query cache n+1` |
| **Tests (any type)** | `test contract unit integration e2e` — **read contract-first-tests.md first** |

Match rule file `triggers:` in YAML frontmatter, or use output from `resolve-rules.sh`.

### Cursor: task-scoped rule file (optional)

Generate a temporary `.mdc` so Cursor surfaces the resolved rules for this task:

```bash
./harness/generate-task-rules.sh api endpoint auth
# Installed project: ./agent-harness/generate-task-rules.sh api endpoint auth
```

Creates `.cursor/rules/_task-active.mdc` (`alwaysApply: false`). **Delete when done:**

```bash
./harness/generate-task-rules.sh --clean
# or: rm .cursor/rules/_task-active.mdc
```

**Index:** `{rules_dir}/STRUCTURE.md`  
**Manifest:** `{rules_dir}/manifest.yaml`  
**Security map:** `{rules_dir}/03-security/README.md`

---

## Agent protocol

1. Run `rules-path.sh` → know `{rules_dir}`.
2. Identify task keywords → run `resolve-rules.sh`.
3. State which rule files you loaded (brief list).
4. **ASK** if AGENT-CORE-PRINCIPLES checklist items are blank — never assume business rules.
5. Smallest diff; one logical change per commit.
6. **Before any test:** read `contract-first-tests.md`; assert contract, never mirror code.
7. Verify after each edit — do not claim tests passed without running them.
8. English only in all artifacts.

---

## Install harness in another project

```bash
git submodule add https://github.com/AlexandreZanata/GoodPraticesForLLMSandAgents.git .agent-harness
./.agent-harness/harness/install.sh . --symlink
```

Then use `./agent-harness/rules-path.sh` and `./agent-harness/resolve-rules.sh`.

Full install docs: [harness/README.md](harness/README.md)

---

## Key references

| Document | Purpose |
|----------|---------|
| [rules/AGENT-CORE-PRINCIPLES.md](rules/AGENT-CORE-PRINCIPLES.md) | Domain architecture contract |
| [rules/04-testing/contract-first-tests.md](rules/04-testing/contract-first-tests.md) | **Mandatory before any test** — contract, not mirror |
| [rules/04-testing/test-pyramid.md](rules/04-testing/test-pyramid.md) | Unit 75% / integration 20% / E2E 5% automated |
| [rules/00-core/size-and-complexity-limits.md](rules/00-core/size-and-complexity-limits.md) | Universal size/complexity caps |
| [rules/STRUCTURE.md](rules/STRUCTURE.md) | Full rule tree + task mapping |
| [rules/03-security/OWASP-TOP10-2025.md](rules/03-security/OWASP-TOP10-2025.md) | Web/API security (A01–A10) |
| [rules/03-security/OWASP-AGENTIC-2026.md](rules/03-security/OWASP-AGENTIC-2026.md) | Agentic AI security (ASI01–ASI10) |
| [harness/README.md](harness/README.md) | Install, resolve, maintenance |
| [rules/09-ai-agent-specific/minimal-implementation.md](rules/09-ai-agent-specific/minimal-implementation.md) | Ponytail YAGNI ladder (MIT attribution) |
| [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) | Third-party licenses |
| [README.md](README.md) | Human-oriented project overview |

---

## Optional local overrides

Project-specific rules not in harness: `.local/overrides/` (gitignored). Harness rules still apply unless user explicitly waives.
