//! The journal (033), in its first form: what the person did to organise
//! their browsing, written before it is done, so that it can be undone.
//! One kind of thing is written so far: a tab that was closed.
//!
//! An entry that is undone is taken out: what was undone is not undone
//! twice. What the journal keeps is places the person visited, so it is
//! forgotten when history is (008).

use rusqlite::{OptionalExtension as _, params};
use wt_core::{LoginSetId, SpaceId, Timestamp};

use crate::{Store, StoreError};

/// How many closed tabs of a Space are kept: the newest. An older one is in
/// history, where every place is.
pub const CLOSED_TABS_KEPT: usize = 50;

/// A tab that was closed, as the journal keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosedTab {
    /// Its place in its Space's row when it was closed: 0 for the first.
    pub position: u32,
    /// Where it was.
    pub url: String,
    /// What the strip showed for it.
    pub title: String,
}

impl Store {
    /// Writes that a tab of a Space is being closed. Only the Space's
    /// newest [`CLOSED_TABS_KEPT`] are kept: the entry and the tidying are
    /// one transaction.
    ///
    /// # Errors
    /// `Query` when the database refuses, or has no such Space; nothing is
    /// then written.
    pub fn journal_closed_tab(
        &mut self,
        space: SpaceId,
        closed: &ClosedTab,
        at: Timestamp,
    ) -> Result<(), StoreError> {
        let space = i64::try_from(space.get()).map_err(|_| StoreError::Query)?;
        let kept = i64::try_from(CLOSED_TABS_KEPT).map_err(|_| StoreError::Query)?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute(
                "INSERT INTO journal (at, kind, space_id, position, url, title) \
                 VALUES (?1, 'tab.closed', ?2, ?3, ?4, ?5)",
                params![at.0, space, closed.position, closed.url, closed.title],
            )
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute(
                "DELETE FROM journal WHERE space_id = ?1 AND kind = 'tab.closed' AND id NOT IN \
                 (SELECT id FROM journal WHERE space_id = ?1 AND kind = 'tab.closed' \
                  ORDER BY id DESC LIMIT ?2)",
                params![space, kept],
            )
            .map_err(|_| StoreError::Query)?;
        transaction.commit().map_err(|_| StoreError::Query)
    }

