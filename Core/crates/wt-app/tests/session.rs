//! The tabs come back (009), through the contract, against the scripted
//! engine, over a store on disk: a shell is dropped, as a quit or a crash
//! leaves it, and another is started in the same folder.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{
    CloseWindow, LoadState, QueryHistory, SubmitAddress, TabView, host_request, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

/// Starts the app in `folder`, with the web it browses.
fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    shell.engine.page("https://example.com/", "Example Domain");
    shell.engine.page("https://example.org/", "Another Place");
    shell.engine.page(
        "https://doc.rust-lang.org/book/",
        "The Rust Programming Language",
    );
    shell.engine.page("https://example.net/", "A Third");
    shell
}

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// The strip: each tab's id.
fn ids(shell: &FakeShell, subscription: u64) -> Vec<u64> {
    shell
        .window(subscription)
        .tabs
        .iter()
        .map(|tab| tab.tab_id)
        .collect()
}

/// The strip: each tab's title and site.
fn strip(shell: &FakeShell, subscription: u64) -> Vec<(String, String)> {
    shell
        .window(subscription)
        .tabs
        .into_iter()
        .map(|tab| (tab.title, tab.host))
        .collect()
}

/// The addresses the shell was asked to load, in order: by a load, or by
/// waking a tab from its kept state (051), which names the address the tab
/// is at and comes to the same page.
fn loads(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewLoad(load) => Some(load.url.clone()),
            host_request::Kind::WebViewRestore(restore) => Some(restore.url.clone()),
            _ => None,
        })
        .collect()
}

fn web_views_made(shell: &FakeShell) -> usize {
    shell
        .host_requests
        .iter()
        .filter(|request| matches!(request, host_request::Kind::WebViewCreate(_)))
        .count()
}

/// The addresses of the tabs the store in `folder` keeps, in order, read
/// from the store itself.
fn kept(folder: &Path) -> Vec<String> {
    let mut store = wt_store::Store::open(&folder.join("webtiger.sqlite")).unwrap();
    // The Space the app made; the id offered here is not used.
    let space = store
        .default_space(
            wt_core::Uuid::from_random_bytes([0; 16]),
            wt_core::Timestamp(0),
        )
        .unwrap();
    store
        .saved_tabs(space.id)
        .unwrap()
        .into_iter()
        .map(|tab| tab.url)
        .collect()
}

fn named(title: &str, host: &str) -> (String, String) {
    (title.to_owned(), host.to_owned())
}

/// Three tabs, the second in front: example.com, example.org, the book.
async fn three_tabs(shell: &mut FakeShell) -> (u64, u64) {
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, "example.com").await;
    let second = shell.open_tab(window, "example.org").await;
    shell.open_tab(window, "doc.rust-lang.org/book/").await;
    shell.activate_tab(second).await;
    (window, subscription)
}

#[test]
fn a_new_store_opens_one_empty_tab() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (_, tab, subscription) = shell.open_window().await;
        assert_eq!(ids(&shell, subscription), [tab]);
        assert_eq!(front(&shell, subscription).address.unwrap().url, "");
        assert_eq!(loads(&shell), [""; 0]);
    });
}

#[test]
fn the_tabs_come_back_in_order_with_the_same_one_in_front() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        three_tabs(&mut shell).await;
        // A crash: nothing is closed, nothing is said.
        drop(shell);

        let mut shell = start(folder.path());
        let (_, tab, subscription) = shell.open_window().await;

        assert_eq!(
            strip(&shell, subscription),
            [
                named("Example Domain", "example.com"),
                named("Another Place", "example.org"),
                named("The Rust Programming Language", "doc.rust-lang.org"),
            ]
        );
        let shown = front(&shell, subscription);
        assert_eq!(shown.tab_id, tab, "the answer names the tab in front");
        assert_eq!(
            ids(&shell, subscription)[1],
            tab,
            "the one that was in front"
        );
        assert_eq!(shown.address.as_ref().unwrap().url, "https://example.org/");
        assert_eq!(shown.title, "Another Place");
        assert_eq!(shown.load_state(), LoadState::Idle, "and it has loaded");
        assert_ne!(shown.web_view_id, 0);
    });
}

