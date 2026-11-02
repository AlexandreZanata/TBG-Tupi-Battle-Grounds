//! Every process converges via the shared database; feed work never enters match ticks.
use crate::{auth::Service, chan::LobbyCmd, store::Store, Config};
use std::sync::Arc;
pub(crate) async fn start(
    store: Arc<Store>,
    config: Arc<Config>,
    lobby: tokio::sync::mpsc::Sender<LobbyCmd>,
    auth: Arc<Service>,
) -> Result<(), &'static str> {
    let snapshot = store
        .release(crate::store::moderation::feed::snapshot)
        .await
        .ok_or("moderation_store")?
        .map_err(|_| "moderation_store")?;
    *config
        .moderation
        .0
        .write()
        .map_err(|_| "moderation_cache")? = snapshot;
    let weak = Arc::downgrade(&store);
    let weak_auth = Arc::downgrade(&auth);
    tokio::spawn(async move {
        drop((store, auth));
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(config.feed_ms));
        let mut cursor = 0;
        loop {
            tick.tick().await;
            let Some(store) = weak.upgrade() else {
                return;
            };
            let events = store
                .release(move |db| crate::store::moderation::feed::read(db, cursor))
                .await;
            let Some(Ok(events)) = events else {
                continue;
            };
            let now = config.auth_clock.now().unwrap_or(0);
            if !events.is_empty()
                || config
                    .moderation
                    .0
                    .read()
                    .is_ok_and(|s| s.states.values().any(|v| v.ban_until == Some(now)))
            {
                store.refresh_leaderboard(now).await;
                let next = store
                    .release(crate::store::moderation::feed::snapshot)
                    .await;
                let Some(Ok(next)) = next else {
                    continue;
                };
                if let Ok(mut cache) = config.moderation.0.write() {
                    *cache = next;
                } else {
                    continue;
                }
                for (id, kind, account) in events {
                    if let Some(account) = account {
                        if kind == "ban" {
                            let state = config.moderation.state(account);
                            if config.auth_clock.now().is_ok_and(|now| state.banned(now)) {
                                let (reply, rx) = tokio::sync::oneshot::channel();
                                if lobby
                                    .send(LobbyCmd::ModerationBan { account, reply })
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                                if rx.await != Ok(true) {
                                    break;
                                }
                                if let Some(auth) = weak_auth.upgrade() {
                                    let _ = auth.revoke_account(account, "banned").await;
                                }
                            }
                        }
                    }
                    cursor = id;
                }
            }
            if let Ok(now) = config.auth_clock.now() {
                let _ = store
                    .release(move |db| {
                        let _ = crate::store::social::prune(db, now);
                        let _ = crate::store::operators::maintenance(db, now);
                        let _ = crate::store::recovery::service::maintenance(db, now);
                        crate::store::moderation::feed::prune(db, now)
                    })
                    .await;
            }
        }
    });
    Ok(())
}
