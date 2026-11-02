# P75 execution prompt — administration, moderation, key recovery and match social

Copy the text between the rules to the executor. Code, comments, docs and
commits stay in English; talk to the owner in Portuguese.

---

Execute **P75 — Administration, moderation, email key recovery and match
social** from `docs/phases/P75-admin-moderation-and-match-social.md`.
Before editing, read `AGENTS.md`, `docs/VALIDATION.md`, the phase file and
its companions in `docs/planning/`: `P75-REQUIREMENTS.md`,
`P75-CONTRACTS.md`, `P75-API.md`, `P75-SCHEMA.sql`, `P75-OPERATIONS.md`.

**Base.** Always start from GitHub main: `git fetch origin`, then create
`phase/p75-admin-moderation-social` from `origin/main` in a new worktree
under `/data` (the owner's main checkout is stale and has uncommitted work;
never use or modify it). If `origin/main` does not yet contain the P75
planning documents, merge `origin/docs/p75-admin-moderation-plan` first.
Check existing branches/worktrees and open PRs; never overwrite another
front (especially `codex/native-settings-updates` and the P65 branch).

**What to build.** Required email at registration with code verification;
"Lost your key?" in game that emails a code to the verified address and
rotates the key **on the same account** — every account-bound row stays
identical, the old key and all sessions die, the new key is shown once;
mail through SMTP with a secret-free outbox and a single dispatcher; a
separate Owner operator created from `TBG_ADMIN_BOOTSTRAP_EMAIL` with no
key, claimed by an emailed code into a single `AdminKey`, recoverable the
same way; the operator console on `admin.tupibg.com/admin-api/*` (loopback
listener per slot) with bans, chat/emoji restrictions, session revocation,
email blocks, reports, emoji review, blocked terms, kill switches,
moderators and audit; Report account from Result/history/profile/directory;
protocol-39 match chat and emojis (3 slots, 16 standard, moderated custom
PNG/GIF/WebP normalized by the server); the Bevy client surfaces (Linux
reference, identical Windows and desktop web) and the web console. Follow
the steps S0–S14 in order and the exact shapes, limits and defaults in the
companions. Record owner answers to O-P75-01…13 at S0 or use the defaults.

**Contracts.** Implement every immutability contract IC-01…IC-20 and
security case S01–S17 as behavioral tests in the named append-only
`contract_p75_*` targets, the `contract_protocol` and `schema_upgrade`
appends. Fix the `migrate_history` `MAX(version)` gate before adding schema
12. Keep the simulation untouched (`contract_determinism` unchanged). Amend
BR-P01/BR-P02 only in the same commits as their contract-test changes.
Keep secrets and emails out of `Config`'s Debug output, logs, metrics and
plaintext storage. Add a reason comment for every new dependency (lettre
with rustls/ring, image with png/gif/webp, the native file picker).

**Validation.** Owner minimal policy: per step format the affected crate
and at most one cached narrow `cargo check`, always as
`flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 …`;
busy lock or cold cache = deferred, never wait, poll or bypass. Use your own
`CARGO_TARGET_DIR` under `/data` (shared targets link stale crates). Write
tests per step and run the union once at S14. No full suite, release build,
benchmark, balance gate, VPS access, DNS change, real email, real secret or
production migration. Native success never proves the browser; record every
unobserved acceptance item as pending.

**Publication (authorized by the owner sending this prompt).** Commit each
step (`feat(p75): …`), push the branch normally and open a **Draft** PR to
`main` titled `feat(p75): administration, moderation, email key recovery and match social`.
Write the description to a file and use `gh pr create --draft --body-file`.
List behavior, contracts and their tests, real command output, screenshots
of new surfaces, migration/rollback notes, the attended rollout from
`P75-OPERATIONS.md` and every pending gate. Never force-push, rewrite
history, push to main, merge, enable auto-merge or deploy. Fill the phase
report, return the PR URL, branch, final commit, checks run and pending
items, then stop for review.

---
