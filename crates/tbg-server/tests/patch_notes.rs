//! Public patch history: real HTTP wiring, bounded output, caching and read-only methods.
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn request(host: &str, method: &str, path: &str) -> (u16, String, Vec<u8>) {
    let mut stream = TcpStream::connect(host).await.unwrap();
    stream
        .write_all(format!("{method} {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let mut raw = Vec::new();
    tokio::time::timeout(
        std::time::Duration::from_secs(5),
        stream.read_to_end(&mut raw),
    )
    .await
    .unwrap()
    .unwrap();
    let end = raw.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
    let header = String::from_utf8(raw[..end].to_vec()).unwrap();
    let status = header.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, header, raw[end..].to_vec())
}

#[tokio::test]
async fn history_is_public_and_cached_but_cannot_write() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let host = listener.local_addr().unwrap().to_string();
    tokio::spawn(tbg_server::run(
        tbg_server::Config {
            db: None,
            ..Default::default()
        },
        listener,
    ));
    let (status, header, body) = request(&host, "GET", "/api/patch-notes").await;
    assert_eq!(status, 200);
    assert!(header
        .to_lowercase()
        .contains("cache-control: public, max-age=60"));
    assert!(body.len() <= 262144);
    let feed: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(feed["schema"], 1);
    assert!(feed.get("accounts").is_none());
    for note in feed["notes"].as_array().unwrap() {
        let id = note["id"].as_str().unwrap();
        let (status, _, body) = request(&host, "GET", &format!("/api/patch-notes/{id}")).await;
        assert_eq!(status, 200);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            *note
        );
    }
    assert_eq!(
        request(&host, "GET", "/api/patch-notes/unknown").await.0,
        404
    );
    assert_eq!(request(&host, "POST", "/api/patch-notes").await.0, 405);
    assert_eq!(
        request(&host, "POST", "/api/patch-notes/native-3-competitive-audit")
            .await
            .0,
        405
    );
}
