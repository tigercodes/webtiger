//! Tidying (049): each Space's time, and the Archive its tidied tabs are
//! listed in until they are brought back or forgotten.
//!
//! What is tidied and when is the core's rule: the store keeps a Space's
//! time, and keeps of the Archive the newest of each Space.

use rusqlite::{OptionalExtension, params};
use wt_core::SpaceId;

use crate::{Store, StoreError};

/// The most tabs the Archive keeps of one Space: the oldest go as new ones
/// come (049 D6).
pub const MAX_ARCHIVED: usize = 1000;

/// A tab that was tidied, as the Archive lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Archived {
    /// Its number in the Archive. Never used twice: a newer one is larger.
    pub id: i64,
    pub url: String,
    pub title: String,
    /// When it was tidied, in milliseconds.
    pub tidied_at: i64,
}

fn key(space: SpaceId) -> Result<i64, StoreError> {
    i64::try_from(space.get()).map_err(|_| StoreError::Query)
}

impl Store {
    /// Each Space's time, in seconds: after how long out of sight its tabs
    /// that are no Sites are tidied. A Space with none (never) is not
    /// listed.
    ///
    /// # Errors
    /// `Query` when the database refuses, `Corrupt` when a row is not what
    /// the schema says.
    pub fn tidy_times(&self) -> Result<Vec<(SpaceId, u32)>, StoreError> {
        let rows: Vec<(i64, i64)> = self
            .connection
            .prepare_cached("SELECT id, tidy_after FROM space WHERE tidy_after > 0 ORDER BY id")
            .and_then(|mut statement| {
                statement
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                    .collect()
            })
            .map_err(|_| StoreError::Query)?;
        rows.into_iter()
            .map(|(id, after)| {
                Ok((
                    SpaceId(u64::try_from(id).map_err(|_| StoreError::Corrupt)?),
                    u32::try_from(after).map_err(|_| StoreError::Corrupt)?,
                ))
            })
            .collect()
    }

