//! Fortification of your own Stronghold (BR-SK05): what each branch offers
//! right now, shared by the desktop and phone HUDs, and the compact phone
//! control. Everything shown comes from the latest authoritative view; a
//! sent `Fortify` only blocks a second click until the view covers it.

use bevy::prelude::*;
use tbg_core::content::FORTIFY_COST;
#[cfg(any(not(target_os = "linux"), test))]
use tbg_core::content::{ENERGY_SCALE, STRONGHOLD_KINDS};
use tbg_core::{MatchView, Phase, StrongholdView};

#[cfg(not(target_os = "linux"))]
use super::{label, UiAction, BUTTON, DISABLED};
use crate::sync::LatestView;
use crate::GameState;

/// Sequence number of the `Fortify` waiting for the view that answers it.
#[derive(Resource, Default)]
pub struct PendingFortify(pub Option<u32>);

/// Whether the Fortification panel is open: it opens on a click on your own
/// Stronghold and closes like a unit's panel (click elsewhere, Escape, new match).
#[derive(Resource, Default)]
pub struct TowerOpen(pub bool);

/// One branch button's state, mirroring the server's checks in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Offer {
    /// Buyable now.
    Open,
    /// Not enough Energy for the next tier.
    Short,
    /// The other branch was chosen at tier 1 (BR-SK05).
    Closed,
    /// Already tier 2.
    Max,
    /// An Augment selection pauses the battle (BR-AG12).
    Paused,
    /// A sent `Fortify` has no answer yet.
    Pending,
    /// The match is not running (countdown or ended).
    Idle,
}

impl Offer {
    /// The desktop projection's word for this state (hud.js styles by it).
    #[cfg(any(target_arch = "wasm32", test))]
    pub(crate) fn code(self) -> &'static str {
        match self {
            Offer::Open => "open",
            Offer::Short => "short",
            Offer::Closed => "closed",
            Offer::Max => "max",
            Offer::Paused => "paused",
            Offer::Pending => "pending",
            Offer::Idle => "idle",
        }
    }
}

pub(crate) fn own(v: &MatchView) -> StrongholdView {
    v.strongholds[v.you.index()]
}

pub(crate) fn offer(v: &MatchView, branch: u8, pending: bool) -> Offer {
    let tower = own(v);
    if tower.tier >= 2 {
        return Offer::Max;
    }
    if branch > 1 || (tower.tier == 1 && branch != tower.branch) {
        return Offer::Closed;
    }
    if v.phase != Phase::Running {
        return Offer::Idle;
    }
    if v.augment_round.is_some() {
        return Offer::Paused;
    }
    if pending {
        return Offer::Pending;
    }
    if v.energy < FORTIFY_COST[usize::from(tower.tier)] {
        return Offer::Short;
    }
    Offer::Open
}

/// `Kind|tier|branch`, the desktop HUD's projection of a seat's tower.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn record(tower: StrongholdView) -> String {
    format!("{:?}|{}|{}", tower.kind, tower.tier, tower.branch)
}

/// `Bastion T1 Fortress`: kind, tier and, once chosen, the branch.
#[cfg(any(not(target_os = "linux"), test))]
fn title(tower: StrongholdView) -> String {
    let def = &STRONGHOLD_KINDS[tower.kind.index()];
    if tower.tier == 0 {
        format!("{} T0", def.name)
    } else {
        let branch = def.branch_names[usize::from(tower.branch.min(1))];
        format!("{} T{} {branch}", def.name, tower.tier)
    }
}

/// Phone button text for `branch`: the next step's name and its price.
#[cfg(any(not(target_os = "linux"), test))]
fn choice_text(v: &MatchView, branch: u8) -> String {
    let tower = own(v);
    let tier = usize::from(tower.tier.min(1));
    let step = STRONGHOLD_KINDS[tower.kind.index()].branches[usize::from(branch.min(1))][tier];
    let letter = if branch == 0 { "A" } else { "B" };
    format!(
        "{letter} {} | {} Energy",
        step.name,
        FORTIFY_COST[tier] / ENERGY_SCALE
    )
}

#[derive(Component)]
#[cfg(not(target_os = "linux"))]
struct FortifyTitle;

#[derive(Component)]
#[cfg(not(target_os = "linux"))]
struct FortifyChoice(u8);

#[derive(Component)]
#[cfg(not(target_os = "linux"))]
struct FortifyChoiceText(u8);

pub fn plugin(app: &mut App) {
    app.init_resource::<PendingFortify>()
        .init_resource::<TowerOpen>()
        .add_systems(
            OnExit(GameState::InMatch),
            |mut p: ResMut<PendingFortify>, mut open: ResMut<TowerOpen>| {
                p.0 = None;
                open.0 = false;
            },
        )
        .add_systems(Update, clear_answered.run_if(in_state(GameState::InMatch)));
    #[cfg(not(target_os = "linux"))]
    app.add_systems(
        Update,
        refresh
            .after(clear_answered)
            .run_if(in_state(GameState::InMatch)),
    );
}

