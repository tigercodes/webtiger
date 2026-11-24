//! A tab's kept state (051): what its engine saved of a web view, for
//! another to be made from. To the store, as to the core, it is bytes, with
//! the engine they are of. A saved tab names its state by the state's id
//! (`SavedTab::state`); a state that no saved tab names is of no use to
//! anybody, and is dropped at a start.
//!
//! What may be kept, and when, are the core's rules: the store keeps what
//! it is given.

use rusqlite::{OptionalExtension, params};

use crate::{Store, StoreError};

impl Store {
    /// Keeps a state, of the engine `engine` (the contract's number for
    /// it). The id it is named by from now: never used twice, and never 0.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn keep_state(&mut self, engine: i32, state: &[u8]) -> Result<i64, StoreError> {
        self.connection
            .execute(
                "INSERT INTO tab_state (engine, state) VALUES (?1, ?2)",
                params![engine, state],
            )
            .map_err(|_| StoreError::Query)?;
        Ok(self.connection.last_insert_rowid())
    }

    /// The state of that id, and the engine it is of. `None` when there is
    /// none: it was dropped, or never kept.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn kept_state(&self, id: i64) -> Result<Option<(i32, Vec<u8>)>, StoreError> {
        self.connection
            .query_row(
                "SELECT engine, state FROM tab_state WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|_| StoreError::Query)
    }

    /// Drops a state: its tab is gone, or has another. One that is not
    /// there is no error.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn drop_state(&mut self, id: i64) -> Result<(), StoreError> {
        self.connection
            .execute("DELETE FROM tab_state WHERE id = ?1", params![id])
            .map_err(|_| StoreError::Query)?;
        Ok(())
    }

    /// Drops states because the person asked for what they hold to be
    /// forgotten (008): a state holds the list of pages its tab goes back
    /// through. They are gone from the store's files, as a forgotten visit
    /// is, and not only from the table.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_states(&mut self, ids: &[i64]) -> Result<(), StoreError> {
        if ids.is_empty() {
            return Ok(());
        }
        for id in ids {
            self.drop_state(*id)?;
        }
        self.leave_no_trace()
    }

    /// Drops every state that no saved tab names: what a run left behind
    /// that the session did not keep. How many went.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn drop_states_no_tab_names(&mut self) -> Result<usize, StoreError> {
        self.connection
            .execute(
                "DELETE FROM tab_state WHERE id NOT IN (SELECT state_id FROM tab)",
                [],
            )
            .map_err(|_| StoreError::Query)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "a test's helpers fail the test the way its assertions do"
)]
mod tests {
    use rusqlite::Connection;
    use wt_core::{Timestamp, Uuid};

    use crate::{SavedTab, Store};

    const FIRST: Uuid = Uuid::from_bytes([1; 16]);

    fn tab(url: &str, state: i64) -> SavedTab {
        SavedTab {
            url: url.to_owned(),
            title: String::new(),
            front: false,
            home: String::new(),
            name: String::new(),
            group: 0,
            favourite: false,
            shown_at: 0,
            state,
        }
    }

    #[test]
    fn a_state_is_kept_by_an_id_of_its_own_with_its_engine_and_dropped() {
        let mut store = Store::open_in_memory().unwrap();
        let one = store.keep_state(1, b"the first").unwrap();
        let two = store.keep_state(2, &[0, 255, 7]).unwrap();
        assert!(one > 0 && two > one, "never nought, and never the same");
        assert_eq!(
            store.kept_state(one).unwrap(),
            Some((1, b"the first".to_vec()))
        );
        assert_eq!(store.kept_state(two).unwrap(), Some((2, vec![0, 255, 7])));
        assert_eq!(store.kept_state(0).unwrap(), None);
        assert_eq!(store.kept_state(two + 1).unwrap(), None);

        store.drop_state(one).unwrap();
        assert_eq!(store.kept_state(one).unwrap(), None);
        assert_eq!(store.kept_state(two).unwrap(), Some((2, vec![0, 255, 7])));
        // One that is not there is no error; and its id is not given again.
        store.drop_state(one).unwrap();
        store.drop_state(two).unwrap();
        let three = store.keep_state(1, b"another").unwrap();
        assert!(three > two);
    }

