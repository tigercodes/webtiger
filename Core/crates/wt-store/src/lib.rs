//! The store: one SQLite database in the data folder (000 §5.6).
//!
//! W0 has the first two tables, `login_set` and `space`, because a web view
//! cannot be created without knowing whose logins it runs in, and that
//! answer has to be the same tomorrow. W1 adds history (`visit`, 008) and
//! the tabs a window had (`tab`, 009). The journal and the
//! one-writer-many-readers arrangement are still to come.
//!
//! Private login sets are never written here: they live and die in memory.

mod download;
mod history;
mod permission;
mod schema;
mod session;
mod setting;
mod site;
mod suggest;

use std::path::Path;

use rusqlite::{Connection, OptionalExtension as _, params};
use wt_core::{LoginSetId, SpaceId, Timestamp, Uuid};

pub use download::{COMING, Download, DownloadPage, FINISHED, INTERRUPTED};
pub use history::{MAX_VISITS_PER_PAGE, Visit, VisitPage};
pub use permission::Allowed;
pub use schema::SCHEMA_VERSION;
pub use session::SavedTab;
pub use suggest::{MAX_SUGGESTIONS, Suggested};

/// Why the store could not do what was asked, as the code a shell is told
/// ("store.open"). Never carries the user's data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    /// The database could not be opened or configured.
    Open,
    /// A migration failed; nothing of it was kept.
    Migrate,
    /// The database was written by a newer WebTiger. It is left untouched.
    Newer,
    /// A query failed.
    Query,
    /// A row held something the schema forbids.
    Corrupt,
}

impl StoreError {
    /// The contract's code for this failure.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            StoreError::Open => "store.open",
            StoreError::Migrate => "store.migrate",
            StoreError::Newer => "store.newer",
            StoreError::Query => "store.query",
            StoreError::Corrupt => "store.corrupt",
        }
    }
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for StoreError {}

/// A Space, as far as W0 knows one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Space {
    pub id: SpaceId,
    /// Empty for the Space the app made itself: the shell shows its own word.
    pub name: String,
    /// Whose logins its pages run in. Several Spaces may name the same one.
    pub login_set: LoginSetId,
}

/// The open database.
#[derive(Debug)]
pub struct Store {
    connection: Connection,
}

impl Store {
    /// Opens the database at `path`, creating and migrating it as needed.
    ///
    /// # Errors
    /// `Open` when the file cannot be opened, `Newer` when a newer WebTiger
    /// wrote it, `Migrate` when a migration fails.
    pub fn open(path: &Path) -> Result<Store, StoreError> {
        let connection = Connection::open(path).map_err(|_| StoreError::Open)?;
        // WAL: readers never block the writer. NORMAL: a power cut may lose
        // the last transaction but never corrupts the file.
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .and_then(|()| connection.pragma_update(None, "synchronous", "NORMAL"))
            .map_err(|_| StoreError::Open)?;
        Store::prepare(connection)
    }

    /// A database in memory: fake mode's and the tests'.
    ///
    /// # Errors
    /// As [`Store::open`].
    pub fn open_in_memory() -> Result<Store, StoreError> {
        let connection = Connection::open_in_memory().map_err(|_| StoreError::Open)?;
        Store::prepare(connection)
    }

    fn prepare(mut connection: Connection) -> Result<Store, StoreError> {
        // What is deleted is overwritten, not only unlinked: history that is
        // forgotten is gone from the file (008).
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .and_then(|()| connection.pragma_update(None, "secure_delete", "ON"))
            .map_err(|_| StoreError::Open)?;
        schema::migrate(&mut connection)?;
        Ok(Store { connection })
    }

    /// The schema version the database is at.
    ///
    /// # Errors
    /// `Query` when it cannot be read.
    pub fn schema_version(&self) -> Result<u32, StoreError> {
        schema::version(&self.connection)
    }

