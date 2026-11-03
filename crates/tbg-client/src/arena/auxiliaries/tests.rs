//! Regression: every War Auxiliary tier, platform, bridge and ship course stays
//! off the BattleLine road and clear of each Arena's existing static scenery.

use super::*;
use crate::arena::{cliffs, life, props, scenery, themes};
use crate::art::mesh;
use bevy::{ecs::system::RunSystemOnce, mesh::VertexAttributeValues};

const PAVED_HALF_WIDTH: f32 = 2.67;
const PAVED_MIN_X: f32 = -1.1;
const PAVED_MAX_X: f32 = 17.1;
/// Contact slack: touching faces (a plank resting on a rock) are not a collision.
const SLACK: f32 = 0.03;

#[derive(Clone, Copy, Debug)]
struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    fn of(points: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let mut it = points.into_iter();
        let first = it.next()?;
        Some(it.fold(
            Self {
                min: first,
                max: first,
            },
            |b, p| Self {
                min: b.min.min(p),
                max: b.max.max(p),
            },
        ))
    }
    fn overlaps(&self, o: &Self) -> bool {
        (self.min + SLACK).cmplt(o.max).all() && (o.min + SLACK).cmplt(self.max).all()
    }
    /// Overlap that shows above the sea surface; contact under water is hidden.
    fn meets_above_water(&self, o: &Self) -> bool {
        self.overlaps(o) && self.max.y.min(o.max.y) > water::LEVEL + SLACK
    }
}

fn points(m: &Mesh) -> Vec<Vec3> {
    let Some(VertexAttributeValues::Float32x3(points)) = m.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        panic!("auxiliary meshes expose positions");
    };
    match m.indices() {
        Some(indices) => indices.iter().map(|i| Vec3::from(points[i])).collect(),
        None => points.iter().copied().map(Vec3::from).collect(),
    }
}

fn triangles(m: &Mesh, world: Transform) -> Vec<Aabb> {
    points(m)
        .chunks_exact(3)
        .filter_map(|t| Aabb::of(t.iter().map(|p| world.transform_point(*p))))
        .collect()
}

/// One labelled world-space box per auxiliary part, every tier built, plus
/// samples of the ship's whole course at its largest (tier 3) size.
fn precinct_boxes(theme: Theme, seat: Seat, kind: AuxiliaryKind) -> Vec<Footprint> {
    let (at, yaw) = placement::anchor(theme, seat, kind);
    let heading = placement::layout(theme, seat).ship_heading;
    boxes_at(theme, seat, kind, at, yaw, heading)
}

