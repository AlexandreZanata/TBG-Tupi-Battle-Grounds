//! Dedicated loopback console configuration; never shared with the public router.
#[derive(Clone, Debug)]
pub struct Settings {
    pub enabled: bool,
    pub addr: String,
    pub origin: String,
    pub allow_ips: Vec<String>,
    pub login_code: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            addr: "127.0.0.1:8090".into(),
            origin: "https://admin.tupibg.com".into(),
            allow_ips: Vec::new(),
            login_code: true,
        }
    }
}
impl Settings {
    pub fn from_env() -> Self {
        let d = Self::default();
        let slot = std::env::var("TBG_SLOT").unwrap_or_default();
        let addr = if slot == "a" || slot == "b" {
            let key = if slot == "a" {
                "TBG_ADMIN_PORT_A"
            } else {
                "TBG_ADMIN_PORT_B"
            };
            format!(
                "127.0.0.1:{}",
                std::env::var(key).unwrap_or(if slot == "a" { "8090" } else { "8092" }.into())
            )
        } else {
            std::env::var("TBG_ADMIN_ADDR").unwrap_or(d.addr)
        };
        Self {
            enabled: std::env::var("TBG_ADMIN").as_deref() == Ok("on"),
            addr,
            origin: std::env::var("TBG_ADMIN_ORIGIN").unwrap_or(d.origin),
            allow_ips: std::env::var("TBG_ADMIN_ALLOW_IPS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().into())
                .collect(),
            login_code: std::env::var("TBG_ADMIN_LOGIN_CODE").as_deref() != Ok("off"),
        }
    }
}
