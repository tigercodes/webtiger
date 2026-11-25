//! The schema and its migrations. `PRAGMA user_version` holds the version;
//! each migration runs in one transaction with the version bump, so a
//! failure leaves the database at the version it had.
//!
//! A migration is never edited once it has shipped: a change is a new one.

use rusqlite::Connection;

use crate::StoreError;

/// The version this build's store is at.
pub const SCHEMA_VERSION: u32 = 18;

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
    // 7 (W1, 020): the app's own settings, each a name and a value. They
    // are the whole app's: no login set and no Space has its own. Where
    // searches go is the first.
    "CREATE TABLE setting (
         key TEXT PRIMARY KEY NOT NULL,
         value TEXT NOT NULL
     ) STRICT, WITHOUT ROWID;",
    // 8 (W2, 025): a Space's colour, by the core's name for it. Empty: none,
    // which is what every Space from before has.
    "ALTER TABLE space ADD COLUMN colour TEXT NOT NULL DEFAULT '';",
    // 9 (W1, 033): the journal, in its first form. What the person did to
    // organise their browsing, written before it is done, so that it can
    // be undone. The table 000 §5.6 names, with the columns its one kind
    // of entry needs so far: a tab that was closed, by the Space it was
    // in, its place in the row, its address and its title. Ids are never
    // used twice: the newest entry is the one with the largest.
    "CREATE TABLE journal (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         at INTEGER NOT NULL,
         kind TEXT NOT NULL CHECK (kind IN ('tab.closed')),
         space_id INTEGER NOT NULL REFERENCES space (id),
         position INTEGER NOT NULL CHECK (position >= 0),
         url TEXT NOT NULL,
         title TEXT NOT NULL
     ) STRICT;
     CREATE INDEX journal_space ON journal (space_id, id);",
    // 10 (W2, 039): a tab that is kept is a Site, and a Site has a home, the
    // address it was kept at, and a name, the title it was kept with. A tab
    // that is no Site has neither. (The tree 000 §5.6 names, `node`, comes
    // with Groups: until a Site has a parent its place is its place in its
    // row.)
    "ALTER TABLE tab ADD COLUMN site_home TEXT NOT NULL DEFAULT '';
     ALTER TABLE tab ADD COLUMN site_name TEXT NOT NULL DEFAULT '';",
    // 11 (W2, 041): Groups. The table 000 §5.6 names, with the one kind of
    // node there is so far and the columns it needs: a Space's Groups by
    // their place, each with its name, its colour or none, and whether it
    // is shut. A Site says which Group it is in by that Group's place,
    // counted from one; nought is none.
    "CREATE TABLE node (
         space_id INTEGER NOT NULL REFERENCES space (id),
         position INTEGER NOT NULL CHECK (position >= 0),
         kind TEXT NOT NULL CHECK (kind IN ('group')),
         name TEXT NOT NULL,
         colour TEXT NOT NULL,
         shut INTEGER NOT NULL CHECK (shut IN (0, 1)),
         PRIMARY KEY (space_id, position)
     ) STRICT, WITHOUT ROWID;
     ALTER TABLE tab ADD COLUMN site_group INTEGER NOT NULL DEFAULT 0;",
    // 12 (W2, 042): Favourites. A Site says whether it is one of its row's.
    "ALTER TABLE tab ADD COLUMN favourite INTEGER NOT NULL DEFAULT 0;",
    // 13 (W2, 046): a site's icon. One picture for a host in a login set,
    // as the bytes it came as, and when a tab last showed it: the ones used
    // most lately are kept. (A table with a row number: its rows are larger
    // than a table without one is good for.)
    "CREATE TABLE icon (
         id INTEGER PRIMARY KEY,
         login_set_id BLOB NOT NULL REFERENCES login_set (id),
         host TEXT NOT NULL CHECK (host <> ''),
         png BLOB NOT NULL CHECK (length(png) BETWEEN 1 AND 65536),
         used_at INTEGER NOT NULL,
         UNIQUE (login_set_id, host)
     ) STRICT;
     CREATE INDEX icon_used ON icon (used_at);",
    // 14 (W2, 049): tidying. Each Space's time, in seconds, after which a
    // tab that is no Site and has not been in front is tidied (0: never);
    // when each saved tab was last in front, so that a quit does not start
    // every tab's time again; and the Archive, where a tidied tab is listed
    // by its Space until it is brought back or forgotten. Ids are never used
    // twice: newest first is by id.
    "ALTER TABLE space ADD COLUMN tidy_after INTEGER NOT NULL DEFAULT 0 CHECK (tidy_after >= 0);
     ALTER TABLE tab ADD COLUMN shown_at INTEGER NOT NULL DEFAULT 0;
     CREATE TABLE archive (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         space_id INTEGER NOT NULL REFERENCES space (id),
         url TEXT NOT NULL CHECK (url <> ''),
         title TEXT NOT NULL,
         tidied_at INTEGER NOT NULL
     ) STRICT;
     CREATE INDEX archive_space ON archive (space_id, id);",
    // 15 (W2, 050): a Space's icon, an emoji of the core's list; empty for
    // none, which is what every Space there was has.
    "ALTER TABLE space ADD COLUMN icon TEXT NOT NULL DEFAULT '';",
    // 16 (W3, 051): a tab's kept state. What an engine saved of a web view,
    // with the engine it is of; and the state a saved tab names (0: none).
    // Ids are never used twice.
    "CREATE TABLE tab_state (
         id INTEGER PRIMARY KEY AUTOINCREMENT,
         engine INTEGER NOT NULL,
         state BLOB NOT NULL
     ) STRICT;
     ALTER TABLE tab ADD COLUMN state_id INTEGER NOT NULL DEFAULT 0;",
    // 17 (W3, 059): the picture of a sleeping tab's page, with its state: a
    // JPEG and its size in pixels. NULL, and nought by nought, for a state
    // with none: which is every state there was.
    "ALTER TABLE tab_state ADD COLUMN picture BLOB;
     ALTER TABLE tab_state ADD COLUMN picture_width INTEGER NOT NULL DEFAULT 0;
     ALTER TABLE tab_state ADD COLUMN picture_height INTEGER NOT NULL DEFAULT 0;",
    // 18 (W4, 067): Guard's switch for a site, in a login set: on (1) or
    // off (0). A site that says nothing has no row, which is every site
    // there was.
    "CREATE TABLE site_guard (
         login_set_id BLOB NOT NULL REFERENCES login_set (id),
         site TEXT NOT NULL,
         guard INTEGER NOT NULL CHECK (guard IN (0, 1)),
         PRIMARY KEY (login_set_id, site)
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
