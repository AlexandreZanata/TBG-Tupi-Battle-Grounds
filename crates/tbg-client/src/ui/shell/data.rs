//! Validated read-only lobby projections from public HTTP endpoints.

use crate::platform::http::Request;
use bevy::prelude::*;
use serde_json::Value;

#[derive(Clone, Debug, Default)]
pub struct Live {
    pub players: u64,
    pub battle: u64,
    pub watching: u64,
    pub matches: Vec<Battle>,
}
#[derive(Clone, Debug)]
pub struct Battle {
    pub id: u64,
    pub mode: String,
    pub arena: String,
    pub names: [String; 2],
    pub hp: [u64; 2],
    pub elapsed: u64,
    pub spectators: u64,
}
#[derive(Clone, Debug)]
pub struct Online {
    pub name: String,
    pub rating: Option<u64>,
    pub status: String,
}
#[derive(Resource, Default)]
pub struct Data {
    pub live: Option<Live>,
    pub online: Vec<Online>,
    pub online_total: u64,
    pub stats: Option<Value>,
    pub unavailable: bool,
    pub update: Option<String>,
    version_request: Option<Request>,
    version_checked: bool,
    live_request: Option<Request>,
    online_request: Option<Request>,
    stats_request: Option<Request>,
    next: f64,
    player: String,
}

fn name(value: &Value) -> Option<String> {
    let s = value.as_str()?;
    (!s.is_empty() && s.len() <= 64).then(|| s.into())
}

pub fn parse_live(value: &Value) -> Option<Live> {
    let online = value.get("online")?;
    for key in ["players", "accounts", "in_battle", "watching"] {
        online.get(key)?.as_u64()?;
    }
    let mut matches: Vec<_> = value
        .get("matches")?
        .as_array()?
        .iter()
        .take(50)
        .filter_map(parse_battle)
        .collect();
    if let Some(id) = value.get("featured").and_then(Value::as_str) {
        matches.sort_by_key(|m| format!("{:016x}", m.id) != id);
    }
    Some(Live {
        players: online["players"].as_u64()?,
        battle: online["in_battle"].as_u64()?,
        watching: online["watching"].as_u64()?,
        matches,
    })
}
fn parse_battle(v: &Value) -> Option<Battle> {
    let raw_id = v.get("id")?.as_str()?;
    if raw_id.len() != 16 || !raw_id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mode = v.get("mode")?.as_str()?;
    if !["casual", "ranked"].contains(&mode) {
        return None;
    }
    let players = v.get("players")?.as_array()?;
    let hp = v.get("hp_bp")?.as_array()?;
    if players.len() != 2 || hp.len() != 2 {
        return None;
    }
    for p in players {
        if !p["rating"].is_null() {
            p["rating"].as_u64()?;
        }
    }
    Some(Battle {
        id: u64::from_str_radix(raw_id, 16).ok()?,
        mode: mode.into(),
        arena: v["arena"].as_str().unwrap_or_default().into(),
        names: [name(&players[0]["name"])?, name(&players[1]["name"])?],
        hp: [hp[0].as_u64()?.min(10_000), hp[1].as_u64()?.min(10_000)],
        elapsed: v["elapsed_s"].as_u64().unwrap_or(0),
        spectators: v["spectators"].as_u64().unwrap_or(0),
    })
}
pub fn parse_online(v: &Value) -> Option<(u64, Vec<Online>)> {
    let total = v.get("total")?.as_u64()?;
    Some((
        total,
        v.get("rows")?
            .as_array()?
            .iter()
            .take(100)
            .filter_map(|row| {
                let rating = if row["rating"].is_null() {
                    None
                } else {
                    Some(row["rating"].as_u64()?)
                };
                Some(Online {
                    name: name(&row["name"])?,
                    rating,
                    status: row["status"].as_str().unwrap_or_default().into(),
                })
            })
            .collect(),
    ))
}