#[test]
fn only_the_tab_in_front_is_loaded() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        three_tabs(&mut shell).await;
        drop(shell);

        let mut shell = start(folder.path());
        shell.open_window().await;

        assert_eq!(loads(&shell), ["https://example.org/"]);
        assert_eq!(
            web_views_made(&shell),
            1,
            "the others cost the shell nothing"
        );
    });
}

#[test]
fn a_tab_that_came_back_is_loaded_when_it_comes_in_front_and_once() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        three_tabs(&mut shell).await;
        drop(shell);
        let mut shell = start(folder.path());
        let (_, second, subscription) = shell.open_window().await;
        let first = ids(&shell, subscription)[0];

        let answer = shell.activate_tab(first).await;

        assert!(matches!(answer, response::Kind::Ack(_)));
        assert_eq!(
            loads(&shell),
            ["https://example.org/", "https://example.com/"]
        );
        let shown = front(&shell, subscription);
        assert_eq!(shown.tab_id, first);
        assert_eq!(shown.title, "Example Domain");
        assert_ne!(shown.web_view_id, 0);

        // Back and forth: each is loaded already.
        shell.activate_tab(second).await;
        shell.activate_tab(first).await;
        assert_eq!(loads(&shell).len(), 2);
        assert_eq!(web_views_made(&shell), 2);
    });
}

#[test]
fn closing_the_tab_in_front_loads_the_one_that_comes_forward() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        three_tabs(&mut shell).await;
        drop(shell);
        let mut shell = start(folder.path());
        let (_, second, subscription) = shell.open_window().await;

        shell.close_tab(second).await;

        // The tab after it comes forward (005), and was asleep.
        let shown = front(&shell, subscription);
        assert_eq!(shown.title, "The Rust Programming Language");
        assert_ne!(
            shown.web_view_id, 0,
            "the tab in front is never left asleep"
        );
        assert_eq!(
            loads(&shell),
            ["https://example.org/", "https://doc.rust-lang.org/book/"]
        );
    });
}

#[test]
fn typing_into_a_tab_that_has_not_loaded_loads_what_was_typed() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        three_tabs(&mut shell).await;
        drop(shell);
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let first = ids(&shell, subscription)[0];

        // Sent somewhere else while it is still behind, and asleep.
        shell.submit(first, "example.net").await;
        shell.activate_tab(first).await;

        assert_eq!(
            loads(&shell),
            ["https://example.org/", "https://example.net/"],
            "where it had been is not loaded as well"
        );
        assert_eq!(front(&shell, subscription).title, "A Third");
    });
}

#[test]
fn closing_a_tab_that_has_not_loaded_asks_the_shell_for_nothing() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        three_tabs(&mut shell).await;
        drop(shell);
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let first = ids(&shell, subscription)[0];
        let asked = shell.host_requests.len();

        shell.close_tab(first).await;

        assert_eq!(shell.host_requests.len(), asked);
        assert_eq!(ids(&shell, subscription).len(), 2);
    });
}

#[test]
fn a_tab_on_its_way_and_a_tab_that_failed_come_back_where_they_were_going() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, _) = shell.open_window().await;
        // A name that does not resolve: the network was down.
        shell.submit(first, "example.net").await;
        shell.engine.fail("https://example.com/", "net.offline");
        shell.submit(first, "example.com").await;
        // And a tab whose load the shell has not even begun.
        let second = shell.open_tab(window, "").await;
        shell.hold_host = true;
        shell.send(request::Kind::SubmitAddress(SubmitAddress {
            tab_id: second,
            text: "example.org".to_owned(),
        }));
        shell.settle().await;
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;

        assert_eq!(
            loads(&shell),
            ["https://example.org/"],
            "the tab in front goes where it was sent"
        );
        assert_eq!(front(&shell, subscription).title, "Another Place");
        // The page that failed has the address it failed at; with the
        // network back it loads.
        let first = ids(&shell, subscription)[0];
        shell.activate_tab(first).await;
        assert_eq!(loads(&shell)[1], "https://example.com/");
        assert_eq!(front(&shell, subscription).title, "Example Domain");
    });
}

