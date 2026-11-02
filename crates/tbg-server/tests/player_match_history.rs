//! BR-AP09/10/12/13/15: atomic identity, keyset paging, journal replay and verified imports.
#[path = "common/player_history.rs"]
mod fixture;
use fixture::*;
use tbg_server::{
    player_history::journal,
    store::player_history::{self, Error, Query},
};

#[tokio::test]
async fn immutable_envelopes_are_atomic_idempotent_and_conflicts_never_overwrite() {
    let (store, _path, accounts) = store().await;
    store
        .release(move |conn| {
            let e = envelope(1, accounts);
            player_history::insert(conn, &e).unwrap();
            player_history::insert(conn, &e).unwrap();
            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM player_match_history", [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                1
            );
            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM player_match_history_sides", [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                2
            );
            let mut conflicting = e.clone();
            conflicting.sides[0].summary.purchases = Some(2);
            assert_eq!(
                player_history::insert(conn, &conflicting),
                Err(Error::Conflict)
            );
            let mut invalid = envelope(2, accounts);
            invalid.sides[1].account = Some(i64::MAX);
            assert!(player_history::insert(conn, &invalid).is_err());
            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM player_match_history", [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                1
            );
            let detail = player_history::detail(
                conn,
                &owner(accounts[0], "FixtureA"),
                &tbg_server::ranked::hex(&e.sides[0].entry_id),
            )
            .unwrap();
            assert_eq!(
                detail["summary"]["purchases"], 0,
                "measured zero remains distinct from unavailable"
            );
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn sixty_five_tied_results_page_without_duplicates_or_cursor_authorization() {
    let (store, _path, accounts) = store().await;
    store.release(move|conn| {
        for i in 1..=65 {player_history::insert(conn,&envelope(i,accounts)).unwrap();}
        let subject=owner(accounts[0],"FixtureA");let mut query=Query::parse(None).unwrap();let mut ids=std::collections::BTreeSet::new();
        let first=player_history::list(conn,&subject,&query).unwrap();let cursor=first["cursor"].as_str().unwrap().to_owned();
        let mut wrong=Query::parse(None).unwrap();wrong.cursor=Some(cursor.clone());
        assert_eq!(player_history::list(conn,&owner(accounts[1],"FixtureB"),&wrong).unwrap_err(),Error::Invalid);
        wrong.mode=Some("ranked".into());assert_eq!(player_history::list(conn,&subject,&wrong).unwrap_err(),Error::Invalid);
        let mut head=envelope(66,accounts);for s in &mut head.sides{s.summary.ended_unix=1001;}player_history::insert(conn,&head).unwrap();
        for v in first["rows"].as_array().unwrap(){assert!(ids.insert(v["entry_id"].as_str().unwrap().to_owned()));}
        query.cursor=Some(cursor);
        loop {let page=player_history::list(conn,&subject,&query).unwrap();for v in page["rows"].as_array().unwrap(){assert!(ids.insert(v["entry_id"].as_str().unwrap().to_owned()));}if page["has_more"]==false{break;}query.cursor=Some(page["cursor"].as_str().unwrap().into());}
        assert_eq!(ids.len(),65,"new head insertion does not shift a keyset cursor");
        let plan:String=conn.prepare("EXPLAIN QUERY PLAN SELECT entry_id FROM player_match_history_sides WHERE account_id=?1 AND (ended_unix,entry_id)<(?2,?3) ORDER BY ended_unix DESC,entry_id DESC LIMIT 21").unwrap().query_map(rusqlite::params![accounts[0],i64::MAX,vec![255u8;16]],|r|r.get::<_,String>(3)).unwrap().map(Result::unwrap).collect::<Vec<_>>().join(" ");
        assert!(plan.contains("player_match_history_by_account"),"{plan}");
    }).await.unwrap();
}