    /// The first Space, made with a login set of its own when there is none
    /// yet. `new_login_set` is the id a new login set takes; it is not used
    /// when the Space already exists.
    ///
    /// # Errors
    /// `Query` when the database refuses, `Corrupt` when a stored id is not
    /// 16 bytes.
    pub fn default_space(
        &mut self,
        new_login_set: Uuid,
        now: Timestamp,
    ) -> Result<Space, StoreError> {
        if let Some(space) = self.first_space()? {
            return Ok(space);
        }
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute(
                "INSERT INTO login_set (id, created_at) VALUES (?1, ?2)",
                params![new_login_set.as_bytes().as_slice(), now.unix_millis()],
            )
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute(
                "INSERT INTO space (name, position, login_set_id, created_at) \
                 VALUES ('', 0, ?1, ?2)",
                params![new_login_set.as_bytes().as_slice(), now.unix_millis()],
            )
            .map_err(|_| StoreError::Query)?;
        let id = transaction.last_insert_rowid();
        transaction.commit().map_err(|_| StoreError::Query)?;
        Ok(Space {
            id: SpaceId(u64::try_from(id).map_err(|_| StoreError::Corrupt)?),
            name: String::new(),
            login_set: LoginSetId(new_login_set),
        })
    }

    fn first_space(&self) -> Result<Option<Space>, StoreError> {
        let row = self
            .connection
            .query_row(
                "SELECT id, name, login_set_id FROM space ORDER BY position, id LIMIT 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|_| StoreError::Query)?;
        let Some((id, name, login_set)) = row else {
            return Ok(None);
        };
        let bytes: [u8; 16] = login_set.try_into().map_err(|_| StoreError::Corrupt)?;
        Ok(Some(Space {
            id: SpaceId(u64::try_from(id).map_err(|_| StoreError::Corrupt)?),
            name,
            login_set: LoginSetId(Uuid::from_bytes(bytes)),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: Uuid = Uuid::from_random_bytes([0x11; 16]);
    const SECOND: Uuid = Uuid::from_random_bytes([0x22; 16]);

    #[test]
    fn a_new_store_is_at_the_current_schema() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn the_default_space_is_made_once() {
        let mut store = Store::open_in_memory().unwrap();
        let made = store.default_space(FIRST, Timestamp(1)).unwrap();
        assert_eq!(made.name, "");
        assert_eq!(made.login_set, LoginSetId(FIRST));
        let again = store.default_space(SECOND, Timestamp(2)).unwrap();
        assert_eq!(again, made, "a second ask must not mint a second login set");
    }

    #[test]
    fn the_default_space_survives_a_restart() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let made = Store::open(&path)
            .unwrap()
            .default_space(FIRST, Timestamp(1))
            .unwrap();
        let reopened = Store::open(&path)
            .unwrap()
            .default_space(SECOND, Timestamp(2))
            .unwrap();
        assert_eq!(reopened, made);
    }

    #[test]
    fn a_store_from_a_newer_webtiger_is_left_alone() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        drop(Store::open(&path).unwrap());
        let connection = Connection::open(&path).unwrap();
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .unwrap();
        drop(connection);
        assert_eq!(Store::open(&path).unwrap_err(), StoreError::Newer);
    }

    #[test]
    fn a_path_that_cannot_be_opened_is_an_error_not_a_panic() {
        let folder = tempfile::tempdir().unwrap();
        let missing = folder.path().join("no-such-folder").join("webtiger.sqlite");
        assert_eq!(Store::open(&missing).unwrap_err(), StoreError::Open);
    }

    #[test]
    fn a_space_cannot_name_a_login_set_that_does_not_exist() {
        let store = Store::open_in_memory().unwrap();
        let refused = store.connection.execute(
            "INSERT INTO space (name, position, login_set_id, created_at) VALUES ('', 0, ?1, 0)",
            params![FIRST.as_bytes().as_slice()],
        );
        assert!(refused.is_err(), "foreign keys must be enforced");
    }

    #[test]
    fn sqlite_is_built_with_the_flags_the_core_asks_for() {
        let store = Store::open_in_memory().unwrap();
        let mut statement = store.connection.prepare("PRAGMA compile_options").unwrap();
        let options: Vec<String> = statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for wanted in [
            "ENABLE_FTS5",
            "DQS=0",
            "DEFAULT_MEMSTATUS=0",
            "OMIT_DEPRECATED",
            "THREADSAFE=2",
        ] {
            assert!(
                options.iter().any(|option| option == wanted),
                "SQLite lacks {wanted}: {options:?}"
            );
        }
    }
}