    /// How many closed tabs of a Space the journal has.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn closed_tabs(&self, space: SpaceId) -> Result<usize, StoreError> {
        let space = i64::try_from(space.get()).map_err(|_| StoreError::Query)?;
        let count: i64 = self
            .connection
            .query_row(
                "SELECT count(*) FROM journal WHERE space_id = ?1 AND kind = 'tab.closed'",
                params![space],
                |row| row.get(0),
            )
            .map_err(|_| StoreError::Query)?;
        usize::try_from(count).map_err(|_| StoreError::Query)
    }

    /// Takes the tab that was last closed in a Space out of the journal:
    /// read and deleted as one. `None` when the journal has none.
    ///
    /// # Errors
    /// `Query` when the database refuses; the journal is then as it was.
    pub fn take_closed_tab(&mut self, space: SpaceId) -> Result<Option<ClosedTab>, StoreError> {
        let space = i64::try_from(space.get()).map_err(|_| StoreError::Query)?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        let newest = transaction
            .query_row(
                "SELECT id, position, url, title FROM journal \
                 WHERE space_id = ?1 AND kind = 'tab.closed' ORDER BY id DESC LIMIT 1",
                params![space],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        ClosedTab {
                            position: row.get(1)?,
                            url: row.get(2)?,
                            title: row.get(3)?,
                        },
                    ))
                },
            )
            .optional()
            .map_err(|_| StoreError::Query)?;
        let Some((id, closed)) = newest else {
            return Ok(None);
        };
        transaction
            .execute("DELETE FROM journal WHERE id = ?1", params![id])
            .map_err(|_| StoreError::Query)?;
        transaction.commit().map_err(|_| StoreError::Query)?;
        Ok(Some(closed))
    }

    /// Forgets every closed tab of every Space whose logins are `login_set`:
    /// gone from the store's files, as forgotten visits are.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_closed_tabs(&mut self, login_set: LoginSetId) -> Result<(), StoreError> {
        self.connection
            .execute(
                "DELETE FROM journal WHERE kind = 'tab.closed' AND space_id IN \
                 (SELECT id FROM space WHERE login_set_id = ?1)",
                params![login_set.0.as_bytes().as_slice()],
            )
            .map_err(|_| StoreError::Query)?;
        self.leave_no_trace()
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use wt_core::Uuid;

    use super::*;

    const MINE: Uuid = Uuid::from_random_bytes([0x11; 16]);
    const THEIRS: Uuid = Uuid::from_random_bytes([0x22; 16]);

    fn closed(position: u32, url: &str, title: &str) -> ClosedTab {
        ClosedTab {
            position,
            url: url.to_owned(),
            title: title.to_owned(),
        }
    }

    /// Another Space, with the logins named.
    fn space(store: &Store, name: &str, login_set: Uuid) -> SpaceId {
        store
            .connection
            .execute(
                "INSERT OR IGNORE INTO login_set (id, created_at) VALUES (?1, 1)",
                params![login_set.as_bytes().as_slice()],
            )
            .unwrap();
        store
            .connection
            .execute(
                "INSERT INTO space (name, position, login_set_id, created_at) VALUES (?1, 1, ?2, 1)",
                params![name, login_set.as_bytes().as_slice()],
            )
            .unwrap();
        SpaceId(u64::try_from(store.connection.last_insert_rowid()).unwrap())
    }

    #[test]
    fn a_closed_tab_is_written_and_taken_newest_first_and_only_once() {
        let mut store = Store::open_in_memory().unwrap();
        let home = store.default_space(MINE, Timestamp(1)).unwrap().id;
        assert_eq!(store.closed_tabs(home).unwrap(), 0, "a new store has none");
        assert_eq!(store.take_closed_tab(home).unwrap(), None);

        let first = closed(0, "https://example.com/", "Example");
        let second = closed(3, "https://example.org/a?b#c", "Another “Place”");
        store
            .journal_closed_tab(home, &first, Timestamp(10))
            .unwrap();
        store
            .journal_closed_tab(home, &second, Timestamp(11))
            .unwrap();
        assert_eq!(store.closed_tabs(home).unwrap(), 2);

        // The last closed is the first back, as it was written; then the
        // one before it; then there is none, and none comes back twice.
        assert_eq!(store.take_closed_tab(home).unwrap(), Some(second));
        assert_eq!(store.closed_tabs(home).unwrap(), 1);
        assert_eq!(store.take_closed_tab(home).unwrap(), Some(first));
        assert_eq!(store.take_closed_tab(home).unwrap(), None);
        assert_eq!(store.closed_tabs(home).unwrap(), 0);
    }

    #[test]
    fn each_space_has_its_own_and_a_space_there_is_not_has_none() {
        let mut store = Store::open_in_memory().unwrap();
        let home = store.default_space(MINE, Timestamp(1)).unwrap().id;
        let work = space(&store, "Work", MINE);
        store
            .journal_closed_tab(
                home,
                &closed(0, "https://example.com/", "Home's"),
                Timestamp(10),
            )
            .unwrap();
        store
            .journal_closed_tab(
                work,
                &closed(1, "https://work.example.com/", "Work's"),
                Timestamp(11),
            )
            .unwrap();

        // The newest anywhere is Work's; Home's own is what Home is given.
        assert_eq!(
            store.take_closed_tab(home).unwrap().unwrap().title,
            "Home's"
        );
        assert_eq!(store.closed_tabs(work).unwrap(), 1, "Work's is still there");
        assert_eq!(
            store.take_closed_tab(work).unwrap().unwrap().title,
            "Work's"
        );

        // Nothing is written for a Space the store does not have.
        assert_eq!(
            store
                .journal_closed_tab(
                    SpaceId(9_999),
                    &closed(0, "https://example.com/", ""),
                    Timestamp(12)
                )
                .unwrap_err(),
            StoreError::Query
        );
        assert_eq!(store.closed_tabs(SpaceId(9_999)).unwrap(), 0);
        assert_eq!(store.take_closed_tab(SpaceId(9_999)).unwrap(), None);
    }

    #[test]
    fn only_the_newest_of_a_space_are_kept() {
        let mut store = Store::open_in_memory().unwrap();
        let home = store.default_space(MINE, Timestamp(1)).unwrap().id;
        let work = space(&store, "Work", MINE);
        store
            .journal_closed_tab(
                work,
                &closed(0, "https://work.example.com/", "Work's"),
                Timestamp(1),
            )
            .unwrap();
        for number in 0..CLOSED_TABS_KEPT + 3 {
            let tab = closed(0, &format!("https://example.com/{number}"), "");
            store.journal_closed_tab(home, &tab, Timestamp(10)).unwrap();
        }

        // The three oldest of Home's are gone; another Space's are not
        // counted among Home's, and are not tidied with them.
        assert_eq!(store.closed_tabs(home).unwrap(), CLOSED_TABS_KEPT);
        assert_eq!(store.closed_tabs(work).unwrap(), 1);
        let mut back = Vec::new();
        while let Some(tab) = store.take_closed_tab(home).unwrap() {
            back.push(tab.url);
        }
        assert_eq!(back.len(), CLOSED_TABS_KEPT);
        assert_eq!(
            back.first().map(String::as_str),
            Some(format!("https://example.com/{}", CLOSED_TABS_KEPT + 2).as_str())
        );
        assert_eq!(
            back.last().map(String::as_str),
            Some("https://example.com/3")
        );
    }

    #[test]
    fn forgetting_takes_every_space_of_those_logins_and_no_other() {
        let mut store = Store::open_in_memory().unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        let shares = space(&store, "Shares Home's logins", MINE);
        let apart = space(&store, "Its own logins", THEIRS);
        for (index, of) in [made.id, shares, apart].into_iter().enumerate() {
            let tab = closed(0, &format!("https://example.com/{index}"), "");
            store.journal_closed_tab(of, &tab, Timestamp(10)).unwrap();
        }

        store.forget_closed_tabs(made.login_set).unwrap();

        assert_eq!(store.closed_tabs(made.id).unwrap(), 0);
        assert_eq!(
            store.closed_tabs(shares).unwrap(),
            0,
            "the same logins: forgotten too"
        );
        assert_eq!(
            store.closed_tabs(apart).unwrap(),
            1,
            "other logins: not touched"
        );
        // Nothing of what was forgotten is left in the table.
        let left: i64 = store
            .connection
            .query_row("SELECT count(*) FROM journal", [], |row| row.get(0))
            .unwrap();
        assert_eq!(left, 1);
    }

    #[test]
    fn closed_tabs_survive_a_restart() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let mut store = Store::open(&path).unwrap();
        let home = store.default_space(MINE, Timestamp(1)).unwrap().id;
        let tab = closed(2, "https://example.com/", "Example");
        store.journal_closed_tab(home, &tab, Timestamp(10)).unwrap();
        // Dropped without a word, as a crash leaves it.
        drop(store);

        let mut store = Store::open(&path).unwrap();
        assert_eq!(store.take_closed_tab(home).unwrap(), Some(tab));
    }

    #[test]
    fn a_store_from_before_the_journal_gains_it_and_keeps_its_tabs() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 8 by taking
        // the journal away: what 025 left.
        let mut store = Store::open(&path).unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        let kept = crate::SavedTab {
            url: "https://example.com/".to_owned(),
            title: "Example".to_owned(),
            front: true,
            home: String::new(),
            name: String::new(),
            group: 0,
            favourite: false,
        };
        store
            .save_tabs(made.id, std::slice::from_ref(&kept))
            .unwrap();
        drop(store);
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "ALTER TABLE tab DROP COLUMN favourite; DROP TABLE node; ALTER TABLE tab DROP COLUMN site_group; ALTER TABLE tab DROP COLUMN site_home; ALTER TABLE tab DROP COLUMN site_name; DROP TABLE journal; PRAGMA user_version = 8;",
        )
        .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        assert_eq!(store.closed_tabs(made.id).unwrap(), 0);
        let tab = closed(0, "https://example.org/", "Another Place");
        store
            .journal_closed_tab(made.id, &tab, Timestamp(10))
            .unwrap();
        assert_eq!(store.take_closed_tab(made.id).unwrap(), Some(tab));
        assert_eq!(
            store.saved_tabs(made.id).unwrap(),
            [kept],
            "the tabs it had are still there"
        );
    }
}
