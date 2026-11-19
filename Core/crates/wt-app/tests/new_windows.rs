//! A page that asks for a new window (006), through the contract, against the
//! scripted engine: the web view the shell had to make is adopted into a tab
//! or refused, and a tab a page opened goes back to the tab it came from.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    EngineEvent, LoadState, NewWindowRequested, TabView, engine_event, host_request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

/// The bit that marks a web view's id as the shell's own.
const THE_SHELLS: u64 = 1 << 63;

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

/// The web views the core has had destroyed, in order.
fn destroyed(shell: &FakeShell) -> Vec<u64> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewDestroy(destroy) => Some(destroy.web_view_id),
            _ => None,
        })
        .collect()
}

/// The web views the core has asked for, in order.
fn created(shell: &FakeShell) -> Vec<u64> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewCreate(create) => Some(create.web_view_id),
            _ => None,
        })
        .collect()
}

/// A window whose one tab shows a page. Its window, tab, subscription and
/// the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    shell.engine.page("https://example.com/", "Example");
    shell.engine.page("https://example.com/popup", "Popup");
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

/// The page in `asking` asks for a window, and the shell reports it. The web
/// view the shell made.
async fn ask(shell: &mut FakeShell, asking: u64, user_gesture: bool) -> u64 {
    let (new, report) =
        shell
            .engine
            .ask_for_window(asking, "https://example.com/popup", user_gesture);
    shell.report(report);
    shell.settle().await;
    new
}

/// The web view goes where its page sends it, and the shell reports it.
async fn goes_to(shell: &mut FakeShell, web_view: u64, url: &str) {
    for report in shell.engine.goes_to(web_view, url) {
        shell.report(report);
    }
    shell.settle().await;
}

// Adopting

#[test]
fn a_window_asked_for_with_a_gesture_is_a_tab_beside_the_one_that_asked() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_web_view) = a_page(&mut shell).await;
        // A tab after the first, so that "beside" is not the same as "at the end".
        let last = shell.open_tab(window, "").await;
        shell.activate_tab(first).await;
        let asked_before = shell.host_requests.len();

        let new = ask(&mut shell, first_web_view, true).await;

        assert_ne!(new & THE_SHELLS, 0, "the shell named it, in its own ids");
        let tabs = ids(&shell, subscription);
        assert_eq!(tabs.len(), 3);
        assert_eq!(
            (tabs[0], tabs[2]),
            (first, last),
            "after the tab that asked"
        );
        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, tabs[1], "in front");
        assert_eq!(tab.web_view_id, new, "with the web view the shell made");
        assert_eq!(
            shell.host_requests.len(),
            asked_before,
            "the core asks the shell for nothing: the web view is there already"
        );
        assert!(shell.engine.web_views.contains_key(&new));
    });
}

#[test]
fn the_new_tab_has_no_address_until_its_page_commits() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, first_web_view) = a_page(&mut shell).await;

        let new = ask(&mut shell, first_web_view, true).await;

        // Where the page that asked wants it to go is that page's claim.
        let tab = front(&shell, subscription);
        assert_eq!(tab.address.as_ref().unwrap().url, "");
        assert_eq!(tab.title, "");
        let summary = &shell.window(subscription).tabs[1];
        assert_eq!((summary.title.as_str(), summary.host.as_str()), ("", ""));

        // A load that has only started says nothing either.
        shell.report(FakeEngine::navigation_started(
            new,
            "https://example.com/popup",
        ));
        shell.settle().await;
        let tab = front(&shell, subscription);
        assert_eq!(tab.address.as_ref().unwrap().url, "");
        assert_eq!(tab.load_state(), LoadState::Loading);

        // The page is there: now it is the tab's address.
        goes_to(&mut shell, new, "https://example.com/popup").await;
        let tab = front(&shell, subscription);
        assert_eq!(
            tab.address.as_ref().unwrap().url,
            "https://example.com/popup"
        );
        assert_eq!(tab.title, "Popup");
        assert_eq!(shell.window(subscription).tabs[1].host, "example.com");
    });
}

