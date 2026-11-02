TBG 0.1.2-rc.2 — Linux and Windows beta candidate

Protocol 31, schema 10, ruleset 6 / 097385fee9a10b0f.

Same gameplay, protocol and contract as 0.1.2-rc.1. This is the first version
published by the automatic main pipeline: one exact-source build per platform,
promoted byte for byte, built with the pinned Rust 1.99.0 from LF-normalized
sources. While the server switches to a new release, a request the old server
refuses is now asked again through the server's admission step (at most three
times in 30 seconds) instead of showing "server full".

Shared Bevy desktop interface, 46 conventional champions, eight Arenas, local
training, winner celebrations, illustrated cards/Augments and protected sessions.
This candidate includes the already documented P70/P71 Proposed changes; it is not
a new competitive balance acceptance.

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
