//! Words handed to a shell to search for, from outside any page (028):
//! another app's selected text. Through the contract, against the scripted
//! engine: where the search opens, what it is sent to, and that the words
//! are never anything but a search. No search is made: the engine here is a
//! script.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_core::SEARCHED_LENGTH;
use wt_proto::v1::{
    ChooseSearchEngine, CreateSpace, SearchWords, TabView, host_request, request, response,
};
use wt_testkit::{FakeShell, run};

/// Where the engine that stands in sends a search for "tigers in the wild".
const TIGERS: &str = "https://duckduckgo.com/?q=tigers+in+the+wild";

/// The ids of the window's tabs, in the order they stand.
fn ids(shell: &FakeShell, subscription: u64) -> Vec<u64> {
    shell
        .window(subscription)
        .tabs
        .iter()
        .map(|tab| tab.tab_id)
        .collect()
}

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the core has asked web views to load, in order.
fn loads(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewLoad(load) => Some(load.url.clone()),
            _ => None,
        })
        .collect()
}

/// The shell hands the core words for a window, as another app handed
/// them to the shell.
async fn search(shell: &mut FakeShell, window: u64, words: &str) -> response::Kind {
    shell
        .request(request::Kind::SearchWords(SearchWords {
            window_id: window,
            text: words.to_owned(),
        }))
        .await
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// A window with two tabs: example.com in front, and a tab after it.
/// The window, its subscription, and the two tabs.
async fn a_window(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    shell.engine.page("https://example.com/", "Example");
    shell.engine.page("https://example.org/", "Another Place");
    shell
        .engine
        .page(TIGERS, "tigers in the wild at DuckDuckGo");
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, "example.com").await;
    let second = shell.open_tab(window, "example.org").await;
    shell.activate_tab(first).await;
    (window, subscription, first, second)
}

#[test]
fn words_from_outside_open_a_search_in_a_new_tab_in_front_after_the_tab_in_front() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, second) = a_window(&mut shell).await;
        let before = front(&shell, subscription);
        assert_eq!(before.tab_id, first);

        let answer = search(&mut shell, window, "tigers in the wild").await;

        // A third tab, after the one that was in front, in front, sent to
        // the chosen engine's search for the words.
        let response::Kind::TabOpened(opened) = answer else {
            panic!("expected the tab that opened, got {answer:?}");
        };
        let now = ids(&shell, subscription);
        assert_eq!(now, [first, opened.tab_id, second]);
        assert_eq!(loads(&shell).last().map(String::as_str), Some(TIGERS));
        let new = front(&shell, subscription);
        assert_eq!(new.tab_id, opened.tab_id);
        assert_eq!(new.title, "tigers in the wild at DuckDuckGo");
        // The person's own search: its field says where it went.
        assert_eq!(new.address.clone().unwrap().url, TIGERS);
        assert_eq!(new.login_set_id, before.login_set_id);

        // The tab that was in front is as it was, behind.
        shell.activate_tab(first).await;
        assert_eq!(front(&shell, subscription), before);
    });
}

#[test]
fn another_apps_words_are_always_searched_for_whatever_they_look_like() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, _, _) = a_window(&mut shell).await;
        // What typed in the address field would be an address, or would be
        // run if anything ran typed text.
        let looks_like = [
            "example.org",
            "https://example.org/",
            "http://192.168.1.1/admin",
            "javascript:alert(document.cookie)",
            "about:blank",
            "file:///etc/passwd",
            "data:text/html,<script>alert(1)</script>",
            "intent://scan/#Intent;scheme=zxing;end",
        ];
        for words in looks_like {
            let answer = search(&mut shell, window, words).await;

            assert!(matches!(answer, response::Kind::TabOpened(_)), "{words}");
            let loaded = loads(&shell).last().cloned().unwrap();
            assert!(
                loaded.starts_with("https://duckduckgo.com/?q="),
                "{words} was sent to {loaded}"
            );
            assert_ne!(loaded, words);
            let shown = front(&shell, subscription).address.unwrap();
            assert_eq!(shown.host, "duckduckgo.com", "{words}");
        }
        // Nothing but searches was loaded since the window's two pages.
        let since: Vec<String> = loads(&shell).into_iter().skip(2).collect();
        assert_eq!(since.len(), looks_like.len());
        assert!(
            since
                .iter()
                .all(|url| url.starts_with("https://duckduckgo.com/?q="))
        );
    });
}