#[test]
fn the_new_tab_is_a_tab_like_any_other() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.org/", "Another");
        let (_, first, subscription, first_web_view) = a_page(&mut shell).await;
        let new = ask(&mut shell, first_web_view, true).await;
        goes_to(&mut shell, new, "https://example.com/popup").await;
        let opened = front(&shell, subscription).tab_id;

        // What is typed in it loads in its own web view, the one the shell made.
        shell.submit(opened, "example.org").await;
        assert!(shell.host_requests.iter().any(|request| matches!(
            request,
            host_request::Kind::WebViewLoad(load)
                if load.web_view_id == new && load.url == "https://example.org/"
        )));
        assert_eq!(created(&shell), [first_web_view], "and no other was made");
        assert_eq!(front(&shell, subscription).title, "Another");

        // Its opener is as it was, behind it.
        shell.activate_tab(first).await;
        let tab = front(&shell, subscription);
        assert_eq!(tab.web_view_id, first_web_view);
        assert_eq!(tab.title, "Example");
    });
}

#[test]
fn a_tab_a_page_opened_can_open_another() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, first, subscription, first_web_view) = a_page(&mut shell).await;
        let second_web_view = ask(&mut shell, first_web_view, true).await;
        let second = front(&shell, subscription).tab_id;

        let third_web_view = ask(&mut shell, second_web_view, true).await;

        assert_ne!(
            third_web_view, second_web_view,
            "the shell never reuses an id"
        );
        let tabs = ids(&shell, subscription);
        assert_eq!(tabs.len(), 3);
        assert_eq!((tabs[0], tabs[1]), (first, second));
        assert_eq!(front(&shell, subscription).web_view_id, third_web_view);
    });
}

// Refusing

#[test]
fn a_window_asked_for_with_no_gesture_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, first, subscription, first_web_view) = a_page(&mut shell).await;
        let snapshots = shell.snapshots(subscription).len();

        let new = ask(&mut shell, first_web_view, false).await;

        assert_eq!(ids(&shell, subscription), [first], "no tab");
        assert_eq!(destroyed(&shell), [new], "and the web view is destroyed");
        assert!(!shell.engine.web_views.contains_key(&new));
        assert_eq!(
            shell.snapshots(subscription).len(),
            snapshots,
            "nothing the window shows has changed"
        );
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn a_window_asked_for_by_a_web_view_the_core_does_not_have_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_web_view) = a_page(&mut shell).await;
        // The tab that asks is closed while its report is on its way.
        let second = shell.open_tab(window, "").await;
        let (new, report) =
            shell
                .engine
                .ask_for_window(first_web_view, "https://example.com/popup", true);
        shell.close_tab(first).await;

        shell.report(report);
        shell.settle().await;

        assert_eq!(ids(&shell, subscription), [second]);
        assert_eq!(destroyed(&shell), [first_web_view, new]);
    });
}

#[test]
fn a_web_view_under_an_id_that_is_not_a_shells_is_not_acted_on() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, first, subscription, first_web_view) = a_page(&mut shell).await;
        let report = |new_web_view_id| EngineEvent {
            web_view_id: first_web_view,
            kind: Some(engine_event::Kind::NewWindowRequested(NewWindowRequested {
                new_web_view_id,
                url: String::new(),
                user_gesture: true,
            })),
        };

        // An id of the core's own kind, the id of a web view there is, and none.
        for id in [7, first_web_view, 0] {
            shell.report(report(id));
            shell.settle().await;
        }

        assert_eq!(ids(&shell, subscription), [first], "no tab");
        assert_eq!(
            destroyed(&shell),
            [0u64; 0],
            "and nothing destroyed: least of all the web view that is the tab's"
        );
        assert!(shell.engine.web_views.contains_key(&first_web_view));
    });
}

#[test]
fn a_web_view_reported_twice_is_adopted_once() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, first_web_view) = a_page(&mut shell).await;
        let (new, report) =
            shell
                .engine
                .ask_for_window(first_web_view, "https://example.com/popup", true);

        shell.report(report.clone());
        shell.report(report);
        shell.settle().await;

        assert_eq!(ids(&shell, subscription).len(), 2);
        assert_eq!(destroyed(&shell), [0u64; 0], "the tab's web view is kept");
        assert_eq!(front(&shell, subscription).web_view_id, new);
    });
}

// Closing

