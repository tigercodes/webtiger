//! The schema and its migrations. `PRAGMA user_version` holds the version;
//! each migration runs in one transaction with the version bump, so a
//! failure leaves the database at the version it had.
//!
//! A migration is never edited once it has shipped: a change is a new one.

use rusqlite::Connection;

use crate::StoreError;

/// The version this build's store is at.
pub const SCHEMA_VERSION: u32 = 1;

/// The migrations, in order: `MIGRATIONS[n]` takes version `n` to `n + 1`.
const MIGRATIONS: [&str; SCHEMA_VERSION as usize] = [
    // 1 (W0): identities and Spaces. Many Spaces may share one login set
    // (WT-D9), which the schema says from the first version.
    "CREATE TABLE login_set (
         id BLOB PRIMARY KEY NOT NULL CHECK (length(id) = 16),
         created_at INTEGER NOT NULL
     ) STRICT, WITHOUT ROWID;
     CREATE TABLE space (
         id INTEGER PRIMARY KEY,
         name TEXT NOT NULL,
         position INTEGER NOT NULL,
         login_set_id BLOB NOT NULL REFERENCES login_set (id),
         created_at INTEGER NOT NULL
     ) STRICT;
     CREATE INDEX space_login_set ON space (login_set_id);",
];

pub(crate) fn version(connection: &Connection) -> Result<u32, StoreError> {
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|_| StoreError::Query)
}

pub(crate) fn migrate(connection: &mut Connection) -> Result<(), StoreError> {
    let at = version(connection)?;
    if at > SCHEMA_VERSION {
        return Err(StoreError::Newer);
    }
    for (index, sql) in MIGRATIONS.iter().enumerate() {
        let target = u32::try_from(index + 1).map_err(|_| StoreError::Migrate)?;
        if target <= at {
            continue;
        }
        let transaction = connection.transaction().map_err(|_| StoreError::Migrate)?;
        transaction
            .execute_batch(sql)
            .map_err(|_| StoreError::Migrate)?;
        transaction
            .pragma_update(None, "user_version", target)
            .map_err(|_| StoreError::Migrate)?;
        transaction.commit().map_err(|_| StoreError::Migrate)?;
    }
    Ok(())
}
