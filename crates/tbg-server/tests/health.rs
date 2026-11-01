//! P71 deploy safety: `/health` fails when a configured database cannot be opened
//! or migrated, and stays `ok` for the in-memory mode that never had one.

use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use tbg_server::Config;

async fn health(db: Option<std::path::PathBuf>) -> (u16, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(tbg_server::run(
        Config {
            db,
            ..Config::default()
        },
        listener,
    ));
    for _ in 0..100 {
        if let Ok(mut stream) = TcpStream::connect(addr).await {
            stream
                .write_all(b"GET /health HTTP/1.1\r\nHost: t\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
            let mut raw = String::new();
            stream.read_to_string(&mut raw).await.unwrap();
            let status = raw[9..12].parse().unwrap();
            let body = raw.split("\r\n\r\n").nth(1).unwrap_or_default().to_string();
            return (status, body);
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("server never listened");
}

#[tokio::test]
async fn p71_health_fails_when_the_configured_database_cannot_open() {
    // A directory is never a SQLite file: open and migration fail.
    let dir = std::env::temp_dir();
    let (status, body) = health(Some(dir)).await;
    assert_eq!(status, 503, "{body}");
    assert!(body.contains("store unavailable"));
}

#[tokio::test]
async fn p71_health_is_ok_without_a_configured_database() {
    let (status, body) = health(None).await;
    assert_eq!(status, 200);
    assert!(body.contains("ok"));
}
