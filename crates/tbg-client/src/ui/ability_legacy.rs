//! Legacy ability facts read directly from the selected canonical form.
use tbg_core::content::Ability;

pub(super) fn text(a: Ability) -> Option<String> {
    use Ability::*;
    Some(match a {
        None => return Option::None,
        Hatch => "Hatch: creates Beetling summons while engaged; summons never count toward Synergy".into(),
        Guard => "Guard: enemies prioritize this unit in range".into(),
        Poison { dps, pulses, cap } => format!("Poison: {dps} damage per second, {pulses} pulses, up to {cap} stacks"),
        Fish { bp, ticks, cap, per_hit } => format!("Fish: Slow {}% for {:.1}s; {per_hit} fish per hit, cap {cap}", bp/100, f32::from(ticks)/20.),
        Echo { every, extra } => format!("Echo: {extra} extra projectiles every {every} attacks"),
        Blast { friendly } => format!("Blast on death; friendly fire: {friendly}"),
        NoStronghold => "Cannot target Strongholds".into(),
        Essence { base, step, dmg, atk_bp } => format!("Essence: {base} initial; +{step} per stack; {dmg} damage and {}% attack speed", atk_bp/100),
        Formation { engage, gap } => format!("Formation: engages within {:.1} tiles; {:.1}-tile spacing", engage as f32/1000.,gap as f32/1000.),
        March { reach, move_bp, dr_bp, ignore_slow, .. } => format!("March: {:.1}-tile bond, movement +{}%, damage reduction {}%, ignores Slow: {ignore_slow}", reach as f32/1000., move_bp/100,dr_bp/100),
        Brew { strip_poison, ward, second } => format!("Brew: removes {strip_poison} Poison stacks; ward {:.1}s; second ally: {second}",f32::from(ward)/20.),
        Puppet { window, immune, bonus_bp, targets, stumble } => format!("Puppet: {:.1}s control, {:.1}s immunity; damage +{}%; {targets} targets; stumble: {stumble}",f32::from(window)/20.,f32::from(immune)/20.,bonus_bp/100),
        Cargo { every, open, table, double_every } => format!("Cargo: crate every {:.1}s; opens after {:.1}s; odds {table:?}; double every {double_every}",f32::from(every)/20.,f32::from(open)/20.),
        Plates { count, hp, move_bp, atk_bp, dmg_bp, shrapnel, rampage } => format!("Armor: {count} plates ×{hp} HP; movement +{}%, attack speed +{}%, damage +{}%; shrapnel {shrapnel}; rampage {rampage}", move_bp/100,atk_bp/100,dmg_bp/100),
        Carry { board, lock, dr_bp, move_bp, shield_bp, surge_bp } => format!("Carry: boards within {:.1} tiles; lock {:.1}s; damage reduction {}%, movement +{}%, shield {}%, surge {}%",board as f32/1000.,f32::from(lock)/20.,dr_bp/100,move_bp/100,shield_bp/100,surge_bp/100),
        Furnace { burn_bp, max_bp } => format!("Furnace: self-burn {}% per hit, bonus up to {}%", burn_bp/100,max_bp/100),
        Siphon { heal_bp, low_bp, split } => format!("Siphon: restores {}% of actual HP damage to the lowest-HP conventional ally; doubled below {}% HP; split: {split}",heal_bp/100,low_bp/100),
        _ => return Option::None,
    })
}
