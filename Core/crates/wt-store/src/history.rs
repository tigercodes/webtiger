//! History (008): the visits of a login set, the words to find them by, and
//! forgetting them.

use rusqlite::{OptionalExtension as _, params};
use wt_core::{LoginSetId, Timestamp};

use crate::{Store, StoreError};

/// The most visits one page of history holds.
pub const MAX_VISITS_PER_PAGE: u32 = 500;

/// The most words of a search that are looked for. Nobody types more, and a
/// query's cost grows with each.
const MAX_WORDS: usize = 16;

/// A page that was seen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Visit {
    /// Never used twice. A later visit has a greater one.
    pub id: i64,
    pub url: String,
    /// Empty until the page has said one, and for a page that has none.
    pub title: String,
    pub visited_at: Timestamp,
}

/// Some of a login set's visits, newest first.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisitPage {
    pub visits: Vec<Visit>,
    /// There are older visits than these that match.
    pub more: bool,
}

impl Store {
    /// Writes a visit down. Its id.
    ///
    /// # Errors
    /// `Query` when the database refuses: among other things, when the login
    /// set is not one the store has, as a private one never is.
    pub fn record_visit(
        &mut self,
        login_set: LoginSetId,
        url: &str,
        title: &str,
        at: Timestamp,
    ) -> Result<i64, StoreError> {
        self.connection
            .execute(
                "INSERT INTO visit (login_set_id, url, title, visited_at) VALUES (?1, ?2, ?3, ?4)",
                params![
                    login_set.0.as_bytes().as_slice(),
                    url,
                    title,
                    at.unix_millis()
                ],
            )
            .map_err(|_| StoreError::Query)?;
        Ok(self.connection.last_insert_rowid())
    }

