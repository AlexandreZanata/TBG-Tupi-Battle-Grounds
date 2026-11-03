//! Secret-free mail configuration and one-time systemd credential loading.

#[derive(Clone, Default)]
pub struct Private<T>(pub T);
impl<T> std::fmt::Debug for Private<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[private]")
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub from: String,
    pub host: String,
    pub port: u16,
    pub security: String,
    pub username: Private<String>,
    pub credential: String,
    pub game_url: String,
    pub admin_url: String,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            from: String::new(),
            host: String::new(),
            port: 587,
            security: "starttls".into(),
            username: Private::default(),
            credential: "smtp_password".into(),
            game_url: "https://app.tupibg.com".into(),
            admin_url: "https://admin.tupibg.com".into(),
        }
    }
}
impl Settings {
    pub fn from_env() -> Self {
        let mut s = Self::default();
        let env = |key, default| std::env::var(key).unwrap_or(default);
        s.from = env("TBG_MAIL_FROM", s.from);
        s.host = env("TBG_SMTP_HOST", s.host);
        s.port = std::env::var("TBG_SMTP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(587);
        s.security = env("TBG_SMTP_SECURITY", s.security);
        s.username = Private(env("TBG_SMTP_USERNAME", String::new()));
        s.credential = env("TBG_SMTP_PASSWORD_CREDENTIAL", s.credential);
        s.game_url = env("TBG_PUBLIC_GAME_URL", s.game_url);
        s.admin_url = env("TBG_PUBLIC_ADMIN_URL", s.admin_url);
        s
    }
    pub(crate) fn load(&self) -> Option<Credentials> {
        if self.host.is_empty()
            || self.from.is_empty()
            || self.username.0.is_empty()
            || !matches!(self.security.as_str(), "starttls" | "tls")
        {
            return None;
        }
        if self.credential.is_empty()
            || !self
                .credential
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        {
            return None;
        }
        let directory = std::env::var_os("CREDENTIALS_DIRECTORY")?;
        let path = std::path::PathBuf::from(directory).join(&self.credential);
        let file = std::fs::File::open(path).ok()?;
        if file.metadata().ok()?.len() > 4096 {
            return None;
        }
        use std::io::Read;
        let mut password = String::new();
        file.take(4097).read_to_string(&mut password).ok()?;
        if password.is_empty() || password.len() > 4096 {
            return None;
        }
        Some(Credentials {
            username: self.username.0.clone(),
            password,
        })
    }
}
// Deliberately no Debug: this is the only struct holding the SMTP password.
pub(crate) struct Credentials {
    pub username: String,
    pub password: String,
}
