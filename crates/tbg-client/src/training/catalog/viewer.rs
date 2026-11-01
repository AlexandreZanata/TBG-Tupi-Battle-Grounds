//! Preview caption and compact rows reserve a clear, draggable stage above the controls.

use super::*;

#[derive(Component)]
pub(crate) struct Viewport;

pub(super) fn controls(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
    // Input uses this actual layout rectangle, so resizing never turns a button into a drag surface.
    p.spawn((
        Node {
            width: percent(100),
            flex_grow: 1.,
            min_height: px(80),
            ..default()
        },
        Viewport,
    ));
    p.spawn((
        Node {
            width: percent(100),
            padding: UiRect::all(px(12)),
            flex_direction: FlexDirection::Column,
            row_gap: px(7),
            border_radius: BorderRadius::all(px(5)),
            flex_shrink: 0.,
            ..default()
        },
        BackgroundColor(Color::srgb_u8(8, 24, 34)),
    ))
    .with_children(|p| {
        p.spawn(kit::heading(content::unit_name(lang, c.selected), 25.));
        p.spawn(kit::muted(lang.key("training.preview_note"), 12.));
        row(p, |r| {
            for (name, t, b) in [
                ("Base", 0, 0),
                ("A1", 1, 0),
                ("A2", 2, 0),
                ("B1", 1, 1),
                ("B2", 2, 1),
            ] {
                if t == 0 || !c.selected.is_summon() {
                    r.spawn(skin::control(
                        name,
                        Action::Form(t, b),
                        58.,
                        c.form == (t, b),
                    ));
                }
            }
        });
        let attack = if def(c.selected).range > 1500 {
            Clip::AttackRanged
        } else {
            Clip::AttackMelee
        };
        row(p, |r| {
            for (key, clip) in [
                ("idle", Clip::Idle),
                ("walk", Clip::Walk),
                ("attack", attack),
                ("cast", Clip::Cast),
            ] {
                r.spawn(skin::control(
                    lang.key(&format!("training.clip.{key}")),
                    Action::Clip(clip),
                    104.,
                    c.clip == clip,
                ));
            }
        });
        row(p, |r| {
            for (key, clip) in [
                ("death", Clip::Death),
                ("evolve", Clip::Evolve),
                ("dance", Clip::Victory),
            ] {
                r.spawn(skin::control(
                    lang.key(&format!("training.clip.{key}")),
                    Action::Clip(clip),
                    142.,
                    c.clip == clip,
                ));
            }
        });
        row(p, |r| {
            for (text, a) in [
                ("<", Action::Rotate(-1)),
                (">", Action::Rotate(1)),
                ("-", Action::Zoom(-1)),
                ("+", Action::Zoom(1)),
            ] {
                r.spawn(skin::control(text, a, 40., false));
            }
            r.spawn(skin::control(
                lang.key(if c.reduced {
                    "training.reduced_on"
                } else {
                    "training.reduced_off"
                }),
                Action::Motion,
                244.,
                c.reduced,
            ));
        });
        p.spawn(kit::muted(lang.key("training.drag_help"), 11.));
    });
}
fn row(p: &mut ChildSpawnerCommands, spawn: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node {
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(6),
        row_gap: px(6),
        ..default()
    })
    .with_children(spawn);
}
