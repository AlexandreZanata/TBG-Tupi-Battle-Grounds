TBG 0.1.4-colossus.2 — Compatible automatic delivery verification beta

This immutable follow-up keeps the colossus.1 gameplay and presentation. The Linux
preview installer reads the committed client version, so later verified packages
remain installable without a hardcoded preview-version update. This release also
provides the owner-authorized compatible timer-only delivery witness.

Protocol 36, schema 10 (P73 history schema 11 remains opt-in), ruleset 13 /
b66993d9e87a9cfc. This contract requires a coordinated release from protocol 31;
compatible-only delivery must not bypass its production contract gate.

The local Practice/Training preview contains 82 champions across ten Factions,
with five-form low-poly kits, refined Ogres, Abyssals, Gods/Colossus and rolling
healing casks. Online FULL_POOL remains the separately gated 46-champion roster.
The existing Proposed tower changes, broader BattleLine, original pushing and
intrinsic body colors are retained. Widening the road no longer widens the
preferred departure formation before contact.

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

Ruleset 13 registers equivalent Rust 1.99 lint fixes; gameplay values and branches
are unchanged from the ruleset-12 preview. No competitive acceptance is implied.

D-104 retirement load is sampled inside its serialized writer turn, excluding
only the observation itself. Later queued work, running matches, reconnect
receipts and Ranked results remain protected during delivery.
