# Blender champion production

**Required for new champions.** Read [Art Direction](../ART-DIRECTION.md)
and the [desktop visual contract](../ui/DESKTOP-UI-CONTRACT.md) before
authoring. The feet circle is the sole on-model allegiance cue. The model,
weapon, fletching, clothes, glow and every Evolution use the same geometry
and intrinsic colors for both seats. Remove ownership geometry from the
source, such as chest badges, arm pennants and team tabards. Preserve
intrinsic material colors and the independent status outline.

## Editable source and export

The Archer and Bone Brute use component modeling. Each Blender object is
named for its physical piece and has a material and a named rig node. The
source is in [scripts/blender_champions](../../scripts/blender_champions).
Blender 4.5.15 LTS was used for these exports.

1. Build the base in joint-local, Y-up game coordinates in
   `characters.py`. Use supported bevels for metal, profile lofts for
   cloth and faces, and tapered tubes for curved bones and bow limbs.
   Keep the skeleton, sockets and grip points from `art/rig.rs` and the
   champion kit. `rig.py` maps game +Y up and +Z forward to Blender +Z
   up and -Y forward.
2. Place separate components on the named v2 bones, springs and sockets.
   Archer has a tailored doublet, piped hood, laminated bow and neutral
   fletching. Bone Brute has a sculpted skull, tapered ribs and a beveled
   cleaver. In the base, chest-local clavicles end at the shoulder pivots,
   and the burial sash shares Hips with the belt.
3. Author Base, A1, A2, B1 and B2 in that order. Keep branch identity in
   quivers, cloaks, bows, shields and blades. Archer's bow uses the kit's
   per-form half lengths and -0.10 depth at each tip. The game still
   animates StringTop, StringBottom and Nock. Reaper uses the current
   longer arm rest offsets and scythe tip.
4. Export from the repository root. The Blender exporter applies the
   evaluated bevels, triangulates surfaces, keeps corner normals, and
   merges geometry by joint and lit/glow layer. TBG4 stores a five-form
   length table, quantized rest offsets, 16-bit positions, normals and
   indices, and RGB vertex colors. Blender and the runtime both check
   the reserved allegiance colors. The runtime also checks the joint
   offsets against the current kit before loading a form.

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_game.py
```

5. Use the same FigureLibrary entry for the Laboratory, BattleLine,
   Evolution swap and inspection. Confirm stance, movement, weapon
   attachment, release/strike, hit and death at match distance and in
   the Laboratory. Use both seats and confirm that the feet circle
   carries allegiance independently of status. Record scoped desktop
   checks in the visual contract.

## Comparison experiment

For editable .blend files and base GLBs, run:

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_study.py
```

The study writes `scripts/exports/`. It builds the earlier prototype rig
and saves separate collections for each form and method. The generated
comparison joins designated body volumes, applies 0.0165 voxel remesh,
two relaxation passes and decimation, then transfers material colors and
up to two weights from nearby source joints. Gear stays rigid. The
generated Archer stretched between its sleeve and tunic in a draw pose,
and both generated bodies had higher triangle counts. The game uses the
modular Blender pieces. This also keeps the existing kit pose and
authoritative battle event path.

`export_game.py` produces the two checked-in game assets. The editable
scenes and intermediate build outputs are generated locally.
