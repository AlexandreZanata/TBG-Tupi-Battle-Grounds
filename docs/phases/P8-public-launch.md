# P8 — Public MVP launch

**Depends on:** P7 reviewed; owner approved balance and look.
**Branch:** `phase/p8-public-launch`.
**Difficulty:** low–medium; the security review (step 2) is done by the
reviewer agent, not the executor.

## Goal

Anyone with the URL can play a fair online match on desktop or phone, and
the owner can operate the server alone.

## Deliverable

Public URL announced; license in place; security review done; off-site
backups; uptime monitoring; final capacity numbers.

## Read first (only these)

- `AGENTS.md`
- `docs/ARCHITECTURE.md` §9–§12
- `deploy/README.md`
- `docs/phases/P5-real-environment.md` (runbook R1–R13)

## Do NOT

- Add accounts, cosmetics, ranking, or new game modes.
- Change gameplay numbers (P7 settled them).

## Steps

1. **Dependency audit.** Run `cargo audit` (install with
   `cargo install --locked cargo-audit` if missing). Update crates with
   critical advisories when a compatible fix exists; list the rest in the
   report. Produce a license list:
   `cargo tree -e normal --prefix none --format "{p} {l}" | sort -u` →
   summarize unusual licenses (anything not MIT/Apache-2.0/BSD/ISC/Zlib/
   Unicode/CC0) in the report.
2. **Security review (reviewer agent).** Re-check ARCHITECTURE §11 and the
   P4 limits against the code; `ssh <host> 'sudo ufw status; sudo ss -tlnp'`;
   confirm `/metrics` is not public; confirm no secrets in git history
   (`git log -p | grep -nE "<host>|<domain>|BEGIN .*PRIVATE KEY"` prints nothing).
3. **License (owner).** The owner adds the root `LICENSE` file (MIT is
   intended). The executor then updates the README "License" section.
4. **Off-site backups.** Extend `deploy/backup.sh` with an optional
   `TBG_BACKUP_REMOTE` (an `rsync` or `scp` target the owner provides, set
   in the server environment, never in git) that receives the newest
   backup weekly (Sunday). Test one restore from the off-site copy.
5. **Uptime monitoring (owner).** Register `https://<domain>/health` in a
   free external uptime checker with alerts to the owner's email or phone.
6. **Final capacity run.** Repeat P5 R6 on the final build (with the bots
   drop-in, removed afterwards). Update ARCHITECTURE §10 "Measured" values.
7. **Public smoke.** Repeat P5 R1–R5, R7–R9, R11 on the final build. Two
   people on different networks finish three matches.
8. **Release.** Tag `v0.1.0` on `main` (`git tag -a v0.1.0 -m "Public MVP"`);
   the owner pushes the tag and announces the URL.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass
- [ ] `cargo audit`: no unaddressed critical advisory
- [ ] Security review signed off by the reviewer
- [ ] `LICENSE` present; README updated
- [ ] Off-site backup restored once successfully
- [ ] Uptime checker configured
- [ ] Capacity and smoke results recorded
- [ ] Tag `v0.1.0` created

## Reviewer checklist

- Every item above has evidence in the report.
- No secrets, IPs, or domains committed.
- After launch, the "After the MVP" list in ROADMAP.md is the next planning input.

## Phase report

*(Executor fills this in at the end.)*

- Commands run and results:
- Audit and license summary:
- Capacity (final):
- Smoke results:
- Deviations:
- Blocked items:
