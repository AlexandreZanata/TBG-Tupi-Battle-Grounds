# New Project Checklist

> Complete **before writing the first line of code**.
> Mirrors `agent-rules/AGENT-CORE-PRINCIPLES.md` checklist.
> If any item is blank, the agent **must ask** — never assume.

---

## Architecture and domain

- [ ] **Layers defined** — Interfaces, Application, Domain, Infrastructure documented
- [ ] **Entities and aggregates** — main aggregates identified with roots
- [ ] **Value Objects** — listed with invariants (invalid values fail at creation)
- [ ] **Business rules** — written in GIVEN/WHEN/THEN (`docs/` or linked files)
- [ ] **State machines** — valid transitions documented for multi-status entities
- [ ] **Access roles** — RBAC/ABAC matrix (action × role) defined
- [ ] **Domain events** — catalogued (past-tense names, payloads)
- [ ] **Use cases** — actor, preconditions, flows documented (`docs/use-cases/`)
- [ ] **API contract** — sketched (`docs/API-CONTRACT.md`)
- [ ] **Glossary** — domain terms defined (`docs/GLOSSARY.md`)

---

## Security (OWASP)

- [ ] **OWASP Top 10:2025** — threats mapped for this project (`agent-rules/03-security/README.md`)
- [ ] **Agentic 2026 (ASI01–ASI10)** — if using AI agents with tools (`agent-rules/03-security/OWASP-AGENTIC-2026.md`)

---

## Agent harness

- [ ] **Harness installed** — `agent-rules/`, `agent-harness/`, `.cursor/rules/`
- [ ] **AGENTS.md** — copied or linked for agent session start
- [ ] **Ponytail (static)** — `.cursor/rules/ponytail.mdc` via harness install ([MIT](https://github.com/DietrichGebert/ponytail))

---

## Testing (contract-first)

- [ ] **Policy read** — `agent-rules/04-testing/contract-first-tests.md` understood before any test
- [ ] **Unit tests** — domain rules, VOs, state machines (GIVEN/WHEN/THEN)
- [ ] **Integration tests** — use cases + adapters (in-memory / test DB)
- [ ] **E2E tests** — critical user journeys documented in `docs/use-cases/` (automated in CI)
- [ ] **CI** — all test layers run automatically on every PR
- [ ] **No mirror tests** — assertions from contract/spec, never copied from production code

---

## Sign-off

| Role | Name | Date |
|------|------|------|
| Product / domain | | |
| Tech lead | | |

When all boxes are checked, implementation may begin.
