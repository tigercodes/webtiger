//! Several Spaces (024 §5): listing them in their order, making one, and
//! renaming one. The schema has had what this needs from its first version:
//! many Spaces may name one login set (000 WT-D9).
//!
//! What a name may be, and how many Spaces there may be, are the core's
//! rules: the store keeps what it is given.

use rusqlite::{OptionalExtension, Row, params};
use wt_core::{LoginSetId, SpaceId, Timestamp, Uuid};

use crate::{Space, Store, StoreError};

/// Whose logins a new Space has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logins {
    /// Its own: a new login set, by this id.
    Own(Uuid),
    /// Those of a Space there is.
    Of(SpaceId),
}

/// The setting that says which Space the window shows.
const SHOWN_SPACE: &str = "session.space";

/// A row of `space` as `SELECT id, name, login_set_id, colour, icon` gives it.
type SpaceRow = (i64, String, Vec<u8>, String, String);

fn row(row: &Row<'_>) -> rusqlite::Result<SpaceRow> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn space((id, name, login_set, colour, icon): SpaceRow) -> Result<Space, StoreError> {
    let bytes: [u8; 16] = login_set.try_into().map_err(|_| StoreError::Corrupt)?;
    Ok(Space {
        id: SpaceId(u64::try_from(id).map_err(|_| StoreError::Corrupt)?),
        name,
        login_set: LoginSetId(Uuid::from_bytes(bytes)),
        colour,
        icon,
    })
}

impl Store {
    /// Every Space, in their order.
    ///
    /// # Errors
    /// `Query` when the database refuses, `Corrupt` when a row is not what
    /// the schema says.
    pub fn spaces(&self) -> Result<Vec<Space>, StoreError> {
        let rows: Vec<SpaceRow> = self
            .connection
            .prepare_cached(
                "SELECT id, name, login_set_id, colour, icon FROM space ORDER BY position, id",
            )
            .and_then(|mut statement| statement.query_map([], row)?.collect())
            .map_err(|_| StoreError::Query)?;
        rows.into_iter().map(space).collect()
    }

    /// Makes a Space called `name`, after the others, with logins of its
    /// own or those of a Space there is, the colour `colour` (the core's name
    /// for one, or empty for none) and the icon `icon` (an emoji of the core's
    /// list, or empty for none). `None`, with nothing made, when the Space
    /// whose logins it was to share is not there.
    ///
    /// # Errors
    /// `Query` when the database refuses, `Corrupt` when a row is not what
    /// the schema says.
    pub fn create_space(
        &mut self,
        name: &str,
        logins: Logins,
        colour: &str,
        icon: &str,
        now: Timestamp,
    ) -> Result<Option<Space>, StoreError> {
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        let login_set: Vec<u8> = match logins {
            Logins::Own(new) => {
                transaction
                    .execute(
                        "INSERT INTO login_set (id, created_at) VALUES (?1, ?2)",
                        params![new.as_bytes().as_slice(), now.unix_millis()],
                    )
                    .map_err(|_| StoreError::Query)?;
                new.as_bytes().to_vec()
            }
            Logins::Of(shared) => {
                let Ok(id) = i64::try_from(shared.0) else {
                    return Ok(None);
                };
                let theirs = transaction
                    .query_row(
                        "SELECT login_set_id FROM space WHERE id = ?1",
                        params![id],
                        |row| row.get::<_, Vec<u8>>(0),
                    )
                    .optional()
                    .map_err(|_| StoreError::Query)?;
                let Some(theirs) = theirs else {
                    return Ok(None);
                };
                theirs
            }
        };
        transaction
            .execute(
                "INSERT INTO space (name, position, login_set_id, created_at, colour, icon) \
                 VALUES (?1, (SELECT COALESCE(MAX(position), -1) + 1 FROM space), ?2, ?3, ?4, ?5)",
                params![name, login_set, now.unix_millis(), colour, icon],
            )
            .map_err(|_| StoreError::Query)?;
        let id = transaction.last_insert_rowid();
        transaction.commit().map_err(|_| StoreError::Query)?;
        space((
            id,
            name.to_owned(),
            login_set,
            colour.to_owned(),
            icon.to_owned(),
        ))
        .map(Some)
    }

