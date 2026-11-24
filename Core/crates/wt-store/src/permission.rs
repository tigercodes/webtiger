//! What a site was allowed to use, in a login set (014): the camera, the
//! microphone, the device's location. By origin, a kind to a row. The
//! store keeps what it is given: which kinds there are is the core's to
//! know.

use rusqlite::params;
use wt_core::LoginSetId;

use crate::{Store, StoreError};

/// An origin with what it was allowed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allowed {
    /// Scheme, host and port: `https://example.com`.
    pub origin: String,
    /// The kinds it may use, in the order of their names.
    pub kinds: Vec<String>,
}

impl Store {
    /// The kinds an origin was allowed in a login set, in the order of
    /// their names. None for an origin nobody was asked about.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn allowed(&self, login_set: LoginSetId, origin: &str) -> Result<Vec<String>, StoreError> {
        self.connection
            .prepare_cached(
                "SELECT kind FROM site_permission \
                 WHERE login_set_id = ?1 AND origin = ?2 AND verdict = 'allow' ORDER BY kind",
            )
            .and_then(|mut statement| {
                statement
                    .query_map(params![login_set.0.as_bytes().as_slice(), origin], |row| {
                        row.get(0)
                    })?
                    .collect()
            })
            .map_err(|_| StoreError::Query)
    }

    /// Writes down that an origin may use each of these kinds, in a login
    /// set. All of them or none: what was allowed before stays.
    ///
    /// # Errors
    /// `Query` when the database refuses, as it does a login set it does
    /// not have.
    pub fn allow(
        &mut self,
        login_set: LoginSetId,
        origin: &str,
        kinds: &[&str],
    ) -> Result<(), StoreError> {
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        for kind in kinds {
            transaction
                .execute(
                    "INSERT INTO site_permission (login_set_id, origin, kind, verdict) \
                     VALUES (?1, ?2, ?3, 'allow') \
                     ON CONFLICT (login_set_id, origin, kind) DO UPDATE SET verdict = 'allow'",
                    params![login_set.0.as_bytes().as_slice(), origin, kind],
                )
                .map_err(|_| StoreError::Query)?;
        }
        transaction.commit().map_err(|_| StoreError::Query)
    }

    /// Every origin that was allowed something in a login set, with what,
    /// in the order of the origins.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn all_allowed(&self, login_set: LoginSetId) -> Result<Vec<Allowed>, StoreError> {
        let rows: Vec<(String, String)> = self
            .connection
            .prepare_cached(
                "SELECT origin, kind FROM site_permission \
                 WHERE login_set_id = ?1 AND verdict = 'allow' ORDER BY origin, kind",
            )
            .and_then(|mut statement| {
                statement
                    .query_map(params![login_set.0.as_bytes().as_slice()], |row| {
                        Ok((row.get(0)?, row.get(1)?))
                    })?
                    .collect()
            })
            .map_err(|_| StoreError::Query)?;
        let mut all: Vec<Allowed> = Vec::new();
        for (origin, kind) in rows {
            match all.last_mut() {
                Some(last) if last.origin == origin => last.kinds.push(kind),
                _ => all.push(Allowed {
                    origin,
                    kinds: vec![kind],
                }),
            }
        }
        Ok(all)
    }

    /// Takes back everything an origin was allowed in a login set, and
    /// leaves no trace of it in the store's files. False when there was
    /// nothing.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_permissions(
        &mut self,
        login_set: LoginSetId,
        origin: &str,
    ) -> Result<bool, StoreError> {
        let forgotten = self
            .connection
            .execute(
                "DELETE FROM site_permission WHERE login_set_id = ?1 AND origin = ?2",
                params![login_set.0.as_bytes().as_slice(), origin],
            )
            .map_err(|_| StoreError::Query)?;
        self.leave_no_trace()?;
        Ok(forgotten > 0)
    }

    /// Takes back everything every origin was allowed in a login set.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_all_permissions(&mut self, login_set: LoginSetId) -> Result<(), StoreError> {
        self.connection
            .execute(
                "DELETE FROM site_permission WHERE login_set_id = ?1",
                params![login_set.0.as_bytes().as_slice()],
            )
            .map_err(|_| StoreError::Query)?;
        self.leave_no_trace()
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use wt_core::{Timestamp, Uuid};

    use super::*;

    const MINE: Uuid = Uuid::from_random_bytes([0x11; 16]);
    const THEIRS: Uuid = Uuid::from_random_bytes([0x22; 16]);
    const MEET: &str = "https://meet.example.com";
    const MAPS: &str = "https://maps.example.org";

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
            .query_row("SELECT count(*) FROM site_permission", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn what_an_origin_was_allowed_is_written_and_read_back() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        assert!(store.allowed(mine, MEET).unwrap().is_empty());

        store.allow(mine, MEET, &["microphone", "camera"]).unwrap();

        assert_eq!(
            store.allowed(mine, MEET).unwrap(),
            ["camera", "microphone"],
            "in the order of their names"
        );
        // Allowed again, and one more: a kind is written once.
        store.allow(mine, MEET, &["camera", "location"]).unwrap();
        assert_eq!(
            store.allowed(mine, MEET).unwrap(),
            ["camera", "location", "microphone"]
        );
        assert_eq!(rows(&store), 3);
    }

    #[test]
    fn an_origin_is_itself_and_no_other() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        store.allow(mine, MEET, &["camera"]).unwrap();

        // Not its site's other names, not another scheme, not another port.
        for other in [
            "https://example.com",
            "https://ads.example.com",
            "http://meet.example.com",
            "https://meet.example.com:8443",
            "https://meet.example.com/",
        ] {
            assert!(store.allowed(mine, other).unwrap().is_empty(), "{other}");
        }
    }

    #[test]
    fn another_login_set_was_never_asked() {
        let mut store = store();
        let (mine, theirs) = (LoginSetId(MINE), LoginSetId(THEIRS));
        store.allow(mine, MEET, &["camera"]).unwrap();

        assert!(store.allowed(theirs, MEET).unwrap().is_empty());
        assert!(store.all_allowed(theirs).unwrap().is_empty());

        store.allow(theirs, MEET, &["microphone"]).unwrap();
        assert_eq!(store.allowed(mine, MEET).unwrap(), ["camera"]);
        assert_eq!(store.allowed(theirs, MEET).unwrap(), ["microphone"]);
    }

    #[test]
    fn the_origins_are_listed_in_order_each_with_what_it_may_use() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        store.allow(mine, MEET, &["microphone", "camera"]).unwrap();
        store.allow(mine, MAPS, &["location"]).unwrap();
        store
            .allow(LoginSetId(THEIRS), "https://else.example.net", &["camera"])
            .unwrap();

        assert_eq!(
            store.all_allowed(mine).unwrap(),
            [
                Allowed {
                    origin: MAPS.into(),
                    kinds: vec!["location".into()]
                },
                Allowed {
                    origin: MEET.into(),
                    kinds: vec!["camera".into(), "microphone".into()]
                },
            ]
        );
    }

    #[test]
    fn taking_back_is_one_origins_or_a_whole_login_sets() {
        let mut store = store();
        let (mine, theirs) = (LoginSetId(MINE), LoginSetId(THEIRS));
        store.allow(mine, MEET, &["camera", "microphone"]).unwrap();
        store.allow(mine, MAPS, &["location"]).unwrap();
        store.allow(theirs, MEET, &["camera"]).unwrap();

        assert!(store.forget_permissions(mine, MEET).unwrap());
        assert!(!store.forget_permissions(mine, MEET).unwrap(), "it is gone");

        assert!(store.allowed(mine, MEET).unwrap().is_empty());
        assert_eq!(store.allowed(mine, MAPS).unwrap(), ["location"]);
        assert_eq!(
            store.allowed(theirs, MEET).unwrap(),
            ["camera"],
            "the same origin in another login set keeps its own"
        );

        store.forget_all_permissions(mine).unwrap();
        assert!(store.all_allowed(mine).unwrap().is_empty());
        assert_eq!(store.all_allowed(theirs).unwrap().len(), 1);
        assert_eq!(rows(&store), 1);
    }

    #[test]
    fn a_refusal_is_not_an_allowance() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        // Nothing writes one yet (014 §5): it is put there as a later build
        // would, and must not read as "allowed" to this one.
        let write = |store: &Store, verdict: &str| {
            store.connection.execute(
                "INSERT INTO site_permission (login_set_id, origin, kind, verdict) \
                 VALUES (?1, ?2, 'camera', ?3)",
                params![MINE.as_bytes().as_slice(), MEET, verdict],
            )
        };
        write(&store, "refuse").unwrap();

        assert!(store.allowed(mine, MEET).unwrap().is_empty());
        assert!(store.all_allowed(mine).unwrap().is_empty());

        // Allowed after all, the same row says so.
        store.allow(mine, MEET, &["camera"]).unwrap();
        assert_eq!(store.allowed(mine, MEET).unwrap(), ["camera"]);
        assert_eq!(rows(&store), 1);
        // And a verdict that is neither is not written at all.
        store.forget_all_permissions(mine).unwrap();
        assert!(write(&store, "perhaps").is_err());
        assert_eq!(rows(&store), 0);
    }

    #[test]
    fn nothing_is_written_for_a_login_set_the_store_does_not_have() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        store.allow(mine, MEET, &["camera"]).unwrap();
        let nobody = LoginSetId(Uuid::from_random_bytes([0x33; 16]));

        assert_eq!(
            store.allow(nobody, MEET, &["camera", "microphone"]),
            Err(StoreError::Query)
        );

        assert_eq!(rows(&store), 1, "nothing of it is written");
    }

    #[test]
    fn a_store_from_before_permissions_gains_them_and_keeps_what_it_had() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 5 by taking
        // the table away: what 013 left.
        let mut store = Store::open(&path).unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        store.set_zoom(made.login_set, "example.com", 125).unwrap();
        drop(store);
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; ALTER TABLE tab DROP COLUMN favourite; DROP TABLE node; ALTER TABLE tab DROP COLUMN site_group; ALTER TABLE tab DROP COLUMN site_home; ALTER TABLE tab DROP COLUMN site_name; DROP TABLE journal; ALTER TABLE space DROP COLUMN colour; DROP TABLE setting; DROP TABLE site_permission; PRAGMA user_version = 5;",
        )
        .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        store.allow(made.login_set, MEET, &["camera"]).unwrap();
        assert_eq!(store.allowed(made.login_set, MEET).unwrap(), ["camera"]);
        assert_eq!(
            store.zoom(made.login_set, "example.com").unwrap(),
            Some(125),
            "what was there is still there"
        );
        drop(store);

        // And what was allowed is there when the store is opened again.
        let store = Store::open(&path).unwrap();
        assert_eq!(store.allowed(made.login_set, MEET).unwrap(), ["camera"]);
    }

    #[test]
    fn what_was_taken_back_is_not_left_in_the_stores_files() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let mut store = Store::open(&path).unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        let origin = "https://a-name-to-look-for.example.com";
        store.allow(made.login_set, origin, &["camera"]).unwrap();

        store.forget_permissions(made.login_set, origin).unwrap();

        for entry in std::fs::read_dir(folder.path()).unwrap() {
            let file = entry.unwrap().path();
            let bytes = std::fs::read(&file).unwrap();
            let found = bytes
                .windows(b"a-name-to-look-for".len())
                .any(|window| window == b"a-name-to-look-for");
            assert!(!found, "{} still names the origin", file.display());
        }
    }
}
