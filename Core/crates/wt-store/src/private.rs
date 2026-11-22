//! What private windows keep while they are open (037): a store that is only
//! ever in memory, and a login set that is put in it and taken out whole.
//!
//! A private window's login set has no row in the store on disk, and so can
//! have no visit, no setting, no permission and no download there: each
//! names its login set. What such a window keeps while it is open (a site's
//! zoom, what a site may use, its downloads' entries) is kept here instead,
//! by the same functions, and is gone when the login set is dropped or the
//! app ends.

use rusqlite::params;
use wt_core::{LoginSetId, Timestamp};

use crate::{Store, StoreError};

/// The number of a private store's first download, less one: far above any
/// a store on disk reaches, so that a number names one download whichever
/// store holds it.
pub const PRIVATE_DOWNLOADS_FROM: i64 = 1 << 40;

impl Store {
    /// A store for private windows: in memory, never a file.
    ///
    /// # Errors
    /// As [`Store::open`].
    pub fn open_private() -> Result<Store, StoreError> {
        let store = Store::open_in_memory()?;
        // Its visits are numbered apart too. None is ever recorded for a
        // private window; a number that named one would name no visit of a
        // store on disk.
        for table in ["download", "visit"] {
            store
                .connection
                .execute(
                    "INSERT INTO sqlite_sequence (name, seq) VALUES (?1, ?2)",
                    params![table, PRIVATE_DOWNLOADS_FROM],
                )
                .map_err(|_| StoreError::Migrate)?;
        }
        Ok(store)
    }

    /// A login set with nothing of its own yet. One that is there already
    /// stays as it is.
    ///
    /// # Errors
    /// `Query` when the store fails.
    pub fn add_login_set(
        &mut self,
        login_set: LoginSetId,
        now: Timestamp,
    ) -> Result<(), StoreError> {
        self.connection
            .execute(
                "INSERT OR IGNORE INTO login_set (id, created_at) VALUES (?1, ?2)",
                params![login_set.0.as_bytes().as_slice(), now.unix_millis()],
            )
            .map(|_| ())
            .map_err(|_| StoreError::Query)
    }

    /// Takes a login set out, with everything that names it: its visits,
    /// its sites' settings and permissions, its downloads. A login set a
    /// Space has is left alone, and so is one there is not.
    ///
    /// # Errors
    /// `Query` when the store fails; nothing is taken out then.
    pub fn drop_login_set(&mut self, login_set: LoginSetId) -> Result<bool, StoreError> {
        let id = login_set.0.as_bytes();
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        let spaces: i64 = transaction
            .query_row(
                "SELECT COUNT(*) FROM space WHERE login_set_id = ?1",
                params![id.as_slice()],
                |row| row.get(0),
            )
            .map_err(|_| StoreError::Query)?;
        if spaces > 0 {
            return Ok(false);
        }
        for table in ["visit", "site_setting", "site_permission", "download"] {
            transaction
                .execute(
                    &format!("DELETE FROM {table} WHERE login_set_id = ?1"),
                    params![id.as_slice()],
                )
                .map_err(|_| StoreError::Query)?;
        }
        let dropped = transaction
            .execute(
                "DELETE FROM login_set WHERE id = ?1",
                params![id.as_slice()],
            )
            .map_err(|_| StoreError::Query)?;
        transaction.commit().map_err(|_| StoreError::Query)?;
        Ok(dropped > 0)
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

    use wt_core::{LoginSetId, Timestamp, Uuid};

    use super::PRIVATE_DOWNLOADS_FROM;
    use crate::{Logins, Store};

    fn login_set(byte: u8) -> LoginSetId {
        LoginSetId(Uuid::from_bytes([byte; 16]))
    }

    const NOW: Timestamp = Timestamp(1_000);

    #[test]
    fn a_private_stores_downloads_are_numbered_far_above_a_real_stores() {
        let mut real = Store::open_in_memory().unwrap();
        let mut private = Store::open_private().unwrap();
        for store in [&mut real, &mut private] {
            store.add_login_set(login_set(1), NOW).unwrap();
        }
        let first = |store: &mut Store| {
            store
                .add_download(
                    login_set(1),
                    "https://example.com/a.zip",
                    "a.zip",
                    "",
                    None,
                    NOW,
                )
                .unwrap()
        };
        assert_eq!(first(&mut real), 1);
        assert_eq!(first(&mut private), PRIVATE_DOWNLOADS_FROM + 1);
        assert_eq!(first(&mut private), PRIVATE_DOWNLOADS_FROM + 2);
        // And its visits, should one ever be recorded there.
        let visit = |store: &mut Store| {
            store
                .record_visit(login_set(1), "https://example.com/", "Example", NOW)
                .unwrap()
        };
        assert_eq!(visit(&mut real), 1);
        assert_eq!(visit(&mut private), PRIVATE_DOWNLOADS_FROM + 1);
    }

    #[test]
    fn a_login_set_is_added_once_and_dropped_with_everything_of_it() {
        let mut store = Store::open_private().unwrap();
        let (mine, other) = (login_set(1), login_set(2));
        store.add_login_set(mine, NOW).unwrap();
        store.add_login_set(mine, NOW).unwrap();
        store.add_login_set(other, NOW).unwrap();
        for of in [mine, other] {
            store
                .record_visit(of, "https://example.com/", "Example", NOW)
                .unwrap();
            store.set_zoom(of, "example.com", 125).unwrap();
            store.allow(of, "https://example.com", &["camera"]).unwrap();
            store
                .add_download(of, "https://example.com/a.zip", "a.zip", "", None, NOW)
                .unwrap();
        }

        assert!(store.drop_login_set(mine).unwrap());

        // Nothing of it is left, in any table that names a login set.
        assert!(store.visits(mine, "", None, 10).unwrap().visits.is_empty());
        assert_eq!(store.zoom(mine, "example.com").unwrap(), None);
        assert!(store.all_allowed(mine).unwrap().is_empty());
        assert!(
            store
                .downloads(mine, None, 10)
                .unwrap()
                .downloads
                .is_empty()
        );
        // And it can have nothing again: it is no login set of this store's.
        assert!(store.set_zoom(mine, "example.com", 150).is_err());
        // The other's is as it was.
        assert_eq!(store.visits(other, "", None, 10).unwrap().visits.len(), 1);
        assert_eq!(store.zoom(other, "example.com").unwrap(), Some(125));
        assert_eq!(store.all_allowed(other).unwrap().len(), 1);
        assert_eq!(store.downloads(other, None, 10).unwrap().downloads.len(), 1);
        // Dropped twice, or never there: nothing to drop.
        assert!(!store.drop_login_set(mine).unwrap());
        assert!(!store.drop_login_set(login_set(9)).unwrap());
    }

    #[test]
    fn a_login_set_a_space_has_is_not_dropped() {
        let mut store = Store::open_in_memory().unwrap();
        let made = store
            .create_space("Work", Logins::Own(Uuid::from_bytes([3; 16])), "", NOW)
            .unwrap()
            .expect("a Space");
        store.set_zoom(made.login_set, "example.com", 125).unwrap();

        assert!(!store.drop_login_set(made.login_set).unwrap());

        assert_eq!(
            store.zoom(made.login_set, "example.com").unwrap(),
            Some(125)
        );
        assert!(
            store
                .spaces()
                .unwrap()
                .iter()
                .any(|space| space.id == made.id)
        );
    }
}