    /// Gives a Space an icon, an emoji, or none (empty). False when there is
    /// no such Space. What an icon may be is the core's rule: the store keeps
    /// what it is given.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn icon_space(&mut self, id: SpaceId, icon: &str) -> Result<bool, StoreError> {
        let Ok(id) = i64::try_from(id.0) else {
            return Ok(false);
        };
        let changed = self
            .connection
            .execute(
                "UPDATE space SET icon = ?1 WHERE id = ?2",
                params![icon, id],
            )
            .map_err(|_| StoreError::Query)?;
        Ok(changed > 0)
    }

    /// Gives a Space a colour, by the core's name for it, or none (empty).
    /// False when there is no such Space. What a colour may be is the
    /// core's rule: the store keeps what it is given.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn colour_space(&mut self, id: SpaceId, colour: &str) -> Result<bool, StoreError> {
        let Ok(id) = i64::try_from(id.0) else {
            return Ok(false);
        };
        let coloured = self
            .connection
            .execute(
                "UPDATE space SET colour = ?1 WHERE id = ?2",
                params![colour, id],
            )
            .map_err(|_| StoreError::Query)?;
        Ok(coloured > 0)
    }

    /// Gives a Space a name. False when there is no such Space.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn rename_space(&mut self, id: SpaceId, name: &str) -> Result<bool, StoreError> {
        let Ok(id) = i64::try_from(id.0) else {
            return Ok(false);
        };
        let renamed = self
            .connection
            .execute(
                "UPDATE space SET name = ?1 WHERE id = ?2",
                params![name, id],
            )
            .map_err(|_| StoreError::Query)?;
        Ok(renamed > 0)
    }

    /// The Space the window showed when this was last written (009, 024):
    /// the one a window shows when the app starts. `None` when none was
    /// written, or the one written is not a Space there is.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn shown_space(&self) -> Result<Option<SpaceId>, StoreError> {
        let id: Option<i64> = self
            .connection
            .prepare_cached(
                "SELECT space.id FROM setting \
                 JOIN space ON space.id = CAST(setting.value AS INTEGER) \
                 WHERE setting.key = ?1",
            )
            .and_then(|mut statement| {
                statement
                    .query_row(params![SHOWN_SPACE], |row| row.get(0))
                    .optional()
            })
            .map_err(|_| StoreError::Query)?;
        Ok(id.and_then(|id| u64::try_from(id).ok()).map(SpaceId))
    }

    /// Writes which Space the window shows.
    ///
    /// # Errors
    /// `Query` when the database refuses; what was written before is then
    /// what is still there.
    pub fn show_space(&mut self, space: SpaceId) -> Result<(), StoreError> {
        self.set_setting(SHOWN_SPACE, &space.0.to_string())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails the way its assertions do"
)]
mod tests {
    use super::*;

    const FIRST: Uuid = Uuid::from_random_bytes([0x11; 16]);
    const WORK: Uuid = Uuid::from_random_bytes([0x22; 16]);
    const PLAY: Uuid = Uuid::from_random_bytes([0x33; 16]);

    /// A store with the app's own Space in it.
    fn a_store() -> (Store, Space) {
        let mut store = Store::open_in_memory().unwrap();
        let own = store.default_space(FIRST, Timestamp(1)).unwrap();
        (store, own)
    }

    fn names(store: &Store) -> Vec<String> {
        store
            .spaces()
            .unwrap()
            .into_iter()
            .map(|space| space.name)
            .collect()
    }

    #[test]
    fn spaces_are_listed_in_the_order_they_were_made() {
        let (mut store, own) = a_store();
        assert_eq!(store.spaces().unwrap(), std::slice::from_ref(&own));

        let work = store
            .create_space("Work", Logins::Own(WORK), "", "", Timestamp(2))
            .unwrap()
            .unwrap();
        let play = store
            .create_space("Play", Logins::Own(PLAY), "", "", Timestamp(3))
            .unwrap()
            .unwrap();

        assert_eq!(store.spaces().unwrap(), [own, work.clone(), play.clone()]);
        assert_ne!(work.id, play.id);
        assert_eq!((work.name.as_str(), play.name.as_str()), ("Work", "Play"));
    }

