//! Several tabs in a window (005), through the contract, against the scripted
//! engine: their order, the one in front, and what closing one leaves.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    CloseWindow, LoadState, NavigationCommand, OpenTab, TabCommand, TabSummary, TabView,
    host_request, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

/// The window's tabs as a strip shows them, in order.
fn strip(shell: &FakeShell, subscription: u64) -> Vec<TabSummary> {
    shell.window(subscription).tabs
}

fn ids(shell: &FakeShell, subscription: u64) -> Vec<u64> {
    strip(shell, subscription)
        .iter()
        .map(|tab| tab.tab_id)
        .collect()
}

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

fn summary(shell: &FakeShell, subscription: u64, tab: u64) -> TabSummary {
    strip(shell, subscription)
        .into_iter()
        .find(|summary| summary.tab_id == tab)
        .expect("the tab is in the strip")
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

fn is_ack(answer: &response::Kind) -> bool {
    matches!(answer, response::Kind::Ack(_))
}

/// Opens a tab with every field of the request chosen.
async fn open(shell: &mut FakeShell, window: u64, text: &str, background: bool, after: u64) -> u64 {
    let answer = shell
        .request(request::Kind::OpenTab(OpenTab {
            window_id: window,
            text: text.to_owned(),
            background,
            after_tab_id: after,
        }))
        .await;
    match answer {
        response::Kind::TabOpened(opened) => opened.tab_id,
        other => panic!("OpenTab answers TabOpened, not {other:?}"),
    }
}

fn web_view_of(shell: &FakeShell, subscription: u64, tab: u64) -> u64 {
    // Only the tab in front says which web view is its own.
    let front = front(shell, subscription);
    assert_eq!(front.tab_id, tab, "the tab is in front");
    front.web_view_id
}

// Opening

#[test]
fn a_new_window_lists_its_one_tab() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription) = shell.open_window().await;
        assert_eq!(ids(&shell, subscription), [tab]);
        assert_eq!(front(&shell, subscription).tab_id, tab);
        let only = summary(&shell, subscription, tab);
        assert_eq!((only.title.as_str(), only.host.as_str()), ("", ""));
        assert!(!only.loading);
    });
}

#[test]
fn a_new_tab_is_empty_in_front_and_at_the_end() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        let requests_before = shell.host_requests.len();

        let second = shell.open_tab(window, "").await;

        assert_ne!(second, first);
        assert_eq!(ids(&shell, subscription), [first, second], "at the end");
        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, second, "in front");
        assert_eq!(tab.web_view_id, 0, "an empty tab has no web view");
        assert_eq!(tab.address.as_ref().unwrap().url, "");
        assert_eq!(tab.load_state(), LoadState::Idle);
        assert_eq!(
            shell.host_requests.len(),
            requests_before,
            "an empty tab costs the shell nothing"
        );
        // The tab behind it is as it was.
        assert_eq!(summary(&shell, subscription, first).title, "Example");
    });
}

#[test]
fn a_tab_opened_on_an_address_loads_it_in_a_web_view_of_its_own() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.org/", "Another");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        let first_web_view = web_view_of(&shell, subscription, first);

        let second = shell.open_tab(window, "example.org").await;

        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, second);
        assert_ne!(tab.web_view_id, 0);
        assert_ne!(
            tab.web_view_id, first_web_view,
            "each tab has its own web view"
        );
        assert_eq!(tab.address.as_ref().unwrap().url, "https://example.org/");
        assert_eq!(tab.title, "Another");
        let [
            ..,
            host_request::Kind::WebViewCreate(create),
            host_request::Kind::WebViewLoad(load),
        ] = shell.host_requests.as_slice()
        else {
            panic!(
                "expected a create then a load, got {:?}",
                shell.host_requests
            );
        };
        assert_eq!(create.web_view_id, tab.web_view_id);
        assert_eq!(load.url, "https://example.org/");
        assert_eq!(shell.engine.web_views.len(), 2, "both tabs are alive");
        let named = summary(&shell, subscription, second);
        assert_eq!(
            (named.title.as_str(), named.host.as_str()),
            ("Another", "example.org")
        );
    });
}