#[test]
fn an_empty_tab_and_a_page_that_is_not_the_webs_come_back_empty() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        shell.open_tab(window, "").await;
        let third = shell.open_tab(window, "example.org").await;
        let web_view = front(&shell, subscription).web_view_id;
        // The third tab's page goes to a blank document of its own.
        for report in [
            FakeEngine::navigation_started(web_view, "about:blank"),
            FakeEngine::navigation_committed(web_view, "about:blank"),
            FakeEngine::navigation_finished(web_view),
        ] {
            shell.report(report);
        }
        shell.settle().await;
        assert_eq!(front(&shell, subscription).tab_id, third);
        drop(shell);

        // What the store itself holds: only an address of the web's is
        // written, whatever is read back.
        assert_eq!(
            kept(folder.path()),
            ["https://example.com/", "", ""],
            "the blank document's address is not kept"
        );

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;

        assert_eq!(
            strip(&shell, subscription),
            [
                named("Example Domain", "example.com"),
                named("", ""),
                named("", "")
            ]
        );
        // The one in front is empty: nothing is loaded for it, and going to
        // the other empty one asks for nothing either.
        assert_eq!(loads(&shell), [""; 0]);
        let second = ids(&shell, subscription)[1];
        shell.activate_tab(second).await;
        assert_eq!(loads(&shell), [""; 0]);
        assert_eq!(front(&shell, subscription).address.unwrap().url, "");
    });
}

#[test]
fn a_closed_tab_is_gone_and_stays_gone() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (_, subscription) = three_tabs(&mut shell).await;
        let first = ids(&shell, subscription)[0];
        shell.close_tab(first).await;
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;

        assert_eq!(
            strip(&shell, subscription),
            [
                named("Another Place", "example.org"),
                named("The Rust Programming Language", "doc.rust-lang.org"),
            ]
        );
    });
}

#[test]
fn closing_the_window_closes_none_of_its_tabs() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, _) = three_tabs(&mut shell).await;
        shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: window,
            }))
            .await;

        // Opened again in the same run: the tabs are there, as they are
        // after a start.
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(ids(&shell, subscription).len(), 3);
        assert_eq!(front(&shell, subscription).title, "Another Place");
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(ids(&shell, subscription).len(), 3);
    });
}

#[test]
fn a_second_window_starts_empty_and_is_not_the_session() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        three_tabs(&mut shell).await;

        // Another window beside the first.
        let (_, other_tab, other) = shell.open_window().await;
        assert_eq!(ids(&shell, other), [other_tab], "one empty tab");
        shell.submit(other_tab, "example.net").await;
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(
            ids(&shell, subscription).len(),
            3,
            "the first window's tabs are what comes back"
        );
        assert!(
            !strip(&shell, subscription).contains(&named("A Third", "example.net")),
            "and not the second's"
        );
    });
}

#[test]
fn the_title_kept_is_the_first_a_page_said() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        // A counter of unread letters, changing all the time.
        for unread in 1..=5 {
            shell.report(FakeEngine::title(
                web_view,
                &format!("({unread}) Example Domain"),
            ));
            shell.settle().await;
        }
        // Behind another tab when the app goes, so that it is not loaded
        // again on the way back.
        shell.open_tab(window, "example.org").await;
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(
            strip(&shell, subscription)[0],
            named("Example Domain", "example.com")
        );
    });
}

#[test]
fn the_last_tab_closed_leaves_an_empty_one_to_come_back() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (_, first, _) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        shell.close_tab(first).await;
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(strip(&shell, subscription), [named("", "")]);
        assert_eq!(loads(&shell), [""; 0]);
    });
}

#[test]
fn loading_a_page_that_came_back_is_a_visit() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (_, first, _) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        drop(shell);

        let mut shell = start(folder.path());
        let (window, _, _) = shell.open_window().await;
        let response::Kind::HistoryPage(page) = shell
            .request(request::Kind::QueryHistory(QueryHistory {
                window_id: window,
                ..Default::default()
            }))
            .await
        else {
            panic!("QueryHistory answers HistoryPage");
        };
        // Before the app went, and when it came back: the page was seen twice.
        assert_eq!(page.entries.len(), 2);
        assert!(
            page.entries
                .iter()
                .all(|entry| entry.url == "https://example.com/")
        );
    });
}

#[test]
fn fake_mode_keeps_no_session() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example Domain");
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        shell.open_tab(window, "").await;
        shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: window,
            }))
            .await;

        // Every window opens the same way: one tab, empty.
        let (_, tab, subscription) = shell.open_window().await;
        assert_eq!(ids(&shell, subscription), [tab]);
        assert_eq!(front(&shell, subscription).address.unwrap().url, "");
    });
}
