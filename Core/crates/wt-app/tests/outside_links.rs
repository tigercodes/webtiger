//! A link for the system (007), through the contract, against the scripted
//! engine: which addresses the core hands to the system, for whom, and how
//! often.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::time::Duration;

use wt_app::OUTSIDE_LINK_INTERVAL;
use wt_proto::v1::{LoadState, TabView, host_request};
use wt_testkit::{FakeEngine, FakeShell, run};

const LETTER: &str = "mailto:ada@example.com";

/// What the system was asked to open, when it was asked for nothing.
const NOTHING: [&str; 0] = [];

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

fn ids(shell: &FakeShell, subscription: u64) -> Vec<u64> {
    shell
        .window(subscription)
        .tabs
        .iter()
        .map(|tab| tab.tab_id)
        .collect()
}

/// The addresses the core has asked the system to open, in order.
fn asked_to_open(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::OpenOutside(open) => Some(open.url.clone()),
            _ => None,
        })
        .collect()
}

/// A window whose one tab shows a page. Its window, tab, subscription and
/// the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    shell.engine.page("https://example.com/", "Example");
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

/// The page in a web view tries to go to an address for the system, and the
/// shell reports it.
async fn follow(shell: &mut FakeShell, web_view: u64, url: &str, user_gesture: bool) {
    shell.report(FakeEngine::outside_link(web_view, url, user_gesture));
    shell.settle().await;
}

#[test]
fn a_clicked_link_to_write_a_letter_is_handed_to_the_system() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        let snapshots = shell.snapshots(subscription).len();

        follow(&mut shell, web_view, LETTER, true).await;

        assert_eq!(asked_to_open(&shell), [LETTER], "once, as it was given");
        assert_eq!(shell.engine.opened_outside, [LETTER]);
        // The page that asked is as it was: no load began in it.
        let shown = front(&shell, subscription);
        assert_eq!(shown.tab_id, tab);
        assert_eq!(shown.title, "Example");
        assert_eq!(shown.load_state(), LoadState::Idle);
        assert_eq!(shown.address.unwrap().url, "https://example.com/");
        assert_eq!(
            shell.snapshots(subscription).len(),
            snapshots,
            "nothing the window shows has changed"
        );
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn a_number_to_ring_and_a_message_to_send_are_handed_on_too() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, _, web_view) = a_page(&mut shell).await;

        for url in ["tel:+15550100", "sms:+15550100?body=hello"] {
            follow(&mut shell, web_view, url, true).await;
            tokio::time::advance(OUTSIDE_LINK_INTERVAL).await;
        }

        assert_eq!(
            asked_to_open(&shell),
            ["tel:+15550100", "sms:+15550100?body=hello"]
        );
    });
}

#[test]
fn nothing_opens_when_nobody_asked() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, _, web_view) = a_page(&mut shell).await;

        // The page goes there by itself, as it loads.
        follow(&mut shell, web_view, LETTER, false).await;
        follow(&mut shell, web_view, "tel:+15550100", false).await;

        assert_eq!(asked_to_open(&shell), NOTHING);
    });
}

#[test]
fn another_apps_own_scheme_is_not_handed_on() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, _, web_view) = a_page(&mut shell).await;

        // Each can make another app act; until there is a way to ask the
        // person first, they are only refused.
        for url in [
            "zoommtg://zoom.us/join?confno=1",
            "itms-apps://apps.apple.com/app/id1",
            "intent://scan/#Intent;scheme=zxing;end",
        ] {
            follow(&mut shell, web_view, url, true).await;
            tokio::time::advance(OUTSIDE_LINK_INTERVAL).await;
        }

        assert_eq!(asked_to_open(&shell), NOTHING);
    });
}

#[test]
fn what_a_shell_calls_an_outside_link_is_checked_against_the_cores_own_policy() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, _, web_view) = a_page(&mut shell).await;

        // A shell that is confused, or worse: none of these is for the system.
        for url in [
            "https://example.com/",
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,x",
            "",
            "ada@example.com",
        ] {
            follow(&mut shell, web_view, url, true).await;
            tokio::time::advance(OUTSIDE_LINK_INTERVAL).await;
        }

        assert_eq!(asked_to_open(&shell), NOTHING);
    });
}

#[test]
fn a_web_view_no_tab_has_opens_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, _, web_view) = a_page(&mut shell).await;
        // Its tab is closed while the report is on its way.
        shell.open_tab(window, "").await;
        shell.close_tab(tab).await;

        follow(&mut shell, web_view, LETTER, true).await;
        follow(&mut shell, 4242, LETTER, true).await;

        assert_eq!(asked_to_open(&shell), NOTHING);
    });
}