#[cfg(not(target_os = "linux"))]
fn choice(branch: u8) -> impl Bundle {
    (
        Node {
            width: px(178.0),
            min_height: px(34.0),
            padding: UiRect::axes(px(6.0), px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(8.0)),
            ..default()
        },
        BackgroundColor(DISABLED),
        UiAction::Disabled,
        FortifyChoice(branch),
        children![(label("", 11.0), FortifyChoiceText(branch))],
    )
}

/// The compact phone row: your tower and the opponent's, then the next
/// Fortification step of each open branch. It sits above the Energy bar,
/// so it never covers the Shop.
#[cfg(not(target_os = "linux"))]
pub(super) fn control() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(6.0),
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(6.0), px(3.0)),
            border_radius: BorderRadius::all(px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.16, 0.12, 0.06, 0.78)),
        children![(label("", 11.0), FortifyTitle), choice(0), choice(1)],
    )
}

fn clear_answered(view: Res<LatestView>, mut pending: ResMut<PendingFortify>) {
    if let (Some(seq), Some(v)) = (pending.0, &view.0) {
        if v.last_seq >= seq {
            pending.0 = None;
        }
    }
}

#[cfg(not(target_os = "linux"))]
type ChoiceQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static FortifyChoice,
        &'static mut UiAction,
        &'static mut BackgroundColor,
        &'static mut Node,
    ),
>;

#[cfg(not(target_os = "linux"))]
fn refresh(
    view: Res<LatestView>,
    pending: Res<PendingFortify>,
    mut titles: Query<&mut Text, (With<FortifyTitle>, Without<FortifyChoiceText>)>,
    mut choices: ChoiceQuery,
    mut texts: Query<(&FortifyChoiceText, &mut Text), Without<FortifyTitle>>,
) {
    let Some(v) = &view.0 else { return };
    let enemy = v.strongholds[v.you.other().index()];
    let mine = if own(v).tier >= 2 {
        format!("{} | Max", title(own(v)))
    } else {
        title(own(v))
    };
    let summary = format!("Fortify: {mine}\nFoe: {}", title(enemy));
    for mut text in &mut titles {
        if text.0 != summary {
            text.0 = summary.clone();
        }
    }
    for (choice, mut action, mut color, mut node) in &mut choices {
        let state = offer(v, choice.0, pending.0.is_some());
        let shown = !matches!(state, Offer::Max | Offer::Closed);
        node.display = if shown { Display::Flex } else { Display::None };
        *action = if state == Offer::Open {
            UiAction::Fortify(choice.0)
        } else {
            UiAction::Disabled
        };
        color.0 = if state == Offer::Open {
            BUTTON
        } else {
            DISABLED
        };
    }
    for (choice, mut text) in &mut texts {
        let next = choice_text(v, choice.0);
        if text.0 != next {
            text.0 = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::StrongholdKind;

    fn view(tier: u8, branch: u8, energy: u32) -> MatchView {
        let mut v = tbg_core::MatchState::new(3).view_for(tbg_core::Seat::A, &[]);
        v.phase = Phase::Running;
        v.augment_round = None;
        v.energy = energy;
        let me = v.you.index();
        v.strongholds[me].kind = StrongholdKind::Artillery;
        v.strongholds[me].tier = tier;
        v.strongholds[me].branch = branch;
        v
    }

    #[test]
    fn offers_follow_the_server_checks() {
        // BR-SK05: tier 0 offers both branches at FORTIFY_COST[0]; tier 1
        // only the chosen branch at FORTIFY_COST[1]; tier 2 nothing.
        let [first, second] = FORTIFY_COST;
        assert_eq!(offer(&view(0, 0, first), 0, false), Offer::Open);
        assert_eq!(offer(&view(0, 0, first), 1, false), Offer::Open);
        assert_eq!(offer(&view(0, 0, first - 1), 1, false), Offer::Short);
        assert_eq!(offer(&view(1, 1, second), 1, false), Offer::Open);
        assert_eq!(offer(&view(1, 1, second - 1), 1, false), Offer::Short);
        assert_eq!(offer(&view(1, 1, second), 0, false), Offer::Closed);
        assert_eq!(offer(&view(2, 1, 99_999), 1, false), Offer::Max);
        assert_eq!(offer(&view(0, 0, first), 0, true), Offer::Pending);
        let mut countdown = view(0, 0, first);
        countdown.phase = Phase::Countdown;
        assert_eq!(offer(&countdown, 0, false), Offer::Idle);
    }

    #[test]
    fn phone_text_names_the_next_step_and_its_price() {
        let v = view(1, 0, 0);
        assert_eq!(choice_text(&v, 0), "A Great Cannon | 4 Energy");
        assert_eq!(title(own(&v)), "Artillery T1 Heavy Cannon");
        assert_eq!(record(own(&v)), "Artillery|1|0");
        assert_eq!(offer(&v, 1, false).code(), "closed");
    }
}