#[tokio::test]
async fn a_committed_file_replays_after_crash_without_changing_participant_ids() {
    let (store, path, accounts) = store().await;
    let dir = journal::directory(&path, Some("fixture"));
    let e = envelope(1, accounts);
    journal::write(&dir, &e).unwrap();
    assert_eq!(journal::load(&dir).unwrap(), vec![e.clone()]);
    let saved = e.clone();
    store
        .release(move |conn| player_history::insert(conn, &saved))
        .await
        .unwrap()
        .unwrap();
    // Crash after DB commit and before journal removal.
    let recovered = journal::load(&dir).unwrap().pop().unwrap();
    assert_eq!(recovered.sides[0].entry_id, e.sides[0].entry_id);
    store
        .release(move |conn| player_history::insert(conn, &recovered))
        .await
        .unwrap()
        .unwrap();
    journal::remove(&dir, &e).unwrap();
    assert!(journal::load(&dir).unwrap().is_empty());
    // BR-AP12: an unrenamed staging file (torn or complete) was never
    // acknowledged; it neither hides valid entries nor replays forever.
    let other = envelope(2, accounts);
    journal::write(&dir, &other).unwrap();
    let torn = dir.join(".tmp-torn.json");
    std::fs::write(&torn, b"{\"partial").unwrap();
    let staged = dir.join(".tmp-complete.json");
    std::fs::write(&staged, serde_json::to_vec(&e).unwrap()).unwrap();
    assert_eq!(journal::load(&dir).unwrap(), vec![other.clone()]);
    journal::remove(&dir, &other).unwrap();
    std::fs::remove_file(torn).unwrap();
    std::fs::remove_file(staged).unwrap();
    let oversized = dir.join("bad.json");
    std::fs::write(&oversized, vec![b' '; 32769]).unwrap();
    assert!(journal::load(&dir).is_err());
    assert!(oversized.exists());
    std::fs::remove_file(oversized).unwrap();
    std::fs::remove_dir(dir).unwrap();
}

#[tokio::test]
async fn additive_migration_backfills_only_verified_ranked_links_with_missing_stats_unavailable() {
    let path = path();
    let store = tbg_server::store::Store::open(&path).unwrap();
    let a = store.register("FixtureA".into(), [1; 32]).await.unwrap().0;
    let b = store.register("FixtureB".into(), [2; 32]).await.unwrap().0;
    store.release(move|conn| {
        conn.execute("INSERT INTO ranked_matches(ended_at,a_id,b_id,winner,reason,duration_s,a_before,a_after,b_before,b_after,a_factions,b_factions) VALUES('2026-11-01T12:00:00Z',?1,?2,0,'Forfeit',30,1000,1010,1000,990,'{}','{}')",[a,b]).unwrap();
    }).await.unwrap();
    let migrated = tbg_server::store::Store::open_history(&path, Default::default(), true).unwrap();
    migrated
        .release(move |conn| {
            assert!(player_history::backfill::batch(conn).unwrap());
            assert!(!player_history::backfill::batch(conn).unwrap());
            let page =
                player_history::list(conn, &owner(a, "FixtureA"), &Query::parse(None).unwrap())
                    .unwrap();
            assert_eq!(page["rows"].as_array().unwrap().len(), 1);
            let id = page["rows"][0]["entry_id"].as_str().unwrap();
            let detail = player_history::detail(conn, &owner(a, "FixtureA"), id).unwrap();
            assert_eq!(detail["quality"], "result_only");
            assert!(detail["summary"]["purchases"].is_null());
            assert_eq!(detail["rating_delta"], 10);
            assert_eq!(
                player_history::preference(conn, a).unwrap()["visibility"],
                "private"
            );
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn draining_legacy_results_backfill_later_without_preempting_a_modern_pending_envelope() {
    let (store, _path, accounts) = store().await;
    store.release(move|conn| {
        let row="INSERT INTO ranked_matches(ended_at,a_id,b_id,winner,reason,duration_s,a_before,a_after,b_before,b_after,a_factions,b_factions,match_key,history_expected) VALUES('2026-11-01T12:00:00Z',?1,?2,0,'Forfeit',30,1000,1010,1000,990,'{}','{}',?3,?4)";
        assert!(!player_history::backfill::batch(conn).unwrap());
        conn.execute(row,rusqlite::params![accounts[0],accounts[1],Option::<Vec<u8>>::None,0]).unwrap();
        let key=[73;16];conn.execute(row,rusqlite::params![accounts[0],accounts[1],key.as_slice(),1]).unwrap();
        assert!(player_history::backfill::batch(conn).unwrap());
        let subject=owner(accounts[0],"FixtureA");let query=Query::parse(None).unwrap();
        assert_eq!(player_history::list(conn,&subject,&query).unwrap()["rows"].as_array().unwrap().len(),1,"legacy terminal rows after migration remain recoverable");
        let mut modern=envelope(73,accounts);modern.key=format!("ranked:{}",tbg_server::ranked::hex(&key));modern.ranked_key=Some(key);
        for side in &mut modern.sides {side.summary.mode=tbg_core::protocol::MatchMode::Ranked;side.summary.origin=tbg_core::protocol::MatchOrigin::PublicQueue;}
        player_history::insert(conn,&modern).unwrap();
        assert!(!player_history::backfill::batch(conn).unwrap());
        let detail=player_history::detail(conn,&subject,&tbg_server::ranked::hex(&modern.sides[0].entry_id)).unwrap();assert_eq!(detail["quality"],"complete");
    }).await.unwrap();
}
