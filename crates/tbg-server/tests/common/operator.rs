//! Owner claim and sign-in through the real dedicated loopback listener.
#![allow(dead_code)]
use super::common::*;
use serde_json::{json, Value};
pub struct Operator {
    pub url: String,
    pub cookie: String,
    pub csrf: String,
    pub key: String,
}
impl Operator {
    pub async fn request(&self, method: &str, path: &str, value: Value) -> Reply {
        request(
            &self.url,
            method,
            &format!("/admin-api/v1/{path}"),
            value,
            vec![
                ("cookie".into(), self.cookie.clone()),
                ("x-tbg-csrf".into(), self.csrf.clone()),
                ("origin".into(), "https://admin.tupibg.com".into()),
            ],
        )
        .await
    }
}
pub async fn server() -> Server {
    Server::configured(|c| {
        c.admin.enabled = true;
        c.admin.addr = "127.0.0.1:0".into();
        c.bootstrap_email = tbg_server::mail::settings::Private("owner@example.invalid".into());
    })
    .await
}
pub async fn preauth(s: &Server) -> Operator {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    let address = loop {
        if let Some(address) = s.shared.admin_address.get() {
            break address;
        }
        assert!(tokio::time::Instant::now() < deadline);
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    };
    let url = format!("http://{address}");
    let reply = request(&url, "GET", "/admin-api/v1/session", json!(null), vec![]).await;
    assert_eq!(reply.status, 401);
    Operator {
        url,
        cookie: reply
            .headers
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .into(),
        csrf: reply.json()["csrf"].as_str().unwrap().into(),
        key: String::new(),
    }
}
pub async fn claim(s: &Server) -> Operator {
    let mut op = preauth(s).await;
    let code = s.mail("operator_claim").await.code.unwrap();
    let reply = op
        .request(
            "POST",
            "claim/complete",
            json!({"email":"owner@example.invalid","code":code}),
        )
        .await;
    assert_eq!(reply.status, 200, "{}", reply.body);
    op.key = reply.json()["admin_key"].as_str().unwrap().into();
    sign_in(s, op).await
}
pub async fn sign_in(s: &Server, mut op: Operator) -> Operator {
    let started = op
        .request("POST", "session/start", json!({"admin_key":op.key}))
        .await;
    assert_eq!(started.status, 202);
    let code = s.mail("operator_login").await.code.unwrap();
    let reply = op
        .request(
            "POST",
            "session/verify",
            json!({"attempt":started.json()["attempt"],"code":code}),
        )
        .await;
    assert_eq!(reply.status, 200, "{}", reply.body);
    let cookie = reply.headers.get("set-cookie").unwrap().to_str().unwrap();
    for flag in [
        "__Host-tbg_admin=",
        "Secure",
        "HttpOnly",
        "SameSite=Strict",
        "Path=/",
    ] {
        assert!(cookie.contains(flag));
    }
    op.cookie = cookie.split(';').next().unwrap().into();
    op.csrf = reply.json()["csrf"].as_str().unwrap().into();
    op
}