fn boxes_at(
    theme: Theme,
    seat: Seat,
    kind: AuxiliaryKind,
    at: Vec2,
    yaw: f32,
    heading: f32,
) -> Vec<Footprint> {
    let pal = AuxPalette::for_theme(theme);
    let blue = seat == Seat::A;
    let place = placement::placement_at(seat, kind, at, yaw);
    let root = place.root;
    let mut parts: Vec<(&str, Mesh, Transform)> = match kind {
        AuxiliaryKind::Miner => vec![
            ("outcrop", models::mine_outcrop(pal), Transform::default()),
            ("shoring", models::mine_shoring(pal), Transform::default()),
            ("rails", models::mine_rails(pal), Transform::default()),
            (
                "cart",
                models::mine_cart(pal),
                Transform::from_xyz(0.78, 0.03, 0.0),
            ),
            (
                "miner",
                models::miner_body(blue),
                Transform::from_xyz(0.46, 0.0, 0.22),
            ),
            (
                "winch",
                structures::winch_frame(pal),
                Transform::from_translation(spots::WINCH),
            ),
            (
                "ore pile",
                structures::ore_pile(pal),
                Transform::from_translation(spots::ORE_PILE),
            ),
            (
                "headframe",
                structures::headframe(pal),
                Transform::from_translation(spots::HEADFRAME),
            ),
            (
                "banner",
                structures::banner(pal, blue),
                Transform::from_translation(spots::BANNER),
            ),
            (
                "apprentice",
                models::miner_body(blue),
                Transform::from_translation(spots::APPRENTICE).with_scale(Vec3::splat(0.82)),
            ),
        ],
        AuxiliaryKind::Merchant => {
            let mut v = vec![
                (
                    "pilings",
                    models::merchant_pier_pilings(pal),
                    Transform::default(),
                ),
                (
                    "deck",
                    models::merchant_pier_deck(pal),
                    Transform::default(),
                ),
                (
                    "stall",
                    models::merchant_stall_frame(pal),
                    Transform::default(),
                ),
                (
                    "awning",
                    models::merchant_stall_awning(pal),
                    Transform::default(),
                ),
                (
                    "merchant",
                    models::merchant_body(blue),
                    Transform::from_xyz(0.0, 0.0, 0.68),
                ),
                (
                    "goods",
                    structures::goods_stack(pal),
                    Transform::from_translation(spots::GOODS),
                ),
                (
                    "pavilion",
                    structures::pavilion(pal, blue),
                    Transform::from_translation(spots::PAVILION),
                ),
                (
                    "chest",
                    structures::chest(pal),
                    Transform::from_translation(spots::CHEST),
                ),
            ];
            // The whole sea lane, at the tier-3 galleon size, bow toward the berth.
            let yaw = (heading.sin()).atan2(-heading.cos());
            for step in 0..=8 {
                let p = placement::ship_point(heading, step as f32 * 0.35);
                let ship = Transform::from_xyz(p.x, water::LEVEL, p.y)
                    .with_rotation(Quat::from_rotation_y(yaw))
                    .with_scale(Vec3::splat(1.32));
                v.push(("ship course", models::merchant_ship(pal, blue), ship));
            }
            v
        }
    };
    if place.on_water {
        let platform = structures::platform(pal, pad_center(kind), pad_half(kind));
        parts.push(("platform", platform, Transform::default()));
    }
    if let Some((land, edge)) = place.bridge {
        let span = edge - land;
        let axis = Transform::from_translation(land)
            .with_rotation(Quat::from_rotation_y((-span.z).atan2(span.x)));
        parts.push(("bridge", structures::bridge_rails(pal, span.length()), axis));
    }
    parts
        .into_iter()
        .filter_map(|(name, m, local)| {
            let world = root * local;
            let tris = triangles(&m, world);
            Aabb::of(tris.iter().flat_map(|t| [t.min, t.max])).map(|b| Footprint {
                name: name.to_string(),
                bounds: b,
                tris,
            })
        })
        .collect()
}

/// One auxiliary part in world space: overall bounds and per-triangle boxes.
struct Footprint {
    name: String,
    bounds: Aabb,
    tris: Vec<Aabb>,
}

/// Every static obstacle the Arena builds near the shore, as world boxes.
fn arena_obstacles(theme: Theme) -> Vec<Obstacle> {
    let mut out = Vec::new();
    let mut kit = themes::build(theme);
    if theme == Theme::Forest {
        kit.props = props::scatter();
    }
    for (layer, parts) in [("prop", kit.props), ("glow", kit.glow)] {
        // Unfiltered: only sea-surface floaters may yield to a harbor (BR-AX15).
        for (shape, _, t) in parts {
            if let Some(b) = Aabb::of(
                points(&mesh::build(shape))
                    .into_iter()
                    .map(|p| t.transform_point(p)),
            ) {
                let radius = (b.max - b.min).xz().max_element() * 0.5;
                let floater = (t.translation.y < placement::FLOATER_Y)
                    .then_some((t.translation.xz(), radius));
                out.push(Obstacle {
                    what: format!("{layer} {shape:?} at {:?}", t.translation),
                    bounds: b,
                    floater,
                });
            }
        }
    }
    for b in triangles(&water::rocks(theme), Transform::default()) {
        out.push(Obstacle::fixed("offshore rock", b));
    }
    // The island rim cliff tops out at the meadow; only higher crags can block.
    for b in triangles(&cliffs::mesh(theme), Transform::default())
        .into_iter()
        .filter(|b| b.max.y > 0.1)
    {
        out.push(Obstacle::fixed("cliff crag", b));
    }
    // One box per tower piece: every kind and Fortification form, both sides.
    for (m, world) in crate::arena::strongholds::all_tower_meshes() {
        if let Some(b) = Aabb::of(points(&m).into_iter().map(|p| world.transform_point(p))) {
            out.push(Obstacle::fixed("Stronghold tower", b));
        }
    }
    for (name, m, world) in spawned_scenery(theme) {
        for b in triangles(&m, world) {
            out.push(Obstacle::fixed(&name, b));
        }
    }
    out
}

