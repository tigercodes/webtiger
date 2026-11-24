//! What was downloaded (013): a record for each download of a login set,
//! from the moment it starts, with how it ended. The files are not the
//! store's: forgetting a record deletes no file.

use rusqlite::{OptionalExtension, params};
use wt_core::{LoginSetId, Timestamp};

use crate::{Store, StoreError};

/// The state of a download that has started and not ended.
pub const COMING: &str = "coming";
/// The state of a download whose file is whole.
pub const FINISHED: &str = "finished";
/// The code a download ends with when the app ended under it.
pub const INTERRUPTED: &str = "download.interrupted";

/// The most downloads one page holds.
const MAX_DOWNLOADS_PER_PAGE: u32 = 500;

/// A download, as it is kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Download {
    /// Never used twice. A later download has a greater one.
    pub id: i64,
    /// Where it came from.
    pub url: String,
    /// The name its file is saved under: the one asked for while it is
    /// coming, the one the shell used once it has finished.
    pub file_name: String,
    pub mime_type: String,
    /// How big the site said it is. `None` when it did not say.
    pub total_bytes: Option<u64>,
    /// [`COMING`], [`FINISHED`], or the code it failed with.
    pub state: String,
    pub started_at: Timestamp,
    /// `None` while it is coming.
    pub ended_at: Option<Timestamp>,
}

/// Some of a login set's downloads, newest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DownloadPage {
    pub downloads: Vec<Download>,
    /// There are older downloads than these.
    pub more: bool,
}

impl Store {
    /// Writes down a download that is starting. Its id.
    ///
    /// # Errors
    /// `Query` when the database refuses: among other things, when the login
    /// set is not one the store has, as a private one never is.
    pub fn add_download(
        &mut self,
        login_set: LoginSetId,
        url: &str,
        file_name: &str,
        mime_type: &str,
        total_bytes: Option<u64>,
        started_at: Timestamp,
    ) -> Result<i64, StoreError> {
        // A size too big for the column is no size.
        let total = total_bytes
            .and_then(|bytes| i64::try_from(bytes).ok())
            .unwrap_or(-1);
        self.connection
            .prepare_cached(
                "INSERT INTO download \
                 (login_set_id, url, file_name, mime_type, total_bytes, state, started_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )
            .and_then(|mut statement| {
                statement.insert(params![
                    login_set.0.as_bytes().as_slice(),
                    url,
                    file_name,
                    mime_type,
                    total,
                    COMING,
                    started_at.0,
                ])
            })
            .map_err(|_| StoreError::Query)
    }

    /// Writes down how a download ended: [`FINISHED`], with the name its
    /// file was saved under, or the code it failed with. True when it was
    /// coming until now: a download ends once, and one that has ended stays
    /// as it ended.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn end_download(
        &mut self,
        id: i64,
        state: &str,
        file_name: Option<&str>,
        ended_at: Timestamp,
    ) -> Result<bool, StoreError> {
        self.connection
            .execute(
                "UPDATE download SET state = ?2, ended_at = ?3, file_name = coalesce(?4, file_name) \
                 WHERE id = ?1 AND state = ?5",
                params![id, state, ended_at.0, file_name, COMING],
            )
            .map(|changed| changed > 0)
            .map_err(|_| StoreError::Query)
    }

