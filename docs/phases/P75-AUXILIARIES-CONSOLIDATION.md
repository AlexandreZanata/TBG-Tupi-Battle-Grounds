# P75 and tiered War Auxiliaries consolidation

Owner request, 2026-11-02: integrate the attached War Auxiliary handoff into
P75, resolve failures, push, verify GitHub CI and merge when validations pass.
The owner acknowledges that main triggers automatic deployment. Schema 12
still requires attended migration; no deployment guard may be bypassed.

## Integrated sources

- P75 S0–S14: `854391b0`, PR #59.
- Main / protected backups and updater: `130cadc1`, PR #57.
- War Auxiliary tiers and presentation: `5c8b9ae2`, three original commits retained.
- Combined protocol: wire 40, append-only P75 tags; auxiliary R16 fingerprint
  `f2e253b98ccb313a` retained. P75 itself introduces no simulation change.
- Client release candidate: `0.1.4-colossus.5`.

## Validation in progress

Level: targeted consolidation and the explicitly requested eight-Arena/both-side
captures and localhost preview. No local full suite is authorized. Every
compilation uses the shared nonblocking host lock and at most two Cargo jobs.
Previous source-branch evidence remains in the P75 report and handoff; this
report records only checks on the combined tree once completed.

## Pending acceptance and activation

- Windows device capture and exact Linux/Windows/desktop-web visual/motion
  correspondence require observed evidence; compilation is not parity.
- Proposed auxiliary values require competitive D-33 acceptance and owner fun
  sign-off; bounded measurements must not be represented as the full gate.
- Production activation: verified backups, copied-database migration rehearsal,
  mail configuration, privacy approval and coordinated compatible client/server
  rollout. Automatic deployment refuses schema changes. No VPS, DNS, real email,
  secrets or production database operation has been performed.
