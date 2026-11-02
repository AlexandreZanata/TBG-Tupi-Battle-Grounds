//! Complete console route inventory and default-deny capability policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability {
    Public,
    Any,
    Review,
    Restrict,
    Ban,
    Owner,
}
pub struct Route {
    pub method: &'static str,
    pub path: &'static str,
    pub capability: Capability,
    pub sensitive: bool,
}
macro_rules! route {
    ($m:literal,$p:literal,$c:ident,$s:literal) => {
        Route {
            method: $m,
            path: $p,
            capability: Capability::$c,
            sensitive: $s,
        }
    };
}
pub const ROUTES: &[Route] = &[
    route!("POST", "claim/request", Public, false),
    route!("POST", "claim/complete", Public, false),
    route!("POST", "session/start", Public, false),
    route!("POST", "session/verify", Public, false),
    route!("GET", "session", Any, false),
    route!("POST", "session/reauth", Any, false),
    route!("POST", "session/logout", Any, false),
    route!("GET", "overview", Any, false),
    route!("GET", "accounts", Any, false),
    route!("GET", "accounts/{name}", Any, false),
    route!("POST", "accounts/{name}/ban", Ban, true),
    route!("POST", "accounts/{name}/unban", Owner, true),
    route!("POST", "accounts/{name}/chat", Restrict, false),
    route!("POST", "accounts/{name}/emoji", Restrict, false),
    route!("POST", "accounts/{name}/revoke-sessions", Restrict, false),
    route!("POST", "accounts/{name}/send-recovery", Owner, true),
    route!("GET", "reports", Any, false),
    route!("GET", "reports/{id}", Any, false),
    route!("POST", "reports/{id}/resolve", Any, false),
    route!("GET", "emoji", Any, false),
    route!("GET", "emoji/{id}", Any, false),
    route!("GET", "emoji/{id}/sheet.png", Any, false),
    route!("POST", "emoji/{id}/approve", Review, false),
    route!("POST", "emoji/{id}/reject", Review, false),
    route!("POST", "emoji/{id}/revoke", Review, false),
    route!("GET", "chat/terms", Any, false),
    route!("POST", "chat/terms", Review, false),
    route!("DELETE", "chat/terms/{term}", Review, false),
    route!("GET", "settings", Any, false),
    route!("PUT", "settings/{key}", Owner, true),
    route!("GET", "email-blocks", Owner, false),
    route!("POST", "email-blocks", Owner, true),
    route!("DELETE", "email-blocks/{email}", Owner, true),
    route!("GET", "operators", Owner, false),
    route!("POST", "operators", Owner, true),
    route!("POST", "operators/{id}/disable", Owner, true),
    route!("POST", "operators/{id}/enable", Owner, true),
    route!("GET", "audit", Owner, false),
    route!("GET", "system", Any, false),
];
pub fn find(method: &str, path: &str) -> Option<&'static Route> {
    ROUTES
        .iter()
        .find(|r| r.method == method && matches(r.path, path))
}
pub fn matches(pattern: &str, path: &str) -> bool {
    let a: Vec<_> = pattern.split('/').collect();
    let b: Vec<_> = path.split('/').collect();
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(a, b)| !b.is_empty() && (a.starts_with('{') || *a == b))
}
pub fn allowed(role: &str, capability: Capability) -> bool {
    match capability {
        Capability::Public => true,
        Capability::Owner => role == "owner",
        _ => matches!(role, "owner" | "moderator"),
    }
}
