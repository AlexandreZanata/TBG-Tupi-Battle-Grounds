# Pyre Warden ground-line removal — 2026-11-01

Owner reference: Screenshot_2026-11-01_09-28-23.png. Remove only the yellow
line on the floor when Pyre Warden fires; preserve the approved animation.

Removed the cosmetic ground-strip spawn from `vfx/flame.rs`. The nozzle-bound
flame pieces, colors, count, motion, lifetimes, model, attack timing and event
endpoints are unchanged. The authoritative segment still determines combat;
no core source or gameplay value changes. No portrait change is needed.

Validation: affected-package formatting and `git diff --check`. The native
debug build required for the authorized reinstall will use the nonblocking
shared lock, two Cargo jobs and existing cache. No new tests or full suite
for this isolated cosmetic removal. Final build/installation evidence is local.
