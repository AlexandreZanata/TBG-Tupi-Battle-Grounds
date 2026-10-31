//! Core content ids mapped to catalog messages.
//!
//! Every `match` here is exhaustive on purpose: a new `tbg-core` variant is a
//! compile error naming the catalog key to add (P64 L06). Ids, numbers and
//! core names stay canonical; only their presentation is resolved here.

use tbg_core::content::AugmentRarity;
use tbg_core::{Archetype, Faction, StrongholdKind, UnitKind};

use super::*;

/// Localized rejection reason (the server sends the `Rejection` code only).
pub fn rejection(lang: Lang, reason: tbg_core::Rejection) -> &'static str {
    lang.t(match reason {
        tbg_core::Rejection::NotRunning => Msg::RejectionNotRunning,
        tbg_core::Rejection::BadOffer => Msg::RejectionBadOffer,
        tbg_core::Rejection::NotEnoughEnergy => Msg::RejectionNotEnoughEnergy,
        tbg_core::Rejection::NotYourUnit => Msg::RejectionNotYourUnit,
        tbg_core::Rejection::UnitGone => Msg::RejectionUnitGone,
        tbg_core::Rejection::MaxTier => Msg::RejectionMaxTier,
        tbg_core::Rejection::WrongBranch => Msg::RejectionWrongBranch,
        tbg_core::Rejection::NotEvolvable => Msg::RejectionNotEvolvable,
        tbg_core::Rejection::Stale => Msg::RejectionStale,
        tbg_core::Rejection::RoundClosed => Msg::RejectionRoundClosed,
        tbg_core::Rejection::BadOption => Msg::RejectionBadOption,
        tbg_core::Rejection::Unavailable => Msg::RejectionUnavailable,
        tbg_core::Rejection::Paused => Msg::RejectionPaused,
    })
}

/// Every message a unit kind presents: name, forms `[A1, A2, B1, B2]` and
/// the Shop matchup lines.
struct UnitMsgs {
    name: Msg,
    forms: [Msg; 4],
    good: Msg,
    weak: Msg,
}

const fn u(name: Msg, forms: [Msg; 4], good: Msg, weak: Msg) -> UnitMsgs {
    UnitMsgs {
        name,
        forms,
        good,
        weak,
    }
}

