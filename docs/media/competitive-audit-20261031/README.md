# Competitive audit evidence — 2026-10-31

These are safe offline aggregates, not private player telemetry. Raw matches,
full duel matrices, generated image masters and build outputs stay ignored.
The audit report records source identity, commands, interpretation and limits.

- discovery-gate.csv / discovery-manifest.txt: the original 64-pair discovery
  run, before the owner approved the normalized purchase-share threshold.
- discovery-gate-normalized.csv: classification derived from those exact
  observations with the approved 0.625/46 floor; no match was resimulated.
- candidate-probes.csv: 25 independent control/candidate measurements on
  screening seeds 200000..200256 (256 paired, seat-swapped seeds each).
  A probe is one Machines-vs-bot matchup, not a full gate acceptance.
- throughput-audited.txt: explicit preview-profile throughput, identical
  results across worker counts; compare rates only with that same profile.
- confirmation-gate.csv / manifest / roster / payoff and duel flags: the
  untouched 1000000..1000128 block, 423,424 complete matches. It preserves
  CANON and --strict correctly exited 1 on the required competitive failures.

The ruleset data fingerprint (Debug-form FNV) differs from the ranked source
hash (watched source bytes). CANON data fingerprint is d365d09fafcb66a4;
RulesetVersion 2 source hash is 3d56156049e626c5. Neither identity changed.
