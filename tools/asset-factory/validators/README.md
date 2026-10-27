# tools/asset-factory/validators/

GLB acceptance gates enforcing `docs/architecture/ASSET-PIPELINE.md`
(Phase 02+). Runtime schemas live in the `tbg-assets` crate; scripts here
are the offline enforcement side.

Contracts:

- Validation failures block the content commit, never the game build.
- Each gate is independently runnable and reports machine-checkable output.
- No validator may require network access or proprietary tooling.