    /// Gives a visit the title its page turned out to have. A visit that has
    /// been forgotten meanwhile stays forgotten.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn retitle_visit(&mut self, visit: i64, title: &str) -> Result<(), StoreError> {
        self.connection
            .execute(
                "UPDATE visit SET title = ?2 WHERE id = ?1 AND title <> ?2",
                params![visit, title],
            )
            .map(|_| ())
            .map_err(|_| StoreError::Query)
    }

    /// A page of a login set's visits, newest first: at most `limit` of them
    /// (and never more than [`MAX_VISITS_PER_PAGE`]), older than the visit
    /// `before` when one is named.
    ///
    /// With `text`, only visits in whose address or title every word of it
    /// is found, as a word or the start of one, whatever its case. Text with
    /// no word in it finds nothing.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn visits(
        &self,
        login_set: LoginSetId,
        text: &str,
        before: Option<i64>,
        limit: u32,
    ) -> Result<VisitPage, StoreError> {
        let limit = limit.clamp(1, MAX_VISITS_PER_PAGE);
        // One more than asked for, to know whether there are more.
        let fetch = i64::from(limit) + 1;
        let before = before.unwrap_or(i64::MAX);
        let login_set = login_set.0.as_bytes().as_slice().to_vec();
        let read = |row: &rusqlite::Row<'_>| {
            Ok(Visit {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                visited_at: Timestamp(row.get(3)?),
            })
        };
        let mut visits: Vec<Visit> = if text.trim().is_empty() {
            let mut statement = self
                .connection
                .prepare_cached(
                    "SELECT id, url, title, visited_at FROM visit \
                     WHERE login_set_id = ?1 AND id < ?2 ORDER BY id DESC LIMIT ?3",
                )
                .map_err(|_| StoreError::Query)?;
            statement
                .query_map(params![login_set, before, fetch], read)
                .and_then(Iterator::collect)
                .map_err(|_| StoreError::Query)?
        } else {
            let Some(words) = match_expression(text) else {
                return Ok(VisitPage::default());
            };
            let mut statement = self
                .connection
                .prepare_cached(
                    "SELECT visit.id, visit.url, visit.title, visit.visited_at \
                     FROM visit_text JOIN visit ON visit.id = visit_text.rowid \
                     WHERE visit_text MATCH ?1 AND visit.login_set_id = ?2 \
                       AND visit_text.rowid < ?3 \
                     ORDER BY visit_text.rowid DESC LIMIT ?4",
                )
                .map_err(|_| StoreError::Query)?;
            statement
                .query_map(params![words, login_set, before, fetch], read)
                .and_then(Iterator::collect)
                .map_err(|_| StoreError::Query)?
        };
        let more = visits.len() > limit as usize;
        visits.truncate(limit as usize);
        Ok(VisitPage { visits, more })
    }

    /// Forgets one visit of a login set. Whether there was one to forget.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_visit(&mut self, login_set: LoginSetId, visit: i64) -> Result<bool, StoreError> {
        let forgotten = self
            .connection
            .execute(
                "DELETE FROM visit WHERE id = ?1 AND login_set_id = ?2",
                params![visit, login_set.0.as_bytes().as_slice()],
            )
            .map_err(|_| StoreError::Query)?;
        self.leave_no_trace()?;
        Ok(forgotten > 0)
    }

    /// Forgets every visit of a login set made at or after `since`.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_visits_since(
        &mut self,
        login_set: LoginSetId,
        since: Timestamp,
    ) -> Result<(), StoreError> {
        self.connection
            .execute(
                "DELETE FROM visit WHERE login_set_id = ?1 AND visited_at >= ?2",
                params![login_set.0.as_bytes().as_slice(), since.unix_millis()],
            )
            .map_err(|_| StoreError::Query)?;
        self.leave_no_trace()
    }

    /// Forgets every visit of a login set.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn forget_all_visits(&mut self, login_set: LoginSetId) -> Result<(), StoreError> {
        self.connection
            .execute(
                "DELETE FROM visit WHERE login_set_id = ?1",
                params![login_set.0.as_bytes().as_slice()],
            )
            .map_err(|_| StoreError::Query)?;
        self.leave_no_trace()
    }

    /// After forgetting: the rows are overwritten already (`secure_delete`),
    /// but the write-ahead log still holds the pages as they were. It is
    /// written back and emptied. A database in memory has none.
    pub(crate) fn leave_no_trace(&self) -> Result<(), StoreError> {
        self.connection
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
            .optional()
            .map(|_| ())
            .map_err(|_| StoreError::Query)
    }
}

