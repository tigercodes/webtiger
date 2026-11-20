//! The search engines the core knows (020): for each, a name, and the
//! address a search is sent to. A person chooses one; none is chosen for
//! them, and none has a place here that it paid for (000 WT-D17).

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
/// Which engines are named here is the owner's to settle (020 §3).
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

/// The engine searches go to until a person has chosen: the one that stood
/// in before there was a choice (001 F3). It needs no account and no
/// agreement. It is not a default anybody was given: the person is asked.
pub const STAND_IN_SEARCH_ENGINE: &str = "duckduckgo";

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
        let stand_in = search_engine(STAND_IN_SEARCH_ENGINE).unwrap();
        assert_eq!(stand_in.name, "DuckDuckGo");
        assert_eq!(stand_in.host(), "duckduckgo.com");
        assert_eq!(search_engine("ecosia").unwrap().host(), "www.ecosia.org");
        assert_eq!(search_engine("Ecosia"), None, "by its id, as written");
        assert_eq!(search_engine(""), None);
        assert_eq!(search_engine("altavista"), None);
    }
}