    #[test]
    fn a_saved_tab_names_its_state_and_a_state_no_tab_names_goes_at_a_start() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let (space, named, unnamed) = {
            let mut store = Store::open(&path).unwrap();
            let space = store.default_space(FIRST, Timestamp(1)).unwrap().id;
            let named = store.keep_state(1, b"named").unwrap();
            let unnamed = store.keep_state(1, b"nobody's").unwrap();
            store
                .save_tabs(
                    space,
                    &[
                        tab("https://example.com/", named),
                        tab("https://example.org/", 0),
                    ],
                )
                .unwrap();
            (space, named, unnamed)
        };
        // Kept over a start: the tab names its state, and the state is there.
        let mut store = Store::open(&path).unwrap();
        let tabs = store.saved_tabs(space).unwrap();
        assert_eq!(
            tabs.iter().map(|tab| tab.state).collect::<Vec<_>>(),
            [named, 0]
        );
        assert_eq!(
            store.kept_state(named).unwrap(),
            Some((1, b"named".to_vec()))
        );
        // The one nobody names goes, and only it.
        assert_eq!(store.drop_states_no_tab_names().unwrap(), 1);
        assert_eq!(store.kept_state(unnamed).unwrap(), None);
        assert_eq!(
            store.kept_state(named).unwrap(),
            Some((1, b"named".to_vec()))
        );
        assert_eq!(store.drop_states_no_tab_names().unwrap(), 0);
    }

    #[test]
    fn a_state_that_is_forgotten_is_gone_from_the_files() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let holds = |needle: &str| {
            std::fs::read_dir(folder.path())
                .unwrap()
                .map(|entry| std::fs::read(entry.unwrap().path()).unwrap())
                .any(|bytes| {
                    bytes
                        .windows(needle.len())
                        .any(|window| window == needle.as_bytes())
                })
        };
        let mut store = Store::open(&path).unwrap();
        store.default_space(FIRST, Timestamp(1)).unwrap();
        // A state as large as a page of the database and more, with words that are nowhere
        // else in it; and others, which stay.
        let mut secret = b"quixotic-zebrafinch ".repeat(400);
        secret.extend_from_slice(b"xylophonist");
        let forgotten = store.keep_state(1, &secret).unwrap();
        let stays = store.keep_state(1, b"a wombatical state").unwrap();
        assert!(holds("zebrafinch") && holds("xylophonist") && holds("wombatical"));

        store.forget_states(&[forgotten]).unwrap();
        assert_eq!(store.kept_state(forgotten).unwrap(), None);
        assert!(
            !holds("zebrafinch") && !holds("xylophonist"),
            "not in the database, nor in its write-ahead log"
        );
        assert!(holds("wombatical"), "the other is there");
        // None to forget is no error, and one that is not there neither.
        store.forget_states(&[]).unwrap();
        store.forget_states(&[forgotten, 999]).unwrap();
        assert_eq!(
            store.kept_state(stays).unwrap(),
            Some((1, b"a wombatical state".to_vec()))
        );
    }

    #[test]
    fn a_store_from_before_kept_states_gains_them_and_keeps_its_tabs() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let space = {
            let mut store = Store::open(&path).unwrap();
            let space = store.default_space(FIRST, Timestamp(1)).unwrap().id;
            store
                .save_tabs(space, &[tab("https://example.com/", 0)])
                .unwrap();
            space
        };
        {
            let old = Connection::open(&path).unwrap();
            old.execute_batch(
                "ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; PRAGMA user_version = 15;",
            )
            .unwrap();
        }
        let mut store = Store::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        let tabs = store.saved_tabs(space).unwrap();
        assert_eq!(
            tabs.iter()
                .map(|tab| (tab.url.as_str(), tab.state))
                .collect::<Vec<_>>(),
            [("https://example.com/", 0)],
            "its tab is there, with no state"
        );
        let kept = store.keep_state(1, b"now").unwrap();
        assert_eq!(store.kept_state(kept).unwrap(), Some((1, b"now".to_vec())));
    }
}
