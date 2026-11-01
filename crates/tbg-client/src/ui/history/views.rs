//! Shared Bevy directory, profile and read-only personal history views.

use super::*;

pub fn players(
    parent: &mut ChildSpawnerCommands,
    data: &shell::data::Data,
    account: &super::super::account::Account,
    lang: Lang,
) {
    parent.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("p73.players"), 30.));
        p.spawn(kit::row()).with_children(|r| {
            control(r, "p73.search", Action::Search, lang, 160.);
            control(r, "p73.clear", Action::ClearSearch, lang, 130.);
            control(
                r,
                if data.online_available {
                    "p73.available_only"
                } else {
                    "p73.all_players"
                },
                Action::Available,
                lang,
                210.,
            );
            control(r, "p73.my_history", Action::Mine, lang, 210.);
        });
        if !data.online_query.is_empty() {
            p.spawn(kit::muted(
                format!("{}: {}", lang.key("p73.search"), data.online_query),
                16.,
            ));
        }
        if data.online_error {
            p.spawn(kit::muted(lang.key("p73.error"), 17.));
        }
        if data.online_stale && data.online_loaded {
            p.spawn(kit::muted(lang.key("p73.stale"), 17.));
        }
        if !data.online_complete && data.online_loaded {
            p.spawn(kit::muted(lang.key("p73.partial"), 17.));
        }
        if !data.online_loaded && !data.online_error {
            p.spawn(kit::muted(lang.key("p73.loading"), 17.));
        }
        if data.online_loaded && data.online.is_empty() {
            p.spawn(kit::muted(
                lang.key(if data.online_query.is_empty() {
                    "p73.no_players"
                } else {
                    "p73.no_results"
                }),
                18.,
            ));
        }
        for row in &data.online {
            let Some(name) = Name::new(&row.name) else {
                continue;
            };
            let mine = account
                .profile
                .as_ref()
                .is_some_and(|p| p.name.eq_ignore_ascii_case(&row.name));
            let slot = crate::auth::slot().map(String::from);
            let same_slot = row.slot == slot;
            let available = row.invitable && same_slot && !data.online_stale && !data.online_error;
            p.spawn(kit::row()).with_children(|r| {
                r.spawn(shell::button(
                    format!(
                        "{}{}",
                        row.name,
                        if mine {
                            format!(" · {}", lang.key("p73.you"))
                        } else {
                            String::new()
                        }
                    ),
                    shell::Action::Social(Action::Profile(name)),
                    260.,
                ));
                r.spawn(kit::muted(
                    row.rating.map_or("—".into(), |v| v.to_string()),
                    17.,
                ));
                r.spawn(kit::muted(
                    lang.key(&format!("live.status.{}", row.status)),
                    17.,
                ));
                if !mine && available {
                    control(r, "p73.challenge", Action::Challenge(name), lang, 150.);
                } else if !mine {
                    r.spawn(kit::muted(
                        lang.key(if same_slot {
                            "p73.unavailable"
                        } else {
                            "p73.other_slot"
                        }),
                        15.,
                    ));
                }
            });
        }
        if data.online_loaded {
            p.spawn(kit::row()).with_children(|r| {
                if data.online_offset > 0 {
                    control(r, "p73.previous", Action::Page(-1), lang, 150.);
                }
                r.spawn(kit::muted(
                    format!(
                        "{}–{} / {}",
                        if data.online.is_empty() {
                            0
                        } else {
                            data.online_offset + 1
                        },
                        data.online_offset + data.online.len(),
                        data.online_total
                    ),
                    16.,
                ));
                if data.online_next.is_some() && !data.online_stale {
                    control(r, "p73.next", Action::Page(1), lang, 150.);
                }
            });
        }
    });
}