/// A static Arena obstacle; floaters lie on the sea surface (center, radius).
struct Obstacle {
    what: String,
    bounds: Aabb,
    floater: Option<(Vec2, f32)>,
}

impl Obstacle {
    fn fixed(what: &str, bounds: Aabb) -> Self {
        Self {
            what: what.to_string(),
            bounds,
            floater: None,
        }
    }
}

/// Meshes spawned as entities (life, themed motion, Forest seascape) with world transforms.
fn spawned_scenery(theme: Theme) -> Vec<(String, Mesh, Transform)> {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world
        .run_system_once(
            move |mut commands: Commands,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut mats: ResMut<Assets<StandardMaterial>>| {
                let material = mats.add(StandardMaterial::default());
                life::spawn(&mut commands, &mut meshes, &mut mats, &material, theme);
                themes::spawn_motion(&mut commands, &mut meshes, &material, theme);
                if theme == Theme::Forest {
                    scenery::spawn(&mut commands, &mut meshes, &material);
                }
            },
        )
        .unwrap();
    let entities: Vec<(Entity, Handle<Mesh>, Option<String>)> = world
        .query::<(Entity, &Mesh3d, Option<&Name>)>()
        .iter(&world)
        .map(|(e, m, n)| (e, m.0.clone(), n.map(|n| n.to_string())))
        .collect();
    entities
        .into_iter()
        .filter_map(|(e, handle, name)| {
            let mut transform = *world.get::<Transform>(e)?;
            let mut at = e;
            while let Some(parent) = world.get::<ChildOf>(at).map(|c| c.parent()) {
                transform = *world.get::<Transform>(parent)? * transform;
                at = parent;
            }
            let mesh = world.resource::<Assets<Mesh>>().get(&handle)?.clone();
            Some((name.unwrap_or_else(|| "scenery".into()), mesh, transform))
        })
        .collect()
}

#[test]
fn p74_auxiliary_precincts_and_ships_clear_the_wide_road_across_all_eight_arenas() {
    for arena in tbg_core::Arena::ALL {
        let theme = Theme::from(arena);
        for seat in Seat::BOTH {
            for kind in AuxiliaryKind::ALL {
                for Footprint {
                    name, bounds: b, ..
                } in precinct_boxes(theme, seat, kind)
                {
                    let in_road = b.max.y > 0.02
                        && b.max.x > PAVED_MIN_X + 0.001
                        && b.min.x < PAVED_MAX_X - 0.001
                        && b.max.z > -PAVED_HALF_WIDTH + 0.001
                        && b.min.z < PAVED_HALF_WIDTH - 0.001;
                    assert!(
                        !in_road,
                        "{theme:?} {seat:?} {kind:?} {name} enters the road: {b:?}"
                    );
                }
            }
        }
    }
}

/// Stronghold footprints near both lane ends, which precincts must not enter.
fn in_keep_out(b: &Aabb) -> bool {
    let near_end = b.max.x > -3.0 && b.min.x < -1.1 || b.max.x > 17.1 && b.min.x < 19.0;
    b.max.y > 0.02 && near_end && b.max.z > -2.3 && b.min.z < 2.3
}

fn in_road(b: &Aabb) -> bool {
    b.max.y > 0.02
        && b.max.x > PAVED_MIN_X + 0.001
        && b.min.x < PAVED_MAX_X - 0.001
        && b.max.z > -PAVED_HALF_WIDTH + 0.001
        && b.min.z < PAVED_HALF_WIDTH - 0.001
}

