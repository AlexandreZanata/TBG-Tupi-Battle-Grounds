# P75 operations — owner runbook for mail, administrator claim and rollout

Companion to [P75](../phases/P75-admin-moderation-and-match-social.md).
**Only the owner (or an agent the owner explicitly authorizes for this
host step) performs these actions on the VPS.** The P75 executor writes the
templates and this runbook; it never touches the VPS, DNS, secrets or the
production database.

Official host: `tupibg-vps` (72.60.147.2). Live layout on 2026-11-02: two
release slots (`a` on 127.0.0.1:8080 active, `b` on 127.0.0.1:8082 idle),
one SQLite file `/var/lib/tbg/tbg.sqlite3`, Caddy in front with Cloudflare
as the trusted proxy, units installed by `deploy/provision.sh` only.

## 1. Gates before activation (all required)

1. **Backups actually run.** `deploy/backup.sh` is scheduled by cron in
   `provision.sh`, but cron is not installed on the live host
   (`docs/phases/HANDOFF-automated-delivery-20261102.md`), so no nightly
   copy exists today. Install the schedule (cron or a systemd timer that
   runs `backup.sh` nightly and `--predeploy` before the switch), configure
   `TBG_BACKUP_REMOTE` for the weekly off-site copy, and run one restore
   drill on a scratch copy (`deploy/README.md`). P75 adds identity,
   moderation and audit data that must not be lost.
2. **Mail sender ready.** Choose an SMTP submission account (any provider
   that offers authenticated SMTP on 587/STARTTLS or 465/TLS). If mail is
   sent as `no-reply@tupibg.com`, publish the provider's SPF include, DKIM
   key and a DMARC record for `tupibg.com` first, then send a test message
   to the Owner address and confirm it does not land in spam.
3. **Privacy and conduct text.** Update `web/landing/privacy` (what email is
   used for, transcripts kept 7 days, reports and moderation records,
   retention, contact) and the terms/conduct rules for chat, emojis and
   bans. Decide how a player asks for account deletion (O-P75-13). This is
   an owner/legal decision; the plan makes no compliance claim.
4. **Clients published first.** P75 changes the wire protocol and the
   schema, so the browser bundle and native packages of the new contract
   are published before the switch (BR-UP09); older clients then receive the
   existing update-required path.

## 2. Host configuration (create before the new generation starts)

The Owner asked that the administrator exist with the address below and
receive the key-claim email as soon as the new version runs with mail
configured. The address is **host configuration**, never compiled in.

`/etc/tbg/p75.env` — root:root, mode 0600 (systemd reads it as root):

```sh
TBG_ADMIN=on
TBG_ADMIN_ORIGIN=https://admin.tupibg.com
TBG_ADMIN_BOOTSTRAP_EMAIL=zanataxandyzv07@gmail.com
TBG_PUBLIC_GAME_URL=https://app.tupibg.com
TBG_PUBLIC_ADMIN_URL=https://admin.tupibg.com
TBG_MAIL_FROM=Tupi Battle Grounds <no-reply@tupibg.com>
TBG_SMTP_HOST=<provider host>
TBG_SMTP_PORT=587
TBG_SMTP_SECURITY=starttls
TBG_SMTP_USERNAME=<provider login>
TBG_SMTP_PASSWORD_CREDENTIAL=smtp_password
# Optional: TBG_ADMIN_ALLOW_IPS=203.0.113.7/32
```

`/etc/tbg/secrets/smtp_password` — root:root, directory 0700, file 0600,
containing only the password (no newline needed). Never put it in
`p75.env`, `server.env`, the repository, a shell history or a chat.

Drop-ins (identical content for `tbg-server.service.d` and
`tbg-server@.service.d`), e.g. `/etc/systemd/system/tbg-server@.service.d/p75.conf`:

```ini
[Service]
EnvironmentFile=-/etc/tbg/p75.env
LoadCredential=smtp_password:/etc/tbg/secrets/smtp_password
```

Then `systemctl daemon-reload`. `provision.sh` overwrites `server.env`
only, so these files survive re-provisioning; keep them out of
`server.env` ("no credentials belong in this file"). `--build-info` keeps
working without any of them. The P75 implementation adds the drop-in and
an example env file to `deploy/` as templates only.

## 3. Caddy

API/app hosts (`deploy/Caddyfile.domains.template`, attended reload):
add `/api/emoji/*` to the `@shared` matcher (both slots, read-only).
Every other P75 player route is under `/api/auth/*` or `/api/me/*` and is
already slot-routed by `X-TBG-Slot`.

Admin host (`web/deploy/Caddyfile.web.template`, published by
`web/deploy/publish.sh`): replace the placeholder block with the console
block written by S12/S13 — static console files plus

