//! Public, offline build identity for attested deployment artifacts; no config or secrets.

pub fn get() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "source_commit": option_env!("TBG_SOURCE_COMMIT").unwrap_or("development"),
        "protocol": tbg_core::protocol::PROTOCOL_VERSION,
        "contract": tbg_core::contract::MatchContract::current().id(),
        "database_schema": crate::store::DATABASE_SCHEMA_VERSION,
        "target": env!("TBG_BUILD_TARGET"),
    })
}
