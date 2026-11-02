//! Single-process-at-a-time outbox dispatcher, outside all simulation ticks.

use super::{
    settings::Settings,
    transport::{CapturedMail, Transport},
};
use crate::{
    auth::Clock,
    metrics::Metrics,
    store::{mail_outbox, Store},
};
use std::sync::Arc;

pub async fn once(
    store: &Store,
    transport: &Transport,
    settings: &Settings,
    holder: &str,
    now: i64,
    metrics: &Metrics,
) -> Result<bool, &'static str> {
    let owner = holder.to_owned();
    let row = store
        .release(move |db| mail_outbox::lease(db, &owner, now))
        .await
        .ok_or("store_unavailable")?
        .map_err(|_| "store_unavailable")?;
    let Some(row) = row else {
        return Ok(false);
    };
    let url = if row.purpose.starts_with("operator_") {
        &settings.admin_url
    } else {
        &settings.game_url
    };
    let body = super::templates::body(&row.purpose, &row.locale, row.code.as_deref(), url);
    let result = transport
        .send(
            settings,
            CapturedMail {
                to: row.to,
                purpose: row.purpose,
                code: row.code,
                body,
            },
        )
        .await;
    let error = result.err();
    let id = row.id;
    let owner = holder.to_owned();
    store
        .release(move |db| mail_outbox::finish(db, id, &owner, now, error))
        .await
        .ok_or("store_unavailable")?
        .map_err(|_| "store_unavailable")?;
    Metrics::add(
        if error.is_none() {
            &metrics.mail_sent
        } else {
            &metrics.mail_failed
        },
        1,
    );
    Ok(true)
}

pub(crate) fn start(
    store: Arc<Store>,
    transport: Arc<Transport>,
    settings: Settings,
    holder: String,
    clock: Clock,
    metrics: Arc<Metrics>,
) {
    let weak = Arc::downgrade(&store);
    tokio::spawn(async move {
        drop(store);
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            tick.tick().await;
            let Some(store) = weak.upgrade() else {
                return;
            };
            let Ok(now) = clock.now() else {
                continue;
            };
            // At most one mail in flight; backlog cannot spawn unbounded work.
            let _ = once(&store, &transport, &settings, &holder, now, &metrics).await;
            let _ = store.release(move |db| mail_outbox::prune(db, now)).await;
        }
    });
}
