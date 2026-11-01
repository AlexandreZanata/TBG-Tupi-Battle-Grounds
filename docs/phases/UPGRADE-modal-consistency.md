# Champion and Stronghold upgrade modal consistency

Owner request: only the single second Evolution choice should expand across the
modal; preserve two initial choices. Style Stronghold Fortification like champion
Evolution, using the supplied three Linux screenshots as reference.

Changes: shared option/header/close geometry and colors; equal flex shares for two
choices, full row for one. Fortification's inline choice text replaces its badges
and coin-box layout; the unchosen branch is hidden after tier 1. Blocking feedback
remains visible when needed. The left information panels remain in their locations.
Prices, purchase conditions, pending-intent handling and authoritative actions are
unchanged. The shared desktop contract and state/motion matrix are updated.

Validation level: work front with owner-authorized native build and reinstall.
Pending: formatting/source gate; native release build; focused A/B champion and
Stronghold tier 0/1/2 real Practice captures; installed package verification.
The existing opt-in focus capture follows real purchase actions, waits for affordable
Fortification offers, and records the two-to-one choice widths in actual UI nodes.
Windows/web runtime checks and disabled-state matrix cases remain pending
consolidation. Queue `cargo test --locked -p tbg-client --bin tbg-client
ui::fortify::tests:: -- --test-threads=1` under the host lock at consolidation.
No full suite, balance gate or broad gallery is requested. No push under the
latest owner-provided AGENTS.md.
