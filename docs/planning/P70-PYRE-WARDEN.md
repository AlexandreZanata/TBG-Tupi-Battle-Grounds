# P70 Pyre Warden pilot

**Status: implementation candidate; final renderer/live-event review pending.**

## Identity and observed baseline

The frozen real-renderer Idle sheet shows a thin exposed skeleton, a flat
dark coffin behind the skull, a short solid-looking gold nozzle end and a
blue rectangular pelvis/chest patch. The five forms differ in armor and
barrel attachments but share mostly gray surfaces. The source event path
discarded `ProfaneFlame.unit` and placed one static green cone at a fixed
world height. Pose sheets alone do not establish how that looked in combat.

Original content: `55510c5363e3f1e555fa5133208a5faf20c5825b`; capture tooling:
`27f98acd98b51fa69eb09f4703bd2ca59a33ed41`. Originals and their hashes are
in `.local/p70-evidence/before-cohort/PyreWarden/ally-threequarter/` and its
`captures.json`, linked through the phase evidence index at consolidation.

## Candidate changes

- Large rust metal and violet cloth regions distinguish the funerary
  furnace from Blood Furnace. The coffin has beveled edges; chest harness,
  straps and shoulder fittings connect its weight to the skeleton. Bone,
  skull openings and skeletal anatomy remain visible.
- The team marker is a small framed round heraldic piece. The nozzle has
  a dark bore, brass rim and spectral pilot light, with the same helper
  defining its visible endpoint and effect socket in all five forms.
- Longer skeletal arms reach one coherent pair of nozzle hand targets.
  Idle checks pressure and looks about; Walk carries the load with a
  guarded gun; the primary action braces before the existing 0.30 s event
  and recoils afterward; Victory raises the nozzle in a compact salute.
  Hit/Death/Spawn/Evolve retain the existing timing and two-handed prop.
- `ProfaneFlame.unit` selects the actual WeaponR joint and per-form bore.
  The flame follows the propagated socket; independent effect roots update
  both local/world transforms to avoid a one-frame emitter lag. A despawned
  emitter leaves its last finite world origin until the short effect ends.
- The essential jet uses a green envelope, hot pale core and traveling
  tongues on both quality settings. Standard adds bounded ember/smoke
  details. Every mesh/material is cached; no per-frame mesh/material is
  created. Jet pieces live 0.20 s and are cleared at session boundaries.
- The original ground strip still spans exactly the event's start/end,
  with the same thickness and 0.28 s lifetime. Damage, friendly-fire rules,
  targets, event ticks and the `Attack::None` generic-shot exclusion are
  unchanged. No particle applies damage or decides an outcome.

## Validation and remaining evidence

Owned Rust files formatted and `git diff --check` passed. Pilot compilation
is consolidated with Trident and Old Finn rather than repeated per file.

Queued narrow regression, under the shared host lock and two build jobs:
`cargo test -p tbg-client --bin tbg-client
pyre_jet_reaches_the_event_endpoint_for_both_seats -- --test-threads=1`.

Required live evidence remains: original/candidate real ProfaneFlame
sequences, both firing directions, Low/standard, all five forms, ally and
enemy damage/status cues, last-origin fallback after despawn, cleanup,
nozzle alignment and no duplicate basic emission. Review the arm/body and
coffin clearance through Hit, Death, Evolve, Victory and blends. Native
compilation does not establish desktop-web or Windows rendering. Owner
appearance approval remains pending.

Source review corrected two event-order hazards before rendering: flame transforms
now update before Bevy visibility culling, and the nozzle offset uses the installed
UnitVisual form rather than a potentially stale Animator form on Evolution. Idle
also applies aim once, avoiding the inherited common-idle plus final-facing turn.
