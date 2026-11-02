//! GD-04 localized Volthar and Astreon facts from the same form parameters the server uses.
use crate::i18n::{Lang, Msg};
use tbg_core::content::{Ability, TICK_HZ};

pub(super) fn text(a: Ability, lang: Lang) -> Option<String> {
    let p = |bp: u16| format!("{}%", lang.compact(f64::from(bp) / 100.0, 0));
    let tiles = |value: i32| lang.compact(f64::from(value) / 1000.0, 1);
    let fill = |msg: Msg, args: &[(&str, String)]| {
        let named: Vec<(&str, &dyn std::fmt::Display)> = args
            .iter()
            .map(|(n, v)| (*n, v as &dyn std::fmt::Display))
            .collect();
        lang.tf(msg, &named)
    };
    match a {
        Ability::StormJudgment {
            every,
            jumps,
            radius,
            ratios,
            primary_bp,
            root,
            extra_ratio,
            extra_targets,
        } => {
            let shares: Vec<String> = ratios[..usize::from(jumps.min(4))]
                .iter()
                .map(|&r| p(r))
                .collect();
            let mut s = fill(
                Msg::GodsAbilityStorm,
                &[
                    ("every", every.to_string()),
                    ("jumps", jumps.to_string()),
                    ("radius", tiles(radius)),
                    ("shares", shares.join(" / ")),
                ],
            );
            if primary_bp > 0 {
                s += &fill(Msg::GodsAbilityStormTrigger, &[("bonus", p(primary_bp))]);
            }
            if root > 0 {
                let secs = format!("{}s", lang.compact(f64::from(root) / f64::from(TICK_HZ), 1));
                s += &fill(Msg::GodsAbilityStormRoot, &[("secs", secs)]);
            }
            if extra_targets > 0 {
                s += &fill(
                    Msg::GodsAbilityStormExtra,
                    &[
                        ("targets", extra_targets.to_string()),
                        ("share", p(extra_ratio)),
                    ],
                );
            }
            Some(s)
        }
        Ability::VerdictLance {
            every,
            max_hp_bp,
            cap,
            low_hp_bp,
            execute_bp,
        } => {
            let mut s = fill(
                Msg::GodsAbilityVerdict,
                &[
                    ("every", every.to_string()),
                    ("share", p(max_hp_bp)),
                    ("cap", cap.to_string()),
                ],
            );
            if execute_bp > 0 {
                s += &fill(
                    Msg::GodsAbilityVerdictExecute,
                    &[("bonus", p(execute_bp)), ("below", p(low_hp_bp))],
                );
            }
            Some(s)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::{content, UnitKind};

    #[test]
    fn gd04_every_form_has_filled_text_in_both_languages() {
        for kind in [UnitKind::Volthar, UnitKind::Astreon] {
            for (t, b) in crate::art::FORMS {
                for lang in [Lang::En, Lang::PtBr] {
                    let text = content::def(kind)
                        .ability(t, b)
                        .into_iter()
                        .find_map(|a| super::text(a, lang))
                        .expect("a Gods ability row");
                    assert!(!text.contains('{'), "{kind:?} {t}/{b}: {text}");
                }
            }
        }
    }
}