#[test]
fn a_tab_opened_on_words_searches_for_them() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription) = shell.open_window().await;
        let tab = shell.open_tab(window, "rust borrow checker").await;
        let shown = front(&shell, subscription);
        assert_eq!(shown.tab_id, tab);
        // The scripted web has no such page: what matters is where the tab was sent.
        let sent: Vec<_> = shell
            .host_requests
            .iter()
            .filter_map(|request| match request {
                host_request::Kind::WebViewLoad(load) => Some(load.url.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(sent, ["https://duckduckgo.com/?q=rust+borrow+checker"]);
    });
}

#[test]
fn a_tab_whose_page_fails_is_still_opened() {
    run(async {
        let mut shell = FakeShell::fake();
        shell
            .engine
            .fail("https://down.example.com/", "net.connect");
        let (window, _, subscription) = shell.open_window().await;
        let tab = shell.open_tab(window, "down.example.com").await;
        let shown = front(&shell, subscription);
        assert_eq!(shown.tab_id, tab);
        assert_eq!(shown.load_state(), LoadState::Failed);
        assert_eq!(shown.error_code, "net.connect");
    });
}

#[test]
fn a_tab_opened_behind_leaves_the_tab_in_front_where_it_is() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.org/", "Another");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;

        let behind = open(&mut shell, window, "example.org", true, 0).await;

        assert_eq!(ids(&shell, subscription), [first, behind]);
        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, first, "the tab in front has not moved");
        assert_eq!(tab.address.as_ref().unwrap().url, "https://example.com/");
        // It loaded all the same, where nobody is looking.
        let named = summary(&shell, subscription, behind);
        assert_eq!(
            (named.title.as_str(), named.host.as_str()),
            ("Another", "example.org")
        );
        assert!(!named.loading);
    });
}

#[test]
fn a_tab_can_be_put_after_another() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription) = shell.open_window().await;
        let second = shell.open_tab(window, "").await;
        let third = shell.open_tab(window, "").await;

        let beside_first = open(&mut shell, window, "", false, first).await;
        assert_eq!(
            ids(&shell, subscription),
            [first, beside_first, second, third]
        );

        // A tab that is not this window's says nothing about where: the end.
        let (_, elsewhere, _) = shell.open_window().await;
        let at_end = open(&mut shell, window, "", false, elsewhere).await;
        let unknown = open(&mut shell, window, "", false, 9_999).await;
        assert_eq!(
            ids(&shell, subscription),
            [first, beside_first, second, third, at_end, unknown]
        );
    });
}

#[test]
fn opening_a_tab_in_a_window_that_is_not_there_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription) = shell.open_window().await;
        let answer = shell
            .request(request::Kind::OpenTab(OpenTab {
                window_id: window + 500,
                ..OpenTab::default()
            }))
            .await;
        assert_eq!(error_code(answer), "window.unknown");
        assert_eq!(ids(&shell, subscription).len(), 1);
    });
}

// Bringing a tab in front

#[test]
fn activating_a_tab_brings_it_in_front_as_it_was_left() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.com/next", "Next");
        shell.engine.page("https://example.org/", "Another");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        shell.submit(first, "example.com/next").await;
        let first_web_view = web_view_of(&shell, subscription, first);
        let second = shell.open_tab(window, "example.org").await;
        assert_eq!(front(&shell, subscription).tab_id, second);

        let answer = shell.activate_tab(first).await;
        assert!(is_ack(&answer), "{answer:?}");

        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, first);
        assert_eq!(tab.web_view_id, first_web_view);
        assert_eq!(
            tab.address.as_ref().unwrap().url,
            "https://example.com/next"
        );
        assert_eq!(tab.title, "Next");
        assert!(tab.can_go_back, "its history is its own");
        assert_eq!(
            ids(&shell, subscription),
            [first, second],
            "the order has not changed"
        );

        // Again changes nothing, and says nothing.
        let before = shell.snapshots(subscription).len();
        assert!(is_ack(&shell.activate_tab(first).await));
        assert_eq!(shell.snapshots(subscription).len(), before);
    });
}

#[test]
fn activating_a_tab_that_is_not_there_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription) = shell.open_window().await;
        assert_eq!(
            error_code(shell.activate_tab(tab + 500).await),
            "tab.unknown"
        );
        assert_eq!(front(&shell, subscription).tab_id, tab);
    });
}