#[rustfmt::skip]
fn unit_msgs(kind: UnitKind) -> UnitMsgs {
    match kind {
        UnitKind::Shieldbearer => u(Msg::UnitShieldbearerName, UNIT_SHIELDBEARER_FORM_NAME, Msg::UnitShieldbearerGood, Msg::UnitShieldbearerWeak),
        UnitKind::Archer => u(Msg::UnitArcherName, UNIT_ARCHER_FORM_NAME, Msg::UnitArcherGood, Msg::UnitArcherWeak),
        UnitKind::BoneBrute => u(Msg::UnitBoneBruteName, UNIT_BONE_BRUTE_FORM_NAME, Msg::UnitBoneBruteGood, Msg::UnitBoneBruteWeak),
        UnitKind::Hexer => u(Msg::UnitHexerName, UNIT_HEXER_FORM_NAME, Msg::UnitHexerGood, Msg::UnitHexerWeak),
        UnitKind::Sawbot => u(Msg::UnitSawbotName, UNIT_SAWBOT_FORM_NAME, Msg::UnitSawbotGood, Msg::UnitSawbotWeak),
        UnitKind::SiegeWalker => u(Msg::UnitSiegeWalkerName, UNIT_SIEGE_WALKER_FORM_NAME, Msg::UnitSiegeWalkerGood, Msg::UnitSiegeWalkerWeak),
        UnitKind::Bloomkeeper => u(Msg::UnitBloomkeeperName, UNIT_BLOOMKEEPER_FORM_NAME, Msg::UnitBloomkeeperGood, Msg::UnitBloomkeeperWeak),
        UnitKind::Rootcaller => u(Msg::UnitRootcallerName, UNIT_ROOTCALLER_FORM_NAME, Msg::UnitRootcallerGood, Msg::UnitRootcallerWeak),
        UnitKind::Nester => u(Msg::UnitNesterName, UNIT_NESTER_FORM_NAME, Msg::UnitNesterGood, Msg::UnitNesterWeak),
        UnitKind::Rampart => u(Msg::UnitRampartName, UNIT_RAMPART_FORM_NAME, Msg::UnitRampartGood, Msg::UnitRampartWeak),
        UnitKind::Beetling => u(Msg::UnitBeetlingName, UNIT_BEETLING_FORM_NAME, Msg::UnitBeetlingGood, Msg::UnitBeetlingWeak),
        UnitKind::Longaxe => u(Msg::UnitLongaxeName, UNIT_LONGAXE_FORM_NAME, Msg::UnitLongaxeGood, Msg::UnitLongaxeWeak),
        UnitKind::Farwatch => u(Msg::UnitFarwatchName, UNIT_FARWATCH_FORM_NAME, Msg::UnitFarwatchGood, Msg::UnitFarwatchWeak),
        UnitKind::Bombard => u(Msg::UnitBombardName, UNIT_BOMBARD_FORM_NAME, Msg::UnitBombardGood, Msg::UnitBombardWeak),
        UnitKind::Rotcap => u(Msg::UnitRotcapName, UNIT_ROTCAP_FORM_NAME, Msg::UnitRotcapGood, Msg::UnitRotcapWeak),
        UnitKind::OldFinn => u(Msg::UnitOldFinnName, UNIT_OLD_FINN_FORM_NAME, Msg::UnitOldFinnGood, Msg::UnitOldFinnWeak),
        UnitKind::AntidoteBrewer => u(Msg::UnitAntidoteBrewerName, UNIT_ANTIDOTE_BREWER_FORM_NAME, Msg::UnitAntidoteBrewerGood, Msg::UnitAntidoteBrewerWeak),
        UnitKind::Shatterback => u(Msg::UnitShatterbackName, UNIT_SHATTERBACK_FORM_NAME, Msg::UnitShatterbackGood, Msg::UnitShatterbackWeak),
        UnitKind::BloodFurnace => u(Msg::UnitBloodFurnaceName, UNIT_BLOOD_FURNACE_FORM_NAME, Msg::UnitBloodFurnaceGood, Msg::UnitBloodFurnaceWeak),
        UnitKind::Bloodweaver => u(Msg::UnitBloodweaverName, UNIT_BLOODWEAVER_FORM_NAME, Msg::UnitBloodweaverGood, Msg::UnitBloodweaverWeak),
        UnitKind::Oilpress => u(Msg::UnitOilpressName, UNIT_OILPRESS_FORM_NAME, Msg::UnitOilpressGood, Msg::UnitOilpressWeak),
        UnitKind::Drumguard => u(Msg::UnitDrumguardName, UNIT_DRUMGUARD_FORM_NAME, Msg::UnitDrumguardGood, Msg::UnitDrumguardWeak),
        UnitKind::Leechroot => u(Msg::UnitLeechrootName, UNIT_LEECHROOT_FORM_NAME, Msg::UnitLeechrootGood, Msg::UnitLeechrootWeak),
        UnitKind::CargoCrab => u(Msg::UnitCargoCrabName, UNIT_CARGO_CRAB_FORM_NAME, Msg::UnitCargoCrabGood, Msg::UnitCargoCrabWeak),
        UnitKind::Puppetmaster => u(Msg::UnitPuppetmasterName, UNIT_PUPPETMASTER_FORM_NAME, Msg::UnitPuppetmasterGood, Msg::UnitPuppetmasterWeak),
        UnitKind::Smuggler => u(Msg::UnitSmugglerName, UNIT_SMUGGLER_FORM_NAME, Msg::UnitSmugglerGood, Msg::UnitSmugglerWeak),
        UnitKind::Crate => u(Msg::UnitCrateName, UNIT_CRATE_FORM_NAME, Msg::UnitCrateGood, Msg::UnitCrateWeak),
    }
}

/// Display name of a unit kind (`UnitDef::name` in English).
pub fn unit_name(lang: Lang, kind: UnitKind) -> &'static str {
    lang.t(unit_msgs(kind).name)
}

/// Display name of the form at `tier`/`branch`; `None` for the base form,
/// matching `tbg_core::content::form_of`.
pub fn form_name(lang: Lang, kind: UnitKind, tier: u8, branch: u8) -> Option<&'static str> {
    if tier == 0 || tier > 2 || branch > 1 {
        return None;
    }
    let slot = usize::from(branch) * 2 + usize::from(tier) - 1;
    Some(lang.t(unit_msgs(kind).forms[slot]))
}

/// Shop matchup lines: (good against, weak against).
pub fn matchup(lang: Lang, kind: UnitKind) -> (&'static str, &'static str) {
    let msgs = unit_msgs(kind);
    (lang.t(msgs.good), lang.t(msgs.weak))
}

