//! `tbg-server` binary: read env config, bind, serve.

use tbg_server::Config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let config = Config::from_env();
    let listener = tokio::net::TcpListener::bind(&config.addr)
        .await
        .unwrap_or_else(|e| panic!("tbg-server: cannot bind {}: {e}", config.addr));
    tracing::info!(addr = %config.addr, ?config, "tbg-server listening");
    if let Err(e) = tbg_server::run(config, listener).await {
        tracing::error!("server stopped: {e}");
    }
}
