//! SMTP submission and bounded synthetic capture, without secret-bearing Debug output.

use super::settings::{Credentials, Settings};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct MailCapture(Arc<Mutex<VecDeque<CapturedMail>>>);
impl std::fmt::Debug for MailCapture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MailCapture([private])")
    }
}
pub struct CapturedMail {
    pub to: String,
    pub purpose: String,
    pub code: Option<String>,
    pub body: String,
}
impl MailCapture {
    pub fn take(&self) -> Option<CapturedMail> {
        self.0.lock().ok()?.pop_front()
    }
    pub fn len(&self) -> usize {
        self.0.lock().map(|v| v.len()).unwrap_or(0)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
pub enum Transport {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    Capture(MailCapture),
}
impl Transport {
    pub(crate) fn smtp(settings: &Settings, credentials: Credentials) -> Option<Self> {
        let builder = if settings.security == "tls" {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&settings.host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&settings.host)
        }
        .ok()?;
        Some(Self::Smtp(
            builder
                .port(settings.port)
                .credentials(lettre::transport::smtp::authentication::Credentials::new(
                    credentials.username,
                    credentials.password,
                ))
                .timeout(Some(std::time::Duration::from_secs(10)))
                .build(),
        ))
    }
    pub(crate) async fn send(
        &self,
        settings: &Settings,
        mail: CapturedMail,
    ) -> Result<(), &'static str> {
        match self {
            Self::Capture(c) => {
                let mut q = c.0.lock().map_err(|_| "capture_unavailable")?;
                if q.len() >= 64 {
                    return Err("capture_full");
                }
                q.push_back(mail);
                Ok(())
            }
            Self::Smtp(s) => {
                let from = settings.from.parse().map_err(|_| "mail_sender")?;
                let to = mail.to.parse().map_err(|_| "mail_recipient")?;
                let message = Message::builder()
                    .from(from)
                    .to(to)
                    .subject("Tupi Battle Grounds")
                    .raw_header(lettre::message::header::HeaderValue::new(
                        lettre::message::header::HeaderName::new_from_ascii_str("Auto-Submitted"),
                        "auto-generated".into(),
                    ))
                    .body(mail.body)
                    .map_err(|_| "mail_message")?;
                match tokio::time::timeout(std::time::Duration::from_secs(10), s.send(message))
                    .await
                {
                    Ok(Ok(_)) => Ok(()),
                    Ok(Err(e)) if e.is_timeout() => Err("smtp_timeout"),
                    Ok(Err(_)) => Err("smtp_failed"),
                    Err(_) => Err("smtp_timeout"),
                }
            }
        }
    }
}
