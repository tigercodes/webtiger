//! Places to offer for what is typed in the address field (011): from a
//! login set's own history, best first.

use std::collections::HashMap;

use rusqlite::params;
use wt_core::{Address, LoginSetId};

use crate::history::match_expression;
use crate::{Store, StoreError};

/// The most places one answer offers.
pub const MAX_SUGGESTIONS: u32 = 20;

/// How many of the newest matching visits are looked at. It is what keeps an
/// answer quick however much history there is: the cost is this many rows,
/// not all that match. A place not been to in so long is seldom what is
/// meant by a few letters.
const VISITS_LOOKED_AT: i64 = 500;

/// A place that was visited, offered again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggested {
    pub url: String,
    /// The title it last had. Empty when it never had one.
    pub title: String,
}

/// A place among the visits looked at.
struct Place {
    url: String,
    title: String,
    visits: u32,
    /// Its newest visit's id: greater is later.
    newest: i64,
    site_begins: bool,
}

impl Store {
    /// The places to offer for `text`, best first, at most `limit` of them
    /// (and never more than [`MAX_SUGGESTIONS`]).
    ///
    /// A place matches when every word of the text is found in its title or
    /// address, as a word or the start of one. It is offered once, however
    /// often it was visited. Places whose site begins with what is typed
    /// come first; then the more often visited; then the more lately.
    ///
    /// # Errors
    /// `Query` when the database refuses.
    pub fn suggestions(
        &self,
        login_set: LoginSetId,
        text: &str,
        limit: u32,
    ) -> Result<Vec<Suggested>, StoreError> {
        let Some(words) = match_expression(text) else {
            return Ok(Vec::new());
        };
        let mut statement = self
            .connection
            .prepare_cached(
                "SELECT visit.id, visit.url, visit.title \
                 FROM visit_text JOIN visit ON visit.id = visit_text.rowid \
                 WHERE visit_text MATCH ?1 AND visit.login_set_id = ?2 \
                 ORDER BY visit_text.rowid DESC LIMIT ?3",
            )
            .map_err(|_| StoreError::Query)?;
        let visits: Vec<(i64, String, String)> = statement
            .query_map(
                params![words, login_set.0.as_bytes().as_slice(), VISITS_LOOKED_AT],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .and_then(Iterator::collect)
            .map_err(|_| StoreError::Query)?;

        let typed = typed_site(text);
        let mut places: Vec<Place> = Vec::new();
        let mut by_url: HashMap<String, usize> = HashMap::new();
        // Newest first: a place's first row here is its newest visit.
        for (id, url, title) in visits {
            if let Some(index) = by_url.get(&url) {
                let place = &mut places[*index];
                place.visits += 1;
                if place.title.is_empty() {
                    place.title = title;
                }
                continue;
            }
            by_url.insert(url.clone(), places.len());
            places.push(Place {
                site_begins: typed
                    .as_deref()
                    .is_some_and(|typed| site_begins(&url, typed)),
                url,
                title,
                visits: 1,
                newest: id,
            });
        }
        places.sort_by(|a, b| {
            b.site_begins
                .cmp(&a.site_begins)
                .then(b.visits.cmp(&a.visits))
                .then(b.newest.cmp(&a.newest))
        });
        places.truncate(limit.clamp(1, MAX_SUGGESTIONS) as usize);
        Ok(places
            .into_iter()
            .map(|place| Suggested {
                url: place.url,
                title: place.title,
            })
            .collect())
    }
}

/// What is typed, as the beginning of a site's name: lower case, without a
/// scheme or "www.", up to its first slash. `None` when it could not be one
/// (it has a space in it, or nothing is left).
fn typed_site(text: &str) -> Option<String> {
    let text = text.trim().to_lowercase();
    if text.contains(char::is_whitespace) {
        return None;
    }
    let text = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
        .unwrap_or(&text);
    let text = text.strip_prefix("www.").unwrap_or(text);
    let site = text.split('/').next().unwrap_or_default();
    (!site.is_empty()).then(|| site.to_owned())
}

/// Whether an address's site, without "www.", begins with what was typed.
fn site_begins(url: &str, typed: &str) -> bool {
    Address::parse(url).is_ok_and(|address| {
        address
            .host()
            .is_some_and(|host| host.strip_prefix("www.").unwrap_or(host).starts_with(typed))
    })
}

#[cfg(test)]
mod tests {
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

