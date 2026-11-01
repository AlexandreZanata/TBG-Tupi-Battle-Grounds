# Admin foundation

There is no server-side administrative authorization yet
([contract](../../docs/planning/WEB-ADMIN-AUTH-CONTRACT.md)). This folder therefore holds two
separate things:

| Folder | Status | Where it may run |
|--------|--------|------------------|
| `public/` | **Implemented.** Static bilingual "in preparation" page; no script, form, data or API. | `admin.tupibg.com` (via `web/deploy`) |
| `app/` | **Demonstration.** Console shell for review: navigation, pages, components, states. | `127.0.0.1` only; `stage.py` refuses it |

`check.py` fails if `public/` ever ships a script, JSON, form, input or fetch.

## Preview

```bash
python3 web/scripts/serve.py admin-preview --port 8603 --public-status https://api.tupibg.com
```

Without `--public-status` the overview shows the public-status card as disabled.

## Structure of `app/`

| Path | Responsibility |
|------|----------------|
| `assets/js/main.js`, `shell.js`, `router.js` | Frame (sidebar, top bar, preview banner), hash routes, loading/error/not-found |
| `assets/js/pages/*.js` | Overview, Champions, Arenas, Matches and players, Updates, Settings |
| `assets/js/ui/*.js` | Reusable pieces: states, cards/panels with data-source badges, table, filter bar, gated actions, form fields, confirmation dialog |
| `assets/js/api/http.js` | Same-origin GET with timeout and size limit; never sends credentials |
| `assets/js/api/public-status.js` | Read-only `/health` and `/api/version` (public routes, not admin access) |
| `assets/js/api/admin.js` | Boundary for future admin endpoints: `ADMIN_AVAILABLE = false`, every request throws |
| `assets/js/data/content.js` | Repository snapshots (champions, Arenas, patch catalog), read-only |
| `assets/js/data/fixtures.js` | **Fictional** matches/players/activity (`Demo…`, `DEMO-…`) for layout only |
| `assets/js/strings.js`, `i18n.js` | pt-BR/English interface strings |

Every panel names its data source (repository, demonstration, public status). Action
buttons stay focusable but explain why they are unavailable; the confirmation dialog can
never enable its confirm button while `ADMIN_AVAILABLE` is false. No login, password, token
or secret exists anywhere in this code, and nothing reaches production except the two public
status reads through the local preview proxy.

## Planned extensions (documented, not implemented)

| Area | Intended scope | Depends on |
|------|----------------|-----------|
| Content | News/notice drafts, preview, publish/withdraw with review | `content.*` capabilities, audit |
| Balance | Read balance-lab reports and proposals; values change only via D-33 evidence and a reviewed PR | `balance.view`; never a write path |
| Moderation | Reports and reversible community restrictions with reasons | `moderation.*`, audit; ranked corrections stay the D-89 CLI |
| Permissions | Named operators and grants | Host maintenance CLI only |
| Audit | Read-only trail of operator actions | `audit.read` |

Turning any area on means implementing its server routes under the contract, then replacing
the matching fixture or gated action behind `api/admin.js`.
