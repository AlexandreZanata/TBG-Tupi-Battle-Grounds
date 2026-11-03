//! Atomic additive P75 storage shape, including constraints and append-only audit guards.

use rusqlite::Connection;

pub(super) const FEATURE: super::schema::Feature = super::schema::Feature {
    tables: include_str!("schema.sql"),
    indexes: "",
    shape: &[
        (
            "recovery_identity",
            &[
                "account_id",
                "email",
                "verified_email",
                "verified_unix",
                "pending_email",
                "pending_unix",
                "epoch",
                "compromised",
                "locale",
                "updated_unix",
            ],
        ),
        (
            "recovery_challenges",
            &[
                "account_id",
                "purpose",
                "digest",
                "email",
                "epoch",
                "expires_unix",
                "attempts",
                "consumed",
            ],
        ),
        ("recovery_dispatch_limits", &["bucket", "issued"]),
        (
            "recovery_audit",
            &["id", "account_id", "at_unix", "epoch", "action"],
        ),
        ("recovery_revocations", &["account_id", "epoch", "at_unix"]),
        (
            "mail_outbox",
            &[
                "id",
                "purpose",
                "principal_kind",
                "principal_id",
                "to_email",
                "locale",
                "created_unix",
                "not_before_unix",
                "attempts",
                "status",
                "lease_owner",
                "lease_until_unix",
                "last_error",
                "sent_unix",
            ],
        ),
        (
            "mail_dispatch_lease",
            &["singleton", "holder", "until_unix"],
        ),
        (
            "operators",
            &[
                "id",
                "email",
                "role",
                "status",
                "key_hash",
                "key_epoch",
                "locale",
                "created_unix",
                "claimed_unix",
                "disabled_unix",
                "origin",
            ],
        ),
        (
            "operator_challenges",
            &[
                "operator_id",
                "purpose",
                "digest",
                "key_epoch",
                "expires_unix",
                "attempts",
                "consumed",
                "attempt_hash",
            ],
        ),
        (
            "operator_sessions",
            &[
                "token_hash",
                "operator_id",
                "csrf_hash",
                "key_epoch",
                "created_unix",
                "last_seen_unix",
                "absolute_expires_unix",
                "reauth_unix",
                "revoked_unix",
                "revoke_reason",
            ],
        ),
        (
            "admin_audit",
            &[
                "id",
                "at_unix",
                "operator_id",
                "actor",
                "action",
                "target_kind",
                "target_id",
                "reason_code",
                "note",
                "before_json",
                "after_json",
                "request_id",
            ],
        ),
        (
            "account_moderation",
            &[
                "account_id",
                "ban_until_unix",
                "ban_reason",
                "chat_until_unix",
                "emoji_until_unix",
                "version",
                "updated_unix",
            ],
        ),
        (
            "moderation_actions",
            &[
                "id",
                "at_unix",
                "operator_id",
                "account_id",
                "action",
                "until_unix",
                "reason",
                "note",
                "report_id",
            ],
        ),
        (
            "moderation_events",
            &["id", "at_unix", "kind", "account_id", "payload_json"],
        ),
        (
            "email_blocks",
            &["email", "created_unix", "operator_id", "reason"],
        ),
        (
            "reports",
            &[
                "id",
                "created_unix",
                "reporter_id",
                "target_id",
                "reason",
                "context",
                "result_id",
                "emoji_id",
                "comment",
                "evidence_json",
                "status",
                "resolved_unix",
                "resolved_by",
                "resolution_note",
            ],
        ),
        (
            "match_social_records",
            &[
                "result_id",
                "ended_unix",
                "expires_unix",
                "mode",
                "a_account",
                "b_account",
                "a_name",
                "b_name",
                "outcome_json",
                "loadouts_json",
                "transcript_json",
            ],
        ),
        (
            "chat_blocked_terms",
            &["term", "created_unix", "operator_id"],
        ),
        (
            "admin_settings",
            &["key", "value", "updated_unix", "operator_id"],
        ),
        (
            "emoji_assets",
            &[
                "id",
                "public_id",
                "owner_account_id",
                "content_hash",
                "status",
                "label",
                "sheet",
                "frame_px",
                "frames",
                "columns",
                "delays_json",
                "source_format",
                "source_bytes",
                "created_unix",
                "reviewed_unix",
                "reviewer_id",
                "decision_reason",
            ],
        ),
        (
            "account_emoji_slots",
            &[
                "account_id",
                "slot",
                "standard_id",
                "emoji_id",
                "updated_unix",
            ],
        ),
    ],
};

/// Column presence alone cannot prove the single-Owner and audit constraints.
pub(super) fn verify(conn: &Connection) -> rusqlite::Result<()> {
    let reference = Connection::open_in_memory()?;
    reference.execute_batch(FEATURE.tables)?;
    let mut expected = reference.prepare("SELECT type,name,sql FROM sqlite_master WHERE sql IS NOT NULL AND name NOT LIKE 'sqlite_%'")?;
    let objects = expected.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    for object in objects {
        let (kind, name, sql) = object?;
        let actual = conn.query_row(
            "SELECT sql FROM sqlite_master WHERE type=?1 AND name=?2",
            [&kind, &name],
            |r| r.get::<_, String>(0),
        );
        if actual
            .map(|v| canonical(&v) != canonical(&sql))
            .unwrap_or(true)
        {
            return Err(super::schema::foreign_shape(&name));
        }
    }
    Ok(())
}

fn canonical(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}
