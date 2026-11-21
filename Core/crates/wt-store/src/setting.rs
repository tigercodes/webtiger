//! The app's own settings (020): each a name and a value, and the whole
//! app's. Where searches go is the first.

use rusqlite::{OptionalExtension, params};

use crate::{Store, StoreError};

impl Store {
    /// What a setting was last given. `None` for one that was never
    /// written.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn setting(&self, key: &str) -> Result<Option<String>, StoreError> {
        self.connection
            .prepare_cached("SELECT value FROM setting WHERE key = ?1")
            .and_then(|mut statement| {
                statement
                    .query_row(params![key], |row| row.get(0))
                    .optional()
            })
            .map_err(|_| StoreError::Query)
    }

    /// Gives a setting a value, in place of the one it had.
    ///
    /// # Errors
    /// `Query` when the database refuses; what was written before is then
    /// what is still there.
    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<(), StoreError> {
        self.connection
            .prepare_cached(
                "INSERT INTO setting (key, value) VALUES (?1, ?2) \
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            )
            .and_then(|mut statement| statement.execute(params![key, value]))
            .map(|_| ())
            .map_err(|_| StoreError::Query)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test fails the way its assertions do"
)]
mod tests {
    use rusqlite::Connection;
    use wt_core::{Timestamp, Uuid};

    use crate::Store;

    #[test]
    fn a_setting_written_is_read_back_and_replaced() {
        let mut store = Store::open_in_memory().unwrap();
        assert_eq!(
            store.setting("search.engine").unwrap(),
            None,
            "never written"
        );

        store.set_setting("search.engine", "ecosia").unwrap();
        assert_eq!(
            store.setting("search.engine").unwrap().as_deref(),
            Some("ecosia")
        );

        store.set_setting("search.engine", "qwant").unwrap();
        assert_eq!(
            store.setting("search.engine").unwrap().as_deref(),
            Some("qwant")
        );
        // One setting is not another's.
        assert_eq!(store.setting("search").unwrap(), None);
        store.set_setting("other", "").unwrap();
        assert_eq!(store.setting("other").unwrap().as_deref(), Some(""));
        assert_eq!(
            store.setting("search.engine").unwrap().as_deref(),
            Some("qwant")
        );
    }

    #[test]
    fn a_setting_is_still_there_when_the_store_is_opened_again() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let mut store = Store::open(&path).unwrap();
        store.set_setting("search.engine", "ecosia").unwrap();
        drop(store);

        let store = Store::open(&path).unwrap();

        assert_eq!(
            store.setting("search.engine").unwrap().as_deref(),
            Some("ecosia")
        );
    }

    #[test]
    fn a_store_from_before_settings_gains_them_and_keeps_what_it_had() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 6 by taking
        // the table away: what 014 left.
        let mut store = Store::open(&path).unwrap();
        let mine = Uuid::from_random_bytes([0x11; 16]);
        let made = store.default_space(mine, Timestamp(1)).unwrap();
        store.set_zoom(made.login_set, "example.com", 125).unwrap();
        drop(store);
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "DROP TABLE journal; ALTER TABLE space DROP COLUMN colour; DROP TABLE setting; PRAGMA user_version = 6;",
        )
        .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        assert_eq!(store.setting("search.engine").unwrap(), None);
        store.set_setting("search.engine", "ecosia").unwrap();
        assert_eq!(
            store.setting("search.engine").unwrap().as_deref(),
            Some("ecosia")
        );
        assert_eq!(
            store.zoom(made.login_set, "example.com").unwrap(),
            Some(125),
            "what was there is still there"
        );
    }
}
