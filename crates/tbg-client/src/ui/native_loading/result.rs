//! Reserve the ranked result's rating slot while the authoritative update is in flight.

use super::super::{account::Account, kit::Pending, result::RatingLabel};
use crate::{i18n::Lang, sync::MatchInfo, GameState};
use bevy::prelude::*;

#[derive(Component)]
struct Since(f64);

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        pending
            .before(bevy::ui::UiSystems::Prepare)
            .run_if(in_state(GameState::Result)),
    );
}

#[allow(clippy::type_complexity)]
fn pending(
    mut commands: Commands,
    time: Res<Time>,
    info: Res<MatchInfo>,
    account: Res<Account>,
    lang: Res<Lang>,
    mut labels: Query<
        (
            Entity,
            &mut Node,
            &mut Text,
            Option<&Since>,
            Option<&mut Pending>,
        ),
        With<RatingLabel>,
    >,
) {
    if !info.ranked {
        return;
    }
    for (entity, mut node, mut text, since, pending) in &mut labels {
        let Some(since) = since else {
            node.width = px(280);
            node.min_height = px(24);
            commands.entity(entity).insert((
                Since(time.elapsed_secs_f64()),
                Pending(account.rating_change.is_none()),
            ));
            continue;
        };
        let Some(mut pending) = pending else {
            continue;
        };
        let expired = time.elapsed_secs_f64() - since.0 >= crate::net::GIVE_UP;
        pending.0 = account.rating_change.is_none() && !expired;
        if account.rating_change.is_none() && expired {
            let message = if *lang == Lang::PtBr {
                "Pontuação indisponível agora."
            } else {
                "Rating unavailable right now."
            };
            if text.0 != message {
                text.0 = message.into();
            }
        }
    }
}