pub fn faction(lang: Lang, faction: Faction) -> &'static str {
    lang.t(match faction {
        Faction::Humans => Msg::FactionHumans,
        Faction::Undead => Msg::FactionUndead,
        Faction::Machines => Msg::FactionMachines,
        Faction::Plants => Msg::FactionPlants,
    })
}

pub fn archetype(lang: Lang, archetype: Archetype) -> &'static str {
    lang.t(match archetype {
        Archetype::Tank => Msg::ArchetypeTank,
        Archetype::Dps => Msg::ArchetypeDps,
        Archetype::Support => Msg::ArchetypeSupport,
        Archetype::Ranged => Msg::ArchetypeRanged,
        Archetype::Control => Msg::ArchetypeControl,
    })
}

pub fn rarity(lang: Lang, rarity: AugmentRarity) -> &'static str {
    lang.t(match rarity {
        AugmentRarity::Bronze => Msg::RarityBronze,
        AugmentRarity::Silver => Msg::RaritySilver,
        AugmentRarity::Gold => Msg::RarityGold,
        AugmentRarity::Diamond => Msg::RarityDiamond,
    })
}

/// Canonical id of a `UnitView::status` code (`1` slowed, `2` rooted); the
/// battle layer styles by this id, so it is never translated.
pub fn status_id(code: u8) -> &'static str {
    match code {
        1 => "Slowed",
        2 => "Rooted",
        _ => "Ready",
    }
}

pub fn status(lang: Lang, code: u8) -> &'static str {
    lang.t(match code {
        1 => Msg::StatusSlowed,
        2 => Msg::StatusRooted,
        _ => Msg::StatusReady,
    })
}

/// Faction and Synergy level, `Humans 1/2`.
pub fn synergy(lang: Lang, f: Faction, level: u8) -> String {
    lang.tf(
        Msg::SynergyLevel,
        &[("faction", &faction(lang, f)), ("level", &level)],
    )
}

pub fn augment_name(lang: Lang, id: u8) -> Option<&'static str> {
    AUGMENT_NAME.get(usize::from(id)).map(|m| lang.t(*m))
}

pub fn stronghold_name(lang: Lang, kind: StrongholdKind) -> &'static str {
    lang.t(match kind {
        StrongholdKind::Classic => Msg::StrongholdClassicName,
        StrongholdKind::Treasury => Msg::StrongholdTreasuryName,
        StrongholdKind::Bastion => Msg::StrongholdBastionName,
        StrongholdKind::Artillery => Msg::StrongholdArtilleryName,
        StrongholdKind::Command => Msg::StrongholdCommandName,
    })
}

/// Branch and step messages; the Classic reference tower has none, like
/// its empty `KindDef::branch_names`.
fn stronghold_msgs(kind: StrongholdKind) -> Option<([Msg; 2], [Msg; 4])> {
    match kind {
        StrongholdKind::Classic => None,
        StrongholdKind::Treasury => Some((
            STRONGHOLD_TREASURY_BRANCH_NAME,
            STRONGHOLD_TREASURY_STEP_NAME,
        )),
        StrongholdKind::Bastion => {
            Some((STRONGHOLD_BASTION_BRANCH_NAME, STRONGHOLD_BASTION_STEP_NAME))
        }
        StrongholdKind::Artillery => Some((
            STRONGHOLD_ARTILLERY_BRANCH_NAME,
            STRONGHOLD_ARTILLERY_STEP_NAME,
        )),
        StrongholdKind::Command => {
            Some((STRONGHOLD_COMMAND_BRANCH_NAME, STRONGHOLD_COMMAND_STEP_NAME))
        }
    }
}

/// Branch name (`KindDef::branch_names`); empty for Classic.
pub fn stronghold_branch(lang: Lang, kind: StrongholdKind, branch: u8) -> &'static str {
    stronghold_msgs(kind).map_or("", |(branches, _)| {
        lang.t(branches[usize::from(branch.min(1))])
    })
}

/// Fortification step name at `tier` (1 or 2) of `branch`; empty for Classic.
pub fn stronghold_step(lang: Lang, kind: StrongholdKind, branch: u8, tier: u8) -> &'static str {
    let slot = usize::from(branch.min(1)) * 2 + usize::from(tier.clamp(1, 2)) - 1;
    stronghold_msgs(kind).map_or("", |(_, steps)| lang.t(steps[slot]))
}
