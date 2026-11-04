# Blender champion production

**Required for new champions.** Read [Art Direction](../ART-DIRECTION.md)
and the [desktop visual contract](../ui/DESKTOP-UI-CONTRACT.md) before
authoring. The feet circle is the champion's sole allegiance marker. The model,
weapon, fletching, clothes, glow and every Evolution use the same geometry
and intrinsic colors for both seats. Remove ownership geometry from the
source, such as chest badges, arm pennants and team tabards. Preserve
intrinsic material colors and the independent status outline.

## Editable source and export

The initial roster and the first expanded-roster figures use component modeling.
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
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_lineup.py -- shieldbearer hexer
```

The renders are in Base, A1, A2, B1, B2 order, from left to right.

![Shieldbearer modular Blender lineup](../media/blender-champions/shieldbearer.webp)

![Hexer modular Blender lineup](../media/blender-champions/hexer.webp)

## Sawbot and Bloomkeeper

The Sawbot has a riveted boiler, piston limbs, a furnace eye and a
directional toothed disk. Overclock adds copper, hot vents and, at A2,
a second saw with paired exhausts. Plating adds steel guards and, at
B2, horns and a larger disk. The disks are centered at hand-local
Y −0.13 and rotate around the existing hand axes. The right-hand
tooth radius follows the strike profile at 0.16 or 0.22. The studio
lineup tilts the arms to show the face of each saw.

The Bloomkeeper has a carved lightwood face, folded leaves and a petal
skirt. The A branch adds shoulder blossoms and a flowering bough on
the back spring. The B branch adds bark guards, thorns and, at B2, a
thorn crown and amber seed. The flower and seed share the cast muzzle
at weapon-local `(0.0, 0.90, 0.03)`.

The exports contain 3,828–4,536 triangles per Sawbot form and
3,094–3,924 per Bloomkeeper form. The files are 424,161 and 248,184
bytes, respectively.

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_game.py -- sawbot bloomkeeper
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_lineup.py -- sawbot bloomkeeper
```

The renders are in Base, A1, A2, B1, B2 order, from left to right.

![Sawbot modular Blender lineup](../media/blender-champions/sawbot.webp)

![Bloomkeeper modular Blender lineup](../media/blender-champions/bloomkeeper.webp)

## Siege Walker and Rootcaller

The Siege Walker's face is sculpted as one shell with defined cheeks,
recessed eyes and fitted tusks. The helmet visor clears the eye line.
The shoulder bombard has a tapered steel barrel, a separate dark bore
liner, a carved stock and a braced support grip. Longshot adds a
centered telescope. Artillery gets twin bores, Shrapnel a flared bell,
and Concussion a shell drum. The bore axis stays at weapon-local Y 0.26.
Keep the five muzzle planes, the support grip at `(0.28, 0.10, 0.36)`
and the native animated reload lever on its own socket. The lineup shows
the shoulder carry.

The Rootcaller has a fluted trunk, jointed root claws, an amber eye glow
and moss beard. The A branch adds vines and a crown of roots. The B
branch adds a broad cap, glowing spores and a fungal cluster on the
back spring. The middle claw on the right-hand socket ends at
`(0, -0.06 - length, 0.04)`, with the kit's per-form claw length.

The refined Siege Walker export has 5,234–6,348 triangles per form and
uses 534,937 bytes. This excludes the native reload lever. The Rootcaller
has 2,386–5,692 triangles per form and uses 253,790 bytes.

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_game.py -- siegewalker rootcaller
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_lineup.py -- siegewalker rootcaller
```

The renders are in Base, A1, A2, B1, B2 order, from left to right.

![Siege Walker modular Blender lineup](../media/blender-champions/siegewalker.webp)

![Rootcaller modular Blender lineup](../media/blender-champions/rootcaller.webp)

## Nester and Rampart

The Nester wears a broadleaf hood and carries a split-willow basket.
Brood grows the clutch and A2 builds a woven straw skep with amber
brood pods. The B branch changes the clutch from green venom eggs to
thorned violet eggs. Keep the basket on BackSpring and the seed
centered at the existing cast point, weapon-local `(0, 0.03, 0.06)`.

The Rampart carries an oak wall with riveted crossbeams over layered
armor. Bastion adds reinforced edges and A2 adds stone crenellations.
The B branch adds wall spikes and a heated forearm ram at B2. Keep
the wall chest-local with its rail at the existing left-hand grip.
The right-hand contact plate remains at hand-local Y −0.15.

The Nester export has 3,720–4,568 triangles per form and uses 276,915
bytes. The Rampart has 4,790–5,546 triangles per form and uses 496,172
bytes.

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_game.py -- nester rampart
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_lineup.py -- nester rampart
```

