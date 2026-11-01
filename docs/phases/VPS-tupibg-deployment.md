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
Runtime backend verification and local native installation are pending below.
