//! Test-only identity extensions; never part of the live v8 migration chain.

use rusqlite::Connection;

pub(super) fn install(conn: &mut Connection) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute_batch(
        "CREATE TABLE recovery_identity (
          account_id INTEGER PRIMARY KEY REFERENCES accounts(id),
          verified_email TEXT, epoch INTEGER NOT NULL DEFAULT 0 CHECK(epoch >= 0),
          compromised INTEGER NOT NULL DEFAULT 0 CHECK(compromised IN (0,1))
        );
        CREATE TABLE recovery_challenges (
          digest BLOB PRIMARY KEY CHECK(length(digest)=32),
          account_id INTEGER NOT NULL REFERENCES recovery_identity(account_id),
          purpose TEXT NOT NULL CHECK(purpose IN ('verify','rotate')),
          email TEXT NOT NULL, epoch INTEGER NOT NULL,
          expires INTEGER NOT NULL, consumed INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE recovery_dispatch_limits (
          bucket BLOB NOT NULL, issued INTEGER NOT NULL
        );
        CREATE INDEX recovery_limits_by_bucket ON recovery_dispatch_limits(bucket, issued);
        CREATE TABLE recovery_audit (
          account_id INTEGER NOT NULL REFERENCES accounts(id),
          epoch INTEGER NOT NULL, action TEXT NOT NULL CHECK(action IN ('verified','rotated')),
          PRIMARY KEY(account_id,epoch,action)
        );
        CREATE TABLE recovery_revocations (
          account_id INTEGER NOT NULL REFERENCES accounts(id), epoch INTEGER NOT NULL,
          PRIMARY KEY(account_id,epoch)
        );",
    )?;
    tx.commit()
}
