//! The person searches for words they selected in a page (023), through the
//! contract, against the scripted engine: when a tab opens, where, and what
//! it is sent to. No search is made: the engine here is a script.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_core::SEARCHED_LENGTH;
use wt_proto::v1::{ChooseSearchEngine, LoadState, TabView, host_request, request, response};
use wt_testkit::{FakeEngine, FakeShell, run};

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

/// The person chooses to search for words of a web view's page, as the
/// shell would report it.
async fn search(shell: &mut FakeShell, web_view: u64, words: &str) {
    shell.report(FakeEngine::search_requested(web_view, words, true));
    shell.settle().await;
}

/// A window whose one tab shows example.com. Its tab, subscription and the
/// tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64) {
    let (_, tab, subscription, web_view) = a_window(shell).await;
    (tab, subscription, web_view)
}

/// The same, with the window.
async fn a_window(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    shell.engine.page("https://example.com/", "Example");
    shell
        .engine
        .page(TIGERS, "tigers in the wild at DuckDuckGo");
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

#[test]
fn words_searched_for_open_in_a_new_tab_in_front_at_the_chosen_engine() {
    run(async {
        let mut shell = FakeShell::fake();
        let (tab, subscription, web_view) = a_page(&mut shell).await;
        let before = front(&shell, subscription);

        search(&mut shell, web_view, "tigers in the wild").await;

        // A second tab, after the one that has the words, in front, sent to
        // the engine's search for them.
        let now = ids(&shell, subscription);
        assert_eq!(now.len(), 2);
        assert_eq!(now[0], tab);
        assert_eq!(loads(&shell).last().map(String::as_str), Some(TIGERS));
        let new = front(&shell, subscription);
        assert_eq!(new.tab_id, now[1]);
        assert_ne!(new.web_view_id, web_view, "in a web view of its own");
        assert_eq!(new.title, "tigers in the wild at DuckDuckGo");
        assert_eq!(new.address.clone().unwrap().url, TIGERS);
        assert_eq!(
            new.login_set_id, before.login_set_id,
            "in the same login set"
        );

        // The page that has the words is as it was, behind.
        shell.activate_tab(tab).await;
        assert_eq!(front(&shell, subscription), before);

        // Closed, the search gives its place back to the tab it came from.
        shell.activate_tab(now[1]).await;
        shell.close_tab(now[1]).await;
        assert_eq!(ids(&shell, subscription), [tab]);
        assert_eq!(front(&shell, subscription).tab_id, tab);
    });
}

#[test]
fn the_search_goes_to_whichever_engine_is_chosen() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, web_view) = a_page(&mut shell).await;

        let chosen = shell
            .request(request::Kind::ChooseSearchEngine(ChooseSearchEngine {
                id: "ecosia".to_owned(),
            }))
            .await;
        assert!(matches!(chosen, response::Kind::Ack(_)), "{chosen:?}");
        search(&mut shell, web_view, "tigers in the wild").await;

        assert_eq!(
            loads(&shell).last().map(String::as_str),
            Some("https://www.ecosia.org/search?q=tigers+in+the+wild")
        );
    });
}

#[test]
fn words_that_look_like_an_address_are_searched_for_all_the_same() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, web_view) = a_page(&mut shell).await;

        // Typed in the address field these would be gone to, or refused.
        for (words, sent) in [
            ("example.org", "https://duckduckgo.com/?q=example.org"),
            (
                "https://example.org/a?b=c",
                "https://duckduckgo.com/?q=https%3A%2F%2Fexample.org%2Fa%3Fb%3Dc",
            ),
            (
                "javascript:alert(1)",
                "https://duckduckgo.com/?q=javascript%3Aalert%281%29",
            ),
        ] {
            search(&mut shell, web_view, words).await;
            assert_eq!(loads(&shell).last().map(String::as_str), Some(sent));
        }
        assert_eq!(ids(&shell, subscription).len(), 4);
    });
}

#[test]
fn the_words_are_tidied_and_long_ones_cut() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, web_view) = a_page(&mut shell).await;

        search(&mut shell, web_view, "  tigers \n in\tthe   wild ").await;
        assert_eq!(loads(&shell).last().map(String::as_str), Some(TIGERS));

        let long = "a".repeat(SEARCHED_LENGTH + 100);
        search(&mut shell, web_view, &long).await;
        assert_eq!(
            loads(&shell).last().cloned(),
            Some(format!(
                "https://duckduckgo.com/?q={}",
                "a".repeat(SEARCHED_LENGTH)
            ))
        );
    });
}

#[test]
fn no_words_and_nobody_asking_open_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (tab, subscription, web_view) = a_page(&mut shell).await;
        let before = loads(&shell);

        // Nothing selected, or nothing but white space.
        search(&mut shell, web_view, "").await;
        search(&mut shell, web_view, " \n\t ").await;
        // A page that only makes it look as if the person asked.
        shell.report(FakeEngine::search_requested(
            web_view,
            "tigers in the wild",
            false,
        ));
        shell.settle().await;
        // A web view that is no tab's.
        search(&mut shell, 4242, "tigers in the wild").await;

        assert_eq!(ids(&shell, subscription), [tab]);
        assert_eq!(loads(&shell), before, "nothing was sent anywhere");
    });
}

#[test]
fn the_new_tab_says_where_the_search_goes_from_the_start() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, web_view) = a_page(&mut shell).await;

        // The shell has not yet done what it was asked: the tab is there,
        // and nothing has come of its load.
        shell.hold_host = true;
        search(&mut shell, web_view, "tigers in the wild").await;

        // The person asked for it, and its item named where it goes: the
        // field shows the search, as for one typed there.
        let new = front(&shell, subscription);
        assert_eq!(new.load_state(), LoadState::Loading);
        assert_eq!(new.address.unwrap_or_default().url, TIGERS);

        shell.release_host();
        shell.settle().await;
        let new = front(&shell, subscription);
        assert_eq!(new.load_state(), LoadState::Idle);
        assert_eq!(new.title, "tigers in the wild at DuckDuckGo");
        assert_eq!(new.address.unwrap_or_default().url, TIGERS);
    });
}

#[test]
fn searches_one_after_another_stand_in_that_order_after_their_tab() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_window(&mut shell).await;
        shell.engine.page("https://example.com/later", "Later");
        shell.open_tab(window, "example.com/later").await;
        let later = *ids(&shell, subscription).last().unwrap();
        shell.activate_tab(tab).await;

        search(&mut shell, web_view, "one").await;
        let first = front(&shell, subscription).tab_id;
        shell.activate_tab(tab).await;
        search(&mut shell, web_view, "two").await;
        let second = front(&shell, subscription).tab_id;

        // After the tab that has the words and what it opened before, and
        // before what stood there already.
        assert_eq!(ids(&shell, subscription), [tab, first, second, later]);
    });
}