    #[test]
    fn a_space_has_logins_of_its_own_or_those_of_a_space_there_is() {
        let (mut store, own) = a_store();

        let work = store
            .create_space("Work", Logins::Own(WORK), "", "", Timestamp(2))
            .unwrap()
            .unwrap();
        assert_eq!(work.login_set, LoginSetId(WORK));
        assert_ne!(work.login_set, own.login_set, "its own, and nobody else's");

        // A second project at the same job: Work's logins. And one that
        // shares the app's own Space's.
        let project = store
            .create_space("Project", Logins::Of(work.id), "", "", Timestamp(3))
            .unwrap()
            .unwrap();
        assert_eq!(project.login_set, work.login_set);
        let beside = store
            .create_space("Beside home", Logins::Of(own.id), "", "", Timestamp(4))
            .unwrap()
            .unwrap();
        assert_eq!(beside.login_set, own.login_set);
        // Sharing the logins of one that shares them is sharing the same.
        let third = store
            .create_space("Third", Logins::Of(project.id), "", "", Timestamp(5))
            .unwrap()
            .unwrap();
        assert_eq!(third.login_set, work.login_set);
    }

    #[test]
    fn the_logins_of_a_space_that_is_not_there_make_nothing() {
        let (mut store, own) = a_store();

        for nowhere in [SpaceId(4242), SpaceId(0), SpaceId(u64::MAX)] {
            let made = store
                .create_space("Lost", Logins::Of(nowhere), "", "", Timestamp(2))
                .unwrap();
            assert_eq!(made, None);
        }

        assert_eq!(store.spaces().unwrap(), [own]);
    }

    #[test]
    fn a_space_is_renamed_and_one_that_is_not_there_is_not() {
        let (mut store, own) = a_store();
        let work = store
            .create_space("Work", Logins::Own(WORK), "", "", Timestamp(2))
            .unwrap()
            .unwrap();

        assert!(store.rename_space(work.id, "The office").unwrap());
        // The app's own Space, which had no name, can be given one.
        assert!(store.rename_space(own.id, "Home").unwrap());
        assert_eq!(names(&store), ["Home", "The office"]);
        // Its logins, and its place, are as they were.
        assert_eq!(store.spaces().unwrap()[1].login_set, LoginSetId(WORK));

        for nowhere in [SpaceId(4242), SpaceId(u64::MAX)] {
            assert!(!store.rename_space(nowhere, "Nobody").unwrap());
        }
        assert_eq!(names(&store), ["Home", "The office"]);
    }

    #[test]
    fn a_space_has_the_colour_it_was_made_with_and_another_when_given_one() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let colours = |store: &Store| -> Vec<String> {
            store
                .spaces()
                .unwrap()
                .into_iter()
                .map(|space| space.colour)
                .collect()
        };
        let (own, work) = {
            let mut store = Store::open(&path).unwrap();
            let own = store.default_space(FIRST, Timestamp(1)).unwrap();
            assert_eq!(own.colour, "", "the app's own Space starts with none");
            let work = store
                .create_space("Work", Logins::Own(WORK), "blue", "", Timestamp(2))
                .unwrap()
                .unwrap();
            assert_eq!(work.colour, "blue");
            let plain = store
                .create_space("Plain", Logins::Of(work.id), "", "", Timestamp(3))
                .unwrap()
                .unwrap();
            assert_eq!(plain.colour, "", "none asked for is none");
            assert_eq!(colours(&store), ["", "blue", ""]);

            // Another, for the one named: the app's own among them; and none again.
            assert!(store.colour_space(work.id, "green").unwrap());
            assert!(store.colour_space(own.id, "red").unwrap());
            assert_eq!(colours(&store), ["red", "green", ""]);
            assert!(store.colour_space(own.id, "").unwrap());
            assert_eq!(colours(&store), ["", "green", ""]);
            assert!(store.colour_space(own.id, "pink").unwrap());

            // A Space that is not there has no colour to change, and nothing changes.
            assert!(!store.colour_space(SpaceId(9_999), "red").unwrap());
            assert!(!store.colour_space(SpaceId(u64::MAX), "red").unwrap());
            assert_eq!(colours(&store), ["pink", "green", ""]);
            (own, work)
        };

