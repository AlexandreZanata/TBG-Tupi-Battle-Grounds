//! Automatic in-place skeletons for every Linux UI image, glyph atlas and pending data label.

use super::{gpu::Prepared, Decoration, Status};
use bevy::{
    asset::{AssetId, LoadState},
    prelude::*,
    text::{FontSource, TextLayoutInfo},
};

#[derive(Component)]
struct ImageMask {
    overlay: Entity,
    error: Entity,
    color: Color,
    written: Color,
    fade: f32,
}
#[derive(Component)]
struct TextMask {
    gradient: BackgroundGradient,
    color: Color,
    written: Color,
    fade: f32,
}

#[derive(Component)]
struct Bar(Entity);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (register, ApplyDeferred)
            .chain()
            .before(bevy::ui::UiSystems::Prepare),
    )
    .add_systems(
        PostUpdate,
        (paint_images, paint_text, fade_bars)
            .chain()
            .after(super::scene::Prepare)
            .after(bevy::ui::UiSystems::Layout)
            .before(bevy::ui::UiSystems::Stack),
    );
}

fn register(
    mut commands: Commands,
    images: Query<(Entity, &ImageNode), (With<Node>, Without<ImageMask>, Without<Decoration>)>,
    texts: Query<
        (Entity, &TextColor, Option<&BackgroundGradient>),
        (
            With<Node>,
            With<Text>,
            Without<TextMask>,
            Without<Decoration>,
        ),
    >,
    parents: Query<&ChildOf>,
    credits: Query<(), With<super::super::attribution::Credit>>,
) {
    for (entity, image) in &images {
        let (overlay, error) = mask(&mut commands, entity);
        commands.entity(entity).insert(ImageMask {
            overlay,
            error,
            color: image.color,
            written: image.color,
            fade: 1.,
        });
    }
    for (entity, color, gradient) in &texts {
        let mut ancestor = entity;
        while let Ok(parent) = parents.get(ancestor) {
            ancestor = parent.parent();
        }
        if credits.contains(ancestor) {
            continue;
        }
        let gradient = gradient.cloned().unwrap_or_default();
        // Text must stay a layout leaf: a child UI box removes its intrinsic measurement.
        commands.entity(entity).insert((
            TextMask {
                gradient: gradient.clone(),
                color: color.0,
                written: color.0,
                fade: 1.,
            },
            gradient,
        ));
    }
}

fn mask(commands: &mut Commands, parent: Entity) -> (Entity, Entity) {
    let overlay = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                border_radius: BorderRadius::all(px(6)),
                overflow: Overflow::clip(),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(super::ink(1.)),
            Visibility::Hidden,
            Decoration,
            ZIndex(-1),
        ))
        .id();
    commands.entity(parent).add_child(overlay);
    commands.entity(overlay).with_children(|block| {
        for (width, top) in [(70., 72.), (45., 84.)] {
            block.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(10),
                    top: percent(top),
                    width: percent(width),
                    height: px(6),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(58, 75, 82)),
                Decoration,
                Bar(overlay),
            ));
        }
    });
    let error = commands
        .spawn((
            Text::new("!"),
            TextFont::from_font_size(FontSize::Px(14.)),
            TextColor(Color::srgb_u8(211, 177, 105)),
            Visibility::Hidden,
            Decoration,
        ))
        .id();
    commands.entity(overlay).add_child(error);
    (overlay, error)
}

fn fade_bars(
    parents: Query<&BackgroundColor, (With<Decoration>, Without<Bar>)>,
    mut bars: Query<(&Bar, &mut BackgroundColor)>,
) {
    for (bar, mut color) in &mut bars {
        if let Ok(parent) = parents.get(bar.0) {
            color.0 = Color::srgb_u8(58, 75, 82).with_alpha(parent.0.alpha());
        }
    }
}

