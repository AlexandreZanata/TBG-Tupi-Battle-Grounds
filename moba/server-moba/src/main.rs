//! MOBA room server entry: `tbg-moba-server [--bind ADDR]` (default 127.0.0.1:7878).

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let args: Vec<String> = std::env::args().collect();
    let bind = args
        .iter()
        .position(|a| a == "--bind")
        .and_then(|i| args.get(i + 1).cloned())
        .or_else(|| std::env::var("TBG_MOBA_BIND").ok())
        .unwrap_or_else(|| "127.0.0.1:7878".into());
    let mut seed = [0u8; 8];
    getrandom::fill(&mut seed).map_err(|e| std::io::Error::other(e.to_string()))?;
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, protocol = tbg_moba_core::protocol::MOBA_PROTOCOL_VERSION, "MOBA room listening on /moba/ws");
    let shutdown = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    tokio::select! {
        result = tbg_moba_server::serve(listener, u64::from_le_bytes(seed)) => result,
        _ = shutdown => Ok(()),
    }
}
