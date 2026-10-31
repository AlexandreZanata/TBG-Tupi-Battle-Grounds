# P64 — Ready-to-send executor prompt

Copy the following prompt to the implementation agent. English is used
here to follow repository documentation policy; conversation may be Portuguese.

```text
Implement phase P64 — complete English/pt-BR presentation in Tupi Battle
Grounds. This is an implementation task. At completion, push the pertinent
commits and open or update a Pull Request targeting main.

Repository: https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds
Planning branch: codex/p64-i18n-completeness-plan

Read the current AGENTS.md and docs/VALIDATION.md, then:
- docs/phases/P64-i18n-completeness.md
- docs/planning/P64-I18N-AUDIT.md
- docs/planning/P64-I18N-SOURCE-MANIFEST.csv
- the P41/read-first documents listed by P64.

Start from current main in an isolated worktree on branch
codex/p64-i18n-completeness. Bring in the necessary planning documents
without overwriting later changes. Preserve all existing work. Refresh the
audit against current main and the actually integrated native shell;
never assume another branch has landed or old line numbers still apply.

Complete the steps in P64's order. Fix content resolvers and consumers:
champions/abilities/forms/status, Augments, Arcanas, Strongholds/Fortification,
battle, accounts, queue/challenge, results/rematch, ranking/API profiles,
standalone ranking routes and the active native desktop UI.

Catalog parity already passes on the audited baselines. The main problem
is text bypassing the catalogs. Reuse existing Lang/Msg/TBG_I18N and move
native bilingual words() strings into checked keys. Preserve legitimately
already-localized surfaces and document technical/proper-name exceptions.

Keep stable IDs, enum values, API keys/codes, storage/credentials, usernames,
assets, gameplay values and outcomes unchanged. Localize their presentation.
Derive numeric parameters from canonical data; use full parameterized
sentences. Do not translate the core mechanics tables or persisted records.
Map API/error/result/status codes at the client, with safe localized
fallbacks. Keep translated/API text safely rendered, never interpolated HTML.

Switching EN/PT while a view is open must refresh cached/dynamic text,
including unchanged match views, without losing drafts, focus, selections,
scroll or progress. Cover async switch ordering, multiple number precisions,
plurals/dates, accessibility attributes, fonts and standalone-page CSP.

Use minimal per-front validation and deduplicated focused tests at
consolidation. Every compiling check uses the shared nonblocking host lock
and at most two jobs. Do not automatically run the full suite, cold builds,
benchmarks, balance gates or mobile QA. Record actual commands/results and
deferred checks honestly. Update P41/P64 reports and every applicable audit
row; don't infer completion from key counts or a lexical scan alone.

Make routine technical choices autonomously. Request owner review only for
real unresolved naming/product choices or subjective language/visual approval;
continue independent work meanwhile. Code/comments/docs/commits are English;
Portuguese product strings belong in the translation catalogs. Speak with
the owner in Portuguese.

Explicit GitHub authorization for this execution: publish the implementation
branch with a normal push and open/update its PR with base main. This
specific instruction overrides AGENTS.md's general no-push rule. No
force-push, history rewrite, PR merge, production migration, installation or
deploy is authorized. Include only pertinent code/tests/docs, never secrets,
builds, credentials or changes from other fronts.

PR description: behavior fixed, affected surfaces/content/API adapters,
validation actually run, deferred checks, reviewed exceptions and any owner
decisions still needed. If mandatory scope remains blocked, publish a Draft
PR that names the gap rather than marking partial work complete.

Finish with the PR URL targeting main, delivered scope, test results and
remaining limitations. Begin now with P64 Step 0 and proceed to implementation.
```