#[test]
fn the_words_are_tidied_and_a_long_run_of_them_is_cut() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, _) = a_window(&mut shell).await;

        search(&mut shell, window, "  tigers \n in\tthe   wild \r\n").await;
        assert_eq!(loads(&shell).last().map(String::as_str), Some(TIGERS));

        let long = "w".repeat(SEARCHED_LENGTH + 250);
        search(&mut shell, window, &long).await;
        let expected = format!("https://duckduckgo.com/?q={}", "w".repeat(SEARCHED_LENGTH));
        assert_eq!(loads(&shell).last(), Some(&expected));
    });
}

#[test]
fn no_words_and_no_window_are_refused_and_open_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, second) = a_window(&mut shell).await;
        let loaded = loads(&shell).len();
        let before = front(&shell, subscription);

        for nothing in ["", " ", " \n\t \r\n "] {
            assert_eq!(
                error_code(search(&mut shell, window, nothing).await),
                "search.no_words"
            );
        }
        assert_eq!(
            error_code(search(&mut shell, window + 999, "tigers").await),
            "window.unknown"
        );
        // A window that is not there is said before anything of the words.
        assert_eq!(
            error_code(search(&mut shell, window + 999, "").await),
            "window.unknown"
        );

        assert_eq!(ids(&shell, subscription), [first, second]);
        assert_eq!(front(&shell, subscription), before);
        assert_eq!(loads(&shell).len(), loaded, "nothing was loaded");
    });
}

#[test]
fn the_search_goes_to_the_engine_the_person_chose() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, _) = a_window(&mut shell).await;
        shell
            .request(request::Kind::ChooseSearchEngine(ChooseSearchEngine {
                id: "ecosia".to_owned(),
            }))
            .await;

        search(&mut shell, window, "tigers in the wild").await;

        assert_eq!(
            loads(&shell).last().map(String::as_str),
            Some("https://www.ecosia.org/search?q=tigers+in+the+wild")
        );
    });
}

#[test]
fn the_search_opens_in_the_space_the_window_shows_with_that_spaces_logins() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, second) = a_window(&mut shell).await;
        let home = front(&shell, subscription).login_set_id;
        let home_space = shell.window(subscription).space_id;
        // A Space with logins of its own, which the window now shows.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                share_logins_with: 0,
                colour: String::new(),
                icon: String::new(),
            }))
            .await;
        let response::Kind::SpaceCreated(work) = made else {
            panic!("expected a Space, got {made:?}");
        };
        shell
            .request(request::Kind::SwitchSpace(wt_proto::v1::SwitchSpace {
                window_id: window,
                space_id: work.space_id,
            }))
            .await;
        let shown = shell.window(subscription);
        assert_eq!(shown.space_id, work.space_id);
        let empty = ids(&shell, subscription);
        assert_eq!(empty.len(), 1);

        let answer = search(&mut shell, window, "tigers in the wild").await;

        let response::Kind::TabOpened(opened) = answer else {
            panic!("expected the tab that opened, got {answer:?}");
        };
        assert_eq!(ids(&shell, subscription), [empty[0], opened.tab_id]);
        let new = front(&shell, subscription);
        assert_eq!(new.address.clone().unwrap().url, TIGERS);
        assert_ne!(new.login_set_id, home, "with the Space's own logins");
        // The first Space's tabs are as they were, when the window shows it again.
        shell
            .request(request::Kind::SwitchSpace(wt_proto::v1::SwitchSpace {
                window_id: window,
                space_id: home_space,
            }))
            .await;
        assert_eq!(ids(&shell, subscription), [first, second]);
    });
}
