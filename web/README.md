# TBG web frontends

Two static sites, independent of the Bevy/WASM game bundle and of the game server process:

| Site | Path | Published at | What it is |
|------|------|--------------|------------|
| Landing | [landing/](landing/README.md) | `https://tupibg.com` (`www` redirects) | Presentation, how to play, champions, Arenas, real captures, patch notes, platforms |
| Admin placeholder | [admin/public/](admin/README.md) | `https://admin.tupibg.com` | One static "in preparation" page: no script, data, sign-in or API |
| Admin preview | [admin/app/](admin/README.md) | **never published** | Local console foundation with fictional fixtures |

Plain HTML, CSS and JavaScript modules. No framework, no npm dependency and no build step;
`web/shared/` holds the design tokens, base components and DOM/language helpers used by
both sites. Game art, fonts and the patch catalog are reused from the repository through
each site's `site.manifest` instead of being copied.

## Develop

```bash
python3 web/scripts/serve.py landing --port 8601
python3 web/scripts/serve.py admin-public --port 8602
python3 web/scripts/serve.py admin-preview --port 8603 --public-status https://api.tupibg.com
```

The preview server binds `127.0.0.1`, maps manifests on the fly (edits show on reload) and
sends the production CSP and security headers, so violations appear in the browser console.
`--public-status` lets only the admin preview read the backend's public `/health` and
`/api/version`; nothing else is proxied.

## Check

```bash
python3 web/scripts/check.py          # manifests, references, JSON, JS syntax, i18n, release gates, forbidden patterns
python3 web/scripts/sync_content.py   # regenerate landing/data/{champions,arenas}.json after roster/Arena/i18n changes
git diff --check
```

`check.py` needs no network; it uses `node --check` for JavaScript syntax when Node is
installed. Content sync derives names, Factions, Archetypes and Arena modifiers from
`tbg-core` and the client i18n files; the web never decides game content.

## Rules

- Build the DOM with `textContent`/`createElement` only (`web/shared/dom.js`); no
  `innerHTML`, `eval`, inline scripts, inline styles or inline handlers. CSP stays
  `default-src 'none'` plus `'self'` for what each page needs.
- `localStorage` holds only the language code (`web/shared/lang.js`), never a credential.
- Interface text exists in pt-BR and English. Glossary terms follow GAME-DESIGN §2 and the
  game's own translations (Fortaleza, Energia, Loja, Evolução, Sinergia, Facção…).
- Announce nothing that is not published and verified: availability lives in
  [landing/config/release.json](landing/config/release.json) and is checked against the game's
  protocol and contract.
- Only reviewed, git-tracked public art. Screenshots come from the native Linux client
  (the visual reference) through `scripts/capture_native.sh` and
  `scripts/media_from_capture.sh`; owner-supplied reference images never enter git.

## Publish

See [deploy/README.md](deploy/README.md): immutable releases under `/srv/tbg/<site>/releases`,
atomic symlink activation, a Caddy include validated before any reload, and rollback.

## Static code cache consistency

Manual staging and attested static packaging both use
`web/scripts/asset_versions.py`. The delivered HTML and every transitive JS/CSS
reference carry one deterministic content revision (`?v=<sha256>`), computed from
the site's text inputs. Source files remain unchanged. A nested module change
therefore invalidates the complete module graph for returning visitors, not just
the entry script. CDN browser-TTL overrides cannot combine newly loaded HTML
with an older cached download controller. No bundler or runtime cache purge is
required; CSP and hash-bound package verification remain intact.

The live `/downloads.json` URL stays unversioned and no-cache so a newly verified
compatible release can change download links independently. Download dialogs are
initialized before metadata requests and explain the pending state even while
those requests are slow or unavailable. A missing native release never disables
the interaction itself. Changing these rules requires the focused static-asset
and static-package regressions, not a Rust rebuild.
