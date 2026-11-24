//! What a site is remembered by, in a login set (012): its zoom, so far.

use rusqlite::{OptionalExtension, params};
use wt_core::LoginSetId;

use crate::{Store, StoreError};

/// Actual size: what a site nobody zoomed has, and what is written as no
/// row at all.
const ACTUAL_SIZE: u32 = 100;

impl Store {
    /// The zoom a site was given in a login set, in per cent. `None` for a
    /// site nobody zoomed.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn zoom(&self, login_set: LoginSetId, site: &str) -> Result<Option<u32>, StoreError> {
        self.connection
            .prepare_cached("SELECT zoom FROM site_setting WHERE login_set_id = ?1 AND site = ?2")
            .and_then(|mut statement| {
                statement
                    .query_row(params![login_set.0.as_bytes().as_slice(), site], |row| {
                        row.get(0)
                    })
                    .optional()
            })
            .map_err(|_| StoreError::Query)
    }

    /// Gives a site a zoom in a login set, in place of the one it had. 100
    /// is no zoom, and leaves nothing written of the site.
    ///
    /// # Errors
    /// `Query` when the database refuses, as it does a zoom outside 25 to
    /// 500 and a login set it does not have; what was written before is
    /// then what is still there.
    pub fn set_zoom(
        &mut self,
        login_set: LoginSetId,
        site: &str,
        percent: u32,
    ) -> Result<(), StoreError> {
        let login_set = login_set.0.as_bytes().as_slice();
        let written = if percent == ACTUAL_SIZE {
            self.connection.execute(
                "DELETE FROM site_setting WHERE login_set_id = ?1 AND site = ?2",
                params![login_set, site],
            )
        } else {
            self.connection.execute(
                "INSERT INTO site_setting (login_set_id, site, zoom) VALUES (?1, ?2, ?3) \
                 ON CONFLICT (login_set_id, site) DO UPDATE SET zoom = excluded.zoom",
                params![login_set, site, percent],
            )
        };
        written.map(|_| ()).map_err(|_| StoreError::Query)
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use wt_core::{Timestamp, Uuid};

    use super::*;

    const MINE: Uuid = Uuid::from_random_bytes([0x11; 16]);
    const THEIRS: Uuid = Uuid::from_random_bytes([0x22; 16]);

    fn store() -> Store {
        let mut store = Store::open_in_memory().unwrap();
        store.default_space(MINE, Timestamp(1)).unwrap();
        store
            .connection
            .execute(
                "INSERT INTO login_set (id, created_at) VALUES (?1, 1)",
                params![THEIRS.as_bytes().as_slice()],
            )
            .unwrap();
        store
    }

    fn rows(store: &Store) -> i64 {
        store
            .connection
            .query_row("SELECT count(*) FROM site_setting", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn a_sites_zoom_is_written_read_and_replaced() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        assert_eq!(store.zoom(mine, "example.com").unwrap(), None);

        store.set_zoom(mine, "example.com", 125).unwrap();
        assert_eq!(store.zoom(mine, "example.com").unwrap(), Some(125));

        store.set_zoom(mine, "example.com", 80).unwrap();
        assert_eq!(store.zoom(mine, "example.com").unwrap(), Some(80));
        assert_eq!(rows(&store), 1, "in place of the one it had");
    }

    #[test]
    fn actual_size_leaves_nothing_written_of_the_site() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        store.set_zoom(mine, "example.com", 150).unwrap();
        store.set_zoom(mine, "example.org", 90).unwrap();

        store.set_zoom(mine, "example.com", 100).unwrap();

        assert_eq!(store.zoom(mine, "example.com").unwrap(), None);
        assert_eq!(rows(&store), 1, "the site is not in the store at all");
        // And a site that was never zoomed is not written by being put at 100.
        store.set_zoom(mine, "example.net", 100).unwrap();
        assert_eq!(rows(&store), 1);
    }

    #[test]
    fn another_sites_and_another_login_sets_are_not_touched() {
        let mut store = store();
        let (mine, theirs) = (LoginSetId(MINE), LoginSetId(THEIRS));
        store.set_zoom(mine, "example.com", 125).unwrap();
        store.set_zoom(theirs, "example.com", 200).unwrap();
        store.set_zoom(mine, "example.org", 75).unwrap();

        assert_eq!(store.zoom(mine, "example.com").unwrap(), Some(125));
        assert_eq!(store.zoom(theirs, "example.com").unwrap(), Some(200));
        assert_eq!(store.zoom(mine, "example.org").unwrap(), Some(75));
        assert_eq!(store.zoom(theirs, "example.org").unwrap(), None);

        store.set_zoom(mine, "example.com", 100).unwrap();
        assert_eq!(
            store.zoom(theirs, "example.com").unwrap(),
            Some(200),
            "the same site in another login set keeps its own"
        );
    }

    #[test]
    fn a_zoom_the_store_cannot_mean_is_refused_and_changes_nothing() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        store.set_zoom(mine, "example.com", 125).unwrap();

        assert_eq!(
            store.set_zoom(mine, "example.com", 0),
            Err(StoreError::Query)
        );
        assert_eq!(
            store.set_zoom(mine, "example.com", 5_000),
            Err(StoreError::Query)
        );
        // A login set the store does not have can have no settings.
        let nobody = LoginSetId(Uuid::from_random_bytes([0x33; 16]));
        assert_eq!(
            store.set_zoom(nobody, "example.com", 125),
            Err(StoreError::Query)
        );

        assert_eq!(store.zoom(mine, "example.com").unwrap(), Some(125));
        assert_eq!(rows(&store), 1);
    }

    #[test]
    fn a_store_from_before_zoom_gains_it_and_keeps_what_it_had() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 3 by taking
        // the table away: what 009 left.
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
        old.execute_batch("ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; ALTER TABLE tab DROP COLUMN favourite; DROP TABLE node; ALTER TABLE tab DROP COLUMN site_group; ALTER TABLE tab DROP COLUMN site_home; ALTER TABLE tab DROP COLUMN site_name; DROP TABLE journal; ALTER TABLE space DROP COLUMN colour; DROP TABLE setting; DROP TABLE site_permission; DROP TABLE download; DROP TABLE site_setting; PRAGMA user_version = 3;")
            .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        store.set_zoom(made.login_set, "example.com", 110).unwrap();
        assert_eq!(
            store.zoom(made.login_set, "example.com").unwrap(),
            Some(110)
        );
        let history = store.visits(made.login_set, "", None, 10).unwrap();
        assert_eq!(history.visits.len(), 1, "what was visited is still there");
        drop(store);

        // And it is there the next time the store is opened.
        let store = Store::open(&path).unwrap();
        assert_eq!(
            store.zoom(made.login_set, "example.com").unwrap(),
            Some(110)
        );
    }
}
