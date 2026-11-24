//! A site's icon, in a login set (046): one picture for a host, kept as the
//! bytes it came as. The thousand used most lately are kept.

use rusqlite::{OptionalExtension, params};
use wt_core::{LoginSetId, Timestamp};

use crate::{Store, StoreError};

/// The most icons a store keeps: the ones used most lately.
pub const MAX_ICONS: u32 = 1000;

/// The largest picture a store keeps, in bytes.
pub const MAX_ICON_BYTES: usize = 65_536;

impl Store {
    /// The picture kept as a host's icon in a login set. `None` for a host
    /// that has none there.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn icon(&self, login_set: LoginSetId, host: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.connection
            .prepare_cached("SELECT png FROM icon WHERE login_set_id = ?1 AND host = ?2")
            .and_then(|mut statement| {
                statement
                    .query_row(params![login_set.0.as_bytes().as_slice(), host], |row| {
                        row.get(0)
                    })
                    .optional()
            })
            .map_err(|_| StoreError::Query)
    }

    /// Keeps a picture as a host's icon in a login set, in place of the one
    /// it had, used now. Of the icons there then are, those used longest ago
    /// beyond [`MAX_ICONS`] are forgotten.
    ///
    /// # Errors
    /// `Query` when the database refuses, as it does a picture of no bytes
    /// or of more than [`MAX_ICON_BYTES`], a host of no name and a login set
    /// it does not have; what was kept before is then what is still there.
    pub fn set_icon(
        &mut self,
        login_set: LoginSetId,
        host: &str,
        png: &[u8],
        now: Timestamp,
    ) -> Result<(), StoreError> {
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute(
                "INSERT INTO icon (login_set_id, host, png, used_at) VALUES (?1, ?2, ?3, ?4) \
                 ON CONFLICT (login_set_id, host) \
                 DO UPDATE SET png = excluded.png, used_at = excluded.used_at",
                params![login_set.0.as_bytes().as_slice(), host, png, now.0],
            )
            .map_err(|_| StoreError::Query)?;
        // The newest by when they were used; of those used at the same time,
        // the ones written last.
        transaction
            .execute(
                "DELETE FROM icon WHERE id NOT IN \
                 (SELECT id FROM icon ORDER BY used_at DESC, id DESC LIMIT ?1)",
                params![MAX_ICONS],
            )
            .map_err(|_| StoreError::Query)?;
        transaction.commit().map_err(|_| StoreError::Query)
    }

    /// Says that hosts' icons in a login set are used now: a tab shows them.
    /// A host that has none is left with none.
    ///
    /// # Errors
    /// `Query` when the database refuses; no icon is then marked.
    pub fn use_icons(
        &mut self,
        login_set: LoginSetId,
        hosts: &[&str],
        now: Timestamp,
    ) -> Result<(), StoreError> {
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        for host in hosts {
            transaction
                .execute(
                    "UPDATE icon SET used_at = ?3 WHERE login_set_id = ?1 AND host = ?2",
                    params![login_set.0.as_bytes().as_slice(), host, now.0],
                )
                .map_err(|_| StoreError::Query)?;
        }
        transaction.commit().map_err(|_| StoreError::Query)
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

    use rusqlite::params;
    use wt_core::Uuid;

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
            .query_row("SELECT count(*) FROM icon", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn a_hosts_icon_is_kept_read_and_replaced() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        assert_eq!(store.icon(mine, "example.com").unwrap(), None);

        store
            .set_icon(mine, "example.com", b"one", Timestamp(5))
            .unwrap();
        assert_eq!(
            store.icon(mine, "example.com").unwrap().as_deref(),
            Some(&b"one"[..])
        );
        store
            .set_icon(mine, "example.com", b"another", Timestamp(6))
            .unwrap();
        assert_eq!(
            store.icon(mine, "example.com").unwrap().as_deref(),
            Some(&b"another"[..])
        );
        assert_eq!(rows(&store), 1, "in place of the one it had");
    }

    #[test]
    fn another_hosts_and_another_login_sets_are_their_own() {
        let mut store = store();
        let (mine, theirs) = (LoginSetId(MINE), LoginSetId(THEIRS));
        store
            .set_icon(mine, "example.com", b"mine", Timestamp(5))
            .unwrap();
        store
            .set_icon(theirs, "example.com", b"theirs", Timestamp(5))
            .unwrap();
        store
            .set_icon(mine, "mail.example.com", b"mail", Timestamp(5))
            .unwrap();

        assert_eq!(
            store.icon(mine, "example.com").unwrap().as_deref(),
            Some(&b"mine"[..])
        );
        assert_eq!(
            store.icon(theirs, "example.com").unwrap().as_deref(),
            Some(&b"theirs"[..]),
            "the same host in another login set has its own"
        );
        assert_eq!(
            store.icon(mine, "mail.example.com").unwrap().as_deref(),
            Some(&b"mail"[..]),
            "a host is not its site: each has its own"
        );
        assert_eq!(store.icon(theirs, "mail.example.com").unwrap(), None);
    }

    #[test]
    fn what_cannot_be_an_icon_is_refused_and_what_was_kept_stays() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        store
            .set_icon(mine, "example.com", b"kept", Timestamp(5))
            .unwrap();
        // No bytes; too many; no host; a login set there is not.
        assert_eq!(
            store.set_icon(mine, "example.com", b"", Timestamp(6)),
            Err(StoreError::Query)
        );
        let large = vec![0_u8; MAX_ICON_BYTES + 1];
        assert_eq!(
            store.set_icon(mine, "example.com", &large, Timestamp(6)),
            Err(StoreError::Query)
        );
        assert_eq!(
            store.set_icon(mine, "", b"one", Timestamp(6)),
            Err(StoreError::Query)
        );
        let nobody = LoginSetId(Uuid::from_random_bytes([0x33; 16]));
        assert_eq!(
            store.set_icon(nobody, "example.com", b"one", Timestamp(6)),
            Err(StoreError::Query)
        );
        assert_eq!(
            store.icon(mine, "example.com").unwrap().as_deref(),
            Some(&b"kept"[..])
        );
        assert_eq!(rows(&store), 1);
        // The largest there may be is kept.
        let largest = vec![7_u8; MAX_ICON_BYTES];
        store
            .set_icon(mine, "example.org", &largest, Timestamp(6))
            .unwrap();
        assert_eq!(
            store
                .icon(mine, "example.org")
                .unwrap()
                .map(|png| png.len()),
            Some(MAX_ICON_BYTES)
        );
    }

    #[test]
    fn the_thousand_used_most_lately_are_kept() {
        let mut store = store();
        let (mine, theirs) = (LoginSetId(MINE), LoginSetId(THEIRS));
        // A thousand, each used a moment after the one before; the first two are of one
        // host in two login sets.
        store
            .set_icon(mine, "first.example.com", b"mine", Timestamp(1))
            .unwrap();
        store
            .set_icon(theirs, "first.example.com", b"theirs", Timestamp(2))
            .unwrap();
        for number in 3..=i64::from(MAX_ICONS) {
            store
                .set_icon(
                    mine,
                    &format!("site{number}.example.com"),
                    b"one",
                    Timestamp(number),
                )
                .unwrap();
        }
        assert_eq!(rows(&store), i64::from(MAX_ICONS));

        // The oldest is used again: a tab shows it. A host with no icon is left with none.
        store
            .use_icons(
                mine,
                &["first.example.com", "nowhere.example.com"],
                Timestamp(5000),
            )
            .unwrap();
        assert_eq!(rows(&store), i64::from(MAX_ICONS));
        assert_eq!(store.icon(mine, "nowhere.example.com").unwrap(), None);

        // One more: the one used longest ago goes, which is now the other login set's of
        // the first host; the one that was used again stays.
        store
            .set_icon(mine, "new.example.com", b"new", Timestamp(5001))
            .unwrap();
        assert_eq!(rows(&store), i64::from(MAX_ICONS));
        assert_eq!(store.icon(theirs, "first.example.com").unwrap(), None);
        assert_eq!(
            store.icon(mine, "first.example.com").unwrap().as_deref(),
            Some(&b"mine"[..])
        );
        assert_eq!(
            store.icon(mine, "new.example.com").unwrap().as_deref(),
            Some(&b"new"[..])
        );
        // A picture put in the place of another is used now, and no icon goes for it:
        // the one used longest ago was that host's, and is now the next one's.
        store
            .set_icon(mine, "site3.example.com", b"again", Timestamp(5002))
            .unwrap();
        assert_eq!(rows(&store), i64::from(MAX_ICONS));
        assert!(store.icon(mine, "site4.example.com").unwrap().is_some());
        store
            .set_icon(mine, "newer.example.com", b"newer", Timestamp(5003))
            .unwrap();
        assert_eq!(rows(&store), i64::from(MAX_ICONS));
        assert_eq!(
            store.icon(mine, "site3.example.com").unwrap().as_deref(),
            Some(&b"again"[..])
        );
        assert_eq!(store.icon(mine, "site4.example.com").unwrap(), None);
    }

    #[test]
    fn a_store_from_before_icons_gains_them_and_keeps_what_it_had() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 12 by taking the table
        // away: what 042 left.
        let mut store = Store::open(&path).unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        store.set_zoom(made.login_set, "example.com", 125).unwrap();
        drop(store);
        let old = rusqlite::Connection::open(&path).unwrap();
        old.execute_batch("ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; PRAGMA user_version = 12;")
            .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        assert_eq!(store.icon(made.login_set, "example.com").unwrap(), None);
        store
            .set_icon(made.login_set, "example.com", b"one", Timestamp(5))
            .unwrap();
        assert_eq!(
            store
                .icon(made.login_set, "example.com")
                .unwrap()
                .as_deref(),
            Some(&b"one"[..])
        );
        assert_eq!(
            store.zoom(made.login_set, "example.com").unwrap(),
            Some(125),
            "what it had is as it was"
        );
    }

    #[test]
    fn a_login_set_that_is_taken_out_takes_its_icons() {
        let mut store = Store::open_private().unwrap();
        let (mine, theirs) = (LoginSetId(MINE), LoginSetId(THEIRS));
        store.add_login_set(mine, Timestamp(1)).unwrap();
        store.add_login_set(theirs, Timestamp(1)).unwrap();
        store
            .set_icon(mine, "example.com", b"mine", Timestamp(5))
            .unwrap();
        store
            .set_icon(theirs, "example.com", b"theirs", Timestamp(5))
            .unwrap();

        assert!(store.drop_login_set(mine).unwrap());
        assert_eq!(rows(&store), 1);
        assert_eq!(
            store.icon(theirs, "example.com").unwrap().as_deref(),
            Some(&b"theirs"[..])
        );
    }
}