#[test]
fn a_window_has_its_own_tab_in_front() {
    run(async {
        let mut shell = FakeShell::fake();
        let (one, first_of_one, watching_one) = shell.open_window().await;
        let (two, first_of_two, watching_two) = shell.open_window().await;
        let second_of_one = shell.open_tab(one, "").await;
        let second_of_two = shell.open_tab(two, "").await;

        shell.activate_tab(first_of_two).await;

        assert_eq!(front(&shell, watching_two).tab_id, first_of_two);
        assert_eq!(
            front(&shell, watching_one).tab_id,
            second_of_one,
            "another window is not moved"
        );
        assert_eq!(ids(&shell, watching_one), [first_of_one, second_of_one]);
        assert_eq!(ids(&shell, watching_two), [first_of_two, second_of_two]);
    });
}

// Closing

#[test]
fn closing_a_tab_behind_destroys_its_web_view_and_moves_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.org/", "Another");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        let first_web_view = web_view_of(&shell, subscription, first);
        let second = shell.open_tab(window, "example.org").await;

        let answer = shell.close_tab(first).await;
        assert!(is_ack(&answer), "{answer:?}");

        assert_eq!(ids(&shell, subscription), [second]);
        assert_eq!(front(&shell, subscription).tab_id, second);
        assert!(
            matches!(shell.host_requests.last(), Some(host_request::Kind::WebViewDestroy(destroy)) if destroy.web_view_id == first_web_view),
            "the closed tab's web view is destroyed: {:?}",
            shell.host_requests.last()
        );
        assert!(!shell.engine.web_views.contains_key(&first_web_view));
        assert_eq!(shell.engine.web_views.len(), 1);
    });
}

#[test]
fn closing_the_tab_in_front_brings_the_one_after_it_forward() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription) = shell.open_window().await;
        let second = shell.open_tab(window, "").await;
        let third = shell.open_tab(window, "").await;
        shell.activate_tab(second).await;

        shell.close_tab(second).await;
        assert_eq!(ids(&shell, subscription), [first, third]);
        assert_eq!(
            front(&shell, subscription).tab_id,
            third,
            "the one after it"
        );

        // The last in the row has none after it: the one before comes forward.
        shell.close_tab(third).await;
        assert_eq!(ids(&shell, subscription), [first]);
        assert_eq!(front(&shell, subscription).tab_id, first);
    });
}

#[test]
fn closing_a_windows_only_tab_leaves_an_empty_one() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (window, only, subscription) = shell.open_window().await;
        shell.submit(only, "example.com").await;

        let answer = shell.close_tab(only).await;
        assert!(is_ack(&answer), "{answer:?}");

        let tabs = ids(&shell, subscription);
        assert_eq!(tabs.len(), 1, "a window always has a tab");
        assert_ne!(tabs[0], only, "and it is a new one");
        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, tabs[0]);
        assert_eq!(
            (tab.web_view_id, tab.address.as_ref().unwrap().url.as_str()),
            (0, "")
        );
        assert!(
            shell.engine.web_views.is_empty(),
            "the page that was there is gone"
        );
        assert_eq!(
            shell.window(subscription).window_id,
            window,
            "the window is still open"
        );
        // The new tab is a tab like any other.
        shell.submit(tabs[0], "example.com").await;
        assert_eq!(front(&shell, subscription).title, "Example");
    });
}

#[test]
fn closing_a_tab_that_is_not_there_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription) = shell.open_window().await;
        let second = shell.open_tab(window, "").await;
        assert_eq!(
            error_code(shell.close_tab(second + 500).await),
            "tab.unknown"
        );
        assert!(is_ack(&shell.close_tab(second).await));
        assert_eq!(
            error_code(shell.close_tab(second).await),
            "tab.unknown",
            "closed once"
        );
        assert_eq!(ids(&shell, subscription), [first]);
        // What is asked of a closed tab is refused too.
        assert_eq!(
            error_code(shell.submit(second, "example.com").await),
            "tab.unknown"
        );
        assert_eq!(error_code(shell.activate_tab(second).await), "tab.unknown");
    });
}

#[test]
fn closing_a_window_destroys_every_tabs_web_view() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.org/", "Another");
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        shell.open_tab(window, "example.org").await;
        shell.open_tab(window, "").await;
        assert_eq!(shell.engine.web_views.len(), 2);

        let answer = shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: window,
            }))
            .await;
        assert!(is_ack(&answer), "{answer:?}");
        assert!(
            shell.engine.web_views.is_empty(),
            "no tab's web view outlives its window"
        );
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

// A tab behind the others