pub fn rematch_card(p: &mut ChildSpawnerCommands, rematch: &crate::sync::RematchInfo, lang: Lang) {
    if !rematch.open {
        return;
    }
    p.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(
            format!("{} · {}", lang.key("p73.recent_opponent"), rematch.opponent),
            22.,
        ));
        p.spawn(kit::muted(
            lang.key(if rematch.you_requested {
                "p73.rematch_waiting"
            } else if rematch.opponent_requested {
                "p73.rematch_received"
            } else {
                "p73.rematch_offer"
            }),
            16.,
        ));
        p.spawn(kit::row()).with_children(|r| {
            if !rematch.you_requested {
                r.spawn(kit::button(
                    lang.key("p73.rematch"),
                    super::super::UiAction::RequestRematch,
                    220.,
                ));
            }
            r.spawn(kit::button(
                lang.key("p73.decline"),
                super::super::UiAction::ToMenu,
                150.,
            ));
        });
    });
}

pub fn profile(p: &mut ChildSpawnerCommands, history: &History, lang: Lang) {
    if !history.current() {
        return;
    }
    p.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(&history.subject, 32.));
        if history.pending() {
            p.spawn(kit::muted(lang.key("p73.loading"), 18.));
        }
        if let Some(error) = history.error {
            p.spawn(kit::muted(lang.key(error), 18.));
        }
        if let Some(v) = &history.profile_data {
            for key in ["rating", "wins", "losses", "draws"] {
                p.spawn(kit::muted(
                    format!(
                        "{}: {}",
                        lang.key(&format!("p73.{key}")),
                        number(&v[key], lang)
                    ),
                    18.,
                ));
            }
            if let Some(name) = Name::new(&history.subject) {
                control(
                    p,
                    "p73.history",
                    if history.owner {
                        Action::Mine
                    } else {
                        Action::History(name)
                    },
                    lang,
                    240.,
                );
                control(p, "p73.full_profile", Action::FullProfile(name), lang, 240.);
            }
        }
    });
}

pub fn spawn(p: &mut ChildSpawnerCommands, history: &History, lang: Lang) {
    if !history.current() {
        return;
    }
    p.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(
            format!("{} · {}", lang.key("p73.history"), history.subject),
            30.,
        ));
        p.spawn(kit::muted(lang.key("p73.history_boundary"), 14.));
        if history.owner {
            p.spawn(kit::heading(lang.key("p73.visibility"), 20.));
            p.spawn(kit::row()).with_children(|r| {
                if !history.preference_pending() && history.preference.is_some() {
                    control(r, "p73.public", Action::Visibility(true), lang, 180.);
                    control(r, "p73.only_me", Action::Visibility(false), lang, 180.);
                }
                if let Some((public, _)) = history.preference {
                    r.spawn(kit::muted(
                        lang.key(if public { "p73.public" } else { "p73.only_me" }),
                        18.,
                    ));
                }
                if !history.save_state.is_empty() {
                    r.spawn(kit::muted(lang.key(history.save_state), 16.));
                }
            });
            control(p, "p73.preview", Action::Preview, lang, 260.);
            p.spawn(kit::muted(lang.key("p73.public_projection"), 14.));
        } else {
            control(p, "p73.my_history", Action::Mine, lang, 240.);
        }
        if let Some(error) = history.error {
            p.spawn(kit::muted(lang.key(error), 18.));
        }
        if history.pending() || history.preference_pending() {
            p.spawn(kit::muted(lang.key("p73.loading"), 18.));
        }
        if let Some(v) = &history.detail {
            control(p, "p73.back_list", Action::HistoryList, lang, 220.);
            item(p, v, lang);
            summary(p, &v["summary"], lang);
            return;
        }
        p.spawn(kit::row()).with_children(|r| {
            r.spawn(shell::button(
                format!(
                    "{}: {}",
                    lang.key("p73.mode"),
                    lang.key(history.mode.map_or("p73.all", |m| match m {
                        "ranked" => "p73.ranked",
                        "casual" => "p73.casual",
                        _ => "p73.challenge_mode",
                    }))
                ),
                shell::Action::Social(Action::HistoryMode),
                230.,
            ));
            r.spawn(shell::button(
                format!(
                    "{}: {}",
                    lang.key("p73.outcome"),
                    lang.key(history.outcome.map_or("p73.all", |o| match o {
                        "win" => "p73.win",
                        "loss" => "p73.loss",
                        _ => "p73.draw",
                    }))
                ),
                shell::Action::Social(Action::HistoryOutcome),
                230.,
            ));
            if !history.pending() && history.now >= history.backoff {
                control(r, "p73.retry", Action::HistoryRefresh, lang, 150.);
            }
        });
        if history.rows.is_empty() && !history.pending() && history.error.is_none() {
            p.spawn(kit::muted(
                lang.key(if history.mode.is_some() || history.outcome.is_some() {
                    "p73.no_results"
                } else {
                    "p73.history_empty"
                }),
                18.,
            ));
        }
        for v in &history.rows {
            p.spawn(kit::panel()).with_children(|p| {
                item(p, v, lang);
                if !history.pending() {
                    if let Some(id) = v["entry_id"].as_str().and_then(entry) {
                        control(p, "p73.details", Action::HistoryDetail(id), lang, 150.);
                    }
                }
            });
        }
        p.spawn(kit::row()).with_children(|r| {
            if history.previous_available() && !history.pending() {
                control(r, "p73.previous", Action::HistoryBack, lang, 150.);
            }
            if history.has_more && !history.pending() {
                control(r, "p73.next", Action::HistoryNext, lang, 150.);
            }
        });
    });
}

