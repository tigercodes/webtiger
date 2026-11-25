//! Fake mode opens a list of addresses by itself (063): for a run that is to
//! be measured with a list of sites open, and nothing in the app automated.
//! Through the contract, against the scripted engine: each address is opened
//! in its turn as one handed in from outside is (031); what is no address of
//! the web is passed over; the list waits for a window that is not private;
//! its timer stops when the list is done; and outside fake mode, or without
//! a number of seconds, nothing is opened.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{CloseWindow, EngineKind, FakeOptions, host_request, request, response};
use wt_testkit::{FakeShell, run};

const HOME: &str = "https://example.com/";
const A: &str = "https://a.example.com/";
const B: &str = "https://b.example.com/one?two=3";
const C: &str = "http://c.example.com/";

/// A configuration for fake mode with a list to open and the seconds between.
fn config(addresses: &[&str], every: u32) -> wt_proto::v1::CoreConfig {
    let mut config = FakeShell::config(EngineKind::Webkit);
    config.fake = Some(FakeOptions {
        open_addresses: addresses.iter().map(|one| (*one).to_owned()).collect(),
        open_every_seconds: every,
        ..FakeOptions::default()
    });
    config
}

/// The scripted engine knows the pages.
fn web(shell: &mut FakeShell) {
    for (url, title) in [(HOME, "Home"), (A, "A"), (B, "B"), (C, "C")] {
        shell.engine.page(url, title);
    }
}

/// The window's tabs by their names, in the order they stand; and the name
/// of the one in front.
fn row(shell: &FakeShell, subscription: u64) -> (Vec<String>, String) {
    let window = shell.window(subscription);
    (
        window.tabs.iter().map(|tab| tab.title.clone()).collect(),
        window.tab.expect("a tab").title,
    )
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

fn names(row: &[&str]) -> Vec<String> {
    row.iter().map(|one| (*one).to_owned()).collect()
}

#[test]
fn each_address_is_opened_in_its_turn_in_a_new_tab_in_front_and_then_the_timer_stops() {
    run(async {
        let mut shell = FakeShell::start(&config(&[A, B, C], 10)).unwrap();
        web(&mut shell);
        let (_, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;

        // Not before its time.
        shell.seconds_pass(9).await;
        assert_eq!(
            row(&shell, subscription),
            (names(&["Home"]), "Home".to_owned())
        );
        assert_eq!(shell.wakeups(), 0);

        // Each at its time: a new tab after the tab in front, in front, at the address as
        // it was given.
        shell.seconds_pass(2).await;
        assert_eq!(
            row(&shell, subscription),
            (names(&["Home", "A"]), "A".to_owned())
        );
        shell.seconds_pass(10).await;
        assert_eq!(
            row(&shell, subscription),
            (names(&["Home", "A", "B"]), "B".to_owned())
        );
        shell.seconds_pass(10).await;
        assert_eq!(
            row(&shell, subscription),
            (names(&["Home", "A", "B", "C"]), "C".to_owned())
        );
        assert_eq!(loads(&shell), [HOME, A, B, C]);
        let front = shell.window(subscription).tab.unwrap();
        assert_eq!(front.address.unwrap().url, C);

        // The core woke itself three times for them, and wakes itself for them no more:
        // the list is done. (The next thing that wakes it is the look for tabs to put to
        // sleep, a minute after the first tab left the front.)
        assert_eq!(shell.wakeups(), 3);
        shell.seconds_pass(30).await;
        assert_eq!(shell.wakeups(), 3);
        assert_eq!(row(&shell, subscription).0.len(), 4);
    });
}

#[test]
fn what_is_no_address_of_the_web_is_passed_over_and_the_next_is_opened_at_its_time() {
    run(async {
        let list = [
            A,
            "javascript:alert(1)",
            "file:///etc/hosts",
            "b.example.com",
            "",
            B,
        ];
        let mut shell = FakeShell::start(&config(&list, 10)).unwrap();
        web(&mut shell);
        let (_, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;

        shell.seconds_pass(11).await;
        assert_eq!(row(&shell, subscription).0, names(&["Home", "A"]));
        // Four that are none: no tab opens for any of them, nothing is loaded, searched
        // for or mended (a host with no scheme is not made an address), and nothing is
        // handed to the system.
        shell.seconds_pass(40).await;
        assert_eq!(row(&shell, subscription).0, names(&["Home", "A"]));
        assert_eq!(loads(&shell), [HOME, A]);
        shell.seconds_pass(10).await;
        assert_eq!(
            row(&shell, subscription),
            (names(&["Home", "A", "B"]), "B".to_owned())
        );
        assert_eq!(loads(&shell), [HOME, A, B]);
        assert!(!shell.host_requests.iter().any(|request| matches!(
            request,
            host_request::Kind::OpenOutside(_) | host_request::Kind::OpenFile(_)
        )));
    });
}

#[test]
fn the_list_waits_for_a_window_that_is_not_private_and_goes_to_the_first_there_is() {
    run(async {
        let mut shell = FakeShell::start(&config(&[A, B, C], 10)).unwrap();
        web(&mut shell);
        // A private window, and no other: what comes from outside never opens in one
        // (037). The list waits, and none of it is lost.
        let (_, _, private) = shell.open_private_window().await;
        shell.seconds_pass(35).await;
        assert_eq!(shell.window(private).tabs.len(), 1);
        assert_eq!(loads(&shell), Vec::<String>::new());

        // Two ordinary windows: the first of them has the list, from its first address.
        let (first_window, first, one) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let (_, _, two) = shell.open_window().await;
        shell.seconds_pass(10).await;
        assert_eq!(row(&shell, one).0, names(&["Home", "A"]));
        assert_eq!(shell.window(two).tabs.len(), 1);
        assert_eq!(shell.window(private).tabs.len(), 1);

        // That window closed, the next address goes to the one that is left.
        let answer = shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: first_window,
            }))
            .await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        shell.seconds_pass(10).await;
        assert_eq!(shell.window(two).tabs.len(), 2);
        assert_eq!(shell.window(two).tab.unwrap().title, "B");
        shell.seconds_pass(10).await;
        assert_eq!(shell.window(two).tab.unwrap().title, "C");
        assert_eq!(shell.window(private).tabs.len(), 1);
        assert_eq!(loads(&shell), [HOME, A, B, C]);
    });
}

