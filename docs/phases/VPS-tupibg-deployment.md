# tupibg.com backend and local Linux delivery — 2026-11-01

## Owner scope and privacy

The owner explicitly authorizes access to their reset VPS, key-only SSH from
this computer, backend deployment and an updated local Linux client. Domain
layout: public backend at `api.tupibg.com`, browser game at `app.tupibg.com`,
future landing at the base/www hosts, future administrative suite at admin.
This task deploys the backend and local Linux client; landing/admin and public
signed client releases are not part of it.

The supplied password is used only at the SSH password prompt, after comparing
the host fingerprint with the provider-console output supplied by the owner.
No password, private key, origin address, host pin or credential belongs in git.
Connection settings use a local SSH alias and an ignored mode-0600 deploy env.
The old origin address previously present in deploy/README was redacted from
the current source. Earlier git history was not rewritten.

## Completed provisioning

- Confirmed a clean Debian 13 amd64 host, approximately 16 GiB RAM, with no prior
  game installation. The provider-console host fingerprint matched the connection.
- Registered this computer's existing Ed25519 public key, proved a fresh key login,
  then limited authorized_keys to that key and disabled password/keyboard auth.
  Validated sshd config, reloaded it and proved another fresh key-only login.
  The private key stays on this computer. Recovery console access remains available.
- Installed official Debian build/network packages and Caddy 2.11.2 from the image's
  already enabled official trixie-backports repository; Rust stable 1.99.1.
- Created the unprivileged game service user and private database/backup directories.
  Enabled firewall ports SSH/HTTP/HTTPS, validated Caddy config, and obtained public
  Let's Encrypt certificates for both API/app hosts. No TLS checks were disabled.

## Source changes

Native production/default release CI endpoint: `wss://api.tupibg.com/ws`.
Browser routing stays same-origin on the app host, including auth cookies and
WebSockets; both approved origins are configured explicitly. No CORS relaxation,
new auth lifetime, gameplay value, protocol or ruleset change. Protocol remains
31 and match contract remains `w31-r6-097385fee9a10b0f`.

The split proxy accepts client-IP metadata only from Cloudflare's official
published IPv4/IPv6 ranges, and overwrites upstream XFF with that verified IP.
This prevents proxy-wide connection caps and caller-forged XFF identities.
References checked 2026-11-01: https://www.cloudflare.com/ips/ and
https://caddyserver.com/docs/caddyfile/options#trusted-proxies.

Deployment supports `--backend-only`, excluding local evidence, agent metadata,
SSH material and environment files from transfer. First install skips a database
backup if there is no database yet; it must not create a root-owned empty database.
Existing databases retain the pre-migration integrity-checked backup procedure.

## Validation and delivery status

Level: targeted deployment/desktop delivery; no full workspace suite or load test.
Source formatting, shell syntax, desktop source gate and mandatory patch-note
checks pass. Caddy configuration validation and public certificate issuance pass.
Runtime backend verification and local native installation completed as recorded below.

## Completed runtime delivery

Runtime source: `8be248e025ed8f56290d94604da75536726d765b`, pushed before the
installation. Both server and installed client use protocol 31 and
`w31-r6-097385fee9a10b0f`. No gameplay source or ruleset revision changed.

- `bash deploy/deploy.sh --backend-only`: passed; remote locked two-job
  `cargo build --locked --release -p tbg-server --bin tbg-server --bin
  tbg-release-admin` finished in **2m 01s**. Public API health returns 200/`ok`.
- Server binary SHA-256:
  `e7daa9c85a950817b68f711e7a3222261cf489546f6efde735092f31fa401984`.
  Startup metadata records this runtime source. Single-slot mode intentionally
  reports null slot/generation fields in the operator endpoint; the contract
  and startup source establish identity.
- Service active; database schema **10**, SQLite integrity **ok**. Database and
  data directory belong to `tbg`, modes **0600/0700**; backups are root-only.
  The first boot created the database as the service user, without a root-owned
  empty database. Manual nightly-backup script verification passed, mode 0600.
- Public HTTPS/WS verification with synthetic data passed: current contract
  admission; old contract and protocol-30 refusal; native registration; unchanged
  900-second Access/2,592,000-second absolute family lifetime; idempotent refresh;
  same-socket renewal; authenticated reconnect preserving result, seat and Arena;
  completed casual surrender; invalid browser Origin refusal; app same-origin
  cookie confirmation with Secure/HttpOnly/SameSite=Strict and no Domain; no
  browser refresh credential in JSON; private operator/metrics routes return 404.
  Synthetic auth families were revoked after the check. This is a focused
  deployment scenario, not a browser UI/vault acceptance campaign or load test.
- Initial Node built-in WebSocket harness was refused because its browser-style
  metadata lacked an approved Origin. The corrected native upgrade harness
  preserved server checks, TLS and WebSocket accept verification and passed.
  No production authentication validation was weakened for the probe.
- The README's suggested server `--version` invocation is not implemented by
  this binary; the inspection attempt failed binding a second process to the
  occupied port. The running systemd service stayed active. Identity was checked
  using the release contract, startup metadata and binary hash instead.

## Installed Linux client

The owner renewed the explicit native-install request after the earlier SIGTERM
interruptions. A locked two-job native build completed in **1m 50s**, using the
existing cache and compile-time `TBG_DEFAULT_SERVER=wss://api.tupibg.com/ws`.
The compilation waited for the other front's lock; no other process was killed.

Installation label: `0.1.1-rc.1-tupibg-8be248e` (local debug playtest build).
Client SHA-256:
`a3666321b21fa3fc9917f7ec5e5e36aeb6fd2ce785497ab5a489cfedfcfcc51a`.
`--build-info` confirms source, native x86_64 target, protocol and contract.
All **368 assets** and **15 license files** were verified by hash. The existing
launcher/desktop entry now points to this installation; the previous version,
player data and existing user windows are preserved. No public release artifact
or signed updater promotion was created.

A real online match was rendered by the **installed** executable in isolated
Xvfb with a synthetic queued opponent. No server override was supplied, proving
the embedded API endpoint. The initial capture showed new Arena **Desert (7)**
and received the match's finished acknowledgment, but its helper's open stdin
prevented process exit. The harness was corrected to close its own stdin and
the bounded scenario passed cleanly on **Volcanic (2)**; both captures were
preserved locally. This was not a repeated roster gallery or performance test.
After the normal reconnect receipt window, operator status confirmed **0 running
matches, 0 pending results, 0 ranked inflight and no retirement blockers**.
Software-rendered capture FPS/RTT is not a player-capacity or hardware benchmark.

## Remaining scope

Landing/base/www and the administrative application have not been published.
The app host has the same-origin backend proxy and TLS, but the web game's
static bundle was intentionally not deployed by `--backend-only`. The owner
requested a Claude bootstrap prompt for those fronts; a local plain-text prompt
was delivered without origin address, host pins, password or key material.

Existing balance approval, complete gallery/Windows visual acceptance, actual
web UI/vault acceptance, signed update promotion and capacity testing remain
separate gates. No full suite, load benchmark, new balance value, DNS change,
main merge, forced history rewrite or frontend administrative authorization
was performed. Historical mentions of the origin address remain in prior git
history; no new credential was versioned.