#[test]
fn closing_a_tab_a_page_opened_brings_its_opener_forward() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_web_view) = a_page(&mut shell).await;
        // Tabs on both sides of where the new one goes: neither is the opener.
        let before = shell.open_tab(window, "").await;
        shell.activate_tab(first).await;
        let new = ask(&mut shell, first_web_view, true).await;
        let opened = front(&shell, subscription).tab_id;
        assert_eq!(ids(&shell, subscription), [first, opened, before]);

        let answer = shell.close_tab(opened).await;

        assert!(matches!(answer, response::Kind::Ack(_)));
        assert_eq!(ids(&shell, subscription), [first, before]);
        assert_eq!(
            front(&shell, subscription).tab_id,
            first,
            "the tab it came from, not the tab after it"
        );
        assert_eq!(destroyed(&shell), [new]);
    });
}

#[test]
fn with_its_opener_gone_closing_brings_the_next_tab_forward() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_web_view) = a_page(&mut shell).await;
        let last = shell.open_tab(window, "").await;
        shell.activate_tab(first).await;
        ask(&mut shell, first_web_view, true).await;
        let opened = front(&shell, subscription).tab_id;
        shell.close_tab(first).await;
        assert_eq!(front(&shell, subscription).tab_id, opened);

        shell.close_tab(opened).await;

        assert_eq!(ids(&shell, subscription), [last]);
        assert_eq!(front(&shell, subscription).tab_id, last);
    });
}

#[test]
fn a_page_may_close_the_tab_a_page_opened() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, first, subscription, first_web_view) = a_page(&mut shell).await;
        let new = ask(&mut shell, first_web_view, true).await;
        goes_to(&mut shell, new, "https://example.com/popup").await;

        shell.report(FakeEngine::close_requested(new));
        shell.settle().await;

        assert_eq!(ids(&shell, subscription), [first]);
        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, first, "back in the tab that opened it");
        assert_eq!(tab.title, "Example");
        assert_eq!(destroyed(&shell), [new]);
        assert!(!shell.engine.web_views.contains_key(&new));
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn a_page_may_not_close_a_tab_the_user_opened() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_web_view) = a_page(&mut shell).await;
        let second = shell.open_tab(window, "example.com/popup").await;
        let second_web_view = front(&shell, subscription).web_view_id;

        shell.report(FakeEngine::close_requested(first_web_view));
        shell.report(FakeEngine::close_requested(second_web_view));
        // And one from a web view there is not.
        shell.report(FakeEngine::close_requested(THE_SHELLS | 99));
        shell.settle().await;

        assert_eq!(ids(&shell, subscription), [first, second]);
        assert_eq!(destroyed(&shell), [0u64; 0]);
    });
}

#[test]
fn a_page_that_closes_the_windows_only_tab_leaves_an_empty_one() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, first, subscription, first_web_view) = a_page(&mut shell).await;
        let new = ask(&mut shell, first_web_view, true).await;
        let opened = front(&shell, subscription).tab_id;
        shell.close_tab(first).await;

        shell.report(FakeEngine::close_requested(new));
        shell.settle().await;

        // A window always has a tab (005, T1).
        let tabs = ids(&shell, subscription);
        assert_eq!(tabs.len(), 1);
        assert_ne!(tabs[0], opened);
        assert_eq!(front(&shell, subscription).web_view_id, 0);
        assert_eq!(destroyed(&shell), [first_web_view, new]);
    });
}

// A window that is refused the only place it was going

#[test]
fn a_window_whose_only_navigation_is_refused_is_closed() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_web_view) = a_page(&mut shell).await;
        let last = shell.open_tab(window, "").await;
        shell.activate_tab(first).await;
        let new = ask(&mut shell, first_web_view, true).await;
        assert_eq!(ids(&shell, subscription).len(), 3);

        // An address for the system (a letter to write, a number to ring):
        // the shell refused it on the core's verdict, and nothing else is
        // on its way.
        shell.report(FakeEngine::navigation_refused(
            new,
            "mailto:ada@example.com",
        ));
        shell.settle().await;

        assert_eq!(ids(&shell, subscription), [first, last], "its tab is gone");
        assert_eq!(
            front(&shell, subscription).tab_id,
            first,
            "and the tab it came from is in front, as if nothing had opened"
        );
        assert_eq!(destroyed(&shell), [new]);
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn a_load_that_was_stopped_or_replaced_closes_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, first_web_view) = a_page(&mut shell).await;
        let new = ask(&mut shell, first_web_view, true).await;

        // Cancelled, and not a refusal: another load took its place, or the
        // user stopped it. The window may yet show a page.
        shell.report(FakeEngine::navigation_started(
            new,
            "https://example.com/popup",
        ));
        shell.report(FakeEngine::navigation_failed(
            new,
            "net.cancelled",
            "https://example.com/popup",
        ));
        shell.settle().await;

        assert_eq!(ids(&shell, subscription).len(), 2);
        assert_eq!(front(&shell, subscription).web_view_id, new);
        assert_eq!(destroyed(&shell), [0u64; 0]);
        // And it does: the page that replaced it arrives.
        goes_to(&mut shell, new, "https://example.com/popup").await;
        assert_eq!(front(&shell, subscription).title, "Popup");
    });
}