    /// Visits, oldest first.
    fn visit(store: &mut Store, pages: &[(&str, &str)]) {
        for (url, title) in pages {
            store
                .record_visit(LoginSetId(MINE), url, title, Timestamp(1))
                .unwrap();
        }
    }

    fn offered(store: &Store, text: &str) -> Vec<String> {
        store
            .suggestions(LoginSetId(MINE), text, 6)
            .unwrap()
            .into_iter()
            .map(|place| place.url)
            .collect()
    }

    const BOOK: (&str, &str) = (
        "https://doc.rust-lang.org/book/",
        "The Rust Programming Language",
    );
    const RUSTIC: (&str, &str) = ("https://example.com/rustic-furniture", "Rustic furniture");
    const NEWS: (&str, &str) = ("https://news.example.org/today", "Today's news");

    #[test]
    fn a_place_is_offered_by_a_word_of_its_title_or_its_address() {
        let mut store = store();
        visit(&mut store, &[BOOK, RUSTIC, NEWS]);
        assert_eq!(offered(&store, "programming"), [BOOK.0]);
        assert_eq!(offered(&store, "furni"), [RUSTIC.0], "the start of a word");
        assert_eq!(
            offered(&store, "lang.org"),
            [BOOK.0],
            "words of the address"
        );
        assert_eq!(offered(&store, "rus boo"), [BOOK.0], "every word typed");
        assert_eq!(offered(&store, "rust zebra"), [""; 0]);
        assert_eq!(offered(&store, "zebra"), [""; 0]);
    }

    #[test]
    fn a_place_is_offered_once_with_the_title_it_last_had() {
        let mut store = store();
        visit(
            &mut store,
            &[
                ("https://example.com/", "An old name"),
                NEWS,
                ("https://example.com/", "Example Domain"),
                ("https://example.com/", ""),
            ],
        );
        let places = store.suggestions(LoginSetId(MINE), "example", 6).unwrap();
        let home: Vec<_> = places
            .iter()
            .filter(|place| place.url == "https://example.com/")
            .collect();
        assert_eq!(home.len(), 1, "once, however often it was visited");
        assert_eq!(
            home[0].title, "Example Domain",
            "the last title it had: a visit that had none says nothing"
        );
    }

    #[test]
    fn a_site_that_begins_with_what_is_typed_comes_first() {
        let mut store = store();
        // Visited once, long ago; the others often and since.
        visit(
            &mut store,
            &[
                ("https://rust-lang.org/", "Rust"),
                BOOK,
                BOOK,
                RUSTIC,
                RUSTIC,
                RUSTIC,
            ],
        );
        assert_eq!(
            offered(&store, "rus"),
            ["https://rust-lang.org/", RUSTIC.0, BOOK.0],
            "the site named so, then the more often visited"
        );
        // With or without "www.", a scheme, or a slash after it.
        visit(&mut store, &[("https://www.rustacean.net/", "Rustacean")]);
        for typed in [
            "rusta",
            "www.rusta",
            "https://rusta",
            "RUSTA",
            "rustacean.net/",
        ] {
            assert_eq!(
                offered(&store, typed).first().map(String::as_str),
                Some("https://www.rustacean.net/"),
                "{typed}"
            );
        }
    }

    #[test]
    fn then_the_more_often_visited_then_the_more_lately() {
        let mut store = store();
        visit(
            &mut store,
            &[
                // Twice, and both times before the others: the more visited is
                // not also the latest.
                ("https://example.com/b", "Page beta"),
                ("https://example.com/b", "Page beta"),
                ("https://example.com/a", "Page alpha"),
                ("https://example.com/c", "Page gamma"),
            ],
        );
        assert_eq!(
            offered(&store, "page"),
            [
                "https://example.com/b",
                "https://example.com/c",
                "https://example.com/a"
            ],
            "twice visited first; of the others, the later one"
        );
    }