fn item(p: &mut ChildSpawnerCommands, v: &Value, lang: Lang) {
    let opponent = v["opponent"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| lang.key("p73.private_opponent"));
    p.spawn(kit::heading(
        format!(
            "{} · {}",
            lang.key(&format!(
                "p73.{}",
                v["outcome"].as_str().unwrap_or("unavailable")
            )),
            opponent
        ),
        21.,
    ));
    let arena = v["arena"]
        .as_str()
        .map(|s| lang.key(&format!("arenas.{s}.name")))
        .unwrap_or_else(|| lang.key("p73.unavailable_metrics"));
    let timestamp = v["ended_unix"]
        .as_i64()
        .map(|t| date(t, lang))
        .unwrap_or_else(|| lang.key("p73.unavailable_metrics"));
    let mode = mode(&v["mode"]);
    p.spawn(kit::muted(
        format!(
            "{} · {} · {} · {}",
            timestamp,
            lang.key(&format!(
                "p73.{}",
                if mode == "challenge" {
                    "challenge_mode"
                } else {
                    &mode
                }
            )),
            arena,
            duration(v["wall_ms"].as_u64().map(|ms| ms / 1000), lang)
        ),
        16.,
    ));
    if v["quality"] != "complete" {
        p.spawn(kit::muted(lang.key("p73.legacy"), 14.));
    }
    p.spawn(kit::muted(
        format!(
            "{}: {}",
            lang.key("p73.end_reason"),
            lang.key(&format!(
                "p73.reason.{}",
                v["reason"].as_str().unwrap_or("unavailable")
            ))
        ),
        15.,
    ));
    let status = v["rating_status"].as_str().unwrap_or("pending");
    p.spawn(kit::muted(
        format!(
            "{}{}",
            lang.key(&format!("p73.rating_{status}")),
            v["rating_delta"]
                .as_i64()
                .map_or(String::new(), |d| format!(" {d:+}"))
        ),
        15.,
    ));
}

