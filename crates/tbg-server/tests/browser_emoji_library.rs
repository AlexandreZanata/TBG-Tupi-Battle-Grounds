//! BR-EM01: real browser-style account reads preserve saved slots and all authentication fences.
#[path = "common/p75.rs"]
mod common;
use common::{Reply, Server};
use serde_json::{json, Value};

const ORIGIN: &str = "https://app.example";

async fn browser(
    s: &Server,
    method: &str,
    access: Option<&str>,
    origin: Option<&str>,
    value: Value,
) -> Reply {
    let url = format!(
        "{}/api/me/emoji{}",
        s.url,
        if method == "PUT" { "/slots" } else { "" }
    );
    let method = method.to_owned();
    let access = access.map(str::to_owned);
    let origin = origin.map(str::to_owned);
    tokio::task::spawn_blocking(move || {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .build()
            .new_agent();
        let mut request = ureq::http::Request::builder()
            .uri(url)
            .method(method.as_str())
            .header("content-type", "application/json")
            .header("x-tbg-auth", "1")
            .header("sec-fetch-site", "same-origin")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-dest", "empty")
            .header("referer", format!("{ORIGIN}/?lang=en"));
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        if let Some(access) = access {
            request = request.header("authorization", format!("Bearer {access}"));
        }
        let body = if method == "GET" {
            Vec::new()
        } else {
            serde_json::to_vec(&value).unwrap()
        };
        let mut reply = agent.run(request.body(body).unwrap()).unwrap();
        Reply {
            status: reply.status().as_u16(),
            headers: reply.headers().clone(),
            body: reply.body_mut().read_to_string().unwrap(),
        }
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn standard_emoji_survives_browser_save_and_originless_library_refresh() {
    let s = Server::configured(|c| c.auth_origins = vec![ORIGIN.into()]).await;
    let grant = s.register("EmojiBrowser", "emoji@example.invalid").await;
    let access = grant["access_token"].as_str();
    let read = browser(&s, "GET", access, None, Value::Null).await;
    assert_eq!(read.status, 200, "{}", read.body);
    assert_eq!(
        read.json()["slots"],
        json!([{"standard":0},{"standard":1},{"standard":2}])
    );
    let choice = json!({"slots":[{"standard":8},{"standard":9},{"standard":15}]});
    let save = browser(&s, "PUT", access, Some(ORIGIN), choice.clone()).await;
    assert_eq!(save.status, 200, "{}", save.body);
    let refreshed = browser(&s, "GET", access, None, Value::Null).await;
    assert_eq!(refreshed.status, 200, "{}", refreshed.body);
    assert_eq!(refreshed.json()["slots"], choice["slots"]);
    assert!(refreshed.headers["cache-control"]
        .to_str()
        .unwrap()
        .contains("no-store"));
    assert_eq!(
        browser(&s, "GET", None, None, Value::Null).await.status,
        403
    );
    assert_eq!(
        browser(
            &s,
            "GET",
            access,
            Some("https://other.example"),
            Value::Null
        )
        .await
        .status,
        403
    );
    assert_eq!(browser(&s, "PUT", access, None, choice).await.status, 403);
    let unchanged = browser(&s, "GET", access, None, Value::Null).await;
    assert_eq!(unchanged.json()["slots"], refreshed.json()["slots"]);
}