    /// Whether any download, of any login set, is still coming.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn any_download_coming(&self) -> Result<bool, StoreError> {
        self.connection
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM download WHERE state = ?1)",
                params![COMING],
                |row| row.get(0),
            )
            .map_err(|_| StoreError::Query)
    }

    /// Ends every download that is still coming, as [`INTERRUPTED`]: the app
    /// that was fetching them is gone. How many there were.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn interrupt_downloads(&mut self, ended_at: Timestamp) -> Result<usize, StoreError> {
        self.connection
            .execute(
                "UPDATE download SET state = ?1, ended_at = ?2 WHERE state = ?3",
                params![INTERRUPTED, ended_at.0, COMING],
            )
            .map_err(|_| StoreError::Query)
    }

    /// A login set's downloads, newest first: at most `limit` of them (and
    /// never more than 500), older than the download `before` when one is
    /// given.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn downloads(
        &self,
        login_set: LoginSetId,
        before: Option<i64>,
        limit: u32,
    ) -> Result<DownloadPage, StoreError> {
        let limit = limit.clamp(1, MAX_DOWNLOADS_PER_PAGE);
        // One more than asked for, to know whether there are more.
        let fetch = i64::from(limit) + 1;
        let mut statement = self
            .connection
            .prepare_cached(
                "SELECT id, url, file_name, mime_type, total_bytes, state, started_at, ended_at \
                 FROM download WHERE login_set_id = ?1 AND id < ?2 ORDER BY id DESC LIMIT ?3",
            )
            .map_err(|_| StoreError::Query)?;
        let mut downloads: Vec<Download> = statement
            .query_map(
                params![
                    login_set.0.as_bytes().as_slice(),
                    before.unwrap_or(i64::MAX),
                    fetch
                ],
                read,
            )
            .and_then(Iterator::collect)
            .map_err(|_| StoreError::Query)?;
        let more = downloads.len() > limit as usize;
        downloads.truncate(limit as usize);
        Ok(DownloadPage { downloads, more })
    }

    /// One download of a login set. `None` when it has none by that id.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn download(&self, login_set: LoginSetId, id: i64) -> Result<Option<Download>, StoreError> {
        self.connection
            .prepare_cached(
                "SELECT id, url, file_name, mime_type, total_bytes, state, started_at, ended_at \
                 FROM download WHERE id = ?1 AND login_set_id = ?2",
            )
            .and_then(|mut statement| {
                statement
                    .query_row(params![id, login_set.0.as_bytes().as_slice()], read)
                    .optional()
            })
            .map_err(|_| StoreError::Query)
    }

    /// Takes one download off a login set's list. False when it had none by
    /// that id. The file is not touched.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_download(&mut self, login_set: LoginSetId, id: i64) -> Result<bool, StoreError> {
        let forgotten = self
            .connection
            .execute(
                "DELETE FROM download WHERE id = ?1 AND login_set_id = ?2",
                params![id, login_set.0.as_bytes().as_slice()],
            )
            .map_err(|_| StoreError::Query)?;
        self.leave_no_trace()?;
        Ok(forgotten > 0)
    }

    /// Takes every download off a login set's list. The files are not
    /// touched.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_all_downloads(&mut self, login_set: LoginSetId) -> Result<(), StoreError> {
        self.connection
            .execute(
                "DELETE FROM download WHERE login_set_id = ?1",
                params![login_set.0.as_bytes().as_slice()],
            )
            .map_err(|_| StoreError::Query)?;
        self.leave_no_trace()
    }
}

