# Proportional Practice arena picture

Owner request: remove the flattened arena thumbnail, let the picture occupy the
unused pane area above the selector/Play buttons, and crop the sides when needed.
Also audit that all laboratory improvements and fixes in this chat propagate to
Windows and desktop web; do not duplicate changes if the implementation is shared.

Implementation: a flexible picture replaces the 92-pixel stretched thumbnail;
post-layout centered cover cropping preserves source proportions. Practice content
uses compact consistent gaps and 30-pixel difficulty rows so the image receives
enough height for a lateral center crop at the observed desktop window sizes.
The existing image loading, selector action and Play action are retained.

Propagation: audited build.rs, training plugin, catalog, viewer, preview, stage,
runtime actions and shared locale/arena assets. They are already one implementation
on Linux, Windows and wasm32; no per-platform port is necessary. Shared source does
not prove exact rendered correspondence, which remains a separate contract gate.

Validation level: work front with the owner-authorized native build/reinstall.
Pending: format/source gate, native release build and focused desktop-size/arena
observations, installation checks. Windows/web runtime checks remain pending
consolidation. No full suite or whole-roster gallery is requested. No push under
the latest owner-provided AGENTS.md.
