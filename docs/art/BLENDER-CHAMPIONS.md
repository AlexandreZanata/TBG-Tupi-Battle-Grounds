# Blender champion production

**Required for new champions.** Read [Art Direction](../ART-DIRECTION.md)
and the [desktop visual contract](../ui/DESKTOP-UI-CONTRACT.md) before
authoring. The feet circle is the champion's sole allegiance marker. The model,
weapon, fletching, clothes, glow and every Evolution use the same geometry
and intrinsic colors for both seats. Remove ownership geometry from the
source, such as chest badges, arm pennants and team tabards. Preserve
intrinsic material colors and the independent status outline.

## Editable source and export

The Archer, Bone Brute, Shieldbearer and Hexer use component modeling.
Each Blender object is named for its physical piece and has a material and
a named rig node. The source is in
[scripts/blender_champions](../../scripts/blender_champions).
Blender 4.5.15 LTS was used for these exports.

1. Build the base in joint-local, Y-up game coordinates in a champion
   model module. Use supported bevels for metal, profile lofts for
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

Pass champion IDs after `--` to export only those champions.

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
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_study.py
```

The study writes `scripts/exports/` and the render step writes the two
comparison PNGs in `scripts/exports/renders/`. The study builds the earlier
prototype rig and saves separate collections for each form and method. The generated
comparison joins designated body volumes, applies 0.0165 voxel remesh,
two relaxation passes and decimation, then transfers material colors and
up to two weights from nearby source joints. Gear stays rigid. The
generated Archer stretched between its sleeve and tunic in a draw pose,
and both generated bodies had higher triangle counts. The game uses the
modular Blender pieces. This also keeps the existing kit pose and
authoritative battle event path.

`export_game.py` produces the game assets. The editable
scenes and intermediate build outputs are generated locally.
The Archer export has 5,058–5,464 triangles per form and occupies 436,544
bytes. The Bone Brute export has 6,384–6,998 triangles per form and occupies
571,208 bytes. These counts exclude runtime bow string and projectile meshes.

## Authoring renders

The Blender Cycles renders show the source design in the rest pose. Each
image has the modular model on the left and the voxel comparison on the
right. The game uses the left-hand method with its shared materials and
animated weapon sockets.

![Archer modular and generated Blender models](../media/blender-champions/archer.webp)

![Bone Brute modular and generated Blender models](../media/blender-champions/bonebrute.webp)

## Shieldbearer and Hexer

The Shieldbearer uses a quilted gambeson, curved armor, a forged kite
shield and a diamond-section sword. The A branch moves from a kettle
helm and wider shield to a great helm and crenellated tower shield.
The B branch adds a cavalry mantle, plume, wing crest and longer sword.
Keep the shield face along the offhand's +X axis. The blade point stays
at weapon-local Z 0.62, 0.72 or 0.82 according to the kit.

The Hexer uses a tattered robe, crooked hat and skull lantern in a
forked blackthorn staff. Withering and Doom add a broken halo and
gold thorns. Plague and Pestilence add a green censer and a stitched
plague beak. The staff mouth stays at the kit's cast muzzle
`(0.0, 0.88, 0.09)`. The runtime renders ritual violet and soul green
in its separate glow layer.

The exports contain 3,418–4,110 triangles per Shieldbearer form and
4,160–4,568 per Hexer form. The files are 360,732 and 313,317 bytes,
respectively. Render the ten forms with the same Blender studio setup.

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_game.py -- shieldbearer hexer
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_lineup.py
```

The renders are in Base, A1, A2, B1, B2 order, from left to right.

![Shieldbearer modular Blender lineup](../media/blender-champions/shieldbearer.webp)

![Hexer modular Blender lineup](../media/blender-champions/hexer.webp)