/// First collision of a precinct footprint, if any: road and Stronghold
/// keep-outs by part bounds, scenery by triangle against triangle.
fn collision(
    parts: &[Footprint],
    obstacles: &[Obstacle],
    yields: impl Fn(Vec2, f32) -> bool,
) -> Option<String> {
    parts.iter().find_map(|part| {
        if in_road(&part.bounds) {
            return Some(format!("{} enters the road", part.name));
        }
        if in_keep_out(&part.bounds) {
            return Some(format!("{} enters a Stronghold footprint", part.name));
        }
        obstacles
            .iter()
            .filter(|o| part.bounds.overlaps(&o.bounds))
            .filter(|o| !o.floater.is_some_and(|(at, r)| yields(at, r)))
            .find(|o| part.tris.iter().any(|t| t.meets_above_water(&o.bounds)))
            .map(|o| format!("{} hits {}", part.name, o.what))
    })
}

/// Parts that must stay in view of the common camera (SIDE-01).
const KEY_PARTS: [&str; 6] = [
    "outcrop",
    "miner",
    "headframe",
    "stall",
    "merchant",
    "pavilion",
];

fn camera_eye() -> Vec3 {
    let mut eye = Transform::default();
    let mut projection = Projection::Perspective(PerspectiveProjection::default());
    crate::camera::frame(&mut eye, &mut projection, 1920.0, 1080.0);
    eye.translation
}

/// Slab test: does the segment `a -> b` pass through box `o`?
fn segment_hits(a: Vec3, b: Vec3, o: &Aabb) -> bool {
    let d = b - a;
    let (mut t0, mut t1) = (0.0_f32, 1.0_f32);
    for axis in 0..3 {
        let (s, v) = (a[axis], d[axis]);
        if v.abs() < 1e-6 {
            if s < o.min[axis] || s > o.max[axis] {
                return false;
            }
            continue;
        }
        let (mut n, mut f) = ((o.min[axis] - s) / v, (o.max[axis] - s) / v);
        if n > f {
            std::mem::swap(&mut n, &mut f);
        }
        t0 = t0.max(n);
        t1 = t1.min(f);
        if t0 > t1 {
            return false;
        }
    }
    true
}

/// Screen pixel (1920x1080, y down) of a world point through the common camera.
fn to_screen(at: Vec3) -> Option<Vec2> {
    let mut eye = Transform::default();
    let mut projection = Projection::Perspective(PerspectiveProjection::default());
    crate::camera::frame(&mut eye, &mut projection, 1920.0, 1080.0);
    let view = eye.compute_affine().inverse();
    let clip = Mat4::perspective_rh(crate::camera::FOV_Y, 1920.0 / 1080.0, 0.1, 500.0)
        * Mat4::from(view)
        * at.extend(1.0);
    (clip.w > 0.0).then(|| {
        let ndc = clip.xy() / clip.w;
        Vec2::new((ndc.x + 1.0) * 960.0, (1.0 - ndc.y) * 540.0)
    })
}

/// Key parts must be on screen, below the score bar and outside the Shop and
/// auxiliary panel that cover the bottom center.
fn offscreen(parts: &[Footprint]) -> Option<String> {
    parts
        .iter()
        .filter(|p| KEY_PARTS.contains(&p.name.as_str()))
        .find_map(|part| {
            let at = (part.bounds.min + part.bounds.max) * 0.5;
            let px = to_screen(at)?;
            let on_screen = (40.0..=1880.0).contains(&px.x) && (150.0..=1040.0).contains(&px.y);
            let under_hud = (470.0..=1730.0).contains(&px.x) && px.y >= 720.0;
            (!on_screen || under_hud)
                .then(|| format!("{} is not visible on screen at {px:?}", part.name))
        })
}

/// First key part hidden from the camera by a Stronghold or tall scenery.
fn occlusion(parts: &[Footprint], obstacles: &[Obstacle], eye: Vec3) -> Option<String> {
    let tall = |o: &&Obstacle| o.what == "Stronghold tower" || o.bounds.max.y > 1.0;
    parts
        .iter()
        .filter(|p| KEY_PARTS.contains(&p.name.as_str()))
        .find_map(|part| {
            let mut at = (part.bounds.min + part.bounds.max) * 0.5;
            at.y = at.y.max(0.25);
            obstacles
                .iter()
                .filter(tall)
                .filter(|o| !(o.bounds.min.cmple(at).all() && o.bounds.max.cmpge(at).all()))
                .find(|o| segment_hits(at, eye, &o.bounds))
                .map(|o| format!("{} hidden behind {}", part.name, o.what))
        })
}