#[test]
fn nothing_is_opened_outside_fake_mode_or_without_a_number_of_seconds() {
    run(async {
        // Fake mode, a list, and no seconds: nothing.
        let mut shell = FakeShell::start(&config(&[A, B], 0)).unwrap();
        web(&mut shell);
        let (_, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        shell.seconds_pass(45).await;
        assert_eq!(row(&shell, subscription).0, names(&["Home"]));
        assert_eq!(shell.wakeups(), 0);

        // Seconds, and no list: no timer.
        let mut shell = FakeShell::start(&config(&[], 5)).unwrap();
        web(&mut shell);
        let (_, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        shell.seconds_pass(45).await;
        assert_eq!(row(&shell, subscription).0, names(&["Home"]));
        assert_eq!(shell.wakeups(), 0);
    });
    run(async {
        // Outside fake mode, with both: a person's own app opens nothing by itself.
        let folder = tempfile::tempdir().unwrap();
        let mut live = FakeShell::live_config(folder.path());
        live.fake = config(&[A, B], 5).fake;
        let mut shell = FakeShell::start(&live).unwrap();
        web(&mut shell);
        let (_, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        shell.seconds_pass(45).await;
        assert_eq!(row(&shell, subscription).0, names(&["Home"]));
        assert_eq!(loads(&shell), [HOME]);
        assert_eq!(shell.wakeups(), 0);
    });
}

/// The windows are told of a tab the moment it is opened, and not when its page has come:
/// nothing a shell sent set this going, so nothing a shell sends is waited for.
#[test]
fn a_tab_is_in_its_window_as_soon_as_it_is_opened_whatever_the_shell_is_doing() {
    run(async {
        let mut shell = FakeShell::start(&config(&[A], 10)).unwrap();
        web(&mut shell);
        let (_, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        // A shell that is busy: it answers nothing the core asks of it.
        shell.hold_host = true;
        shell.seconds_pass(11).await;
        let window = shell.window(subscription);
        assert_eq!(window.tabs.len(), 2, "the tab is in the row");
        let front = window.tab.unwrap();
        assert_eq!(front.tab_id, window.tabs[1].tab_id, "and in front");
        assert_eq!(front.title, "", "with no page yet");

        // When the shell answers, its page comes.
        shell.release_host();
        shell.settle().await;
        assert_eq!(
            row(&shell, subscription),
            (names(&["Home", "A"]), "A".to_owned())
        );
    });
}
