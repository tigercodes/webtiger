//! The schema and its migrations. `PRAGMA user_version` holds the version;
//! each migration runs in one transaction with the version bump, so a
//! failure leaves the database at the version it had.
//!
//! A migration is never edited once it has shipped: a change is a new one.

use rusqlite::Connection;

use crate::StoreError;

/// The version this build's store is at.
pub const SCHEMA_VERSION: u32 = 6;

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
    // 2 (W1, 008): history. A visit names its login set, so a private one,
    // which has no row, can have none. Ids are never used twice: newest
    // first is by id, and a visit that was forgotten names nothing after.
    // The words of a visit's address and title are indexed beside it, kept
    // in step by triggers, and the index forgets what a row forgets.
    "CREATE TABLE visit (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         login_set_id BLOB NOT NULL REFERENCES login_set (id),
         url TEXT NOT NULL,
         title TEXT NOT NULL,
         visited_at INTEGER NOT NULL
     ) STRICT;
     CREATE INDEX visit_login_set ON visit (login_set_id, id);
     CREATE VIRTUAL TABLE visit_text USING fts5 (
         url, title,
         content = 'visit', content_rowid = 'id',
         tokenize = 'unicode61 remove_diacritics 2'
     );
     INSERT INTO visit_text (visit_text, rank) VALUES ('secure-delete', 1);
     CREATE TRIGGER visit_text_insert AFTER INSERT ON visit BEGIN
         INSERT INTO visit_text (rowid, url, title) VALUES (new.id, new.url, new.title);
     END;
     CREATE TRIGGER visit_text_delete AFTER DELETE ON visit BEGIN
         INSERT INTO visit_text (visit_text, rowid, url, title)
         VALUES ('delete', old.id, old.url, old.title);
     END;
     CREATE TRIGGER visit_text_update AFTER UPDATE OF url, title ON visit BEGIN
         INSERT INTO visit_text (visit_text, rowid, url, title)
         VALUES ('delete', old.id, old.url, old.title);
         INSERT INTO visit_text (rowid, url, title) VALUES (new.id, new.url, new.title);
     END;",
    // 3 (W1, 009): the tabs a Space's window had, so that they come back.
    // The table 000 §5.6 names, with the columns that need it so far.
    "CREATE TABLE tab (
         space_id INTEGER NOT NULL REFERENCES space (id),
         position INTEGER NOT NULL,
         url TEXT NOT NULL,
         title TEXT NOT NULL,
         front INTEGER NOT NULL CHECK (front IN (0, 1)),
         PRIMARY KEY (space_id, position)
     ) STRICT, WITHOUT ROWID;",
    // 4 (W1, 012): what a site is remembered by, in a login set: its zoom,
    // so far. The table 000 §5.6 names, with the one column that needs it.
    // A site at 100% has no row.
    "CREATE TABLE site_setting (
         login_set_id BLOB NOT NULL REFERENCES login_set (id),
         site TEXT NOT NULL,
         zoom INTEGER NOT NULL CHECK (zoom BETWEEN 25 AND 500),
         PRIMARY KEY (login_set_id, site)
     ) STRICT, WITHOUT ROWID;",
    // 5 (W1, 013): what was downloaded, per login set. The table 000 §5.6
    // names. Ids are never used twice, as a visit's are not: newest first is
    // by id. How far a download has got is not here: it changes many times
    // a second and means nothing after a quit. `state` is 'coming',
    // 'finished', or the code it failed with.
    "CREATE TABLE download (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         login_set_id BLOB NOT NULL REFERENCES login_set (id),
         url TEXT NOT NULL,
         file_name TEXT NOT NULL,
         mime_type TEXT NOT NULL,
         total_bytes INTEGER NOT NULL,
         state TEXT NOT NULL,
         started_at INTEGER NOT NULL,
         ended_at INTEGER
     ) STRICT;
     CREATE INDEX download_login_set ON download (login_set_id, id);",
    // 6 (W1, 014): what an origin was allowed to use, in a login set: a row
    // for each kind (the camera, the microphone, the location). By origin
    // and not by site: what one of a site's names was allowed, another was
    // not. Only 'allow' is written so far.
    "CREATE TABLE site_permission (
         login_set_id BLOB NOT NULL REFERENCES login_set (id),
         origin TEXT NOT NULL,
         kind TEXT NOT NULL,
         verdict TEXT NOT NULL CHECK (verdict IN ('allow', 'refuse')),
         PRIMARY KEY (login_set_id, origin, kind)
     ) STRICT, WITHOUT ROWID;",
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