fn opacity(color: Color, alpha: f32) -> Color {
    color.with_alpha(color.alpha() * alpha)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn paint_images(
    time: Res<Time>,
    server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    gpu: Res<Prepared>,
    mut status: ResMut<Status>,
    mut fields: Query<(
        &mut ImageNode,
        &mut ImageMask,
        Option<&super::super::kit::Pending>,
        &InheritedVisibility,
    )>,
    mut masks: Query<(&mut Visibility, &mut BackgroundColor), With<Decoration>>,
    mut errors: Query<&mut Visibility, (With<Decoration>, Without<BackgroundColor>)>,
) {
    status.images = 0;
    status.failed = 0;
    for (mut image, mut mask, pending, visible) in &mut fields {
        if image.color != mask.written {
            mask.color = image.color;
        }
        let id = image.image.id();
        let failed = matches!(server.load_state(id), LoadState::Failed(_));
        let waiting = super::forced()
            || pending.is_some_and(|p| p.0)
            || id == AssetId::<Image>::default()
            || images.get(id).is_none()
            || !gpu.image_ready(id);
        if failed || waiting {
            mask.fade = 0.;
        } else {
            mask.fade = (mask.fade + time.delta_secs() / 0.14).min(1.);
        }
        let color = opacity(mask.color, mask.fade);
        if image.color != color {
            image.color = color;
        }
        mask.written = color;
        if let Ok((mut visibility, mut background)) = masks.get_mut(mask.overlay) {
            *visibility = if mask.fade < 1. {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            let pulse = if failed {
                1.
            } else {
                1. + time.elapsed_secs().sin() * 0.035
            };
            background.0 = Color::srgba(0.06 * pulse, 0.105 * pulse, 0.14 * pulse, 1. - mask.fade);
        }
        if let Ok(mut error) = errors.get_mut(mask.error) {
            *error = if failed {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
        if visible.get() && mask.fade == 0. {
            if failed {
                status.failed += 1;
            } else {
                status.images += 1;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn paint_text(
    time: Res<Time>,
    fonts: Res<Assets<Font>>,
    gpu: Res<Prepared>,
    state: Res<State<crate::GameState>>,
    mut status: ResMut<Status>,
    mut fields: Query<(
        &Text,
        &TextFont,
        Option<&TextLayoutInfo>,
        &mut TextColor,
        &mut TextMask,
        &mut BackgroundGradient,
        Option<&super::super::kit::Pending>,
        &InheritedVisibility,
    )>,
) {
    status.texts = 0;
    for (text, font, layout, mut color, mut mask, mut gradient, pending, visible) in &mut fields {
        if color.0 != mask.written {
            mask.color = color.0;
        }
        let gpu_ready = layout.is_some_and(|l| {
            l.glyphs
                .iter()
                .all(|g| gpu.image_ready(g.atlas_info.texture))
        });
        // Never replace a ready battle clock/choice with a startup animation.
        let boot_wait = !status.boot_ready && *state.get() != crate::GameState::InMatch;
        let font_missing =
            matches!(&font.font, FontSource::Handle(handle) if fonts.get(handle.id()).is_none());
        let waiting = pending.is_some_and(|p| p.0)
            || (!text.0.trim().is_empty() && (boot_wait || font_missing || !gpu_ready));
        if waiting {
            mask.fade = 0.;
        } else {
            mask.fade = (mask.fade + time.delta_secs() / 0.10).min(1.);
        }
        let want = opacity(mask.color, mask.fade);
        if color.0 != want {
            color.0 = want;
        }
        mask.written = want;
        let want = if mask.fade < 1. {
            let bar = Color::srgba(0.24, 0.32, 0.36, 1. - mask.fade);
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::percent(Color::NONE, 38),
                ColorStop::percent(bar, 38),
                ColorStop::percent(bar, 62),
                ColorStop::percent(Color::NONE, 62),
            ]))
        } else {
            mask.gradient.clone()
        };
        if *gradient != want {
            *gradient = want;
        }
        if visible.get() && waiting {
            status.texts += 1;
        }
    }
}
