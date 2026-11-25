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
    /// The tab is a Site (039), and this is its home: the address it was
    /// kept at. Empty for a tab that is no Site.
    pub home: String,
    /// A Site's name: the title it was kept with. It is what a Site is
    /// called while it is asleep, wherever it had wandered.
    pub name: String,
    /// The Group a Site is in (041): its place among its Space's Groups,
    /// counted from one. Nought for a tab that is in none.
    pub group: u32,
    /// The Site is one of its row's Favourites (042).
    pub favourite: bool,
    /// When the tab was last in front, in milliseconds (049): what a Space's
    /// tidying counts from. Nought for one that is not known to have been.
    pub shown_at: i64,
    /// The tab's kept state (051), by its id in the store. Nought: none.
    pub state: i64,
}

/// A Group as it is kept between runs (041).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedGroup {
    pub name: String,
    /// The name of its colour, one of a Space's (025). Empty for none.
    pub colour: String,
    /// It is shut: its Sites are out of the way.
    pub shut: bool,
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
                    "INSERT INTO tab (space_id, position, url, title, front, site_home, site_name, site_group, favourite, shown_at, state_id) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                )
                .map_err(|_| StoreError::Query)?;
            for (position, tab) in tabs.iter().enumerate() {
                let position = i64::try_from(position).map_err(|_| StoreError::Query)?;
                insert
                    .execute(params![
                        space,
                        position,
                        tab.url,
                        tab.title,
                        tab.front,
                        tab.home,
                        tab.name,
                        tab.group,
                        tab.favourite,
                        tab.shown_at,
                        tab.state
                    ])
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
                "SELECT url, title, front, site_home, site_name, site_group, favourite, shown_at, state_id FROM tab \
                 WHERE space_id = ?1 ORDER BY position",
            )
            .map_err(|_| StoreError::Query)?;
        statement
            .query_map(params![space], |row| {
                Ok(SavedTab {
                    url: row.get(0)?,
                    title: row.get(1)?,
                    front: row.get(2)?,
                    home: row.get(3)?,
                    name: row.get(4)?,
                    group: row.get(5)?,
                    favourite: row.get(6)?,
                    shown_at: row.get(7)?,
                    state: row.get(8)?,
                })
            })
            .and_then(Iterator::collect)
            .map_err(|_| StoreError::Query)
    }

    /// Replaces a Space's saved Groups with `groups`, in their order, as a
    /// whole (041).
    ///
    /// # Errors
    /// `Query` when the database refuses; what was saved before is then
    /// what is still there.
    pub fn save_groups(&mut self, space: SpaceId, groups: &[SavedGroup]) -> Result<(), StoreError> {
        let space = i64::try_from(space.get()).map_err(|_| StoreError::Query)?;
        let transaction = self
            .connection
            .transaction()
            .map_err(|_| StoreError::Query)?;
        transaction
            .execute("DELETE FROM node WHERE space_id = ?1", params![space])
            .map_err(|_| StoreError::Query)?;
        {
            let mut insert = transaction
                .prepare_cached(
                    "INSERT INTO node (space_id, position, kind, name, colour, shut) \
                     VALUES (?1, ?2, 'group', ?3, ?4, ?5)",
                )
                .map_err(|_| StoreError::Query)?;
            for (position, group) in groups.iter().enumerate() {
                let position = i64::try_from(position).map_err(|_| StoreError::Query)?;
                insert
                    .execute(params![
                        space,
                        position,
                        group.name,
                        group.colour,
                        group.shut
                    ])
                    .map_err(|_| StoreError::Query)?;
            }
        }
        transaction.commit().map_err(|_| StoreError::Query)
    }

    /// A Space's saved Groups, in their order. None saved is none.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn saved_groups(&self, space: SpaceId) -> Result<Vec<SavedGroup>, StoreError> {
        let space = i64::try_from(space.get()).map_err(|_| StoreError::Query)?;
        let mut statement = self
            .connection
            .prepare_cached(
                "SELECT name, colour, shut FROM node \
                 WHERE space_id = ?1 AND kind = 'group' ORDER BY position",
            )
            .map_err(|_| StoreError::Query)?;
        statement
            .query_map(params![space], |row| {
                Ok(SavedGroup {
                    name: row.get(0)?,
                    colour: row.get(1)?,
                    shut: row.get(2)?,
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
            home: String::new(),
            name: String::new(),
            group: 0,
            favourite: false,
            shown_at: 0,
            state: 0,
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
            "DROP TABLE site_guard; ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; ALTER TABLE tab DROP COLUMN favourite; DROP TABLE node; ALTER TABLE tab DROP COLUMN site_group; ALTER TABLE tab DROP COLUMN site_home; ALTER TABLE tab DROP COLUMN site_name; DROP TABLE journal; ALTER TABLE space DROP COLUMN colour; DROP TABLE setting; DROP TABLE site_permission; DROP TABLE download; DROP TABLE site_setting; DROP TABLE tab; PRAGMA user_version = 2;",
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

    #[test]
    fn a_sites_home_is_kept_with_it_and_a_tab_that_is_no_site_has_none() {
        let mut store = Store::open_in_memory().unwrap();
        let space = store.default_space(MINE, Timestamp(1)).unwrap().id;
        let mut tabs = three();
        // The first is a Site, kept at one address and since gone to another.
        tabs[0].home = "https://example.com/".to_owned();
        tabs[0].name = "Mail".to_owned();
        tabs[0].url = "https://example.com/inbox/7".to_owned();

        store.save_tabs(space, &tabs).unwrap();

        let read = store.saved_tabs(space).unwrap();
        assert_eq!(read, tabs);
        assert_eq!(
            read.iter()
                .map(|tab| (tab.home.as_str(), tab.name.as_str()))
                .collect::<Vec<_>>(),
            [("https://example.com/", "Mail"), ("", ""), ("", "")]
        );
    }

    #[test]
    fn the_tabs_of_a_store_from_before_sites_are_tabs_with_no_home() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 9 by taking a
        // tab's home away: what 033 left.
        let mut store = Store::open(&path).unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        store.save_tabs(made.id, &three()).unwrap();
        drop(store);
        let old = Connection::open(&path).unwrap();
        old.execute_batch("DROP TABLE site_guard; ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; ALTER TABLE tab DROP COLUMN favourite; DROP TABLE node; ALTER TABLE tab DROP COLUMN site_group; ALTER TABLE tab DROP COLUMN site_home; ALTER TABLE tab DROP COLUMN site_name; PRAGMA user_version = 9;")
            .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        assert_eq!(
            store.saved_tabs(made.id).unwrap(),
            three(),
            "the tabs it had are still there, and none is a Site"
        );
        // And a Site can be kept in it now.
        let mut tabs = three();
        tabs[1].home = "https://example.org/".to_owned();
        tabs[1].name = "Another Place".to_owned();
        store.save_tabs(made.id, &tabs).unwrap();
        assert_eq!(store.saved_tabs(made.id).unwrap(), tabs);
    }

    #[test]
    fn a_spaces_groups_are_kept_in_their_order_and_a_site_says_which_it_is_in() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let mut store = Store::open(&path).unwrap();
        let space = store.default_space(MINE, Timestamp(1)).unwrap().id;
        assert_eq!(
            store.saved_groups(space).unwrap(),
            [],
            "a new store has none"
        );
        let group = |name: &str, colour: &str, shut: bool| SavedGroup {
            name: name.to_owned(),
            colour: colour.to_owned(),
            shut,
        };
        let groups = [group("Work", "blue", false), group("House", "", true)];
        let mut tabs = three();
        tabs[0].home = "https://example.com/".to_owned();
        tabs[0].name = "Example".to_owned();
        tabs[0].group = 2;

        store.save_groups(space, &groups).unwrap();
        store.save_tabs(space, &tabs).unwrap();
        assert_eq!(store.saved_groups(space).unwrap(), groups);
        assert_eq!(store.saved_tabs(space).unwrap()[0].group, 2);
        // Saved again, they are replaced as a whole.
        store
            .save_groups(space, std::slice::from_ref(&groups[1]))
            .unwrap();
        assert_eq!(
            store.saved_groups(space).unwrap(),
            [group("House", "", true)]
        );
        drop(store);

        // After a restart, as they were.
        let store = Store::open(&path).unwrap();
        assert_eq!(
            store.saved_groups(space).unwrap(),
            [group("House", "", true)]
        );
        assert_eq!(store.saved_tabs(space).unwrap(), tabs);
    }

    #[test]
    fn a_store_from_before_groups_has_none_and_its_sites_are_in_none() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 10 by taking
        // the Groups away: what 039 left.
        let mut store = Store::open(&path).unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        let mut tabs = three();
        tabs[0].home = "https://example.com/".to_owned();
        tabs[0].name = "Example".to_owned();
        store.save_tabs(made.id, &tabs).unwrap();
        drop(store);
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "DROP TABLE site_guard; ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; ALTER TABLE tab DROP COLUMN favourite; DROP TABLE node; \
             ALTER TABLE tab DROP COLUMN site_group; PRAGMA user_version = 10;",
        )
        .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        assert_eq!(store.saved_groups(made.id).unwrap(), []);
        assert_eq!(
            store.saved_tabs(made.id).unwrap(),
            tabs,
            "its tabs and its Site are as they were, in no Group"
        );
        // And a Group can be kept in it now.
        let group = SavedGroup {
            name: "Work".to_owned(),
            colour: String::new(),
            shut: false,
        };
        store
            .save_groups(made.id, std::slice::from_ref(&group))
            .unwrap();
        assert_eq!(store.saved_groups(made.id).unwrap(), [group]);
    }

    #[test]
    fn a_site_says_whether_it_is_a_favourite_and_a_store_from_before_has_none() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        let mut store = Store::open(&path).unwrap();
        let space = store.default_space(MINE, Timestamp(1)).unwrap().id;
        let mut tabs = three();
        tabs[0].home = "https://example.com/".to_owned();
        tabs[0].name = "Example".to_owned();
        tabs[0].favourite = true;
        tabs[1].home = "https://example.org/".to_owned();
        tabs[1].name = "Another".to_owned();
        store.save_tabs(space, &tabs).unwrap();
        assert_eq!(store.saved_tabs(space).unwrap(), tabs);
        drop(store);

        // After a restart, as they were: the one that is, and the one that is not.
        let store = Store::open(&path).unwrap();
        let kept = store.saved_tabs(space).unwrap();
        assert_eq!(
            kept.iter().map(|tab| tab.favourite).collect::<Vec<_>>(),
            [true, false, false]
        );
        drop(store);

        // The same store put back to version 11, what 041 left: its tabs and Sites are
        // as they were, and none is a Favourite.
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "DROP TABLE site_guard; ALTER TABLE tab DROP COLUMN state_id; DROP TABLE tab_state; ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; ALTER TABLE tab DROP COLUMN favourite; PRAGMA user_version = 11;",
        )
        .unwrap();
        drop(old);
        let mut store = Store::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        tabs[0].favourite = false;
        assert_eq!(store.saved_tabs(space).unwrap(), tabs);
        // And one can be kept in it now.
        tabs[1].favourite = true;
        store.save_tabs(space, &tabs).unwrap();
        assert_eq!(store.saved_tabs(space).unwrap(), tabs);
    }
}