#[test]
fn br_ax15_every_tier_bridge_and_ship_course_clears_existing_arena_scenery() {
    let mut failures = Vec::new();
    for arena in tbg_core::Arena::ALL {
        let theme = Theme::from(arena);
        let obstacles = arena_obstacles(theme);
        let eye = camera_eye();
        for seat in Seat::BOTH {
            for kind in AuxiliaryKind::ALL {
                let runtime = |p: Vec2, r: f32| {
                    placement::clears_floater(theme, Vec3::new(p.x, -1.4, p.y), r)
                };
                let parts = precinct_boxes(theme, seat, kind);
                let hit = collision(&parts, &obstacles, runtime)
                    .or_else(|| occlusion(&parts, &obstacles, eye))
                    .or_else(|| offscreen(&parts));
                if let Some(hit) = hit {
                    failures.push(format!("{theme:?} {seat:?} {kind:?} {hit}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "auxiliary collisions:\n{}",
        failures.join("\n")
    );
}

/// Layout lab: prints a collision-free `LAYOUTS` table, preferring the
/// smallest move and turn from the base anchors and the default sea lane.
/// `cargo test -p tbg-client --bin tbg-client search_collision_free_layouts -- --ignored --nocapture`
#[test]
#[ignore]
fn search_collision_free_layouts() {
    use std::f32::consts::{PI, TAU};
    let steps: Vec<f32> = (-9..=9).map(|i| i as f32 * 0.25).collect();
    const YAWS: [f32; 7] = [0.0, 0.5, -0.5, 1.0, -1.0, 1.57, -1.57];
    // Candidates ordered by how far they move the precinct from its base.
    let mut poses: Vec<(Vec2, f32)> = steps
        .iter()
        .flat_map(|&x| {
            steps
                .iter()
                .flat_map(move |&z| YAWS.iter().map(move |&y| (Vec2::new(x, z), y)))
        })
        .collect();
    poses.sort_by(|a, b| (a.0.length() + a.1.abs()).total_cmp(&(b.0.length() + b.1.abs())));
    let gap = |h: f32| ((h - placement::DEFAULT_HEADING + PI).rem_euclid(TAU) - PI).abs();
    let mut headings: Vec<f32> = (0..24).map(|i| i as f32 * TAU / 24.0).collect();
    headings.sort_by(|a, b| gap(*a).total_cmp(&gap(*b)));
    for arena in tbg_core::Arena::ALL {
        let theme = Theme::from(arena);
        let obstacles = arena_obstacles(theme);
        let eye = camera_eye();
        let mut row = Vec::new();
        for seat in Seat::BOTH {
            let mut found = [None, None];
            let mut heading = None;
            for kind in AuxiliaryKind::ALL {
                let base = placement::base(seat, kind);
                let lanes: &[f32] = if kind == AuxiliaryKind::Merchant {
                    &headings
                } else {
                    &[placement::DEFAULT_HEADING]
                };
                // Then the whole coast of this seat's quadrant, pier facing out to sea.
                let quadrant = match (seat, kind) {
                    (Seat::A, AuxiliaryKind::Merchant) => (0.5 * PI, PI),
                    (Seat::B, AuxiliaryKind::Merchant) => (0.0, 0.5 * PI),
                    (Seat::A, AuxiliaryKind::Miner) => (PI, 1.5 * PI),
                    (Seat::B, AuxiliaryKind::Miner) => (1.5 * PI, TAU),
                };
                let coast = (0..=24).flat_map(|i| {
                    let a = quadrant.0 + (quadrant.1 - quadrant.0) * i as f32 / 24.0;
                    [1.0, 1.04, 1.08, 1.13, 1.18]
                        .into_iter()
                        .flat_map(move |k| {
                            let at = Vec2::new(8.0 + 12.0 * k * a.cos(), 5.7 * k * a.sin());
                            let normal = Vec2::new(a.cos() / 12.0, a.sin() / 5.7).normalize();
                            // Local +Z (the pier) along the outward normal, or turned along the coast.
                            let out = normal.x.atan2(normal.y);
                            [0.0, 0.5, -0.5, 1.0, -1.0, 1.57, -1.57]
                                .map(move |turn| (at - base, out + turn))
                        })
                });
                let candidates: Vec<(Vec2, f32)> = poses.iter().copied().chain(coast).collect();
                let mut why: std::collections::BTreeMap<String, usize> = Default::default();
                'search: for &(off, yaw) in &candidates {
                    let at = base + off;
                    if at.y.abs() > 5.3 || !(-4.8..=20.8).contains(&at.x) {
                        continue;
                    }
                    let parts = boxes_at(theme, seat, kind, at, yaw, placement::DEFAULT_HEADING);
                    let (_, dock): (Vec<_>, Vec<_>) =
                        parts.into_iter().partition(|p| p.name == "ship course");
                    let harbor = |h: f32| {
                        move |p: Vec2, r: f32| placement::in_harbor(seat, kind, at, yaw, h, p, r)
                    };
                    let blocked = collision(&dock, &obstacles, harbor(placement::DEFAULT_HEADING))
                        .or_else(|| occlusion(&dock, &obstacles, eye))
                        .or_else(|| offscreen(&dock));
                    if let Some(hit) = blocked {
                        *why.entry(hit.split(" at ").next().unwrap_or("").into())
                            .or_default() += 1;
                        continue;
                    }
                    *why.entry("DOCK CLEAR".into()).or_default() += 1;
                    for &h in lanes {
                        let hit = collision(
                            &boxes_at(theme, seat, kind, at, yaw, h),
                            &obstacles,
                            harbor(h),
                        );
                        if let Some(hit) = &hit {
                            *why.entry(format!(
                                "lane: {}",
                                hit.split(" at ").next().unwrap_or("")
                            ))
                            .or_default() += 1;
                        }
                        if hit.is_none() {
                            found[usize::from(kind.slot())] = Some((off, yaw));
                            heading = heading.or((kind == AuxiliaryKind::Merchant).then_some(h));
                            break 'search;
                        }
                    }
                }
                if found[usize::from(kind.slot())].is_none() {
                    let mut top: Vec<_> = why.into_iter().collect();
                    top.sort_by(|a, b| b.1.cmp(&a.1));
                    top.truncate(8);
                    println!("  {theme:?} {seat:?} {kind:?} blockers: {top:?}");
                }
            }
            row.push(match (found, heading) {
                ([Some((m, my)), Some((c, cy))], Some(h)) => format!(
                    "layout_of([{:.2}, {:.2}, {my:.2}], [{:.2}, {:.2}, {cy:.2}], {h:.3})",
                    m.x, m.y, c.x, c.y
                ),
                _ => format!("UNSOLVED {seat:?} {found:?}"),
            });
        }
        println!("    // {theme:?}\n    [{}],", row.join(", "));
    }
}

#[test]
fn red_side_precincts_render_both_faces_under_their_mirror() {
    for kind in AuxiliaryKind::ALL {
        assert!(
            placement(Theme::Forest, Seat::B, kind).root.scale.x < 0.0,
            "Seat B mirrors along X"
        );
    }
    assert_eq!(
        precinct_material().cull_mode,
        None,
        "mirrored winding must not cull outward faces"
    );
}

#[test]
fn br_ax15_static_land_props_never_yield_even_at_a_precinct_anchor() {
    for arena in tbg_core::Arena::ALL {
        let theme = Theme::from(arena);
        let kit = themes::build(theme);
        for (shape, color, _) in kit.props.into_iter().chain(kit.glow) {
            for seat in Seat::BOTH {
                for kind in AuxiliaryKind::ALL {
                    let (at, _) = placement::anchor(theme, seat, kind);
                    let part = (shape, color, Transform::from_xyz(at.x, 0.0, at.y));
                    assert!(
                        !yields_to_precinct(theme, &part),
                        "BR-AX15: preserve static land scenery instead of hiding a collision"
                    );
                }
            }
        }
    }
}