The renders are in Base, A1, A2, B1, B2 order, from left to right.

![Nester modular Blender lineup](../media/blender-champions/nester.webp)

![Rampart modular Blender lineup](../media/blender-champions/rampart.webp)

## Longaxe and Farwatch

Longaxe has a stitched ochre gambeson, a fitted kettle helmet and a
forged ash poleaxe. The A branch adds plate, a broader edge and a gilded
crescent. The B branch adds a twin blade, then a hook and horned helmet.
Keep the working edge at weapon-local `(0, 1.728, 0.22)` and the existing
two-hand grip. The lineup tilts the polearm to show the whole haft.

Farwatch has a slate field coat, fitted gloves and a cranked arbalest
with laminated steel limbs. The A branch adds a centered scope and
a wide feathered hat. The B branch adds a scarf and heated bolt point.
The bolt ends at weapon-local `(0, 0.05, 0.58)`. The fore-stock
encloses the existing support-hand point `(0.02, 0, 0.14)`.

Longaxe uses 2,938–3,334 triangles per form and 255,053 bytes. Farwatch
uses 3,674–4,098 triangles per form and 353,534 bytes.

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_game.py -- longaxe farwatch
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_lineup.py -- longaxe farwatch
```

![Longaxe modular Blender lineup](../media/blender-champions/longaxe.webp)

![Farwatch modular Blender lineup](../media/blender-champions/farwatch.webp)

## Bombard and Rotcap

Bombard's leather coat, riveted cuirass, helmet and smoked goggles are
separate components on the existing rig. The right hand carries the
mortar at the hip and the support grip encloses weapon-local
`(0, -0.03, 0.24)`. The open muzzle planes sit at weapon-local Y 0.10
and Z 0.43, 0.45, 0.51, 0.43 and 0.47 in Base, A1, A2, B1 and B2 order.
A1 adds a flared mortar and bomb rack, and A2 adds a hot collar and a
furnace pack. B1 adds a hull shield, and B2 adds paired bores on either
side of the shared launch axis and a rear ammunition drum.

Rotcap's stem, collar, limbs, cap skin, membrane and radial gills are
individually modeled. The spitting rim is centered at head-local
`(0, -0.05187, 0.13923)`, with the throat recessed behind it and the
gills above it. A1 adds toxin freckles and vials, and A2 adds a broader
violet cap with venom drops. B1 grows shoulder caps, and B2 grows a
cluster of caps and pods on BackSpring. The palette stays the same
across seats. The game uses the existing recoil, reload and spit clips.

Bombard uses 5,266–6,526 triangles per form and 513,618 bytes. Rotcap
uses 4,686–7,998 triangles per form and 408,832 bytes.

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_game.py -- bombard rotcap
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_lineup.py -- bombard rotcap
```

![Bombard modular Blender lineup](../media/blender-champions/bombard.webp)

![Rotcap modular Blender lineup](../media/blender-champions/rotcap.webp)

## Old Finn and Antidote Brewer

Old Finn wears a waxed ochre coat, a sou'wester and a forked white
beard. The rod, wrapped grip, working reel silhouette and line are
separate pieces on WeaponR. The rod and line end at the existing
fish-release point `(0, 0.902, 0)`. The ochre branch adds a larger catch
and a hauling float in A1, then an offset silver-bellied catfish with
long barbels in A2. The maritime-blue branch adds a two-tone storm
bib and hip rope coil in B1, then lighter blue oilskin, net floats,
extra catches and a brass hat clasp in B2. The basket and catches
move with BackSpring.

Antidote Brewer wears a linen apron and a two-leaf hood. The rim of
the remedy flask sits at WeaponR-local `(0, 0.07, 0.03)`. The amber
branch adds a gilt collar, braided herb crown and pouch in A1, then a
copper still with a curved condenser and glowing port in A2. The violet
branch adds a four-vial bandolier in B1, then a silver-collared second
flask on OffhandL, a mixing towel and cast back cauldron in B2.
The lower apron follows Hips and the workshop gear follows BackSpring.
Both champions keep their 0.22 s cast releases.

The branch palettes are intrinsic and match across seats. Old Finn
uses 4,016–5,618 triangles per form and 360,707 bytes. Antidote Brewer
uses 2,992–4,150 triangles per form and 299,567 bytes.

```sh
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/export_game.py -- oldfinn antidotebrewer
blender -b -t 2 --python-exit-code 1 --python scripts/blender_champions/render_lineup.py -- oldfinn antidotebrewer
```

![Old Finn modular Blender lineup](../media/blender-champions/oldfinn.webp)

![Antidote Brewer modular Blender lineup](../media/blender-champions/antidotebrewer.webp)
