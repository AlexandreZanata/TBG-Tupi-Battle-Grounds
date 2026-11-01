//! HTTPS transport for the TUF client (P66 §4): blocking `ureq` on a
//! worker thread, streamed to tough in chunks. No redirects (a mirror
//! cannot send the client elsewhere), HTTPS only except loopback test
//! servers, and `file://` for local repositories. tough enforces every
//! length and digest; this layer only moves bytes.

use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures::channel::mpsc;
use futures::SinkExt;
use tough::{FilesystemTransport, Transport, TransportError, TransportErrorKind, TransportStream};
use url::Url;

/// Bytes per streamed chunk.
const CHUNK: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct HttpsTransport {
    agent: ureq::Agent,
}

impl Default for HttpsTransport {
    fn default() -> Self {
        let config = ureq::Agent::config_builder()
            .max_redirects(0)
            .http_status_as_error(true)
            .timeout_connect(Some(Duration::from_secs(10)))
            .timeout_recv_response(Some(Duration::from_secs(30)))
            .build();
        HttpsTransport {
            agent: ureq::Agent::new_with_config(config),
        }
    }
}

fn loopback(url: &Url) -> bool {
    matches!(
        url.host_str(),
        Some("127.0.0.1") | Some("localhost") | Some("[::1]")
    )
}

#[async_trait]
impl Transport for HttpsTransport {
    async fn fetch(&self, url: Url) -> Result<TransportStream, TransportError> {
        match url.scheme() {
            "file" => FilesystemTransport.fetch(url).await,
            "https" => Ok(self.stream(url)),
            "http" if loopback(&url) => Ok(self.stream(url)),
            _ => Err(TransportError::new(
                TransportErrorKind::UnsupportedUrlScheme,
                url,
            )),
        }
    }
}

impl HttpsTransport {
    /// Start the request on a blocking thread and return its chunk stream;
    /// dropping the stream stops the reader at its next chunk.
    fn stream(&self, url: Url) -> TransportStream {
        let (mut tx, rx) = mpsc::channel::<Result<Bytes, TransportError>>(4);
        let agent = self.agent.clone();
        std::thread::spawn(move || {
            let send =
                |tx: &mut mpsc::Sender<_>, item| futures::executor::block_on(tx.send(item)).is_ok();
            let response = match agent.get(url.as_str()).call() {
                Ok(r) => r,
                Err(e) => {
                    let kind = match e {
                        ureq::Error::StatusCode(404) => TransportErrorKind::FileNotFound,
                        _ => TransportErrorKind::Other,
                    };
                    send(
                        &mut tx,
                        Err(TransportError::new_with_cause(kind, url.as_str(), e)),
                    );
                    return;
                }
            };
            let mut reader = response
                .into_body()
                .into_with_config()
                .limit(u64::MAX)
                .reader();
            let mut buf = vec![0u8; CHUNK];
            loop {
                match std::io::Read::read(&mut reader, &mut buf) {
                    Ok(0) => return,
                    Ok(n) => {
                        if !send(&mut tx, Ok(Bytes::copy_from_slice(&buf[..n]))) {
                            return;
                        }
                    }
                    Err(e) => {
                        let err = TransportError::new_with_cause(
                            TransportErrorKind::Other,
                            url.as_str(),
                            e,
                        );
                        send(&mut tx, Err(err));
                        return;
                    }
                }
            }
        });
        Box::pin(rx)
    }
}