    /// Gives a Space its time, in seconds; nought for never. False when
    /// there is no such Space. Which times there are is the core's rule.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn set_tidy_time(&mut self, space: SpaceId, seconds: u32) -> Result<bool, StoreError> {
        let Ok(id) = i64::try_from(space.get()) else {
            return Ok(false);
        };
        let set = self
            .connection
            .execute(
                "UPDATE space SET tidy_after = ?1 WHERE id = ?2",
                params![seconds, id],
            )
            .map_err(|_| StoreError::Query)?;
        Ok(set > 0)
    }

    /// Lists a tidied tab in its Space's Archive, and keeps of that Space
    /// the [`MAX_ARCHIVED`] newest.
    ///
    /// # Errors
    /// `Query` when the database refuses: a Space there is not, or a tab
    /// with no address. Nothing is then listed.
    pub fn archive(
        &mut self,
        space: SpaceId,
        url: &str,
        title: &str,
        tidied_at: i64,
    ) -> Result<(), StoreError> {
        let space = key(space)?;
        let most = i64::try_from(MAX_ARCHIVED).map_err(|_| StoreError::Query)?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute(
                "INSERT INTO archive (space_id, url, title, tidied_at) VALUES (?1, ?2, ?3, ?4)",
                params![space, url, title, tidied_at],
            )
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute(
                "DELETE FROM archive WHERE space_id = ?1 AND id NOT IN \
                 (SELECT id FROM archive WHERE space_id = ?1 ORDER BY id DESC LIMIT ?2)",
                params![space, most],
            )
            .map_err(|_| StoreError::Query)?;
        transaction.commit().map_err(|_| StoreError::Query)
    }

    /// A page of a Space's Archive, newest first: at most `limit`, those
    /// older than `before` (nought for the newest); and whether there are
    /// older ones still.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn archived(
        &self,
        space: SpaceId,
        before: i64,
        limit: usize,
    ) -> Result<(Vec<Archived>, bool), StoreError> {
        let space = key(space)?;
        let asked = i64::try_from(limit.saturating_add(1)).map_err(|_| StoreError::Query)?;
        let before = if before > 0 { before } else { i64::MAX };
        let mut entries: Vec<Archived> = self
            .connection
            .prepare_cached(
                "SELECT id, url, title, tidied_at FROM archive \
                 WHERE space_id = ?1 AND id < ?2 ORDER BY id DESC LIMIT ?3",
            )
            .and_then(|mut statement| {
                statement
                    .query_map(params![space, before, asked], |row| {
                        Ok(Archived {
                            id: row.get(0)?,
                            url: row.get(1)?,
                            title: row.get(2)?,
                            tidied_at: row.get(3)?,
                        })
                    })?
                    .collect()
            })
            .map_err(|_| StoreError::Query)?;
        let more = entries.len() > limit;
        entries.truncate(limit);
        Ok((entries, more))
    }

    /// Takes a tab out of a Space's Archive, to be brought back: what was
    /// listed, which is listed no longer. `None` when the Space's Archive
    /// has no such tab.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn take_archived(
        &mut self,
        space: SpaceId,
        id: i64,
    ) -> Result<Option<Archived>, StoreError> {
        let space = key(space)?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        let found = transaction
            .query_row(
                "SELECT id, url, title, tidied_at FROM archive WHERE space_id = ?1 AND id = ?2",
                params![space, id],
                |row| {
                    Ok(Archived {
                        id: row.get(0)?,
                        url: row.get(1)?,
                        title: row.get(2)?,
                        tidied_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(|_| StoreError::Query)?;
        if found.is_some() {
            transaction
                .execute(
                    "DELETE FROM archive WHERE space_id = ?1 AND id = ?2",
                    params![space, id],
                )
                .map_err(|_| StoreError::Query)?;
        }
        transaction.commit().map_err(|_| StoreError::Query)?;
        Ok(found)
    }

    /// Forgets a tab of a Space's Archive, or with `None` all of them. How
    /// many were forgotten.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_archived(
        &mut self,
        space: SpaceId,
        id: Option<i64>,
    ) -> Result<usize, StoreError> {
        let space = key(space)?;
        match id {
            Some(id) => self.connection.execute(
                "DELETE FROM archive WHERE space_id = ?1 AND id = ?2",
                params![space, id],
            ),
            None => self
                .connection
                .execute("DELETE FROM archive WHERE space_id = ?1", params![space]),
        }
        .map_err(|_| StoreError::Query)
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "a test fails the way its assertions do"
    )]

    use rusqlite::Connection;
    use wt_core::{Timestamp, Uuid};

    use super::*;
    use crate::space::Logins;

    const FIRST: Uuid = Uuid::from_random_bytes([0x11; 16]);
    const WORK: Uuid = Uuid::from_random_bytes([0x22; 16]);

    fn store() -> (Store, SpaceId) {
        let mut store = Store::open_in_memory().unwrap();
        let first = store.default_space(FIRST, Timestamp(1)).unwrap();
        (store, first.id)
    }

    fn second(store: &mut Store) -> SpaceId {
        store
            .create_space("Work", Logins::Own(WORK), "", "", Timestamp(2))
            .unwrap()
            .unwrap()
            .id
    }

    fn urls(page: &(Vec<Archived>, bool)) -> Vec<&str> {
        page.0.iter().map(|entry| entry.url.as_str()).collect()
    }

    #[test]
    fn a_space_has_no_time_until_it_is_given_one() {
        let (mut store, home) = store();
        let work = second(&mut store);
        assert_eq!(store.tidy_times().unwrap(), []);
        assert!(store.set_tidy_time(work, 43_200).unwrap());
        assert_eq!(store.tidy_times().unwrap(), [(work, 43_200)]);
        assert!(store.set_tidy_time(home, 86_400).unwrap());
        assert_eq!(
            store.tidy_times().unwrap(),
            [(home, 86_400), (work, 43_200)]
        );
        // Never again: it is not listed.
        assert!(store.set_tidy_time(work, 0).unwrap());
        assert_eq!(store.tidy_times().unwrap(), [(home, 86_400)]);
        // A Space there is not has no time to be given.
        assert!(!store.set_tidy_time(SpaceId(99), 43_200).unwrap());
        assert_eq!(store.tidy_times().unwrap(), [(home, 86_400)]);
    }

    #[test]
    fn a_tidied_tab_is_listed_newest_first_in_its_own_spaces_archive() {
        let (mut store, home) = store();
        let work = second(&mut store);
        store
            .archive(home, "https://a.example.com/", "A", 10)
            .unwrap();
        store
            .archive(work, "https://w.example.com/", "W", 11)
            .unwrap();
        store
            .archive(home, "https://b.example.com/", "", 12)
            .unwrap();
        let listed = store.archived(home, 0, 10).unwrap();
        assert_eq!(
            urls(&listed),
            ["https://b.example.com/", "https://a.example.com/"]
        );
        assert!(!listed.1);
        assert_eq!(listed.0[1].title, "A");
        assert_eq!(listed.0[1].tidied_at, 10);
        assert!(
            listed.0[0].id > listed.0[1].id,
            "a newer one has a larger number"
        );
        assert_eq!(
            urls(&store.archived(work, 0, 10).unwrap()),
            ["https://w.example.com/"]
        );
        // A tab with no address, and a Space there is not, are not listed.
        assert_eq!(
            store.archive(home, "", "Nothing", 13),
            Err(StoreError::Query)
        );
        assert_eq!(
            store.archive(SpaceId(99), "https://c.example.com/", "C", 14),
            Err(StoreError::Query)
        );
        assert_eq!(store.archived(home, 0, 10).unwrap().0.len(), 2);
        assert_eq!(
            store.archived(SpaceId(99), 0, 10).unwrap(),
            (Vec::new(), false)
        );
    }

    #[test]
    fn the_archive_is_read_a_page_at_a_time() {
        let (mut store, home) = store();
        for n in 1..=5 {
            store
                .archive(home, &format!("https://example.com/{n}"), "", n)
                .unwrap();
        }
        let first = store.archived(home, 0, 2).unwrap();
        assert_eq!(
            urls(&first),
            ["https://example.com/5", "https://example.com/4"]
        );
        assert!(first.1, "there are older ones");
        let next = store.archived(home, first.0[1].id, 2).unwrap();
        assert_eq!(
            urls(&next),
            ["https://example.com/3", "https://example.com/2"]
        );
        assert!(next.1);
        let last = store.archived(home, next.0[1].id, 2).unwrap();
        assert_eq!(urls(&last), ["https://example.com/1"]);
        assert!(!last.1, "and then no more");
        // A page of exactly what is left says there is no more.
        let all = store.archived(home, 0, 5).unwrap();
        assert_eq!(all.0.len(), 5);
        assert!(!all.1);
        assert_eq!(store.archived(home, 0, 0).unwrap(), (Vec::new(), true));
    }

    #[test]
    fn a_tab_taken_out_is_listed_no_longer_and_one_forgotten_is_gone() {
        let (mut store, home) = store();
        let work = second(&mut store);
        for (space, url) in [(home, "a"), (home, "b"), (home, "c"), (work, "w")] {
            store
                .archive(space, &format!("https://example.com/{url}"), url, 1)
                .unwrap();
        }
        let listed = store.archived(home, 0, 10).unwrap().0;
        let (c, b, a) = (listed[0].id, listed[1].id, listed[2].id);
        // Taken out: what was listed, once.
        let taken = store.take_archived(home, b).unwrap().unwrap();
        assert_eq!(
            (taken.url.as_str(), taken.title.as_str()),
            ("https://example.com/b", "b")
        );
        assert_eq!(store.take_archived(home, b).unwrap(), None);
        // Another Space's tab is not this Space's to take, or to forget.
        assert_eq!(store.take_archived(work, a).unwrap(), None);
        assert_eq!(store.forget_archived(work, Some(a)).unwrap(), 0);
        assert_eq!(
            urls(&store.archived(home, 0, 10).unwrap()),
            ["https://example.com/c", "https://example.com/a"]
        );
        // Forgotten: one, and then all of a Space's; the other Space's stay.
        assert_eq!(store.forget_archived(home, Some(c)).unwrap(), 1);
        assert_eq!(store.forget_archived(home, Some(c)).unwrap(), 0);
        assert_eq!(
            urls(&store.archived(home, 0, 10).unwrap()),
            ["https://example.com/a"]
        );
        store
            .archive(home, "https://example.com/d", "d", 2)
            .unwrap();
        assert_eq!(store.forget_archived(home, None).unwrap(), 2);
        assert_eq!(store.archived(home, 0, 10).unwrap(), (Vec::new(), false));
        assert_eq!(
            urls(&store.archived(work, 0, 10).unwrap()),
            ["https://example.com/w"]
        );
    }

    #[test]
    fn a_space_keeps_its_thousand_newest() {
        let (mut store, home) = store();
        let work = second(&mut store);
        store
            .archive(work, "https://example.com/work", "", 1)
            .unwrap();
        for n in 1..=MAX_ARCHIVED + 3 {
            store
                .archive(home, &format!("https://example.com/{n}"), "", 1)
                .unwrap();
        }
        let kept = store.archived(home, 0, MAX_ARCHIVED + 10).unwrap();
        assert_eq!(kept.0.len(), MAX_ARCHIVED);
        assert_eq!(
            kept.0[0].url,
            format!("https://example.com/{}", MAX_ARCHIVED + 3)
        );
        assert_eq!(
            kept.0[MAX_ARCHIVED - 1].url,
            "https://example.com/4",
            "the three oldest went"
        );
        // Another Space's are not counted with them.
        assert_eq!(store.archived(work, 0, 10).unwrap().0.len(), 1);
    }

    #[test]
    fn a_store_from_before_tidying_gains_it_and_keeps_what_it_had() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite3");
        let home = {
            let mut store = Store::open(&path).unwrap();
            let first = store.default_space(FIRST, Timestamp(1)).unwrap();
            store
                .save_tabs(
                    first.id,
                    &[crate::SavedTab {
                        url: "https://example.com/".to_owned(),
                        title: "Example".to_owned(),
                        front: true,
                        home: String::new(),
                        name: String::new(),
                        group: 0,
                        favourite: false,
                        shown_at: 77,
                        state: 0,
                    }],
                )
                .unwrap();
            first.id
        };
        {
            let old = Connection::open(&path).unwrap();
            old.execute_batch(
                "ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; \
                 ALTER TABLE space DROP COLUMN tidy_after; PRAGMA user_version = 13;",
            )
            .unwrap();
        }
        let mut store = Store::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        // Its Space has no time, its tab is there and is not known to have been in front,
        // and its Archive is empty and can be written.
        assert_eq!(store.tidy_times().unwrap(), []);
        let tabs = store.saved_tabs(home).unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!(
            (tabs[0].url.as_str(), tabs[0].shown_at),
            ("https://example.com/", 0)
        );
        assert_eq!(store.archived(home, 0, 10).unwrap(), (Vec::new(), false));
        store
            .archive(home, "https://example.com/", "Example", 5)
            .unwrap();
        assert_eq!(store.archived(home, 0, 10).unwrap().0.len(), 1);
    }
}
