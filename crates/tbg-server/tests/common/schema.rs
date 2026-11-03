//! Test-only historical database fixtures strip later features before replaying migrations.

pub fn remove_p75(db: &rusqlite::Connection) {
    let tables: Vec<String> = {
        let mut q = db
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        q.query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let p75 = [
        "recovery_identity",
        "recovery_challenges",
        "recovery_dispatch_limits",
        "recovery_audit",
        "recovery_revocations",
        "mail_outbox",
        "mail_dispatch_lease",
        "operators",
        "operator_challenges",
        "operator_sessions",
        "admin_audit",
        "account_moderation",
        "moderation_actions",
        "moderation_events",
        "email_blocks",
        "reports",
        "match_social_records",
        "chat_blocked_terms",
        "admin_settings",
        "emoji_assets",
        "account_emoji_slots",
    ];
    db.execute_batch("PRAGMA foreign_keys=OFF").unwrap();
    for table in tables {
        if p75.contains(&table.as_str()) {
            db.execute_batch(&format!("DROP TABLE {table}")).unwrap();
        }
    }
    db.execute("DELETE FROM schema_version WHERE version=12", [])
        .unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON").unwrap();
}
