//! The search engines the core knows (020): for each, a name, and the
//! address a search is sent to. One is the default, which a person keeps or
//! changes; none has a place here that it paid for (000 WT-D17).

use crate::address::{Address, search_address};

/// One search engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchEngine {
    /// The core's name for it: what a choice names it by, and what is
    /// written in the store.
    pub id: &'static str,
    /// Its own name, which is not translated.
    pub name: &'static str,
    /// The address a search is sent to, with `{query}` for the words.
    pub template: &'static str,
}

/// The engines a person chooses among, in the order of their names: an
/// order nobody can buy. Each is one anybody can use without an account.
///
/// Which engines are named here, and which is the default, the owner has
/// settled (020 §3).
pub const SEARCH_ENGINES: [SearchEngine; 7] = [
    SearchEngine {
        id: "bing",
        name: "Bing",
        template: "https://www.bing.com/search?q={query}",
    },
    SearchEngine {
        id: "brave",
        name: "Brave Search",
        template: "https://search.brave.com/search?q={query}",
    },
    SearchEngine {
        id: "duckduckgo",
        name: "DuckDuckGo",
        template: "https://duckduckgo.com/?q={query}",
    },
    SearchEngine {
        id: "ecosia",
        name: "Ecosia",
        template: "https://www.ecosia.org/search?q={query}",
    },
    SearchEngine {
        id: "google",
        name: "Google",
        template: "https://www.google.com/search?q={query}",
    },
    SearchEngine {
        id: "qwant",
        name: "Qwant",
        template: "https://www.qwant.com/?q={query}",
    },
    SearchEngine {
        id: "startpage",
        name: "Startpage",
        template: "https://www.startpage.com/do/search?q={query}",
    },
];

/// The default: where searches go until a person chooses another (the
/// owner's decision, 020 §3). It needs no account and no agreement, and it
/// pays nothing to be the default. At a first run the person is shown the
/// choice, with this one to keep.
pub const DEFAULT_SEARCH_ENGINE: &str = "duckduckgo";

/// The most characters of selected words that are searched for (023 §3): a
/// search for a whole page is nobody's wish.
pub const SEARCHED_LENGTH: usize = 500;

/// Words a person selected, as they are searched for (023 §3): with no white
/// space at the ends, every run of white space inside as one space, and no
/// more than [`SEARCHED_LENGTH`] characters. `None` when nothing is left.
#[must_use]
pub fn searched_words(selected: &str) -> Option<String> {
    let tidy = selected.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = tidy.chars().take(SEARCHED_LENGTH).collect();
    // What the cut left at its end is no part of a word.
    let words = cut.trim_end();
    (!words.is_empty()).then(|| words.to_owned())
}

/// The engine the core knows by that name.
#[must_use]
pub fn search_engine(id: &str) -> Option<&'static SearchEngine> {
    SEARCH_ENGINES.iter().find(|engine| engine.id == id)
}

impl SearchEngine {
    /// The address a search for `words` loads.
    #[must_use]
    pub fn search(&self, words: &str) -> Option<Address> {
        search_address(self.template, words)
    }

    /// Where searches are sent, for a shell to show under the name. Empty
    /// for a template that makes no address.
    #[must_use]
    pub fn host(&self) -> String {
        self.search("")
            .and_then(|address| address.host().map(str::to_owned))
            .unwrap_or_default()
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

    #[test]
    fn selected_words_are_tidied_and_cut_and_may_come_to_nothing() {
        assert_eq!(searched_words("tigers").as_deref(), Some("tigers"));
        // The ends, and every run of white space of any kind inside.
        assert_eq!(
            searched_words("  tigers \n\t in\u{a0}the   wild\r\n").as_deref(),
            Some("tigers in the wild")
        );
        // Nothing, and nothing but white space.
        assert_eq!(searched_words(""), None);
        assert_eq!(searched_words(" \n\t\u{a0} "), None);
        // Cut at so many characters, not bytes, and never in the middle of one.
        let long = "é".repeat(SEARCHED_LENGTH + 40);
        let cut = searched_words(&long).unwrap();
        assert_eq!(cut.chars().count(), SEARCHED_LENGTH);
        assert!(cut.chars().all(|c| c == 'é'));
        // Exactly as long is not cut; a space the cut leaves at the end goes.
        let exact = "a".repeat(SEARCHED_LENGTH);
        assert_eq!(searched_words(&exact).as_deref(), Some(exact.as_str()));
        let spaced = format!("{} b", "a".repeat(SEARCHED_LENGTH - 1));
        assert_eq!(
            searched_words(&spaced).as_deref(),
            Some("a".repeat(SEARCHED_LENGTH - 1).as_str())
        );
    }

    #[test]
    fn the_list_is_in_the_order_of_the_names_and_names_each_engine_once() {
        let names: Vec<String> = SEARCH_ENGINES
            .iter()
            .map(|engine| engine.name.to_lowercase())
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(names, sorted, "by name, and no name twice");

        let mut ids: Vec<&str> = SEARCH_ENGINES.iter().map(|engine| engine.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), SEARCH_ENGINES.len(), "no id twice");
        for engine in &SEARCH_ENGINES {
            assert!(
                engine
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()),
                "{}: an id is plain",
                engine.id
            );
        }
    }

    #[test]
    fn every_engine_makes_an_address_over_https_with_the_words_in_it() {
        for engine in &SEARCH_ENGINES {
            let address = engine
                .search("rust borrow checker & more")
                .unwrap_or_else(|| panic!("{} makes no address", engine.id));
            assert_eq!(address.scheme(), "https", "{}", engine.id);
            let written = address.as_str();
            assert!(
                written.ends_with("rust+borrow+checker+%26+more"),
                "{}: the words, written so that they stay words: {written}",
                engine.id
            );
            let host = engine.host();
            assert!(host.contains('.'), "{}: {host}", engine.id);
            assert!(written.contains(&host), "{}", engine.id);
        }
    }

    #[test]
    fn the_engine_that_stands_in_is_one_of_the_list_and_others_are_found_by_name() {
        let default = search_engine(DEFAULT_SEARCH_ENGINE).unwrap();
        assert_eq!(default.name, "DuckDuckGo");
        assert_eq!(default.host(), "duckduckgo.com");
        // Google and Bing are among the engines to choose (the owner's list).
        assert!(search_engine("google").is_some());
        assert!(search_engine("bing").is_some());
        assert_eq!(search_engine("ecosia").unwrap().host(), "www.ecosia.org");
        assert_eq!(search_engine("Ecosia"), None, "by its id, as written");
        assert_eq!(search_engine(""), None);
        assert_eq!(search_engine("altavista"), None);
    }
}