```caddyfile
handle /admin-api/* {
	reverse_proxy 127.0.0.1:8090 127.0.0.1:8092 {
		header_up X-Forwarded-For {client_ip}
		lb_policy first
		lb_try_duration 2s
		fail_duration 10s
	}
}
```

and the console CSP (`default-src 'none'; script-src 'self'; style-src
'self'; img-src 'self' blob:; connect-src 'self'; font-src 'self';
base-uri 'none'; form-action 'none'; frame-ancestors 'none'`). Caddy reloads
follow the existing rule: only with zero live connections or as an
attended maintenance step.

## 4. Attended rollout (schema + protocol change)

The automatic pipeline refuses this release by design (schema mismatch is
always rejected; a contract change needs `--coordinated-contract`). Follow
`deploy/README.md` and `deploy/auto/README.md`:

1. Disable the backend timer for the window; confirm slot states with
   `deploy/auto/status.py`.
2. `deploy/backup.sh --predeploy` (integrity check + copy).
3. Rehearse: copy the database to a scratch directory, start the new
   build against the copy on a spare loopback port, confirm the migration
   and `/health`, then delete the scratch copy.
4. Make sure §2 files exist (the new generation must start with them).
5. Install the new generation in the **idle** slot with `release-slot.sh`,
   promote it through the documented coordinated contract switch; the old
   slot drains its matches without a cutoff (D-104).
6. Verify `/health`, a casual match between two updated clients, chat and
   one emoji, then re-enable timers per the delivery notes.

Rollback before any P75 write: promote the previous generation back. After
P75 tables hold data, a rollback keeps the additive tables in place (older
builds ignore them); restoring the pre-switch backup loses newer
accounts/results and is an explicit owner decision (`deploy/README.md`).

## 5. Claiming the administrator key

1. When the new generation starts with §2 in place, it creates the Owner
   operator for the bootstrap address (no key) and queues the claim email;
   the mail dispatcher sends it within a minute. It repeats at most once
   per 24 h until claimed.
2. Open the email "Tupi Battle Grounds — claim your administrator key"
   (check spam). Go to `https://admin.tupibg.com`, choose **Claim or
   recover key**, enter the address and the code (`XXXXX-XXXXX`, 24 h).
3. The console shows the AdminKey (`TBGADM-…`) **once**. Save it in a
   password manager. It is never emailed and cannot be shown again.
4. Sign in: AdminKey → a login code arrives by email → enter it → console.
   Every sign-in sends a notice to the same address.
5. Lost AdminKey: repeat steps 2–3. The previous key and all console
   sessions stop working immediately. Protect the mailbox with its own
   two-factor authentication: whoever controls that mailbox controls the
   console (accepted by design, D-107).

Break-glass without mail (SSH to the host as an administrator):
`sudo -u tbg env TBG_DB=/var/lib/tbg/tbg.sqlite3 /opt/tbg/slots/<active>/tbg-server operator claim-code --owner`
prints a claim code on the terminal (audited as `cli:<unix user>`). Other
subcommands: `revoke-sessions`, `disable`, `enable`, `set-owner-email`
(asks for confirmation on a TTY; invalidates the old key). It is a
subcommand of `tbg-server` because the backend artifact may only contain
`tbg-server` and `tbg-release-admin` (`deploy/auto/policy.py`).

## 6. Turning features on

Defaults after activation: chat **off**, emojis on, custom emoji display
off, uploads off, reports on, registration on. Recommended order from the
console's Settings: invite moderators if wanted → enable chat → watch the
reports queue for a few days → enable uploads → approve a few emojis →
enable custom emoji display.

## 7. Incidents

| Situation | Action |
|---|---|
| Chat abuse wave | Settings → chat off (both slots within ~2 s); act on reports; add blocked terms |
| Upload abuse | Settings → uploads off; reject/revoke; restrict the uploaders' emoji rights |
| Suspected console compromise | `tbg-server operator revoke-sessions --operator <id>`; claim a new key; review `admin_audit` |
| Owner mailbox compromised | `tbg-server operator set-owner-email <new>` over SSH, then claim with the new address |
| SMTP outage or password change | Console System card shows the queue and last error code; replace the credential file, then start the next generation in the idle slot and promote it (no restart of a live slot) |
| Database restore | Follow `deploy/README.md`; all console sessions are invalid after a restore; re-claim if needed |

## 8. What never happens

No console action reveals or sets a player key, changes a player email,
edits ratings/results/history, or deletes accounts. The executor and the
console never read the SMTP password. No secret is pasted into chats,
issues, PRs, logs or screenshots.