    #[test]
    fn words_typed_with_a_space_are_not_a_sites_name() {
        let mut store = store();
        visit(
            &mut store,
            &[("https://page.example.com/", "Somewhere"), RUSTIC, RUSTIC],
        );
        // "page" alone begins a site's name; "rustic page" does not.
        visit(&mut store, &[("https://example.com/p", "Rustic page")]);
        assert_eq!(
            offered(&store, "rustic p"),
            ["https://example.com/p"],
            "only the place with both words"
        );
        assert_eq!(typed_site("rustic page"), None);
        assert_eq!(
            typed_site("  Example.COM/path "),
            Some("example.com".to_owned())
        );
        assert_eq!(typed_site("https://"), None);
    }

    #[test]
    fn how_many_are_offered() {
        let mut store = store();
        let many: Vec<(String, String)> = (0..40)
            .map(|n| (format!("https://example.com/{n}"), format!("Page {n}")))
            .collect();
        for (url, title) in &many {
            store
                .record_visit(LoginSetId(MINE), url, title, Timestamp(1))
                .unwrap();
        }
        let count = |limit| {
            store
                .suggestions(LoginSetId(MINE), "page", limit)
                .unwrap()
                .len()
        };
        assert_eq!(count(6), 6);
        assert_eq!(count(1), 1);
        assert_eq!(count(0), 1, "asking for none is asking for one");
        assert_eq!(count(u32::MAX), MAX_SUGGESTIONS as usize);
    }

    #[test]
    fn another_login_sets_places_are_not_offered() {
        let mut store = store();
        visit(&mut store, &[BOOK]);
        store
            .record_visit(
                LoginSetId(THEIRS),
                "https://example.net/rust",
                "Their Rust",
                Timestamp(1),
            )
            .unwrap();
        assert_eq!(offered(&store, "rust"), [BOOK.0]);
    }

    #[test]
    fn text_with_no_word_offers_nothing() {
        let mut store = store();
        visit(&mut store, &[BOOK, RUSTIC]);
        for text in ["", "   ", "/", "://", "\"", "*"] {
            assert_eq!(offered(&store, text), [""; 0], "{text:?}");
        }
    }

    /// "Address-field suggestions over 100,000 history rows: ≤ 10 ms at p95"
    /// (000 §7; 011 §7). A time, so not among the tests that run every
    /// time (060): `Tools/test.sh budgets` runs it, on a release build, and
    /// it fails at the budget.
    #[test]
    #[ignore = "timed (060): `Tools/test.sh budgets` runs it, on a release build; it fails at the plan's 10 ms at p95"]
    fn measure_offering_places_among_a_hundred_thousand_visits() {
        use std::time::{Duration, Instant};

        const BUDGET: Duration = Duration::from_millis(10);

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
        // 100,000 visits of 5,000 places: some places many times over.
        for n in 0..100_000_usize {
            let place = (n * 7919) % 5_000;
            let (a, b, c) = (
                WORDS[place % 16],
                WORDS[(place / 16) % 16],
                WORDS[(place / 256) % 16],
            );
            let url = format!("https://{a}{place}.example.com/{b}/{c}");
            let title = format!("{a} {b} {c} place {place}");
            store
                .record_visit(LoginSetId(MINE), &url, &title, Timestamp(n as i64))
                .unwrap();
        }
        let measure = |what: &str, text: &str| {
            let mut times: Vec<_> = (0..200)
                .map(|_| {
                    let asked = Instant::now();
                    let places = store.suggestions(LoginSetId(MINE), text, 6).unwrap();
                    assert!(!places.is_empty(), "{what} offers something");
                    asked.elapsed()
                })
                .collect();
            times.sort();
            println!(
                "places offered for {what}, among 100,000 visits: median {:?}, p95 {:?}, worst {:?} ({} {}, {} build)",
                times[100],
                times[190],
                times[199],
                std::env::consts::OS,
                std::env::consts::ARCH,
                if cfg!(debug_assertions) {
                    "a debug"
                } else {
                    "a release"
                }
            );
            assert!(
                times[190] <= BUDGET,
                "{what}: {:?} at p95, and it may be {BUDGET:?}",
                times[190]
            );
        };
        measure("one letter", "r");
        measure("two letters", "ru");
        measure("a word", "rust");
        measure("the start of a site's name", "rust1");
        measure("two words", "garden rail");
        measure("three words", "rust garden railway");
    }
}
