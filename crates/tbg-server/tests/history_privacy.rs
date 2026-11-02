//! BR-AP11/16: current durable visibility, ownership, public projection and concurrent saves.
#[path = "common/player_history.rs"]
mod fixture;
use fixture::*;
use tbg_server::store::player_history::{self, Error, Query, Subject};

#[tokio::test]
async fn private_by_default_owner_reads_and_independent_entry_ids_never_grant_access() {
    let (store, _path, accounts) = store().await;
    store
        .release(move |conn| {
            let e = envelope(1, accounts);
            player_history::insert(conn, &e).unwrap();
            for account in accounts {
                assert_eq!(
                    player_history::preference(conn, account).unwrap()["visibility"],
                    "private"
                );
            }
            let mine = owner(accounts[0], "FixtureA");
            let other = owner(accounts[1], "FixtureB");
            let id = tbg_server::ranked::hex(&e.sides[0].entry_id);
            assert!(player_history::detail(conn, &mine, &id).is_ok());
            assert_eq!(
                player_history::detail(conn, &other, &id).unwrap_err(),
                Error::NotFound
            );
            assert_eq!(
                player_history::list(
                    conn,
                    &Subject::Public("FixtureA".into()),
                    &Query::parse(None).unwrap()
                )
                .unwrap_err(),
                Error::Private
            );
            assert!(player_history::detail(
                conn,
                &mine,
                &tbg_server::ranked::hex(&e.sides[1].entry_id)
            )
            .is_err());
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn public_opt_in_never_publishes_private_opponents_or_other_side_statistics() {
    let (store, _path, accounts) = store().await;
    store
        .release(move |conn| {
            let mut e = envelope(1, accounts);
            e.sides[1].summary.purchases = Some(938);
            player_history::insert(conn, &e).unwrap();
            player_history::set_preference(conn, accounts[0], "public", 0).unwrap();
            let id = tbg_server::ranked::hex(&e.sides[0].entry_id);
            let public =
                player_history::detail(conn, &Subject::Public("FixtureA".into()), &id).unwrap();
            assert!(public["opponent"].is_null());
            assert_eq!(public["summary"]["purchases"], 0);
            assert_eq!(public["origin"], "private");
            let text = public.to_string();
            for forbidden in [
                "FixtureB",
                "938",
                "history_key",
                "result_id",
                "account_id",
                "sides",
                "entry_id_b",
            ] {
                assert!(!text.contains(forbidden), "{forbidden}: {text}");
            }
            let owner = player_history::detail(conn, &owner(accounts[0], "FixtureA"), &id).unwrap();
            assert_eq!(owner["opponent"], "FixtureB");
            assert_eq!(
                player_history::list(
                    conn,
                    &Subject::Public("FixtureB".into()),
                    &Query::parse(None).unwrap()
                )
                .unwrap_err(),
                Error::Private
            );
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn stale_public_saves_cannot_reverse_a_committed_private_revision() {
    let (store, _path, accounts) = store().await;
    store
        .release(move |conn| {
            let e = envelope(1, accounts);
            player_history::insert(conn, &e).unwrap();
            assert_eq!(
                player_history::set_preference(conn, accounts[0], "public", 0).unwrap()["revision"],
                1
            );
            assert_eq!(
                player_history::set_preference(conn, accounts[0], "private", 1).unwrap()
                    ["revision"],
                2
            );
            assert_eq!(
                player_history::set_preference(conn, accounts[0], "public", 0).unwrap_err(),
                Error::Revision
            );
            assert_eq!(
                player_history::preference(conn, accounts[0]).unwrap()["visibility"],
                "private"
            );
            let id = tbg_server::ranked::hex(&e.sides[0].entry_id);
            assert_eq!(
                player_history::detail(conn, &Subject::Public("FixtureA".into()), &id).unwrap_err(),
                Error::Private
            );
        })
        .await
        .unwrap();
}
