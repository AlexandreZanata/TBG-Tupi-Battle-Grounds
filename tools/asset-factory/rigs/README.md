# tools/asset-factory/rigs/

Shared skeleton definitions and rig-once-per-base tooling (Phase 02+).

Contracts:

- Rig once per base body class, never per assembled character.
- Skeleton changes are breaking: bump the rig version and re-validate
  all dependent assets.
- Prefer articulated rigid-part rigs where skinning cost is unjustified.
