//! BR-RP01–RP07, S02/S07/S09/S12/S17: report targets/evidence come from the server.
#[path = "common/p75.rs"]
mod common;
use common::*;
use serde_json::json;
#[tokio::test]
async fn rp04_only_participants_report_a_match_and_evidence_is_frozen_once() {
    let s = Server::new().await;
    let a = s.register("Reporter", "reporter@example.invalid").await;
    let b = s.register("Target", "target@example.invalid").await;
    let stranger = s.register("Stranger", "stranger@example.invalid").await;
    s.sql("INSERT INTO match_social_records VALUES(zeroblob(16),1000000,1604800,'casual',1,2,'Reporter','Target','{\"winner\":0}','[]','[{\"from\":1,\"text\":\"masked ****\"}]');");
    let input = json!({"context":"match","result_id":"0".repeat(32),"reason":"abusive_chat","comment":"Review https://example.invalid as text"});
    assert_eq!(
        s.request(
            "POST",
            "/api/me/reports",
            input.clone(),
            stranger["access_token"].as_str()
        )
        .await
        .status,
        422
    );
    let (one, two) = tokio::join!(
        s.request(
            "POST",
            "/api/me/reports",
            input.clone(),
            a["access_token"].as_str()
        ),
        s.request(
            "POST",
            "/api/me/reports",
            input.clone(),
            a["access_token"].as_str()
        )
    );
    let mut statuses = [one.status, two.status];
    statuses.sort();
    assert_eq!(statuses, [201, 409], "S12 one effect under race");
    assert_eq!(s.number("SELECT COUNT(*) FROM reports"), 1);
    s.sql("UPDATE match_social_records SET transcript_json='[]';");
    let db = rusqlite::Connection::open(&s.db).unwrap();
    let evidence: String = db
        .query_row("SELECT evidence_json FROM reports", [], |r| r.get(0))
        .unwrap();
    assert!(evidence.contains("masked ****"), "RP03 frozen evidence");
    for route in [
        "/api/players/Target",
        "/api/players/Reporter",
        "/api/online",
    ] {
        let r = s
            .request("GET", route, json!(null), b["access_token"].as_str())
            .await;
        assert!(!r.body.contains("reporter_id"));
        assert!(!r.body.contains("Review https"), "S17 report privacy");
    }
    let forged =
        json!({"context":"match","result_id":"0".repeat(32),"reason":"other","target_id":3});
    assert_eq!(
        s.request(
            "POST",
            "/api/me/reports",
            forged,
            a["access_token"].as_str()
        )
        .await
        .status,
        400
    );
}
#[tokio::test]
async fn rp05_profile_reports_are_limited_and_guests_cannot_file() {
    let s = Server::new().await;
    let a = s.register("Reporter", "reporter@example.invalid").await;
    for i in 0..11 {
        s.register(&format!("Target{i}"), &format!("target{i}@example.invalid"))
            .await;
    }
    for i in 0..10 {
        let r = s
            .request(
                "POST",
                "/api/me/reports",
                json!({"context":"profile","target":format!("Target{i}"),"reason":"other"}),
                a["access_token"].as_str(),
            )
            .await;
        assert_eq!(r.status, 201);
    }
    assert_eq!(
        s.request(
            "POST",
            "/api/me/reports",
            json!({"context":"profile","target":"Target10","reason":"other"}),
            a["access_token"].as_str()
        )
        .await
        .status,
        429
    );
    assert_eq!(
        s.request(
            "POST",
            "/api/me/reports",
            json!({"context":"profile","target":"Target1","reason":"other"}),
            None
        )
        .await
        .status,
        403
    );
}
