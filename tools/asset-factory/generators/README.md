# tools/asset-factory/generators/

Versioned procedural generation scripts: part builders, body bases,
faction variants (Phase 02+).

Contracts:

- Deterministic output: same inputs + same script version = same asset.
- Every script records its version in the output manifest.
- No generative-AI inference and no network calls at generation time
  unless the dependency is vendored and license-audited.
