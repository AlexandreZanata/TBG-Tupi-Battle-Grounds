//! Localized completion facts use catalog templates and authoritative form parameters.
use crate::i18n::{Lang, Msg};
use tbg_core::content::{completion::Ability, TICK_HZ};
pub(super) fn text(a: Ability, lang: Lang) -> String {
    use Ability::*;
    let p = |bp: u16| format!("{}%", lang.compact(f64::from(bp) / 100.0, 0));
    let s = |ticks: u16| {
        format!(
            "{}s",
            lang.compact(f64::from(ticks) / f64::from(TICK_HZ), 1)
        )
    };
    let r = |value: i32| lang.compact(f64::from(value) / 1000.0, 1);
    let (msg, args): (Msg, Vec<(&str, String)>) = match a {
        Barrel {
            speed,
            radius,
            damage,
        } => (
            Msg::CompletionAbilityBarrel,
            vec![
                ("speed", r(speed * TICK_HZ as i32)),
                ("radius", r(radius)),
                ("damage", damage.to_string()),
            ],
        ),
        Finisher { below_bp, bonus_bp } => (
            Msg::CompletionAbilityFinisher,
            vec![("below", p(below_bp)), ("bonus", p(bonus_bp))],
        ),
        Echo { heal, ticks } => (
            Msg::CompletionAbilityEcho,
            vec![("heal", heal.to_string()), ("secs", s(ticks))],
        ),
        Sweep {
            every,
            damage,
            radius,
            targets,
        } => (
            Msg::CompletionAbilitySweep,
            vec![
                ("every", every.to_string()),
                ("damage", damage.to_string()),
                ("radius", r(radius)),
                ("targets", targets.to_string()),
            ],
        ),
        Pounce {
            walk_ticks,
            bonus_bp,
        } => (
            Msg::CompletionAbilityPounce,
            vec![("secs", s(walk_ticks)), ("bonus", p(bonus_bp))],
        ),
        Pierce {
            every,
            ratio_bp,
            behind,
        } => (
            Msg::CompletionAbilityPierce,
            vec![
                ("every", every.to_string()),
                ("ratio", p(ratio_bp)),
                ("behind", r(behind)),
            ],
        ),
        Regrowth { heal, pulses, gap } => (
            Msg::CompletionAbilityRegrowth,
            vec![
                ("heal", heal.to_string()),
                ("pulses", pulses.to_string()),
                ("secs", s(gap)),
            ],
        ),
        Interrupt {
            period, lock_ticks, ..
        } => (
            Msg::CompletionAbilityInterrupt,
            vec![("period", s(period)), ("lock", s(lock_ticks))],
        ),
        Spark {
            every,
            damage,
            ticks,
        } => (
            Msg::CompletionAbilitySpark,
            vec![
                ("every", every.to_string()),
                ("damage", damage.to_string()),
                ("secs", s(ticks)),
            ],
        ),
        Eclipse {
            high_bp,
            solar_bp,
            lunar_bp,
        } => (
            Msg::CompletionAbilityEclipse,
            vec![
                ("high", p(high_bp)),
                ("solar", p(solar_bp)),
                ("lunar", p(lunar_bp)),
            ],
        ),
        Crusher { max_hp_bp, cap } => (
            Msg::CompletionAbilityCrusher,
            vec![("hp", p(max_hp_bp)), ("cap", cap.to_string())],
        ),
        Courage {
            heal_per_enemy,
            enemies,
            radius,
        } => (
            Msg::CompletionAbilityCourage,
            vec![
                ("heal", heal_per_enemy.to_string()),
                ("enemies", enemies.to_string()),
                ("radius", r(radius)),
            ],
        ),
        Beam {
            period,
            damage,
            radius,
            targets,
        } => (
            Msg::CompletionAbilityBeam,
            vec![
                ("period", s(period)),
                ("damage", damage.to_string()),
                ("radius", r(radius)),
                ("targets", targets.to_string()),
            ],
        ),
        Bleeder { damage, pulses } => (
            Msg::CompletionAbilityBleeder,
            vec![
                ("damage", damage.to_string()),
                ("pulses", pulses.to_string()),
            ],
        ),
        DeathPop {
            damage,
            radius,
            targets,
        } => (
            Msg::CompletionAbilityDeathpop,
            vec![
                ("damage", damage.to_string()),
                ("radius", r(radius)),
                ("targets", targets.to_string()),
            ],
        ),
        Opportunity { every, bonus_bp } => (
            Msg::CompletionAbilityOpportunity,
            vec![("every", every.to_string()), ("bonus", p(bonus_bp))],
        ),
        Channel {
            pulses,
            gap,
            first_bp,
        } => (
            Msg::CompletionAbilityChannel,
            vec![
                ("pulses", pulses.to_string()),
                ("gap", s(gap)),
                ("first", p(first_bp)),
            ],
        ),
        Rhythm {
            period,
            bp,
            ticks,
            radius,
            targets,
        } => (
            Msg::CompletionAbilityRhythm,
            vec![
                ("period", s(period)),
                ("bp", p(bp)),
                ("ticks", s(ticks)),
                ("radius", r(radius)),
                ("targets", targets.to_string()),
            ],
        ),
    };
    let named: Vec<(&str, &dyn std::fmt::Display)> = args
        .iter()
        .map(|(n, v)| (*n, v as &dyn std::fmt::Display))
        .collect();
    lang.tf(msg, &named)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_all_ability_templates_fill_from_each_real_form_in_both_languages() {
        for kind in tbg_core::COMPLETION_POOL.into_iter().skip(46) {
            for (t, b) in crate::art::FORMS {
                for a in tbg_core::content::def(kind).ability(t, b) {
                    if let tbg_core::content::Ability::Completion(a) = a {
                        for lang in [Lang::En, Lang::PtBr] {
                            let shown = text(a, lang);
                            assert!(
                                !shown.contains('{') && !shown.contains('}') && !shown.is_empty(),
                                "{kind:?} {t}/{b}: {shown}"
                            );
                        }
                    }
                }
            }
        }
    }
}
