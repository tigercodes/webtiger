//! The session (009): the tabs a Space's window had, written as they change
//! so that they come back after a quit or a crash.

use rusqlite::params;
use wt_core::SpaceId;

use crate::{Store, StoreError};

/// A tab as it is kept between runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedTab {
    /// Where the tab was. Empty for an empty tab.
    pub url: String,
    /// What the strip showed for it.
    pub title: String,
    /// It was the tab in front.
    pub front: bool,
}

impl Store {
    /// Replaces a Space's saved tabs with `tabs`, in their order, as a
    /// whole: a half-written session is worse than an old one.
    ///
    /// # Errors
    /// `Query` when the database refuses; what was saved before is then
    /// what is still there.
    pub fn save_tabs(&mut self, space: SpaceId, tabs: &[SavedTab]) -> Result<(), StoreError> {
        let space = i64::try_from(space.get()).map_err(|_| StoreError::Query)?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute("DELETE FROM tab WHERE space_id = ?1", params![space])
            .map_err(|_| StoreError::Query)?;
        {
            let mut insert = transaction
                .prepare_cached(
                    "INSERT INTO tab (space_id, position, url, title, front) \
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .map_err(|_| StoreError::Query)?;
            for (position, tab) in tabs.iter().enumerate() {
                let position = i64::try_from(position).map_err(|_| StoreError::Query)?;
                insert
                    .execute(params![space, position, tab.url, tab.title, tab.front])
                    .map_err(|_| StoreError::Query)?;
            }
        }
        transaction.commit().map_err(|_| StoreError::Query)
    }

    /// A Space's saved tabs, in their order. None saved is none.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn saved_tabs(&self, space: SpaceId) -> Result<Vec<SavedTab>, StoreError> {
        let space = i64::try_from(space.get()).map_err(|_| StoreError::Query)?;
        let mut statement = self
            .connection
            .prepare_cached(
                "SELECT url, title, front FROM tab WHERE space_id = ?1 ORDER BY position",
            )
            .map_err(|_| StoreError::Query)?;
        statement
            .query_map(params![space], |row| {
                Ok(SavedTab {
                    url: row.get(0)?,
                    title: row.get(1)?,
                    front: row.get(2)?,
                })
            })
            .and_then(Iterator::collect)
            .map_err(|_| StoreError::Query)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use wt_core::{Timestamp, Uuid};

    use super::*;

    const MINE: Uuid = Uuid::from_random_bytes([0x11; 16]);

    fn tab(url: &str, title: &str, front: bool) -> SavedTab {
        SavedTab {
            url: url.to_owned(),
            title: title.to_owned(),
            front,
        }
    }

    fn three() -> Vec<SavedTab> {
        vec![
            tab("https://example.com/", "Example Domain", false),
            tab("https://example.org/a", "Another Place", true),
            tab("", "", false),
        ]
    }

    #[test]
    fn tabs_are_saved_and_read_back_in_order_with_the_one_in_front() {
        let mut store = Store::open_in_memory().unwrap();
        let space = store.default_space(MINE, Timestamp(1)).unwrap().id;
        assert_eq!(store.saved_tabs(space).unwrap(), [], "a new store has none");

        store.save_tabs(space, &three()).unwrap();

        assert_eq!(store.saved_tabs(space).unwrap(), three());
    }

    #[test]
    fn saving_again_replaces_what_was_saved() {
        let mut store = Store::open_in_memory().unwrap();
        let space = store.default_space(MINE, Timestamp(1)).unwrap().id;
        store.save_tabs(space, &three()).unwrap();

        // Fewer, in another order, another one in front.
        let fewer = vec![
            tab("https://example.org/a", "Another Place", false),
            tab("https://example.com/", "Example Domain", true),
        ];
        store.save_tabs(space, &fewer).unwrap();
        assert_eq!(store.saved_tabs(space).unwrap(), fewer);

        store.save_tabs(space, &[]).unwrap();
        assert_eq!(store.saved_tabs(space).unwrap(), []);
    }

    #[test]
    fn another_spaces_tabs_are_not_touched() {
        let mut store = Store::open_in_memory().unwrap();
        let space = store.default_space(MINE, Timestamp(1)).unwrap().id;
        store
            .connection
            .execute(
                "INSERT INTO space (name, position, login_set_id, created_at) VALUES ('Work', 1, ?1, 1)",
                params![MINE.as_bytes().as_slice()],
            )
            .unwrap();
        let other = SpaceId(u64::try_from(store.connection.last_insert_rowid()).unwrap());
        store.save_tabs(space, &three()).unwrap();
        store
            .save_tabs(other, &[tab("https://work.example.com/", "Work", true)])
            .unwrap();

        store.save_tabs(space, &[]).unwrap();

        assert_eq!(store.saved_tabs(space).unwrap(), []);
        assert_eq!(store.saved_tabs(other).unwrap().len(), 1);
        // A Space the store does not have can have no tabs.
        assert_eq!(
            store.save_tabs(SpaceId(9_999), &three()).unwrap_err(),
            StoreError::Query
        );
    }

    #[test]
    fn tabs_survive_a_restart() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let mut store = Store::open(&path).unwrap();
        let space = store.default_space(MINE, Timestamp(1)).unwrap().id;
        store.save_tabs(space, &three()).unwrap();
        // Dropped without a word, as a crash leaves it.
        drop(store);

        let store = Store::open(&path).unwrap();
        assert_eq!(store.saved_tabs(space).unwrap(), three());
    }

    #[test]
    fn a_store_from_before_the_session_gains_it_and_keeps_its_history() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 2 by taking
        // away the session's table and what came after it: what 008 left.
        let mut store = Store::open(&path).unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        store
            .record_visit(
                made.login_set,
                "https://example.com/",
                "Example",
                Timestamp(2),
            )
            .unwrap();
        drop(store);
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "ALTER TABLE space DROP COLUMN colour; DROP TABLE setting; DROP TABLE site_permission; DROP TABLE download; DROP TABLE site_setting; DROP TABLE tab; PRAGMA user_version = 2;",
        )
            .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        store.save_tabs(made.id, &three()).unwrap();
        assert_eq!(store.saved_tabs(made.id).unwrap(), three());
        let history = store.visits(made.login_set, "", None, 10).unwrap();
        assert_eq!(history.visits.len(), 1, "what was visited is still there");
    }
}