        // Kept: the list, and the first Space as it is given at a start.
        let mut store = Store::open(&path).unwrap();
        assert_eq!(colours(&store), ["pink", "green", ""]);
        let first = store.default_space(PLAY, Timestamp(9)).unwrap();
        assert_eq!((first.id, first.colour.as_str()), (own.id, "pink"));
        // A colour changes nothing else of a Space.
        let listed = store.spaces().unwrap();
        assert_eq!((listed[1].id, listed[1].name.as_str()), (work.id, "Work"));
        assert_eq!(listed[1].login_set, work.login_set);
    }

    #[test]
    fn a_space_has_the_icon_it_was_made_with_and_another_when_given_one() {
        const HOUSE: &str = "\u{1F3E0}";
        const CASE: &str = "\u{1F4BC}";
        const PLANE: &str = "\u{2708}\u{FE0F}";
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let icons = |store: &Store| -> Vec<String> {
            store
                .spaces()
                .unwrap()
                .into_iter()
                .map(|space| space.icon)
                .collect()
        };
        let (own, work) = {
            let mut store = Store::open(&path).unwrap();
            let own = store.default_space(FIRST, Timestamp(1)).unwrap();
            assert_eq!(own.icon, "", "the app's own Space starts with none");
            let work = store
                .create_space("Work", Logins::Own(WORK), "blue", CASE, Timestamp(2))
                .unwrap()
                .unwrap();
            assert_eq!(work.icon, CASE);
            let plain = store
                .create_space("Plain", Logins::Of(work.id), "", "", Timestamp(3))
                .unwrap()
                .unwrap();
            assert_eq!(plain.icon, "", "none asked for is none");
            assert_eq!(icons(&store), ["", CASE, ""]);

            // Another, for the one named: the app's own among them; and none again. An
            // icon written with a selector is kept with it.
            assert!(store.icon_space(work.id, PLANE).unwrap());
            assert!(store.icon_space(own.id, HOUSE).unwrap());
            assert_eq!(icons(&store), [HOUSE, PLANE, ""]);
            assert!(store.icon_space(own.id, "").unwrap());
            assert_eq!(icons(&store), ["", PLANE, ""]);
            assert!(store.icon_space(own.id, HOUSE).unwrap());

            // A Space that is not there has no icon to change, and nothing changes.
            assert!(!store.icon_space(SpaceId(9_999), CASE).unwrap());
            assert!(!store.icon_space(SpaceId(u64::MAX), CASE).unwrap());
            assert_eq!(icons(&store), [HOUSE, PLANE, ""]);
            (own, work)
        };

        // Kept: the list, and the first Space as it is given at a start.
        let mut store = Store::open(&path).unwrap();
        assert_eq!(icons(&store), [HOUSE, PLANE, ""]);
        let first = store.default_space(PLAY, Timestamp(9)).unwrap();
        assert_eq!((first.id, first.icon.as_str()), (own.id, HOUSE));
        // An icon changes nothing else of a Space.
        let listed = store.spaces().unwrap();
        assert_eq!(
            (
                listed[1].id,
                listed[1].name.as_str(),
                listed[1].colour.as_str()
            ),
            (work.id, "Work", "blue")
        );
        assert_eq!(listed[1].login_set, work.login_set);
    }

    #[test]
    fn a_store_from_before_icons_gains_them_and_keeps_its_spaces() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 14 by taking away the one
        // thing that came after: what 049 left.
        let before = {
            let mut store = Store::open(&path).unwrap();
            store.default_space(FIRST, Timestamp(1)).unwrap();
            store
                .create_space("Work", Logins::Own(WORK), "blue", "\u{1F4BC}", Timestamp(2))
                .unwrap()
                .unwrap();
            store.spaces().unwrap()
        };
        let old = rusqlite::Connection::open(&path).unwrap();
        old.execute_batch("ALTER TABLE space DROP COLUMN icon; PRAGMA user_version = 14;")
            .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        let after = store.spaces().unwrap();
        assert_eq!(after.len(), 2);
        for (was, is) in before.iter().zip(&after) {
            assert_eq!(
                (is.id, is.name.as_str(), is.login_set, is.colour.as_str()),
                (
                    was.id,
                    was.name.as_str(),
                    was.login_set,
                    was.colour.as_str()
                )
            );
            assert_eq!(is.icon, "", "a Space from before has none");
        }
        assert!(store.icon_space(after[1].id, "\u{1F331}").unwrap());
        assert_eq!(store.spaces().unwrap()[1].icon, "\u{1F331}");
    }

    #[test]
    fn a_store_from_before_colours_gains_them_and_keeps_its_spaces() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 7 by taking
        // away what came after: what 024 left.
        let before = {
            let mut store = Store::open(&path).unwrap();
            store.default_space(FIRST, Timestamp(1)).unwrap();
            store
                .create_space("Work", Logins::Own(WORK), "blue", "", Timestamp(2))
                .unwrap()
                .unwrap();
            store.spaces().unwrap()
        };
        let old = rusqlite::Connection::open(&path).unwrap();
        old.execute_batch(
            "ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; ALTER TABLE tab DROP COLUMN favourite; DROP TABLE node; ALTER TABLE tab DROP COLUMN site_group; ALTER TABLE tab DROP COLUMN site_home; ALTER TABLE tab DROP COLUMN site_name; DROP TABLE journal; ALTER TABLE space DROP COLUMN colour; PRAGMA user_version = 7;",
        )
        .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        let after = store.spaces().unwrap();
        assert_eq!(after.len(), 2);
        for (was, is) in before.iter().zip(&after) {
            assert_eq!(
                (is.id, is.name.as_str(), is.login_set),
                (was.id, was.name.as_str(), was.login_set)
            );
            assert_eq!(is.colour, "", "a Space from before has none");
        }
        assert!(store.colour_space(after[1].id, "teal").unwrap());
        assert_eq!(store.spaces().unwrap()[1].colour, "teal");
    }

    #[test]
    fn the_space_the_window_shows_is_kept_and_only_while_it_is_a_space() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let work = {
            let mut store = Store::open(&path).unwrap();
            let own = store.default_space(FIRST, Timestamp(1)).unwrap();
            assert_eq!(store.shown_space().unwrap(), None, "none was written");
            let work = store
                .create_space("Work", Logins::Own(WORK), "", "", Timestamp(2))
                .unwrap()
                .unwrap();
            store.show_space(own.id).unwrap();
            assert_eq!(store.shown_space().unwrap(), Some(own.id));
            // Written again, it is the one written last.
            store.show_space(work.id).unwrap();
            work
        };

        let mut store = Store::open(&path).unwrap();
        assert_eq!(store.shown_space().unwrap(), Some(work.id));

        // One that is no Space, or no number at all (a store that was
        // meddled with), is none.
        store.show_space(SpaceId(9_999)).unwrap();
        assert_eq!(store.shown_space().unwrap(), None);
        store.set_setting(SHOWN_SPACE, "Work").unwrap();
        assert_eq!(store.shown_space().unwrap(), None);
        store.set_setting(SHOWN_SPACE, "").unwrap();
        assert_eq!(store.shown_space().unwrap(), None);
    }

    #[test]
    fn the_spaces_are_still_there_when_the_store_is_opened_again() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let before = {
            let mut store = Store::open(&path).unwrap();
            let own = store.default_space(FIRST, Timestamp(1)).unwrap();
            let work = store
                .create_space("Work", Logins::Own(WORK), "", "", Timestamp(2))
                .unwrap()
                .unwrap();
            store
                .create_space("Project", Logins::Of(work.id), "", "", Timestamp(3))
                .unwrap()
                .unwrap();
            store.rename_space(own.id, "Home").unwrap();
            store.spaces().unwrap()
        };

        let mut store = Store::open(&path).unwrap();
        assert_eq!(store.spaces().unwrap(), before);
        assert_eq!(names(&store), ["Home", "Work", "Project"]);
        // The first Space is still the app's own, whatever it is called now.
        assert_eq!(store.default_space(PLAY, Timestamp(9)).unwrap(), before[0]);
    }
}
