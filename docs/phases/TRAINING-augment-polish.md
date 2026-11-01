# Laboratory proportions and Augment card cleanup

Owner request: use the supplied laboratory screenshot's proportions, with a
smaller arena picture and standard difficulty controls above it. Remove the
misaligned green Augment shadow and all Bronze/Silver/Gold/Diamond text labels.
Retain visual rarity frames, prices and the selected-card outline.

The laboratory now uses the standard 38-pixel controls, 8-pixel gaps and a
10-pixel difficulty-note inset. The remaining height still belongs to the
centered, proportional arena picture; selector and Play stay at the bottom.
No arena asset, rule, default difficulty, selection action or crop code changes.

Augment cards no longer contain a BoxShadow component or a rarity text node.
The obsolete rarity text/color update code is removed. Frame assets, illustration,
name/description/price positions, selected border, affordability guards and
confirmation actions are unchanged. No assets or game-rule values change.

Validation level: work front with the owner's standing native build/reinstall
authorization. Formatting, source routing and diff checks passed on the current
Rust changes. Final build, targeted Linux captures, packaging and installation
are pending. Windows/web rendered correspondence and the full locale/viewport/
DPI matrix remain pending consolidation. No full suite or push is authorized.