/// What is typed, as the index is asked for it: every word, each as a word
/// or the start of one. What is typed is never handed over as it is, since
/// the index has a syntax of its own (`OR`, `NEAR`, `*`, quotes, columns):
/// it is cut into words, which are letters and digits and nothing else, and
/// each is quoted. `None` when there is no word in it.
pub(crate) fn match_expression(text: &str) -> Option<String> {
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .take(MAX_WORDS)
        .collect();
    if words.is_empty() {
        return None;
    }
    let mut expression = String::new();
    for word in words {
        if !expression.is_empty() {
            expression.push(' ');
        }
        expression.push('"');
        expression.push_str(word);
        expression.push_str("\"*");
    }
    Some(expression)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use wt_core::Uuid;

    use super::*;

    const MINE: Uuid = Uuid::from_random_bytes([0x11; 16]);
    const THEIRS: Uuid = Uuid::from_random_bytes([0x22; 16]);

    /// A store with two login sets: `MINE`, the first Space's, and `THEIRS`.
    fn store() -> Store {
        let mut store = Store::open_in_memory().unwrap();
        store.default_space(MINE, Timestamp(1)).unwrap();
        add_login_set(&store);
        store
    }

    fn add_login_set(store: &Store) {
        store
            .connection
            .execute(
                "INSERT INTO login_set (id, created_at) VALUES (?1, 1)",
                params![THEIRS.as_bytes().as_slice()],
            )
            .unwrap();
    }

    /// Writes visits down a second apart, oldest first. Their ids.
    fn visit(store: &mut Store, pages: &[(&str, &str)]) -> Vec<i64> {
        pages
            .iter()
            .enumerate()
            .map(|(index, (url, title))| {
                let at = Timestamp(1_000 * (index as i64 + 1));
                store
                    .record_visit(LoginSetId(MINE), url, title, at)
                    .unwrap()
            })
            .collect()
    }

    fn titles(page: &VisitPage) -> Vec<&str> {
        page.visits
            .iter()
            .map(|visit| visit.title.as_str())
            .collect()
    }

    fn found(store: &Store, text: &str) -> Vec<String> {
        store
            .visits(LoginSetId(MINE), text, None, 100)
            .unwrap()
            .visits
            .into_iter()
            .map(|visit| visit.title)
            .collect()
    }

    const PAGES: [(&str, &str); 4] = [
        (
            "https://doc.rust-lang.org/book/",
            "The Rust Programming Language",
        ),
        ("https://example.com/caf%C3%A9", "Café menu"),
        ("https://news.example.org/today", "Today's news"),
        ("https://example.com/rustic-furniture", "Rustic furniture"),
    ];

    #[test]
    fn visits_come_back_newest_first() {
        let mut store = store();
        let ids = visit(&mut store, &PAGES);
        let page = store.visits(LoginSetId(MINE), "", None, 100).unwrap();
        assert_eq!(
            titles(&page),
            [
                "Rustic furniture",
                "Today's news",
                "Café menu",
                "The Rust Programming Language"
            ]
        );
        assert!(!page.more);
        let newest = &page.visits[0];
        assert_eq!(newest.id, ids[3]);
        assert_eq!(newest.url, "https://example.com/rustic-furniture");
        assert_eq!(newest.visited_at, Timestamp(4_000));
    }

    #[test]
    fn a_page_at_a_time_and_whether_there_are_more() {
        let mut store = store();
        visit(&mut store, &PAGES);
        let first = store.visits(LoginSetId(MINE), "", None, 3).unwrap();
        assert_eq!(first.visits.len(), 3);
        assert!(first.more, "one more is older than these");

        let last = first.visits.last().unwrap().id;
        let second = store.visits(LoginSetId(MINE), "", Some(last), 3).unwrap();
        assert_eq!(titles(&second), ["The Rust Programming Language"]);
        assert!(!second.more);

        // A page of exactly what is left says there are no more.
        let exact = store.visits(LoginSetId(MINE), "", None, 4).unwrap();
        assert_eq!(exact.visits.len(), 4);
        assert!(!exact.more);
    }

    #[test]
    fn a_page_is_never_longer_than_the_most_a_page_holds() {
        let mut store = store();
        let many: Vec<(String, String)> = (0..MAX_VISITS_PER_PAGE + 20)
            .map(|n| (format!("https://example.com/{n}"), format!("Page {n}")))
            .collect();
        for (index, (url, title)) in many.iter().enumerate() {
            store
                .record_visit(LoginSetId(MINE), url, title, Timestamp(index as i64))
                .unwrap();
        }
        let page = store.visits(LoginSetId(MINE), "", None, u32::MAX).unwrap();
        assert_eq!(page.visits.len(), MAX_VISITS_PER_PAGE as usize);
        assert!(page.more);
        // Asking for none is asking for one.
        assert_eq!(
            store
                .visits(LoginSetId(MINE), "", None, 0)
                .unwrap()
                .visits
                .len(),
            1
        );
    }

    #[test]
    fn words_are_found_in_titles_and_addresses() {
        let mut store = store();
        visit(&mut store, &PAGES);
        // In a title; in an address; whatever its case.
        assert_eq!(found(&store, "menu"), ["Café menu"]);
        assert_eq!(found(&store, "news.example.org"), ["Today's news"]);
        assert_eq!(
            found(&store, "PROGRAMMING"),
            ["The Rust Programming Language"]
        );
        // The start of a word is enough, newest first.
        assert_eq!(
            found(&store, "rus"),
            ["Rustic furniture", "The Rust Programming Language"]
        );
        // Not the middle of one.
        assert_eq!(found(&store, "ustic"), [""; 0]);
        // An accent typed or left out finds the same.
        assert_eq!(found(&store, "cafe"), ["Café menu"]);
        assert_eq!(found(&store, "café"), ["Café menu"]);
    }

    #[test]
    fn every_word_must_be_found() {
        let mut store = store();
        visit(&mut store, &PAGES);
        assert_eq!(
            found(&store, "rust book"),
            ["The Rust Programming Language"]
        );
        assert_eq!(found(&store, "example furniture"), ["Rustic furniture"]);
        assert_eq!(found(&store, "rust menu"), [""; 0]);
    }

    #[test]
    fn what_is_typed_is_words_and_never_a_query() {
        let mut store = store();
        visit(&mut store, &PAGES);
        // Each of these means something to the index when handed over as it
        // is: an operator, a column, a phrase left open, a prefix of nothing.
        assert_eq!(found(&store, "rust OR menu"), [""; 0], "OR is a word");
        assert_eq!(found(&store, "title:rust"), [""; 0], "no column is named");
        assert_eq!(found(&store, "\"rust"), found(&store, "rust"));
        assert_eq!(found(&store, "rust*"), found(&store, "rust"));
        assert_eq!(found(&store, "NEAR(rust book)"), [""; 0]);
        assert_eq!(
            found(&store, "rust -book"),
            ["The Rust Programming Language"]
        );
        // Nothing with a word in it finds nothing, and breaks nothing.
        for text in ["*", "\"", "()", "-", "'; DROP TABLE visit; --"] {
            let _ = found(&store, text);
        }
        assert_eq!(found(&store, "*"), [""; 0]);
        assert_eq!(found(&store, "").len(), 4, "and the table is still there");
    }

    #[test]
    fn the_words_typed_become_quoted_prefixes() {
        assert_eq!(match_expression("rust"), Some("\"rust\"*".to_owned()));
        assert_eq!(
            match_expression("  doc.rust-lang.org/book  "),
            Some("\"doc\"* \"rust\"* \"lang\"* \"org\"* \"book\"*".to_owned())
        );
        assert_eq!(
            match_expression("\"a\" OR b*"),
            Some("\"a\"* \"OR\"* \"b\"*".to_owned())
        );
        assert_eq!(match_expression(" ./-* "), None);
        let many = "word ".repeat(40);
        assert_eq!(
            match_expression(&many).unwrap().matches('*').count(),
            MAX_WORDS
        );
    }

    #[test]
    fn another_login_sets_visits_are_not_seen() {
        let mut store = store();
        visit(&mut store, &PAGES);
        let theirs = store
            .record_visit(
                LoginSetId(THEIRS),
                "https://example.net/rust",
                "Their Rust",
                Timestamp(9),
            )
            .unwrap();

        assert_eq!(
            store
                .visits(LoginSetId(MINE), "", None, 100)
                .unwrap()
                .visits
                .len(),
            4
        );
        assert!(!found(&store, "rust").contains(&"Their Rust".to_owned()));
        let page = store.visits(LoginSetId(THEIRS), "rust", None, 100).unwrap();
        assert_eq!(titles(&page), ["Their Rust"]);
        // Nor forgotten by a login set that is not theirs.
        assert!(!store.forget_visit(LoginSetId(MINE), theirs).unwrap());
        store.forget_all_visits(LoginSetId(MINE)).unwrap();
        assert_eq!(
            store
                .visits(LoginSetId(THEIRS), "", None, 100)
                .unwrap()
                .visits
                .len(),
            1
        );
    }

    #[test]
    fn a_login_set_the_store_does_not_have_can_have_no_visit() {
        let mut store = Store::open_in_memory().unwrap();
        // As a private window's: it is never in the store.
        let refused =
            store.record_visit(LoginSetId(MINE), "https://example.com/", "", Timestamp(1));
        assert_eq!(refused.unwrap_err(), StoreError::Query);
    }

    #[test]
    fn a_title_that_arrives_later_is_found_by_its_words() {
        let mut store = store();
        let id = store
            .record_visit(LoginSetId(MINE), "https://example.com/", "", Timestamp(1))
            .unwrap();
        assert_eq!(found(&store, "domain"), [""; 0]);

        store.retitle_visit(id, "Example Domain").unwrap();
        assert_eq!(found(&store, "domain"), ["Example Domain"]);

        // And the words of the title it had before find it no longer.
        store.retitle_visit(id, "Another name").unwrap();
        assert_eq!(found(&store, "domain"), [""; 0]);
        assert_eq!(found(&store, "another"), ["Another name"]);
        // A visit that is not there is no error.
        store.retitle_visit(9_999, "Nothing").unwrap();
    }

    #[test]
    fn forgetting_one_visit_since_a_time_and_everything() {
        let mut store = store();
        let ids = visit(&mut store, &PAGES);

        assert!(store.forget_visit(LoginSetId(MINE), ids[1]).unwrap());
        assert!(
            !store.forget_visit(LoginSetId(MINE), ids[1]).unwrap(),
            "once"
        );
        assert_eq!(found(&store, "menu"), [""; 0], "its words find nothing");
        assert_eq!(found(&store, "").len(), 3);

        // At or after the third page's time: the third and the fourth.
        store
            .forget_visits_since(LoginSetId(MINE), Timestamp(3_000))
            .unwrap();
        assert_eq!(found(&store, ""), ["The Rust Programming Language"]);

        store.forget_all_visits(LoginSetId(MINE)).unwrap();
        assert_eq!(found(&store, ""), [""; 0]);
        assert_eq!(found(&store, "rust"), [""; 0]);
    }

    #[test]
    fn an_id_is_never_used_twice() {
        let mut store = store();
        let ids = visit(&mut store, &PAGES);
        store.forget_all_visits(LoginSetId(MINE)).unwrap();
        let next = store
            .record_visit(LoginSetId(MINE), "https://example.com/", "", Timestamp(1))
            .unwrap();
        assert!(
            next > ids[3],
            "a forgotten visit's id names nothing afterwards"
        );
    }

    #[test]
    fn what_is_forgotten_is_gone_from_the_files() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // Words that are nowhere else in the database.
        let secret_url = "https://quixotic-zebrafinch.example.com/xylophonist";
        let secret_title = "Wombatical Zeppelinry";
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
        store.default_space(MINE, Timestamp(1)).unwrap();
        let id = store
            .record_visit(LoginSetId(MINE), secret_url, "", Timestamp(2))
            .unwrap();
        store.retitle_visit(id, secret_title).unwrap();
        // Other visits stay, so the forgotten one's pages are not simply dropped with the table's.
        for n in 0..50 {
            store
                .record_visit(
                    LoginSetId(MINE),
                    &format!("https://example.com/{n}"),
                    "Kept",
                    Timestamp(3),
                )
                .unwrap();
        }
        assert!(
            holds("zebrafinch") && holds("Zeppelinry"),
            "the test can see what is written"
        );

        assert!(store.forget_visit(LoginSetId(MINE), id).unwrap());

        for needle in [
            "zebrafinch",
            "xylophonist",
            "Wombatical",
            "Zeppelinry",
            "wombatical",
            "zeppelinry",
        ] {
            assert!(
                !holds(needle),
                "{needle} is still in a file after it was forgotten"
            );
        }
        assert_eq!(found(&store, "kept").len(), 50, "and the rest is as it was");
    }

    /// A measurement, not a gate (008 §7): run it with
    /// `cargo test -p wt-store --release -- --ignored --nocapture measure`.
    #[test]
    #[ignore = "a measurement: it takes seconds and asserts nothing about time"]
    fn measure_looking_for_words_among_a_hundred_thousand_visits() {
        use std::time::Instant;

        const WORDS: [&str; 16] = [
            "rust",
            "garden",
            "railway",
            "timetable",
            "recipe",
            "weather",
            "museum",
            "library",
            "bicycle",
            "harbour",
            "orchestra",
            "lantern",
            "meadow",
            "compass",
            "quarry",
            "violin",
        ];
        let folder = tempfile::tempdir().unwrap();
        let mut store = Store::open(&folder.path().join("webtiger.sqlite")).unwrap();
        store.default_space(MINE, Timestamp(1)).unwrap();
        let filling = Instant::now();
        for n in 0..100_000_usize {
            let (a, b, c) = (WORDS[n % 16], WORDS[(n / 16) % 16], WORDS[(n / 256) % 16]);
            let url = format!("https://{a}.example.com/{b}/{c}/{n}");
            let title = format!("{a} {b} {c} number {n}");
            store
                .record_visit(LoginSetId(MINE), &url, &title, Timestamp(n as i64))
                .unwrap();
        }
        println!(
            "writing 100,000 visits, one at a time: {:?}",
            filling.elapsed()
        );

        let measure = |what: &str, text: &str| {
            let mut times: Vec<_> = (0..200)
                .map(|_| {
                    let asked = Instant::now();
                    let page = store.visits(LoginSetId(MINE), text, None, 100).unwrap();
                    assert!(!page.visits.is_empty(), "{what} finds something");
                    asked.elapsed()
                })
                .collect();
            times.sort();
            println!(
                "{what}: median {:?}, p95 {:?}, worst {:?}",
                times[100], times[190], times[199]
            );
        };
        measure("the newest hundred, no words", "");
        measure("a common word (one visit in sixteen)", "rust");
        measure("the start of a word", "rai");
        measure("three words together", "rust garden railway");
        measure("a word only one visit has", "99999");
    }

    #[test]
    fn a_store_from_before_history_gains_it_and_keeps_its_space() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("webtiger.sqlite");
        // Version 1, as W0 wrote it, with a Space in it.
        let old = Connection::open(&path).unwrap();
        old.execute_batch(
            "CREATE TABLE login_set (
                 id BLOB PRIMARY KEY NOT NULL CHECK (length(id) = 16),
                 created_at INTEGER NOT NULL
             ) STRICT, WITHOUT ROWID;
             CREATE TABLE space (
                 id INTEGER PRIMARY KEY,
                 name TEXT NOT NULL,
                 position INTEGER NOT NULL,
                 login_set_id BLOB NOT NULL REFERENCES login_set (id),
                 created_at INTEGER NOT NULL
             ) STRICT;
             CREATE INDEX space_login_set ON space (login_set_id);
             PRAGMA user_version = 1;",
        )
        .unwrap();
        old.execute(
            "INSERT INTO login_set (id, created_at) VALUES (?1, 1)",
            params![MINE.as_bytes().as_slice()],
        )
        .unwrap();
        old.execute(
            "INSERT INTO space (name, position, login_set_id, created_at) VALUES ('', 0, ?1, 1)",
            params![MINE.as_bytes().as_slice()],
        )
        .unwrap();
        drop(old);

        let mut store = Store::open(&path).unwrap();
        assert_eq!(store.schema_version().unwrap(), crate::SCHEMA_VERSION);
        let space = store.default_space(THEIRS, Timestamp(2)).unwrap();
        assert_eq!(
            space.login_set,
            LoginSetId(MINE),
            "the Space it had is the Space it has"
        );
        store
            .record_visit(
                space.login_set,
                "https://example.com/",
                "Example",
                Timestamp(3),
            )
            .unwrap();
        assert_eq!(found(&store, "example"), ["Example"]);
    }
}
