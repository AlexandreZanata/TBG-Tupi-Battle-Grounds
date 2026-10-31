# R07 — Trident delivery

Own articulated shark jaw/fin/chain/trident library and five forms. Two-handed
short thrust and independent low rend sweep; authoritative primary-centered area
and one bleed slot per surviving enemy (D-78/§13). Static damage snapshots once
at application; four pulses retain their next deadline on refresh and survive
source death. Ward blocks new bleed only; no allied/tower splash or heal-cut.
A shared red HP-bar badge shows persistent bleed even under another control ring.
Lab pulse attribution retains source seat after death. Public pools stay closed.

Work-front result: package formatting/diff passed. Shared-lock/two-job cached
`cargo check -p tbg-client --bin tbg-client --target wasm32-unknown-unknown`
passed without warnings in2.26s. No per-front native tests or duplicate target.

Queued for combined R04–R07 consolidation under the same shared lock/two jobs:
`cargo test -p tbg-core --features lab --lib r07 -- --test-threads=1`, affected
roster_slots/specials and roster/protocol/determinism contracts. Cases cover exact
primary center/radius, carried enemy versus ally, no cut, equal/stronger refresh,
source death, four pulses/lab credit, Ward/raw damage and branch replacements.
Desktop screenshot/character commit pending one combined build. Full-clip/ability/
HUD/live reconnect/performance, balance gates, full suite and owner look/fun remain
pending, not passing. No merge, production deployment or pool activation.

Consolidation R04–R07: 75 distinct focused tests passed on the integrated tree:
--features lab --lib r0 (34), r10 (4), specials::tests (6, one overlap),
roster_slots::tests (12), staged_kinds (1), and named roster/protocol/determinism
contracts (6/11/2). Shared host lock, CARGO_BUILD_JOBS=2, --test-threads=1. The
initial test-only selector-call E0425 was corrected with every old assertion kept.
Shared-lock offline Trunk release preview passed (2m49s Rust compilation,
BINARYEN_CORES=2); actual desktop lineup observations follow below. No full suite,
balance gate, cold native client build, CI waiting or production operation.

Initial desktop review requested clearer A-branch armor/crest and B-branch sash:
added pale shell shoulder guards, raised the final dorsal armor above the head
silhouette, and exposed the red-brown waist wrap. Competitive code is unchanged.
The earlier image is superseded; final evidence awaits the next combined preview.

Corrected desktop preview passed in the next shared-lock/two-job WASM build
(3m39s Rust compilation, BINARYEN_CORES=2). Pale shell shoulder guards and the
raised final dorsal crest now distinguish A; the B waist wrap is visible. The
actual screenshot replaces the superseded initial capture. Competitive code
remains the passing75-test revision;15 additional R12/shared/roster tests passed
in this combined build. No gameplay/contract value changed for this visual fix.

Actual desktop lineup observed at1280×720: Base/A1/A2/B1/B2 in idle/walk/attack,
with the own silhouette and branch attachments, using the combined P59 WASM
preview. Triangles: 1702/2070/2070/1714/1954. Screenshot: ../../media/p59/trident/lineup.jpg.
These observations and the consolidation above resolve earlier pending compile,
test and lineup entries. Full-clip/ability/HUD/live reconnect/performance/balance,
full suite and owner look/fun acceptance remain open. No pool activation/deploy.
