//! A tab's kept state (051): what its engine saved of a web view, for
//! another to be made from. To the store, as to the core, it is bytes, with
//! the engine they are of. A saved tab names its state by the state's id
//! (`SavedTab::state`); a state that no saved tab names is of no use to
//! anybody, and is dropped at a start.
//!
//! A state may have a picture with it (059): of its tab's page as it was
//! when the tab was put to sleep. It is a part of the state's row: it is
//! there for as long as the state is, and goes when the state goes, however
//! it goes.
//!
//! What may be kept, and when, are the core's rules: the store keeps what
//! it is given.

use rusqlite::{OptionalExtension, params};

use crate::{Store, StoreError};

/// The picture kept with a state (059): a JPEG, and its size in pixels. To
/// the store it is bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeptPicture {
    pub jpeg: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

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

    /// Keeps a picture with the state of that id (059), in the place of any
    /// it had. Whether there was such a state: with none, nothing is kept.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn keep_picture(&mut self, state: i64, picture: &KeptPicture) -> Result<bool, StoreError> {
        self.connection
            .execute(
                "UPDATE tab_state SET picture = ?2, picture_width = ?3, picture_height = ?4 WHERE id = ?1",
                params![state, picture.jpeg, picture.width, picture.height],
            )
            .map(|rows| rows > 0)
            .map_err(|_| StoreError::Query)
    }

    /// The picture kept with the state of that id (059). `None` when the
    /// state has none, or there is no such state.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn kept_picture(&self, state: i64) -> Result<Option<KeptPicture>, StoreError> {
        self.connection
            .query_row(
                "SELECT picture, picture_width, picture_height FROM tab_state
                 WHERE id = ?1 AND picture IS NOT NULL",
                params![state],
                |row| {
                    Ok(KeptPicture {
                        jpeg: row.get(0)?,
                        width: row.get(1)?,
                        height: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(|_| StoreError::Query)
    }

    /// Drops a state, and its picture with it: its tab is gone, or has
    /// another. One that is not there is no error.
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

    use super::KeptPicture;
    use crate::{SavedTab, Store};

    const FIRST: Uuid = Uuid::from_bytes([1; 16]);

    fn picture(jpeg: &[u8], width: u32, height: u32) -> KeptPicture {
        KeptPicture {
            jpeg: jpeg.to_vec(),
            width,
            height,
        }
    }

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

    #[test]
    fn a_picture_is_kept_with_a_state_and_goes_with_it() {
        let mut store = Store::open_in_memory().unwrap();
        let one = store.keep_state(1, b"the first").unwrap();
        let two = store.keep_state(1, b"the second").unwrap();
        // A state has no picture until it is given one; and no state, none.
        assert_eq!(store.kept_picture(one).unwrap(), None);
        assert_eq!(store.kept_picture(0).unwrap(), None);
        assert_eq!(store.kept_picture(two + 1).unwrap(), None);

        assert!(
            store
                .keep_picture(one, &picture(b"a page", 800, 600))
                .unwrap()
        );
        assert_eq!(
            store.kept_picture(one).unwrap(),
            Some(picture(b"a page", 800, 600))
        );
        assert_eq!(
            store.kept_picture(two).unwrap(),
            None,
            "the other's has none"
        );
        // The state is the one it was.
        assert_eq!(
            store.kept_state(one).unwrap(),
            Some((1, b"the first".to_vec()))
        );
        // Another picture takes the place of the first.
        assert!(
            store
                .keep_picture(one, &picture(b"later", 400, 900))
                .unwrap()
        );
        assert_eq!(
            store.kept_picture(one).unwrap(),
            Some(picture(b"later", 400, 900))
        );
        // With no such state nothing is kept, and that is said.
        assert!(
            !store
                .keep_picture(two + 1, &picture(b"nobody's", 1, 1))
                .unwrap()
        );
        assert_eq!(store.kept_picture(two + 1).unwrap(), None);
        assert_eq!(
            store.kept_state(two + 1).unwrap(),
            None,
            "and no state is made for it"
        );

        // It goes with its state.
        store.drop_state(one).unwrap();
        assert_eq!(store.kept_picture(one).unwrap(), None);
        assert!(!store.keep_picture(one, &picture(b"again", 1, 1)).unwrap());
    }

    #[test]
    fn a_picture_goes_with_a_state_no_tab_names_and_stays_with_one_that_is_named() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let (named, unnamed) = {
            let mut store = Store::open(&path).unwrap();
            let space = store.default_space(FIRST, Timestamp(1)).unwrap().id;
            let named = store.keep_state(1, b"named").unwrap();
            let unnamed = store.keep_state(1, b"nobody's").unwrap();
            assert!(
                store
                    .keep_picture(named, &picture(b"kept", 800, 600))
                    .unwrap()
            );
            assert!(
                store
                    .keep_picture(unnamed, &picture(b"left", 800, 600))
                    .unwrap()
            );
            store
                .save_tabs(space, &[tab("https://example.com/", named)])
                .unwrap();
            (named, unnamed)
        };
        // Kept over a start, with its state.
        let mut store = Store::open(&path).unwrap();
        assert_eq!(
            store.kept_picture(named).unwrap(),
            Some(picture(b"kept", 800, 600))
        );
        assert_eq!(store.drop_states_no_tab_names().unwrap(), 1);
        assert_eq!(store.kept_picture(unnamed).unwrap(), None);
        assert_eq!(
            store.kept_picture(named).unwrap(),
            Some(picture(b"kept", 800, 600))
        );
    }

    #[test]
    fn a_picture_whose_state_is_forgotten_is_gone_from_the_files() {
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
        // A picture as large as many pages of the database, with words that are nowhere else
        // in it; and another, which stays.
        let mut secret = b"perspicacious-axolotl ".repeat(3000);
        secret.extend_from_slice(b"zymurgist");
        let forgotten = store.keep_state(1, b"a state").unwrap();
        let stays = store.keep_state(1, b"another state").unwrap();
        assert!(
            store
                .keep_picture(forgotten, &picture(&secret, 800, 600))
                .unwrap()
        );
        assert!(
            store
                .keep_picture(stays, &picture(b"a marsupial picture", 800, 600))
                .unwrap()
        );
        assert!(holds("axolotl") && holds("zymurgist") && holds("marsupial"));

        store.forget_states(&[forgotten]).unwrap();
        assert_eq!(store.kept_picture(forgotten).unwrap(), None);
        assert!(
            !holds("axolotl") && !holds("zymurgist"),
            "not in the database, nor in its write-ahead log"
        );
        assert!(holds("marsupial"), "the other is there");
        assert_eq!(
            store.kept_picture(stays).unwrap(),
            Some(picture(b"a marsupial picture", 800, 600))
        );
    }

    #[test]
    fn a_picture_that_another_takes_the_place_of_is_gone_when_its_state_is_forgotten() {
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
        let state = store.keep_state(1, b"a state").unwrap();
        let mut first = b"sesquipedalian-pangolin ".repeat(3000);
        first.extend_from_slice(b"ultracrepidarian");
        assert!(
            store
                .keep_picture(state, &picture(&first, 800, 600))
                .unwrap()
        );
        assert!(
            store
                .keep_picture(state, &picture(b"the later one", 800, 600))
                .unwrap()
        );
        store.forget_states(&[state]).unwrap();
        assert!(
            !holds("pangolin") && !holds("ultracrepidarian") && !holds("the later one"),
            "neither the picture it had nor the one before it"
        );
    }

    #[test]
    fn a_store_from_before_pictures_gains_them_and_keeps_its_states() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let (space, kept) = {
            let mut store = Store::open(&path).unwrap();
            let space = store.default_space(FIRST, Timestamp(1)).unwrap().id;
            let kept = store.keep_state(1, b"from before").unwrap();
            store
                .save_tabs(space, &[tab("https://example.com/", kept)])
                .unwrap();
            (space, kept)
        };
        {
            let old = Connection::open(&path).unwrap();
            old.execute_batch(
                "ALTER TABLE tab_state DROP COLUMN picture;
                 ALTER TABLE tab_state DROP COLUMN picture_width;
                 ALTER TABLE tab_state DROP COLUMN picture_height;
                 PRAGMA user_version = 16;",
            )
            .unwrap();
        }
        let mut store = Store::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        assert_eq!(
            store
                .saved_tabs(space)
                .unwrap()
                .iter()
                .map(|tab| tab.state)
                .collect::<Vec<_>>(),
            [kept]
        );
        assert_eq!(
            store.kept_state(kept).unwrap(),
            Some((1, b"from before".to_vec())),
            "its state is there"
        );
        assert_eq!(store.kept_picture(kept).unwrap(), None, "with no picture");
        assert!(
            store
                .keep_picture(kept, &picture(b"now", 800, 600))
                .unwrap()
        );
        assert_eq!(
            store.kept_picture(kept).unwrap(),
            Some(picture(b"now", 800, 600))
        );
    }
}