pub fn poll(
    time: Res<Time>,
    account: Res<super::Account>,
    mut data: ResMut<Data>,
    mut shell: ResMut<super::Shell>,
) {
    if !crate::config::bevy_ui() {
        return;
    }
    if !data.version_checked {
        data.version_checked = true;
        data.version_request = Some(Request::get("/api/version"));
    }
    if let Some(result) = data.version_request.as_ref().and_then(Request::take) {
        data.version_request = None;
        data.update = result.ok().and_then(|v| {
            newer_client(
                &v,
                env!("CARGO_PKG_VERSION"),
                tbg_core::protocol::PROTOCOL_VERSION,
            )
        });
        shell.dirty = true;
    }
    let now = time.elapsed_secs_f64();
    if now >= data.next && data.live_request.is_none() {
        data.next = now + 5.;
        data.live_request = Some(Request::get("/api/live"));
        data.online_request = Some(Request::get("/api/online"));
    }
    let player = account.profile.as_ref().map_or("", |p| p.name.as_str());
    if player != data.player {
        data.player = player.into();
        data.stats = None;
        data.stats_request =
            (!player.is_empty()).then(|| Request::get(&format!("/api/players/{player}")));
    }
    if let Some(result) = data.live_request.as_ref().and_then(Request::take) {
        data.live_request = None;
        if let Some(live) = result.ok().and_then(|v| parse_live(&v)) {
            data.live = Some(live);
            data.unavailable = false;
        } else {
            data.unavailable = true;
        }
        shell.dirty = true;
    }
    if let Some(result) = data.online_request.as_ref().and_then(Request::take) {
        data.online_request = None;
        if let Some((total, rows)) = result.ok().and_then(|v| parse_online(&v)) {
            data.online = rows;
            data.online_total = total;
        }
        shell.dirty = true;
    }
    if let Some(result) = data.stats_request.as_ref().and_then(Request::take) {
        data.stats_request = None;
        data.stats = result.ok().filter(|v| {
            ["wins", "losses", "draws", "games"]
                .iter()
                .all(|k| v[k].as_u64().is_some())
                && v["win_rate"].as_f64().is_some()
        });
        shell.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn release_discovery_handles_prereleases_and_protocol_mismatch() {
        let release = json!({"latest_client":"0.1.0-native.2","protocol":17});
        assert_eq!(
            newer_client(&release, "0.1.0-native.1", 17),
            Some("0.1.0-native.2".into())
        );
        assert_eq!(newer_client(&release, "0.1.0-native.2", 17), None);
        assert!(newer_client(&release, "0.1.0-native.2", 16).is_some());
        assert!(newer_client(&json!({}), "0.1.0", 17).is_none());
    }
    #[test]
    fn malformed_live_rows_are_dropped_without_trusting_private_data() {
        let valid = json!({"id":"0000000000000001","mode":"casual","players":[{"name":"Guest","rating":null},{"name":"Alex","rating":1000}],"hp_bp":[12000,3000],"elapsed_s":70});
        let payload = json!({"online":{"players":2,"accounts":1,"in_battle":2,"watching":0},"matches":[{},valid]});
        let parsed = parse_live(&payload).unwrap();
        assert_eq!(parsed.matches.len(), 1);
        assert_eq!(parsed.matches[0].hp, [10000, 3000]);
        assert!(parse_live(&json!({"online":{"players":-1},"matches":[]})).is_none());
        let online = json!({"total":2,"rows":[{"name":"Alex","rating":1000,"status":"playing"},{"name":"","rating":null}]});
        assert_eq!(parse_online(&online).unwrap().1.len(), 1);
    }
}

/// Missing discovery support never blocks Practice or a compatible server.
fn newer_client(value: &Value, current: &str, protocol: u16) -> Option<String> {
    let latest = value.get("latest_client")?.as_str()?;
    let remote = semver::Version::parse(latest).ok()?;
    let current = semver::Version::parse(current).ok()?;
    let incompatible = value.get("protocol")?.as_u64()? != u64::from(protocol);
    (incompatible || remote > current).then(|| latest.to_string())
}
