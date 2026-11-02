TBG 0.1.4-colossus.2 — War Auxiliaries and Gods consolidation beta

Miner and Merchant now provide authoritative War Auxiliary choices during a match.
The consolidated preview includes Volthar, Astreon and the refined Colossus,
while preserving the original contact-based pushing and randomized match sides.
The Linux preview installer reads the committed version and preserves existing data.

Protocol 38, schema 10 (P73 history schema 11 remains opt-in), ruleset 15 /
cc459351581b7763. This contract requires a coordinated release from protocol 31;
compatible-only delivery must not bypass its production contract gate.

The local Practice/Training preview contains 84 champions across ten Factions,
with five-form low-poly kits, refined Ogres, Abyssals, Gods/Colossus, Volthar and Astreon and rolling
healing casks. Online FULL_POOL remains the separately gated 46-champion roster.
The existing Proposed tower changes, broader BattleLine, original pushing and
intrinsic body colors are retained. Widening the road no longer widens the
preferred departure formation before contact. CROWD-EXIT-01 also bounds live
walking envelopes to the actual body and held equipment, while engaged Colossus
arms and Ogre Archer equipment retain directional clearance.

Blue stays left and red stays right for every desktop player and spectator.
Online and Practice sides are randomized; the side announcement, private controls,
Stronghold, reconnection and result ownership follow the assigned seat. P73 adds
participant-bound social state and opt-in private history; laboratory refinements
and the landing dialog scroll fix are included.

Includes reviewed A/B backend delivery and bounded re-admission (three retries
within 30 seconds), main-based publication and exact-source native packages.
Native publication does not install an update on a player's computer.

Linux: x86_64 archive and AppImage (glibc 2.35+, Vulkan).
Windows: x86_64 MSVC portable ZIP and per-user installer (Windows 11, DirectX 12).
Both packages use the same source, assets, dependency lock and declared features.

The Windows executable/installer are unsigned beta artifacts. GitHub build provenance
and SHA-256 checksums authenticate the release pipeline and integrity; they are not an
Authenticode publisher signature. SmartScreen reputation is not claimed.

Windows VM and visual acceptance results are recorded in the linked execution report.
Do not infer complete GPU/audio/DPI acceptance or exact pixel equality from shared
code, a successful compile or a partial capture. The current release manifest keeps
visual acceptance pending until the required full comparison is completed.

Authentication uses the real OS vault when persistence is chosen. P67 email recovery
and commerce remain offline foundations. Native TUF signing and two-slot rollout are
separate gated mechanisms. Preserve existing accounts, preferences and older packages.

Source, individual asset checksums and the exact compatible contract are in
`tbg-native-release.json`; `native-provenance.jsonl` binds the artifacts to GitHub CI.

Ruleset 15 consolidates P74 and GD-04, including serialized auxiliary state and
the owning seat's reroll quote. Miner and Merchant keep the proposed 6/4 Energy
costs and single-hire lifecycle. Two new Gods remain in the local preview.
No competitive acceptance is implied.

D-104 retirement load is sampled inside its serialized writer turn, excluding
only the observation itself. Later queued work, running matches, reconnect
receipts and Ranked results remain protected during delivery.
