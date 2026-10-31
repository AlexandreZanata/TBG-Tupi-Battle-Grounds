//! P59 ability text from the same form parameters the server uses; never predicts outcomes.
use tbg_core::{
    content::{self, Ability},
    UnitKind, UnitView,
};
fn secs(t: u16) -> String {
    format!("{:.1}s", f32::from(t) / 20.0)
}
fn tiles(r: i32) -> String {
    format!("{:.1}", r as f32 / 1000.0)
}
fn pct(bp: u16) -> String {
    format!("{:.0}%", f32::from(bp) / 100.0)
}
/// Full next-form facts also cover branches whose Evolution changes no basic stat.
pub(super) fn describe(kind: UnitKind, tier: u8, branch: u8) -> String {
    let mut out: Vec<String> = content::def(kind)
        .ability(tier, branch)
        .into_iter()
        .filter_map(ability)
        .collect();
    match kind {
        UnitKind::Drillnest => {
            out.push("8 living drills per player; equal ranks fuse within 0.2 tiles; HP and harmful effects survive fusion".into());
            if tier == 2 && branch == 0 {
                out.push("Merged drill shield 25 for 2.0s".into());
            }
            if tier > 0 && branch == 1 {
                out.push("Duodrill damage +15%".into());
                if tier == 2 {
                    out.push(
                        "Megadrill +25% damage vs printed base HP ≥1000 units; no tower bonus"
                            .into(),
                    );
                }
            }
        }
        UnitKind::Microdrill => {
            out.push("Rank 1; expires after 12.0s; fuses with another Microdrill".into())
        }
        UnitKind::Duodrill => {
            out.push("Rank 2; expires after 14.0s; fuses with another Duodrill".into())
        }
        UnitKind::Megadrill => out.push(
            if tier == 2 && branch == 1 {
                "Rank 3; expires after 16.0s; +25% damage vs printed base HP ≥1000 units"
            } else {
                "Rank 3; expires after 16.0s; final fusion rank"
            }
            .into(),
        ),
        _ => {}
    }
    out.join("; ")
}
fn ability(a: Ability) -> Option<String> {
    use Ability::*;
    Some(match a{
 LivingWall{period,bp,ticks}=>format!("Living Wall every {}: self reduction {} for {}; anchored while active",secs(period),pct(bp),secs(ticks)),
 AreaCrush{period,radius,damage}=>format!("Area Crush every {}: {damage} base damage in {} tiles around self",secs(period),tiles(radius)),
 CrowMark{period,ticks,slow_bp,cut_bp,bonus_bp}=>format!("Crow Mark ready every {}: next unit hit marks for {}, Slow {}, healing -{}, own hits +{}",secs(period),secs(ticks),pct(slow_bp),pct(cut_bp),pct(bonus_bp)),
 Crossfire{every,ratio,radius}=>format!("Crossfire every {every} completed unit hits: strikes one other enemy within {} tiles for {} of static basic damage",tiles(radius),pct(ratio)),
 DoubleMayhem{period,radius,damage,bp,ticks}=>format!("Double Mayhem every {}: {damage} base damage within {} tiles of primary; Slow {} for {}",secs(period),tiles(radius),pct(bp),secs(ticks)),
 SwampBelly{period,radius,damage,bp,ticks,protection,protection_ticks}=>{let mut s=format!("Swamp Belly every {}: {damage} base damage within {} tiles of self; Slow {} for {}",secs(period),tiles(radius),pct(bp),secs(ticks));if protection>0{s+=&format!("; self reduction {} for {}",pct(protection),secs(protection_ticks));}s},
 BloodyRend{period,radius,damage,bleed,pulses}=>format!("Bloody Rend every {}: {damage} base damage within {} tiles; bleed {bleed} ×{pulses}, one pulse each second",secs(period),tiles(radius)),
 PrimordialCarapace{step_bp,cap_bp,gap,short_range,short_bonus_bp}=>{let mut s=format!("Primordial Carapace: +{} reduction per HP-damaging hit, cap {}; expires after {} without HP damage",pct(step_bp),pct(cap_bp),secs(gap));if short_bonus_bp>0{s+=&format!("; +{} basic damage within {} tiles",pct(short_bonus_bp),tiles(short_range));}s},
 PredatoryFrenzy{step_bp,increments,gap}=>format!("Predatory Frenzy: +{} per prior hit on the same enemy, up to {increments} increments; resets after {}",pct(step_bp),secs(gap)),
 CorrosiveSpit{every,damage,radius,targets}=>format!("Corrosive Spit: every {every}rd unit hit adds {damage} base damage to up to {targets} other enemies within {} tiles",tiles(radius)),
 FearPulse{period,bp,ticks}=>format!("Fear Pulse every {}: Slow {} for {} on one enemy",secs(period),pct(bp),secs(ticks)),
 SolarBarrier{period,range,amount,ticks,second}=>{let mut s=format!("Solar Barrier every {}: shield {amount} for {} on lowest HP-fraction other ally within {} tiles",secs(period),secs(ticks),tiles(range));if second>0{s+=&format!("; second ally shield {second}");}s},
 HarmonicPulse{period,bp,ticks}=>format!("Harmonic Pulse every {}: enemy basic damage -{} for {}; no heal",secs(period),pct(bp),secs(ticks)),
 PurifyingArrow{every,damage}=>format!("Purifying Arrow every {every} completed unit hits: deals {damage} base damage instead and strips its temporary shield before damage"),
 CelestialRain{period,radius,damage,bp,ticks}=>{let mut s=format!("Celestial Rain every {}: one {damage} base damage pulse within {} tiles of primary",secs(period),tiles(radius));if bp>0{s+=&format!("; Slow {} for {}",pct(bp),secs(ticks));}s},
 RaisedCarapace{period,bp,ticks,retaliation}=>{let mut s=format!("Raised Carapace every {}: self reduction {} for {}",secs(period),pct(bp),secs(ticks));if retaliation>0{s+=&format!("; next unit hit after expiry adds {retaliation} base damage once");}s},
 PreciseStrike{every,damage}=>format!("Precise Strike: every {every}th consecutive hit on the same enemy adds {damage} base damage; no idle timeout"),
 VenomousSting{every,damage,pulses}=>format!("Venomous Sting: every {every}rd unit hit applies venom {damage} ×{pulses}, one pulse each second; refreshes one slot"),
 StickyWeb{period,damage,root,range_add,slow_bp,slow_ticks}=>{let mut s=format!("Sticky Web every {}: {damage} base damage and Root {}",secs(period),secs(root));if range_add>0{s+=&format!("; +{} special range",tiles(range_add));}if slow_bp>0{s+=&format!("; Slow {} for {} after Root ends; cleanse discards it",pct(slow_bp),secs(slow_ticks));}s},
 ProfaneFire{cut_bp,cut_ticks}=>format!("Profane Fire: every basic burns ALL units ahead, including allies; healing -{} for {}; self excluded",pct(cut_bp),secs(cut_ticks)),
 DrillNest{period}=>format!("Drill Nest: Microdrill every {} with an enemy target; no direct damage",secs(period)),
 _=>return Option::None,
 })
}
pub(super) fn live(u: &UnitView) -> String {
    let mut text = describe(u.kind, u.tier, u.branch);
    let n = u.ability_counter();
    let suffix = match u.kind {
        UnitKind::Trigger | UnitKind::CelestialArcher | UnitKind::Mantis => {
            Some(format!("Sequence {n}/4"))
        }
        UnitKind::Acidmaw | UnitKind::Hornet => Some(format!("Sequence {n}/3")),
        UnitKind::Ravager => Some(format!("Frenzy increments {n}")),
        UnitKind::Jawguard if n > 0 => Some("Retaliation charged".into()),
        _ => None,
    };
    if let Some(s) = suffix {
        if !text.is_empty() {
            text.push_str("; ");
        }
        text.push_str(&s);
    }
    text
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ability_only_evolutions_report_their_actual_new_contract() {
        assert!(describe(UnitKind::Weaver, 2, 1).contains("after Root ends"));
        assert!(describe(UnitKind::Hornet, 2, 0).contains("venom 7 ×4"));
        assert!(describe(UnitKind::Jawguard, 1, 1).contains("adds 40"));
        assert!(describe(UnitKind::PyreWarden, 2, 1).contains("including allies"));
        assert!(describe(UnitKind::Drillnest, 2, 0).contains("shield 25 for 2.0s"));
        assert!(describe(UnitKind::Drillnest, 2, 1).contains("no tower bonus"));
    }
}