#[test]
fn a_tab_behind_goes_on_loading_and_its_name_follows() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        let first_web_view = web_view_of(&shell, subscription, first);
        let second = shell.open_tab(window, "").await;

        // The page behind starts a load by itself, and it lands.
        shell.report(FakeEngine::navigation_started(
            first_web_view,
            "https://example.com/next",
        ));
        shell.settle().await;
        assert!(summary(&shell, subscription, first).loading);
        assert_eq!(
            summary(&shell, subscription, first).host,
            "example.com",
            "where a page is heading is not yet where it is"
        );
        shell.report(FakeEngine::navigation_committed(
            first_web_view,
            "https://other.example.org/",
        ));
        shell.report(FakeEngine::navigation_finished(first_web_view));
        shell.settle().await;

        let behind = summary(&shell, subscription, first);
        assert_eq!(behind.host, "other.example.org");
        assert!(!behind.loading);
        let tab = front(&shell, subscription);
        assert_eq!(
            tab.tab_id, second,
            "what a tab behind does moves nothing forward"
        );
        assert_eq!(
            tab.address.as_ref().unwrap().url,
            "",
            "nor does it reach the address field"
        );
    });
}

#[test]
fn what_a_tab_behind_reports_that_no_strip_shows_sends_no_snapshot() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        let first_web_view = web_view_of(&shell, subscription, first);
        shell.open_tab(window, "").await;
        let before = shell.snapshots(subscription).len();

        // Progress is the page's own, and a strip shows only whether a tab is loading.
        shell.report(FakeEngine::progress(first_web_view, 0.3));
        shell.report(FakeEngine::progress(first_web_view, 0.6));
        shell.settle().await;

        assert_eq!(shell.snapshots(subscription).len(), before);
    });
}

#[test]
fn a_command_goes_to_the_tab_it_names() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.com/next", "Next");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, "example.com").await;
        shell.submit(first, "example.com/next").await;
        let first_web_view = web_view_of(&shell, subscription, first);
        shell.open_tab(window, "").await;

        // Back, in the tab behind.
        let answer = shell
            .request(request::Kind::TabCommand(TabCommand {
                tab_id: first,
                command: NavigationCommand::Back as i32,
            }))
            .await;
        assert!(is_ack(&answer), "{answer:?}");
        assert!(
            matches!(shell.host_requests.last(), Some(host_request::Kind::WebViewCommand(command)) if command.web_view_id == first_web_view)
        );
        assert_eq!(summary(&shell, subscription, first).title, "Example");
    });
}

// Order, and what is published

#[test]
fn changes_that_arrive_together_are_one_snapshot() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription) = shell.open_window().await;
        let before = shell.snapshots(subscription).len();

        // Three requests handed over at once, before the core says anything.
        let empty = OpenTab {
            window_id: window,
            ..OpenTab::default()
        };
        shell.send(request::Kind::OpenTab(empty.clone()));
        shell.send(request::Kind::OpenTab(empty));
        shell.send(request::Kind::ActivateTab(wt_proto::v1::ActivateTab {
            tab_id: first,
            chosen_while_typing: false,
        }));
        shell.settle().await;

        assert_eq!(shell.snapshots(subscription).len(), before + 1);
        assert_eq!(ids(&shell, subscription).len(), 3);
        assert_eq!(
            front(&shell, subscription).tab_id,
            first,
            "they took effect in the order they came"
        );
    });
}

#[test]
fn a_tabs_page_is_asked_of_the_shell_before_the_next_request_is_looked_at() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.org/", "Another");
        let (window, _, _) = shell.open_window().await;
        shell.hold_host = true;

        for text in ["example.com", "example.org"] {
            shell.send(request::Kind::OpenTab(OpenTab {
                window_id: window,
                text: text.to_owned(),
                ..OpenTab::default()
            }));
        }
        shell.settle().await;

        // Each tab's web view is asked for and then its load, the first tab's before the second's.
        let asked: Vec<&str> = shell
            .host_requests
            .iter()
            .map(|request| match request {
                host_request::Kind::WebViewCreate(_) => "create",
                host_request::Kind::WebViewLoad(_) => "load",
                _ => "other",
            })
            .collect();
        assert_eq!(asked, ["create", "load", "create", "load"]);
        shell.release_host();
        shell.settle().await;
        assert_eq!(shell.engine.web_views.len(), 2);
    });
}
