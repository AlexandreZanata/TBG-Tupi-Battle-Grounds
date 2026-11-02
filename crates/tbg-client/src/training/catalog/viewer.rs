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
    #[cfg(not(target_os = "android"))]
    panel(p, c, lang);
    #[cfg(target_os = "android")]
    {
        if c.preview_controls {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    right: px(0),
                    top: px(0),
                    bottom: px(56),
                    min_height: px(0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(kit::INK),
                ZIndex(20),
                crate::ui::UiAction::Disabled,
            ))
            .with_children(|drawer| panel(drawer, c, lang));
        }
        p.spawn(skin::control(
            lang.key(if c.preview_controls {
                "android.hide_preview_controls"
            } else {
                "android.preview_controls"
            }),
            Action::PreviewControls,
            0.,
            c.preview_controls,
        ));
    }
}

fn panel(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
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
                        if cfg!(target_os = "android") {
                            48.
                        } else {
                            58.
                        },
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
                    if cfg!(target_os = "android") {
                        84.
                    } else {
                        104.
                    },
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
                    if cfg!(target_os = "android") {
                        84.
                    } else {
                        142.
                    },
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
                r.spawn(skin::control(
                    text,
                    a,
                    if cfg!(target_os = "android") {
                        48.
                    } else {
                        40.
                    },
                    false,
                ));
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
