# P75 operator console

`app/` is the real operator console. `public/` remains the no-script fallback. The `admin-app` site may be staged for the attended P75 rollout; the automatic static package continues to include `admin-public` only. Staging does not activate the console.

The app uses `/admin-api/v1/*` on the same origin, HttpOnly host-only sessions, a CSRF header for every mutation, reasoned actions and text-only DOM construction. Pages cover claim/recovery, sign-in with emailed code, accounts, reports, every emoji frame, terms, switches, email blocks, operators, audit and system. The server enforces the capability matrix even for direct API calls. There are no rating, history or player-key editing tools.

Run `python3 web/scripts/check.py` for static references, i18n parity, JS syntax, text-only rendering and the CSRF source gate. A local static preview can use `python3 web/scripts/serve.py admin-app --port 8603`; authenticated QA additionally requires a local same-origin proxy to the synthetic capture-backed admin listener. Never point a preview at a live operator API. See [operations](../../docs/planning/P75-OPERATIONS.md) for the attended activation and exact CSP.

Real capture-backed browser sign-in and the owner's subjective sign-off are recorded separately in the phase report; source checks are not runtime evidence.
