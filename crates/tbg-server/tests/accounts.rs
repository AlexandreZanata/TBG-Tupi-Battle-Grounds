//! Account store and protocol tests (GAME-DESIGN BR-P01–BR-P05).
//! Each test names the rule it protects.

use tbg_core::protocol::AccountError;
use tbg_server::store::Store;

struct TempDb {
    _dir: std::path::PathBuf,
    db: std::path::PathBuf,
}

fn temp_db(name: &str) -> TempDb {
    let dir = std::env::temp_dir().join(format!("tbg-accounts-{}-{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("test.sqlite3");
    let _ = std::fs::remove_file(&db);
    TempDb { _dir: dir, db }
}

fn hash_of(secret: &[u8; 16]) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(secret).into()
}

fn secret_of(byte: u8) -> [u8; 16] {
    let mut s = [0u8; 16];
    for (i, b) in s.iter_mut().enumerate() {
        *b = byte.wrapping_add(i as u8);
    }
    s
}

#[tokio::test]
async fn br_p02_migrate_twice_is_noop() {
    let t = temp_db("migrate");
    let db = &t.db;
    Store::open(db).unwrap();
    // Second open migrates again: must be a no-op, data intact.
    let store = Store::open(db).unwrap();
    let secret = secret_of(1);
    let (_, profile) = store
        .register("Aldric".into(), hash_of(&secret))
        .await
        .unwrap();
    assert_eq!(profile.name, "Aldric");
    assert_eq!(store.match_count().await.unwrap(), 0);
}

#[tokio::test]
async fn br_p01_register_then_login() {
    let t = temp_db("reglogin");
    let db = &t.db;
    let store = Store::open(db).unwrap();
    let secret = secret_of(2);
    let (id, profile) = store
        .register("Aldric".into(), hash_of(&secret))
        .await
        .unwrap();
    assert_eq!(profile.rating, 1000);
    assert_eq!(profile.ranked_games, 0);
    let (same_id, same) = store
        .login("aldric".into(), hash_of(&secret))
        .await
        .unwrap();
    assert_eq!(id, same_id);
    assert_eq!(same, profile);
}

#[tokio::test]
async fn br_p01_duplicate_name_other_case_taken() {
    let t = temp_db("dup");
    let db = &t.db;
    let store = Store::open(db).unwrap();
    store
        .register("Aldric".into(), hash_of(&secret_of(3)))
        .await
        .unwrap();
    assert_eq!(
        store
            .register("ALDRIC".into(), hash_of(&secret_of(4)))
            .await,
        Err(AccountError::NameTaken)
    );
    assert_eq!(
        store
            .register("aldric".into(), hash_of(&secret_of(5)))
            .await,
        Err(AccountError::NameTaken)
    );
}

#[tokio::test]
async fn br_p02_bad_secret_and_unknown_name_share_badkey() {
    // BR-P02/BR-P03: no name oracle — identical error and code path.
    let t = temp_db("badkey");
    let db = &t.db;
    let store = Store::open(db).unwrap();
    let secret = secret_of(6);
    store
        .register("Aldric".into(), hash_of(&secret))
        .await
        .unwrap();
    let mut tampered = secret;
    tampered[0] ^= 0xff;
    assert_eq!(
        store.login("aldric".into(), hash_of(&tampered)).await,
        Err(AccountError::BadKey)
    );
    assert_eq!(
        store.login("unknown".into(), hash_of(&secret)).await,
        Err(AccountError::BadKey)
    );
}
