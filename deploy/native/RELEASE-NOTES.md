TBG 0.1.1-rc.1 - consolidated Linux candidate, protocol 30, schema 10.

One build from the consolidation pull request that joins #18-#33: the
complete 46-champion roster with forms and abilities, five refined arenas
with training in each, the troops/training laboratory, the winner-only
victory celebration, English/pt-BR presentation, ranked records, the
competitive audit with illustrated patch notes, protected sign-in (P68) and
the recoverable Linux updater foundation (P66). Gameplay and balance numbers
are unchanged: Ruleset 2 / 3d56156049e626c5.

Compatibility. This candidate requires a protocol-30 server built from the
same consolidation. The production server currently deployed uses protocol 29
and is NOT compatible: online play reports that an update is required, while
offline Practice and the training laboratory keep working. No server was
deployed and no existing installation is replaced by this download.

Sign-in and resume. Access stays in memory. "Keep me signed in" and the
short-lived match-resume record use the current user's Secret Service only;
a locked or missing vault shows a visible temporary session. AccountKeys keep
working for manual sign-in and recovery.

Not included or not active. Official signing and automatic updates wait for
owner gate O02 (no update root is bundled). Windows is not supported by this
release: any Windows test build attached here is unofficial, unsigned,
cross-compiled and checked only under Wine. Two-slot server rollout stays
inactive. Competitive balance acceptance and owner look/fun sign-off remain
open; the audit's recorded failures are not hidden.

Linux x86_64, glibc 2.39 or newer, Vulkan-capable graphics, ALSA, X11 or
Wayland. Extract the tar.gz and run ./tbg-client (or ./install.sh for a
per-user launcher), or run the AppImage. Verify SHA256SUMS first. Each archive
carries build-info.json (version, protocol, contract, source commit,
Cargo.lock and asset digests) and file-manifest.json.
