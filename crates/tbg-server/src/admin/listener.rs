//! Separate loopback-only router; public game routing never mounts operator endpoints.
use super::{policy, request, session};
use crate::{
    store::operators::{self, Error},
    AppState,
};
use axum::{
    extract::{ConnectInfo, Request, State},
    http::Method,
    response::{IntoResponse, Response},
    routing::any,
    Json, Router,
};
use serde_json::json;
use std::{net::SocketAddr, time::Duration};
pub(crate) struct Task(tokio::task::JoinHandle<()>);
impl Drop for Task {
    fn drop(&mut self) {
        self.0.abort();
    }
}
pub(crate) async fn start(
    app: AppState,
    address: &std::sync::OnceLock<SocketAddr>,
) -> std::io::Result<Option<Task>> {
    let settings = &app.config.admin;
    if !settings.enabled {
        return Ok(None);
    }
    let addr: SocketAddr = settings
        .addr
        .parse()
        .map_err(|_| std::io::Error::other("invalid admin address"))?;
    if !addr.ip().is_loopback() || !crate::auth::valid_origin(&settings.origin) {
        return Err(std::io::Error::other(
            "admin listener must be loopback with a canonical configured origin",
        ));
    }
    for entry in &settings.allow_ips {
        if !request::valid_cidr(entry) {
            return Err(std::io::Error::other("invalid admin allow-list"));
        }
    }
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let _ = address.set(listener.local_addr()?);
    let router = Router::new()
        .route("/admin-api/v1/{*path}", any(handler))
        .with_state(app);
    let task = tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await;
    });
    Ok(Some(Task(task)))
}
async fn handler(
    State(app): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request,
) -> Response {
    let id = super::key::random::<16>().unwrap_or_else(|_| "0".repeat(32));
    let request_id = id.clone();
    let mut response =
        match tokio::time::timeout(Duration::from_secs(5), execute(app, peer, req, request_id))
            .await
        {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => session::error(e),
            Err(_) => session::error(Error::Unavailable),
        };
    for (key, value) in [
        ("cache-control", "no-store"),
        ("x-content-type-options", "nosniff"),
        ("x-request-id", &id),
    ] {
        if let Ok(value) = value.parse() {
            response.headers_mut().insert(
                axum::http::HeaderName::from_bytes(key.as_bytes()).unwrap(),
                value,
            );
        }
    }
    response
}
async fn execute(
    app: AppState,
    peer: SocketAddr,
    req: Request,
    id: String,
) -> Result<Response, Error> {
    let ip = crate::ws::client_ip(peer, req.headers());
    if !request::allowed_ip(ip, &app.config.admin.allow_ips) {
        return Err(Error::Forbidden);
    }
    let query = req.uri().query().unwrap_or("").to_owned();
    let method = req.method().as_str().to_owned();
    let system = json!({"mail_configured":app.mail_configured,"slot":app.release.slot(),"build":app.config.server_build,"online":app.metrics.connections.load(std::sync::atomic::Ordering::Relaxed),"live_matches":app.metrics.matches_running.load(std::sync::atomic::Ordering::Relaxed),"feed_cursor":app.config.moderation.cursor()});
    let path = req
        .uri()
        .path()
        .strip_prefix("/admin-api/v1/")
        .ok_or(Error::Invalid)?
        .to_owned();
    let route = policy::find(req.method().as_str(), &path).ok_or(Error::Invalid)?;
    request::mutation(req.headers(), req.method(), &app.config.admin.origin)?;
    let token = request::cookie(req.headers(), "__Host-tbg_admin").unwrap_or_default();
    let csrf = req
        .headers()
        .get("x-tbg-csrf")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let mutation = req.method() != Method::GET;
    let anonymous = request::cookie(req.headers(), "__Host-tbg_admin_pre");
    if route.capability == policy::Capability::Public && mutation {
        let expected = anonymous.as_ref().ok_or(Error::Forbidden)?;
        if expected.len() != 64
            || !crate::identity::codes::equal(
                &super::key::hash(&expected),
                &super::key::hash(csrf.as_deref().ok_or(Error::Forbidden)?),
            )
        {
            return Err(Error::Forbidden);
        }
    }
    let attempt = anonymous
        .map(|nonce| {
            super::key::hash(&nonce)
                .iter()
                .take(16)
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        })
        .unwrap_or_default();
    let permit = app
        .history_reads
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::Unavailable)?;
    let bytes = axum::body::to_bytes(req.into_body(), 8192)
        .await
        .map_err(|_| Error::Invalid)?;
    let store = app.store.as_ref().ok_or(Error::Unavailable)?;
    let now = app.auth.now().map_err(|_| Error::Unavailable)?;
    let second = app.config.admin.login_code;
    let ip = ip.to_string();
    store.release(move |db| {
        let _permit=permit;
        let principal=if route.capability!=policy::Capability::Public {
            match operators::sessions::authenticate(db,&token,mutation.then_some(csrf.as_deref().unwrap_or("")),now) {
                Ok(p)=>{if !policy::allowed(&p.role,route.capability){return Err(Error::Forbidden);}if route.sensitive&&now-p.reauth>=300{return Err(Error::Reauth);}Some(p)},
                Err(Error::Unauthorized) if path=="session"=>{
                    // Before a session exists, a host-only nonce protects claim/sign-in against login CSRF.
                    let nonce=super::key::random::<32>().map_err(|_|Error::Unavailable)?;
                    let mut r=(axum::http::StatusCode::UNAUTHORIZED,Json(json!({"error":"auth_required","csrf":nonce}))).into_response();
                    r.headers_mut().insert("set-cookie",format!("__Host-tbg_admin_pre={nonce}; Path=/; Secure; HttpOnly; SameSite=Strict").parse().map_err(|_|Error::Unavailable)?);return Ok(r);
                },Err(e)=>return Err(e)
            }
        }else{None};
        if path.starts_with("session")||path.starts_with("claim/"){session::execute(db,&path,&bytes,&token,&ip,&id,&attempt,now,second)}else{
            let principal=principal.as_ref().ok_or(Error::Unauthorized)?;
            if path.starts_with("emoji/")&&path.ends_with("/sheet.png") {
                let asset=path.split('/').nth(1).and_then(|v|v.parse::<i64>().ok()).ok_or(Error::Invalid)?;
                let bytes=operators::areas::private_sheet(db,asset)?;
                return Ok(([("content-type","image/png")],bytes).into_response());
            }
            let context=operators::areas::Context{principal,request:&id,now,query:&query,system};
            Ok(Json(operators::areas::authorized(db,&path,&method,&bytes,&context,&token,csrf.as_deref())?).into_response())
        }
    }).await.ok_or(Error::Unavailable)?
}