fn read(row: &rusqlite::Row<'_>) -> rusqlite::Result<Download> {
    let total: i64 = row.get(4)?;
    let ended: Option<i64> = row.get(7)?;
    Ok(Download {
        id: row.get(0)?,
        url: row.get(1)?,
        file_name: row.get(2)?,
        mime_type: row.get(3)?,
        total_bytes: u64::try_from(total).ok(),
        state: row.get(5)?,
        started_at: Timestamp(row.get(6)?),
        ended_at: ended.map(Timestamp),
    })
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
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

    fn add(store: &mut Store, login_set: Uuid, name: &str, at: i64) -> i64 {
        store
            .add_download(
                LoginSetId(login_set),
                &format!("https://example.com/files/{name}"),
                name,
                "application/pdf",
                Some(1_000),
                Timestamp(at),
            )
            .unwrap()
    }

    fn names(page: &DownloadPage) -> Vec<&str> {
        page.downloads
            .iter()
            .map(|download| download.file_name.as_str())
            .collect()
    }

    #[test]
    fn a_download_is_written_as_it_starts_and_once_as_it_ends() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        let id = add(&mut store, MINE, "report.pdf", 10);

        let coming = store.download(mine, id).unwrap().unwrap();
        assert_eq!(coming.state, COMING);
        assert_eq!(coming.url, "https://example.com/files/report.pdf");
        assert_eq!(coming.total_bytes, Some(1_000));
        assert_eq!((coming.started_at, coming.ended_at), (Timestamp(10), None));

        // It finished under another name: the one it asked for was taken.
        assert!(
            store
                .end_download(id, FINISHED, Some("report (2).pdf"), Timestamp(20))
                .unwrap()
        );
        let finished = store.download(mine, id).unwrap().unwrap();
        assert_eq!(finished.state, FINISHED);
        assert_eq!(finished.file_name, "report (2).pdf");
        assert_eq!(finished.ended_at, Some(Timestamp(20)));

        // A download ends once: a late word of it changes nothing.
        assert!(
            !store
                .end_download(id, "download.network", None, Timestamp(30))
                .unwrap()
        );
        assert_eq!(store.download(mine, id).unwrap().unwrap(), finished);
        assert!(
            !store
                .end_download(id + 100, FINISHED, None, Timestamp(30))
                .unwrap()
        );
    }

    #[test]
    fn a_failure_keeps_the_name_that_was_asked_for_and_a_size_not_said_is_none() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        let id = store
            .add_download(
                mine,
                "https://example.com/stream",
                "stream.bin",
                "",
                None,
                Timestamp(1),
            )
            .unwrap();
        assert_eq!(store.download(mine, id).unwrap().unwrap().total_bytes, None);

        store
            .end_download(id, "download.network", None, Timestamp(2))
            .unwrap();

        let failed = store.download(mine, id).unwrap().unwrap();
        assert_eq!(failed.state, "download.network");
        assert_eq!(failed.file_name, "stream.bin");
        // A size no column holds is no size, not an error.
        let huge = store
            .add_download(
                mine,
                "https://example.com/h",
                "h",
                "",
                Some(u64::MAX),
                Timestamp(3),
            )
            .unwrap();
        assert_eq!(
            store.download(mine, huge).unwrap().unwrap().total_bytes,
            None
        );
    }

    #[test]
    fn downloads_are_listed_newest_first_a_page_at_a_time_by_login_set() {
        let mut store = store();
        let (mine, theirs) = (LoginSetId(MINE), LoginSetId(THEIRS));
        for (index, name) in ["a.pdf", "b.pdf", "c.pdf", "d.pdf", "e.pdf"]
            .into_iter()
            .enumerate()
        {
            add(&mut store, MINE, name, index as i64);
        }
        add(&mut store, THEIRS, "theirs.pdf", 9);

        let first = store.downloads(mine, None, 2).unwrap();
        assert_eq!(names(&first), ["e.pdf", "d.pdf"]);
        assert!(first.more);
        let second = store
            .downloads(mine, first.downloads.last().map(|download| download.id), 2)
            .unwrap();
        assert_eq!(names(&second), ["c.pdf", "b.pdf"]);
        let last = store
            .downloads(mine, second.downloads.last().map(|download| download.id), 2)
            .unwrap();
        assert_eq!(names(&last), ["a.pdf"]);
        assert!(!last.more);

        assert_eq!(
            names(&store.downloads(theirs, None, 10).unwrap()),
            ["theirs.pdf"]
        );
        // And one login set's download is not another's to read.
        let theirs_id = store.downloads(theirs, None, 1).unwrap().downloads[0].id;
        assert_eq!(store.download(mine, theirs_id).unwrap(), None);
    }

    #[test]
    fn forgetting_takes_records_off_one_login_sets_list() {
        let mut store = store();
        let (mine, theirs) = (LoginSetId(MINE), LoginSetId(THEIRS));
        let first = add(&mut store, MINE, "a.pdf", 1);
        add(&mut store, MINE, "b.pdf", 2);
        let other = add(&mut store, THEIRS, "theirs.pdf", 3);

        assert!(store.forget_download(mine, first).unwrap());
        assert!(!store.forget_download(mine, first).unwrap(), "it is gone");
        assert!(
            !store.forget_download(mine, other).unwrap(),
            "another login set's is not this one's to forget"
        );
        assert_eq!(names(&store.downloads(mine, None, 10).unwrap()), ["b.pdf"]);

        store.forget_all_downloads(mine).unwrap();
        assert_eq!(
            store.downloads(mine, None, 10).unwrap(),
            DownloadPage::default()
        );
        assert_eq!(
            names(&store.downloads(theirs, None, 10).unwrap()),
            ["theirs.pdf"]
        );

        // An id is never used twice: a later download is not taken for one forgotten.
        let later = add(&mut store, MINE, "c.pdf", 4);
        assert!(later > other);
    }

    #[test]
    fn downloads_left_coming_are_interrupted() {
        let mut store = store();
        let mine = LoginSetId(MINE);
        let done = add(&mut store, MINE, "done.pdf", 1);
        store
            .end_download(done, FINISHED, None, Timestamp(2))
            .unwrap();
        let coming = add(&mut store, MINE, "coming.pdf", 3);
        let also = add(&mut store, THEIRS, "also.pdf", 4);

        assert!(store.any_download_coming().unwrap());
        assert_eq!(store.interrupt_downloads(Timestamp(9)).unwrap(), 2);
        assert!(!store.any_download_coming().unwrap());

        let interrupted = store.download(mine, coming).unwrap().unwrap();
        assert_eq!(interrupted.state, INTERRUPTED);
        assert_eq!(interrupted.ended_at, Some(Timestamp(9)));
        assert_eq!(
            store
                .download(LoginSetId(THEIRS), also)
                .unwrap()
                .unwrap()
                .state,
            INTERRUPTED
        );
        let finished = store.download(mine, done).unwrap().unwrap();
        assert_eq!(
            (finished.state.as_str(), finished.ended_at),
            (FINISHED, Some(Timestamp(2))),
            "what had ended is as it ended"
        );
        assert_eq!(store.interrupt_downloads(Timestamp(10)).unwrap(), 0);
    }

    #[test]
    fn a_store_from_before_downloads_gains_them_and_keeps_what_it_had() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // A store as this build makes it, put back to version 4 by taking
        // the table away: what 012 left.
        let mut store = Store::open(&path).unwrap();
        let made = store.default_space(MINE, Timestamp(1)).unwrap();
        store.set_zoom(made.login_set, "example.com", 125).unwrap();
        drop(store);
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "ALTER TABLE space DROP COLUMN icon; DROP INDEX archive_space; DROP TABLE archive; ALTER TABLE tab DROP COLUMN shown_at; ALTER TABLE space DROP COLUMN tidy_after; DROP TABLE icon; ALTER TABLE tab DROP COLUMN favourite; DROP TABLE node; ALTER TABLE tab DROP COLUMN site_group; ALTER TABLE tab DROP COLUMN site_home; ALTER TABLE tab DROP COLUMN site_name; DROP TABLE journal; ALTER TABLE space DROP COLUMN colour; DROP TABLE setting; DROP TABLE site_permission; DROP TABLE download; PRAGMA user_version = 4;",
        )
        .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();

        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        let id = store
            .add_download(
                made.login_set,
                "https://example.com/a.zip",
                "a.zip",
                "application/zip",
                Some(5),
                Timestamp(2),
            )
            .unwrap();
        assert_eq!(
            store.zoom(made.login_set, "example.com").unwrap(),
            Some(125),
            "what was there is still there"
        );
        drop(store);

        // And it is there the next time the store is opened, still coming:
        // what becomes of it then is the core's to say.
        let store = Store::open(&path).unwrap();
        assert_eq!(
            store.download(made.login_set, id).unwrap().unwrap().state,
            COMING
        );
    }
}
