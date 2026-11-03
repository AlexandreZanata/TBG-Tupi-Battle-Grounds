//! One medieval presentation for identity, recovery, report and emoji-library states.
use super::*;
#[derive(Component)]
pub(super) struct Root;
#[allow(clippy::too_many_arguments)]
pub(super) fn render(
    mut commands: Commands,
    s: Res<State>,
    chat: Res<super::super::match_chat::Chat>,
    info: Res<MatchInfo>,
    game: Res<bevy::prelude::State<GameState>>,
    lang: Res<Lang>,
    form: NonSend<ActiveForm>,
    roots: Query<Entity, With<Root>>,
    time: Res<Time>,
    mut shown: Local<String>,
) {
    let signature = format!(
        "{:?}:{}:{:?}:{}:{}:{}",
        s.page,
        s.revision,
        game.get(),
        lang.code(),
        chat.mode,
        textform::is_open(&form)
    );
    if *shown == signature {
        return;
    }
    *shown = signature;
    for e in &roots {
        commands.entity(e).despawn();
    }
    if *game.get() == GameState::Menu && !textform::is_open(&form) && s.page == Page::None {
        commands
            .spawn((
                Root,
                GlobalZIndex(40),
                Node {
                    position_type: PositionType::Absolute,
                    right: percent(2.),
                    bottom: percent(2.),
                    column_gap: px(8),
                    ..default()
                },
            ))
            .with_children(|p| {
                if crate::auth::snapshot().access.is_some() {
                    toolbar_button(p, *lang, "p75.account", Action::Account);
                    toolbar_button(p, *lang, "p75.emojis", Action::Emojis);
                } else {
                    toolbar_button(p, *lang, "p75.lost_key", Action::LostKey);
                }
            });
    }
    if *game.get() == GameState::Result
        && !info.spectating
        && s.page == Page::None
        && crate::auth::snapshot().access.is_some()
        && s.eligibility
            .is_some_and(|(id, eligible)| eligible && Some(id) == info.result_id)
    {
        commands
            .spawn((
                Root,
                GlobalZIndex(40),
                Node {
                    position_type: PositionType::Absolute,
                    right: percent(2.),
                    bottom: percent(15.),
                    ..default()
                },
            ))
            .with_children(|p| {
                toolbar_button(p, *lang, "p75.report", Action::ReportMatch);
                if let (Some(you), Some(policy)) = (info.you, chat.policy.as_ref()) {
                    for emoji in &policy.loadouts[1 - you.index()] {
                        if let tbg_core::protocol::EmojiRef::Custom { id, .. } = emoji {
                            toolbar_button(p, *lang, "p75.report_emoji", Action::ReportEmoji(*id));
                        }
                    }
                }
            });
    }
    if s.page == Page::None {
        return;
    }
    commands
        .spawn((
            Root,
            GlobalZIndex(90),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.8)),
        ))
        .with_children(|root| {
            root.spawn(kit::panel())
                .insert((
                    DialogPanel,
                    Node {
                        width: px(840),
                        max_height: percent(88),
                        overflow: Overflow::scroll_y(),
                        padding: UiRect::all(px(24)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(kit::heading(
                        lang.key(match s.page {
                            Page::Account => "p75.account",
                            Page::Recovery => "p75.lost_key",
                            Page::Emojis => "p75.emojis",
                            Page::Report => "p75.report",
                            Page::Suspended => "p75.suspended",
                            Page::None => "p75.account",
                        }),
                        28.,
                    ));
                    p.spawn(kit::muted(lang.key("p75.keyboard"), 14.));
                    if !s.notice.is_empty() {
                        p.spawn(kit::muted(lang.key(s.notice), 16.));
                    }
                    match s.page {
                        Page::Account => {
                            account(p, &s, *lang);
                            button(p, *lang, &format!("p75.mode.{}", chat.mode), Action::Mode);
                        }
                        Page::Recovery => recovery(p, &s, *lang),
                        Page::Report => report(p, &s, *lang),
                        Page::Emojis => emojis(p, &s, *lang, time.elapsed_secs_f64()),
                        Page::Suspended => {
                            if let Some((until, reason)) = s.suspension {
                                p.spawn(kit::muted(
                                    lang.key(&format!("p75.reason.{}", reason.as_str())),
                                    20.,
                                ));
                                p.spawn(kit::muted(
                                    until
                                        .map(|v| {
                                            i64::try_from(v)
                                                .map(|v| super::super::history::date(v, *lang))
                                                .unwrap_or_else(|_| {
                                                    lang.key("p73.unavailable_metrics")
                                                })
                                        })
                                        .unwrap_or_else(|| lang.key("p75.permanent")),
                                    18.,
                                ));
                            }
                        }
                        _ => {}
                    }
                    button(p, *lang, "common.close", Action::Close);
                });
        });
    let _ = &chat;
}
fn account(p: &mut ChildSpawnerCommands, s: &State, lang: Lang) {
    p.spawn(kit::muted(
        s.data["email_masked"].as_str().unwrap_or(""),
        20.,
    ));
    p.spawn(kit::muted(
        lang.key(&format!(
            "p75.email_state.{}",
            s.data["email_state"].as_str().unwrap_or("none")
        )),
        18.,
    ));
    p.spawn(kit::row()).with_children(|r| {
        button(r, lang, "p75.enter_code", Action::EmailVerify);
        button(r, lang, "p75.resend", Action::EmailResend);
    });
    p.spawn(kit::muted(lang.key("p75.change_safeguard"), 15.));
    p.spawn(kit::row()).with_children(|r| {
        button(r, lang, "p75.key_proof", Action::KeyProof);
        button(r, lang, "p75.new_email", Action::NewEmail);
    });
    p.spawn(kit::muted(&s.email.email, 16.));
    button(
        p,
        lang,
        if s.email.hold {
            "p75.hold_on"
        } else {
            "p75.hold_off"
        },
        Action::Hold,
    );
    button(p, lang, "p75.change_email", Action::EmailChange);
    p.spawn(kit::row()).with_children(|r| {
        button(r, lang, "p75.old_code", Action::OldCode);
        button(r, lang, "p75.new_code", Action::NewCode);
    });
    button(p, lang, "p75.confirm_change", Action::EmailConfirm);
    button(p, lang, "p75.cancel_change", Action::EmailCancel);
    if let Some(until) = s.data["pending_change"]["until"]
        .as_i64()
        .filter(|until| *until > 0)
    {
        p.spawn(kit::muted(
            format!(
                "{}: {}",
                lang.key("p75.hold_until"),
                super::super::history::date(until, lang)
            ),
            16.,
        ));
    }
}
fn recovery(p: &mut ChildSpawnerCommands, s: &State, lang: Lang) {
    p.spawn(kit::muted(lang.key("p75.recovery_guidance"), 18.));
    p.spawn(kit::row()).with_children(|r| {
        button(r, lang, "p75.name_prompt", Action::RecoveryName);
        button(r, lang, "p75.email_prompt", Action::RecoveryEmail);
    });
    p.spawn(kit::muted(&s.recovery.name, 18.));
    button(p, lang, "p75.send_code", Action::RecoveryRequest);
    if s.recovery.requested {
        p.spawn(kit::muted(lang.key("p75.recovery_generic"), 17.));
        button(p, lang, "p75.enter_code", Action::RecoveryCode);
    }
}
fn report(p: &mut ChildSpawnerCommands, s: &State, lang: Lang) {
    p.spawn(kit::muted(lang.key("p75.report_privacy"), 16.));
    for row in 0..4 {
        p.spawn(kit::row()).with_children(|r| {
            for col in 0..2 {
                let index = row * 2 + col;
                button(
                    r,
                    lang,
                    &format!("p75.reason.{}", super::super::report::REASONS[index]),
                    Action::Reason(index as u8),
                );
            }
        });
    }
    p.spawn(kit::muted(
        lang.key(&format!(
            "p75.reason.{}",
            super::super::report::REASONS[usize::from(s.report.reason)]
        )),
        18.,
    ));
    button(p, lang, "p75.comment", Action::Comment);
    p.spawn(kit::muted(&s.report.comment, 16.));
    button(p, lang, "p75.send_report", Action::SendReport);
}
fn emojis(p: &mut ChildSpawnerCommands, s: &State, lang: Lang, now: f64) {
    p.spawn(kit::row()).with_children(|r| {
        for slot in 0..3 {
            button(
                r,
                lang,
                &format!("p75.slot{}", slot + 1),
                Action::Slot(slot),
            );
        }
    });
    p.spawn(kit::muted(
        format!("{} {}", lang.key("p75.selected_slot"), s.library.slot + 1),
        17.,
    ));
    for row in 0..4 {
        p.spawn(kit::row()).with_children(|r| {
            for col in 0..4 {
                let id = row * 4 + col;
                r.spawn((
                    kit::control(id.to_string(), true, 44.),
                    UiAction::P75(Action::Standard(id)),
                    DialogButton,
                ))
                .with_children(|p| {
                    crate::emoji::spawn(
                        p,
                        tbg_core::protocol::EmojiRef::Standard { id },
                        id,
                        now,
                        32.,
                    )
                });
            }
        });
    }
    p.spawn(kit::muted(lang.key("p75.upload_guidance"), 16.));
    button(p, lang, "p75.choose_file", Action::Upload);
    button(p, lang, "p75.label", Action::UploadLabel);
    p.spawn(kit::muted(&s.library.label, 16.));
    if let Some((_, name)) = &s.library.selected {
        p.spawn(kit::muted(name, 16.));
        button(p, lang, "p75.upload_send", Action::UploadSend);
    }
    if let Some(uploads) = s.data["uploads"].as_array() {
        for asset in uploads.iter().take(10) {
            p.spawn(kit::muted(
                format!(
                    "{} · {}",
                    asset["label"].as_str().unwrap_or(""),
                    lang.key(&format!(
                        "p75.asset.{}",
                        asset["status"].as_str().unwrap_or("pending")
                    ))
                ),
                16.,
            ));
            if let Some(id) = asset["id"].as_i64() {
                let reference = tbg_core::protocol::EmojiRef::Custom {
                    id: [0; 16],
                    frame_px: 128,
                    frames: asset["frames"].as_u64().unwrap_or(0) as u8,
                    columns: asset["columns"].as_u64().unwrap_or(0) as u8,
                    delays_ms: serde_json::from_value(asset["delays_ms"].clone())
                        .unwrap_or_default(),
                };
                if reference.valid() {
                    crate::emoji::preview(p, id, reference, now);
                }
                if let Some(reason) = asset["reason"].as_str() {
                    p.spawn(kit::muted(reason, 16.));
                }
                if matches!(asset["status"].as_str(), Some("pending" | "approved")) {
                    button(p, lang, "p75.withdraw", Action::Withdraw(id));
                }
            }
            if let Some(public) = asset["public_id"]
                .as_str()
                .and_then(parse_id)
                .filter(|_| asset["status"] == "approved")
            {
                button(p, lang, "p75.equip", Action::EquipCustom(public));
            }
        }
    }
}
fn parse_id(s: &str) -> Option<[u8; 16]> {
    if s.len() != 32 || !s.is_ascii() {
        return None;
    }
    let mut b = [0; 16];
    for (i, p) in s.as_bytes().chunks(2).enumerate() {
        b[i] = u8::from_str_radix(std::str::from_utf8(p).ok()?, 16).ok()?;
    }
    Some(b)
}