pub fn summary(p: &mut ChildSpawnerCommands, s: &Value, lang: Lang) {
    if s["version"] != 1 {
        p.spawn(kit::muted(lang.key("p73.unsupported"), 18.));
        return;
    }
    for (key, label) in [
        ("purchases", "p73.purchases"),
        ("evolutions", "p73.evolutions"),
        ("fortifications", "p73.fortifications"),
    ] {
        p.spawn(kit::muted(
            format!("{}: {}", lang.key(label), number(&s[key], lang)),
            18.,
        ));
    }
    p.spawn(kit::muted(
        format!(
            "{}: {} · {}: {}",
            lang.key("p73.combat_time"),
            duration(
                s["combat_ticks"]
                    .as_u64()
                    .zip(s["tick_hz"].as_u64())
                    .and_then(|(t, h)| t.checked_div(h)),
                lang
            ),
            lang.key("p73.total_time"),
            duration(s["wall_ms"].as_u64().map(|ms| ms / 1000), lang)
        ),
        16.,
    ));
    let tower = &s["stronghold"];
    if !tower.is_null() {
        let kind = tower["kind"].as_str().unwrap_or("unavailable");
        p.spawn(kit::muted(
            format!(
                "{}: {} · HP {}/{}",
                lang.key("p73.stronghold"),
                lang.key(&format!("stronghold.{kind}.name")),
                number(&tower["hp"], lang),
                number(&tower["max_hp"], lang)
            ),
            17.,
        ));
    }
    p.spawn(kit::heading(lang.key("p73.champions"), 21.));
    if let Some(champions) = s["champions"].as_array() {
        for champion in champions {
            let id = champion["champion"].as_str().unwrap_or("");
            let name = tbg_core::types::ALL_KINDS
                .iter()
                .find(|k| format!("{k:?}") == id)
                .map(|k| crate::i18n::content::unit_name(lang, *k))
                .unwrap_or(id);
            p.spawn(kit::muted(
                format!(
                    "{} × {} · {} {}/{}",
                    name,
                    number(&champion["purchases"], lang),
                    lang.key("p73.evolution_tiers"),
                    number(&champion["evolution_tiers"][0], lang),
                    number(&champion["evolution_tiers"][1], lang)
                ),
                16.,
            ));
        }
        if champions.is_empty() {
            p.spawn(kit::muted("0", 16.));
        }
    } else {
        p.spawn(kit::muted(lang.key("p73.unavailable_metrics"), 16.));
    }
    p.spawn(kit::heading(lang.key("p73.augments"), 21.));
    if let Some(augments) = s["augments"].as_array() {
        for augment in augments {
            let name = augment["id"]
                .as_u64()
                .and_then(|id| u8::try_from(id).ok())
                .and_then(|id| crate::i18n::content::augment_name(lang, id))
                .map(str::to_owned)
                .unwrap_or_else(|| lang.key("p73.unsupported"));
            p.spawn(kit::muted(
                format!("{} × {}", name, number(&augment["copies"], lang)),
                16.,
            ));
        }
        if augments.is_empty() {
            p.spawn(kit::muted("0", 16.));
        }
    } else {
        p.spawn(kit::muted(lang.key("p73.unavailable_metrics"), 16.));
    }
}
fn number(v: &Value, lang: Lang) -> String {
    v.as_i64()
        .map_or_else(|| lang.key("p73.unavailable_metrics"), |n| n.to_string())
}
fn duration(seconds: Option<u64>, lang: Lang) -> String {
    seconds.map_or_else(
        || lang.key("p73.unavailable_metrics"),
        |s| format!("{}:{:02}", s / 60, s % 60),
    )
}
fn date(seconds: i64, lang: Lang) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        jiff::Timestamp::from_second(seconds)
            .ok()
            .map(|t| {
                t.to_zoned(jiff::tz::TimeZone::system())
                    .strftime(if lang == Lang::PtBr {
                        "%d/%m/%Y %H:%M %:z"
                    } else {
                        "%m/%d/%Y %H:%M %:z"
                    })
                    .to_string()
            })
            .unwrap_or_else(|| lang.key("p73.unavailable_metrics"))
    }
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        #[wasm_bindgen(module = "/assets/auth/history.js")]
        extern "C" {
            #[wasm_bindgen(js_name=historyTime)]
            fn local_time(seconds: f64, locale: &str) -> String;
        }
        local_time(seconds as f64, lang.code())
    }
}