#[test]
fn a_refusal_closes_only_a_window_that_has_had_no_page() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, first, subscription, first_web_view) = a_page(&mut shell).await;
        let new = ask(&mut shell, first_web_view, true).await;
        goes_to(&mut shell, new, "https://example.com/popup").await;
        let opened = front(&shell, subscription).tab_id;

        // The window has a page, which then tries to go somewhere refused:
        // the page is still there to look at.
        shell.report(FakeEngine::navigation_refused(
            new,
            "mailto:ada@example.com",
        ));
        // And the same in a tab the user opened, with a page or without.
        shell.report(FakeEngine::navigation_refused(
            first_web_view,
            "mailto:ada@example.com",
        ));
        shell.settle().await;

        assert_eq!(ids(&shell, subscription), [first, opened]);
        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, opened);
        assert_eq!(tab.title, "Popup");
        assert_eq!(tab.load_state(), LoadState::Idle);
        assert_eq!(destroyed(&shell), [0u64; 0]);
    });
}

#[test]
fn a_refusal_never_closes_a_tab_the_user_opened() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, _) = a_page(&mut shell).await;
        // A tab of the user's with a web view and no page: its load was stopped.
        shell.engine.page("https://example.org/", "Another");
        shell.hold_host = true;
        let opening = shell.send(wt_proto::v1::request::Kind::OpenTab(
            wt_proto::v1::OpenTab {
                window_id: window,
                text: "example.org".to_owned(),
                background: false,
                after_tab_id: 0,
            },
        ));
        shell.settle().await;
        let second = front(&shell, subscription).tab_id;
        let web_view = created(&shell)[1];
        shell.report(FakeEngine::navigation_failed(
            web_view,
            "net.cancelled",
            "https://example.org/",
        ));

        shell.report(FakeEngine::navigation_refused(
            web_view,
            "mailto:ada@example.com",
        ));
        shell.settle().await;

        assert_eq!(ids(&shell, subscription), [first, second]);
        assert_eq!(destroyed(&shell), [0u64; 0]);
        shell.release_host();
        shell.settle().await;
        assert!(shell.take_response(opening).is_some());
    });
}

#[test]
fn a_refusal_does_not_close_a_window_the_user_has_sent_somewhere() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.org/", "Another");
        let (_, _, subscription, first_web_view) = a_page(&mut shell).await;
        let new = ask(&mut shell, first_web_view, true).await;
        let opened = front(&shell, subscription).tab_id;

        // The user types an address in the new tab before its page has said
        // where it wants to go; the shell is slow to load it.
        shell.hold_host = true;
        let typed = shell.send(wt_proto::v1::request::Kind::SubmitAddress(
            wt_proto::v1::SubmitAddress {
                tab_id: opened,
                text: "example.org".to_owned(),
            },
        ));
        shell.settle().await;
        // Meanwhile what the page wanted is refused.
        shell.report(FakeEngine::navigation_refused(
            new,
            "mailto:ada@example.com",
        ));
        shell.settle().await;

        assert_eq!(
            ids(&shell, subscription).len(),
            2,
            "the tab is the user's now"
        );
        shell.release_host();
        shell.settle().await;
        assert!(matches!(
            shell.take_response(typed),
            Some(response::Kind::Ack(_))
        ));
        let tab = front(&shell, subscription);
        assert_eq!(tab.tab_id, opened);
        assert_eq!(tab.title, "Another");
    });
}

#[test]
fn closing_the_window_destroys_the_web_views_the_shell_made_too() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, first_web_view) = a_page(&mut shell).await;
        let new = ask(&mut shell, first_web_view, true).await;

        shell
            .request(wt_proto::v1::request::Kind::CloseWindow(
                wt_proto::v1::CloseWindow { window_id: window },
            ))
            .await;

        let mut gone = destroyed(&shell);
        gone.sort_unstable();
        assert_eq!(gone, [first_web_view, new]);
        assert!(shell.engine.web_views.is_empty());
    });
}
