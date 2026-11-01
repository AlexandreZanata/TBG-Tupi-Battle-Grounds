# Web landing and admin foundation — 2026-11-01

Branch `phase/web-landing-admin-foundation`, based on `origin/phase/backend-main-integration`
at `8be248e025ed8f56290d94604da75536726d765b` (protocol 31, contract `w31-r6-097385fee9a10b0f`,
schema 10, eight Arenas). Game rules, numbers, protocol and server code are unchanged.

## Owner scope

Create independent frontends for the landing (`tupibg.com`, `www` redirect) and a
foundation for the future admin suite (`admin.tupibg.com`), prepare a safe static deploy
with rollback and a separate Caddy include, and validate it, without publishing: visible
publication and any real administrative access wait for the owner's review. Desktop
browsers only. Owner correction during the work: battle visuals come from the local native
Linux client, the up-to-date visual reference.

## Delivered

**Implemented**
- `web/landing`: header, hero, pillars, how to play, 46 champions with a Faction filter, the
  eight Arenas and their modifiers, real native captures with a keyboard-accessible viewer,
  patch notes from the reviewed catalog, platforms, footer with the license credit; contact,
  privacy and terms drafts; 404 page; pt-BR and English; SEO/social metadata; strict CSP.
- Availability gates (`config/release.json`): browser game `pending`, Linux `pending`
  (the published `v0.1.1-rc.1` package is protocol 30 and the server is 31). Gates are
  re-validated in the browser and by `check.py` against tbg-core's protocol and contract.
- `web/admin/public`: static bilingual "in preparation" page for the admin host.
- `web/shared`: tokens, base components, safe DOM and language helpers.
- `web/scripts`: content sync from game sources, manifest-based staging, loopback preview
  server with production headers, static checker, native capture and media scripts.
- `web/deploy`: publisher with immutable releases, atomic activation, rollback, private Caddy
  backups, candidate validation before reload and a kill switch; Caddy include template.
- `deploy/Caddyfile.domains.template`: final `import tbg-web/*.caddy` so backend provisioning
  keeps the web hosts (empty glob is a warning on a fresh host).

**Demonstration (local preview only)**
- `web/admin/app`: console shell (sidebar, top bar, preview banner), Overview, Champions,
  Arenas, Matches and players, Updates and Settings; reusable cards, table, filters, gated
  actions, form fields and confirmation dialog; loading, error, empty and not-found states.
  Content panels read repository snapshots; operational panels use fictional fixtures
  (`Demo…`, `DEMO-…`); the overview reads the public `/health` and `/api/version` through
  the local preview proxy. No action can succeed (`ADMIN_AVAILABLE = false`).

**Pending (owner decisions or later work)**
- Publication: releases `20261101T194012Z-93648c8e9013` (landing, 115 files) and
  `20261101T194016Z-93648c8e9013` (admin placeholder, 16 files) are uploaded to the host and
  inspected (root-owned, 644/755, no hidden, secret or script files in the admin release) but
  **not active**; `activate … --confirm` and `--confirm caddy-apply` wait for the owner's
  direct approval (commands in `web/deploy/README.md`). Public hosts are unchanged.
- Flip `webGame` to `live` once the web build is published and verified on `app.tupibg.com`;
  offer Linux once a protocol-31 package is published with checksums.
- Server-side admin authentication/authorization:
  [WEB-ADMIN-AUTH-CONTRACT.md](../planning/WEB-ADMIN-AUTH-CONTRACT.md) (O-W1–O-W3).
- Legal review of privacy/terms (drafts are `noindex`), official contact channel.
- Owner look sign-off on the landing and the selection of captures.

## Decisions taken (routine, for review)

Proposed as one DECISIONS entry at consolidation (not added here to avoid numbering
collisions with concurrent branches):
1. Static HTML/CSS/ES modules with no build or npm dependency; repository art reused through
   manifests, so preview and release are the same files.
2. pt-BR authored in HTML (readable without JavaScript); dictionaries checked for parity.
3. Availability stays data-driven and fails closed; nothing is announced unverified.
4. Admin host stays a static placeholder until operator authorization exists on the server;
   the console foundation is never staged.
5. Web hosts live in their own Caddy include imported by the main file; API/app blocks,
   Cloudflare trust and private-route blocks are untouched.
6. Captures: English UI (the pt-BR native HUD still mixes English labels), 16:9 crop that
   removes the client's debug performance line; crop and resize only.

## Validation (work-front level; no Cargo, Trunk, suite, balance or load runs)

| Check | Result |
|-------|--------|
| `python3 web/scripts/check.py` (manifests, references, JSON, `node --check` on every module, i18n parity and HTML/pt-BR equality, release gates vs tbg-core, forbidden patterns, admin-public restrictions, content freshness) | passed: landing 115 files, admin-public 16, admin-preview 115 |
| `python3 web/scripts/sync_content.py --check` | up to date (46 champions, 7 Factions, 8 Arenas) |
| `shellcheck` on `web/deploy/*.sh` and `web/scripts/*.sh`; `bash -n` | clean |
| Chrome 156 via Playwright, 1440x900: landing pt/en, viewer, filter, privacy, terms, contact, 404, admin placeholder, six admin pages pt/en, gated action, empty, error (injected 503) and not-found states, confirmation dialog | no console/CSP errors or failed requests except the intended 404/503; single `h1`, alt on every image, named controls, labelled inputs, no duplicate ids or heading jumps |
| In-page gate tests (mocked config only) | published protocol-30 Linux build refused; foreign/HTTP URLs refused; matching checksummed asset accepted; restored config shows pending |
| Keyboard | filters keep focus after activation; viewer opens on Enter, closes on Escape, returns focus; visible focus ring |
| Caddy 2.10.2 offline (Docker, no network) | template+include, template without include, live file+import+include: valid; adapted JSON shows API/app routes and server settings identical; 404 handler scoped to the landing host |
| Caddy 2.10.2 functional run of the include | CSP/HSTS/nosniff/frame headers, cache policy, real 404 status, hidden dotfiles, no authoring files, `www` 301 with path and query, admin `noindex` and no scripts/data/`/health` |
| VPS Caddy 2.11.2 `publish.sh caddy-check` | candidate valid; live Caddyfile hash unchanged, no include installed, `/srv/tbg` untouched, Caddy and `tbg-server` active, API `ok`; private backup in `/var/backups/tbg-web` |

Native captures: 15 isolated runs of the installed client `0.1.1-rc.1-tupibg-8be248e`
(private Xvfb, throwaway HOME/data, no session D-Bus, closed loopback server); 14 succeeded,
the Evolution-panel capture timed out and was not used.

## Deferred, with reasons

- Mobile layout and touch checks: desktop-only owner rule.
- Public HTTPS checks of the new hosts and certificate issuance: they require `caddy-apply`,
  which waits for the owner's review.
- Lighthouse/axe audits: not installed locally; covered by the basic automated checks above.
- `deploy/deploy.sh` and backend validation: not affected beyond the template's final import
  line, validated with both Caddy versions.

## How to review

```bash
git log --oneline 8be248e..phase/web-landing-admin-foundation
git diff --stat 8be248e..phase/web-landing-admin-foundation
python3 web/scripts/check.py
python3 web/scripts/serve.py landing --port 8601
```