#[test]
fn one_click_opens_one_thing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;

        // A page that asks ten times within the one click.
        for _ in 0..10 {
            follow(&mut shell, web_view, LETTER, true).await;
        }
        assert_eq!(asked_to_open(&shell), [LETTER]);

        // Not quite a second later: still no.
        tokio::time::advance(OUTSIDE_LINK_INTERVAL - Duration::from_millis(1)).await;
        follow(&mut shell, web_view, "tel:+15550100", true).await;
        assert_eq!(asked_to_open(&shell), [LETTER]);

        // Another tab's page is not held back by this one's.
        shell.engine.page("https://example.org/", "Another");
        shell.open_tab(window, "example.org").await;
        let other = front(&shell, subscription).web_view_id;
        follow(&mut shell, other, "sms:+15550100", true).await;
        assert_eq!(asked_to_open(&shell), [LETTER, "sms:+15550100"]);

        // A second after its last one, the first page may again.
        tokio::time::advance(Duration::from_millis(1)).await;
        follow(&mut shell, web_view, "tel:+15550100", true).await;
        assert_eq!(
            asked_to_open(&shell),
            [LETTER, "sms:+15550100", "tel:+15550100"]
        );
    });
}

#[test]
fn a_window_opened_by_a_click_carries_the_click_once() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, first, subscription, web_view) = a_page(&mut shell).await;
        // A link that opens a window on a letter to write. The engine makes
        // the window first, and asks about its address in it, where it no
        // longer says anyone clicked.
        let (new, report) = shell.engine.ask_for_window(web_view, "", true);
        shell.report(report);
        shell.settle().await;
        assert_eq!(ids(&shell, subscription).len(), 2);

        shell.report(FakeEngine::outside_link(new, LETTER, false));
        shell.report(FakeEngine::navigation_refused(new, LETTER));
        shell.settle().await;

        assert_eq!(
            asked_to_open(&shell),
            [LETTER],
            "the click that opened it asked"
        );
        // And the window, which had nowhere else to go, is closed (006).
        assert_eq!(ids(&shell, subscription), [first]);
        assert_eq!(front(&shell, subscription).tab_id, first);
    });
}

#[test]
fn the_click_that_opened_a_window_is_good_for_one_address() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, _, web_view) = a_page(&mut shell).await;
        let (new, report) = shell.engine.ask_for_window(web_view, "", true);
        shell.report(report);
        shell.settle().await;

        // A window left open and empty, which the page that opened it sends
        // to one address after another; no refusal closes it.
        follow(&mut shell, new, LETTER, false).await;
        tokio::time::advance(OUTSIDE_LINK_INTERVAL).await;
        follow(&mut shell, new, "tel:+15550100", false).await;
        tokio::time::advance(OUTSIDE_LINK_INTERVAL).await;
        follow(&mut shell, new, "sms:+15550100", false).await;

        assert_eq!(asked_to_open(&shell), [LETTER]);
    });
}

#[test]
fn a_window_that_has_had_a_page_does_not_carry_the_click() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/popup", "Popup");
        let (_, _, _, web_view) = a_page(&mut shell).await;
        let (new, report) = shell.engine.ask_for_window(web_view, "", true);
        shell.report(report);
        for event in shell.engine.goes_to(new, "https://example.com/popup") {
            shell.report(event);
        }
        shell.settle().await;

        // Its page now goes somewhere by itself: nobody asked.
        follow(&mut shell, new, LETTER, false).await;
        assert_eq!(asked_to_open(&shell), NOTHING);

        // A click in it is a click.
        follow(&mut shell, new, LETTER, true).await;
        assert_eq!(asked_to_open(&shell), [LETTER]);
    });
}

#[test]
fn a_device_with_nothing_to_open_it_breaks_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        shell.engine.nothing_opens_outside = true;

        follow(&mut shell, web_view, LETTER, true).await;

        assert_eq!(asked_to_open(&shell), [LETTER], "the system was asked");
        assert_eq!(shell.engine.opened_outside, NOTHING);
        let shown = front(&shell, subscription);
        assert_eq!(shown.tab_id, tab);
        assert_eq!(
            shown.load_state(),
            LoadState::Idle,
            "and the page is as it was"
        );
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn a_shell_that_does_not_know_the_request_breaks_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        // A shell built before the System service existed.
        shell.answer_next_with = Some(wt_proto::v1::host_response::Kind::Error(
            wt_proto::v1::Error {
                code: "host.unsupported".to_owned(),
                ..Default::default()
            },
        ));

        follow(&mut shell, web_view, LETTER, true).await;

        assert_eq!(asked_to_open(&shell), [LETTER]);
        assert_eq!(front(&shell, subscription).title, "Example");
        assert_eq!(shell.pending_host_requests(), 0);
        // The core is as alive as before.
        tokio::time::advance(OUTSIDE_LINK_INTERVAL).await;
        follow(&mut shell, web_view, "tel:+15550100", true).await;
        assert_eq!(asked_to_open(&shell), [LETTER, "tel:+15550100"]);
    });
}
