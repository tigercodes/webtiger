//! Find on the page (010), through the contract, against the scripted
//! engine: what is asked of a tab's web view, and what the tab's snapshot
//! says was found.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{FindInPage, FindView, TabView, request, response};
use wt_testkit::{FakeEngine, FakeShell, run};

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// The find of the tab in front.
fn found(shell: &FakeShell, subscription: u64) -> Option<FindView> {
    front(shell, subscription).find
}

async fn find(shell: &mut FakeShell, tab: u64, text: &str, backwards: bool) -> response::Kind {
    shell
        .request(request::Kind::FindInPage(FindInPage {
            tab_id: tab,
            text: text.to_owned(),
            backwards,
        }))
        .await
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// A window whose one tab shows a page. Its window, tab, subscription and
/// the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    shell.engine.page("https://example.com/", "Example");
    shell.engine.page("https://example.org/", "Another");
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

#[test]
fn text_is_looked_for_in_the_tabs_web_view() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        assert_eq!(
            found(&shell, subscription),
            None,
            "a tab has no find at first"
        );

        let answer = find(&mut shell, tab, "needle", false).await;

        assert!(matches!(answer, response::Kind::Ack(_)));
        assert_eq!(shell.engine.finds, [(web_view, "needle".to_owned(), false)]);
        // The tab's find says what is looked for, and that nothing is known
        // of it yet: it is not "not found".
        let looking = found(&shell, subscription).expect("the tab has a find");
        assert_eq!(looking.text, "needle");
        assert!(!looking.answered);
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn the_same_text_again_goes_on_and_backwards_goes_back() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, _, web_view) = a_page(&mut shell).await;

        find(&mut shell, tab, "needle", false).await;
        find(&mut shell, tab, "needle", false).await;
        find(&mut shell, tab, "needle", true).await;

        assert_eq!(
            shell.engine.finds,
            [
                (web_view, "needle".to_owned(), false),
                (web_view, "needle".to_owned(), false),
                (web_view, "needle".to_owned(), true),
            ],
            "each is asked of the web view, as it was asked"
        );
    });
}

#[test]
fn what_the_engine_found_is_the_tabs_find() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        find(&mut shell, tab, "needle", false).await;

        // An engine that counts: twelve, and the third is shown.
        shell.report(FakeEngine::find_result(web_view, true, Some((12, 3))));
        shell.settle().await;
        let counted = found(&shell, subscription).unwrap();
        assert!(counted.answered && counted.found && counted.counted);
        assert_eq!((counted.matches, counted.current), (12, 3));
        assert_eq!(counted.text, "needle");

        // The next match: the same text, and what was known still holds
        // until the engine says more.
        find(&mut shell, tab, "needle", false).await;
        assert_eq!(found(&shell, subscription).unwrap().current, 3);
        shell.report(FakeEngine::find_result(web_view, true, Some((12, 4))));
        shell.settle().await;
        assert_eq!(found(&shell, subscription).unwrap().current, 4);

        // An engine that does not count says only that the text is there.
        shell.report(FakeEngine::find_result(web_view, true, None));
        shell.settle().await;
        let uncounted = found(&shell, subscription).unwrap();
        assert!(uncounted.found && !uncounted.counted);
        assert_eq!((uncounted.matches, uncounted.current), (0, 0));
    });
}

#[test]
fn text_that_is_not_there_is_not_found() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        find(&mut shell, tab, "zebra", false).await;

        shell.report(FakeEngine::find_result(web_view, false, Some((0, 0))));
        shell.settle().await;

        let nothing = found(&shell, subscription).unwrap();
        assert!(nothing.answered, "the engine has spoken");
        assert!(!nothing.found);
        assert_eq!(nothing.matches, 0);
    });
}

#[test]
fn other_text_starts_from_nothing_known() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        find(&mut shell, tab, "zebra", false).await;
        shell.report(FakeEngine::find_result(web_view, false, None));
        shell.settle().await;

        // Another letter typed: what was said of "zebra" is not said of it.
        find(&mut shell, tab, "zebras", false).await;

        let looking = found(&shell, subscription).unwrap();
        assert_eq!(looking.text, "zebras");
        assert!(
            !looking.answered,
            "not \"not found\" until the engine says so"
        );
    });
}

#[test]
fn empty_text_stops_finding() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        find(&mut shell, tab, "needle", false).await;
        shell.report(FakeEngine::find_result(web_view, true, Some((2, 1))));
        shell.settle().await;

        let answer = find(&mut shell, tab, "", false).await;

        assert!(matches!(answer, response::Kind::Ack(_)));
        assert_eq!(found(&shell, subscription), None, "the tab has no find");
        assert_eq!(
            shell.engine.finds.last(),
            Some(&(web_view, String::new(), false)),
            "and the web view is asked to clear what it found"
        );
        // The engine's last word arrives late: it is about nothing now.
        shell.report(FakeEngine::find_result(web_view, true, Some((2, 2))));
        shell.settle().await;
        assert_eq!(found(&shell, subscription), None);
    });
}

#[test]
fn a_new_page_ends_the_find() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        find(&mut shell, tab, "needle", false).await;
        shell.report(FakeEngine::find_result(web_view, true, Some((2, 1))));
        shell.settle().await;

        shell.submit(tab, "example.org").await;

        assert_eq!(
            found(&shell, subscription),
            None,
            "what was found went with the page it was found in"
        );
        // The page changing its own address is the same page: a find there
        // goes on.
        find(&mut shell, tab, "needle", false).await;
        shell.report(FakeEngine::address_changed(
            web_view,
            "https://example.org/#part",
        ));
        shell.settle().await;
        assert!(found(&shell, subscription).is_some());
    });
}

#[test]
fn a_tab_with_no_page_and_no_such_tab() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, _) = a_page(&mut shell).await;
        let empty = shell.open_tab(window, "").await;
        let asked = shell.engine.finds.len();

        let no_page = find(&mut shell, empty, "needle", false).await;
        let nobody = find(&mut shell, 9_999, "needle", false).await;

        assert_eq!(error_code(no_page), "tab.no_page");
        assert_eq!(error_code(nobody), "tab.unknown");
        assert_eq!(
            shell.engine.finds.len(),
            asked,
            "nothing was asked of the shell"
        );
    });
}

#[test]
fn each_tab_has_its_own_find() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_web_view) = a_page(&mut shell).await;
        find(&mut shell, first, "needle", false).await;
        shell.report(FakeEngine::find_result(first_web_view, true, Some((5, 1))));
        shell.settle().await;

        // Another tab in front, with its own page and no find.
        let second = shell.open_tab(window, "example.org").await;
        assert_eq!(found(&shell, subscription), None);
        // A find in the tab behind is still its own: a result for it does
        // not show in the tab in front.
        shell.report(FakeEngine::find_result(first_web_view, true, Some((5, 2))));
        shell.settle().await;
        assert_eq!(front(&shell, subscription).tab_id, second);
        assert_eq!(found(&shell, subscription), None);

        shell.activate_tab(first).await;
        assert_eq!(found(&shell, subscription).unwrap().current, 2);
    });
}

#[test]
fn a_count_the_engine_cannot_mean_is_not_passed_on() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        find(&mut shell, tab, "needle", false).await;

        // The seventh of three.
        shell.report(FakeEngine::find_result(web_view, true, Some((3, 7))));
        shell.settle().await;

        let shown = found(&shell, subscription).unwrap();
        assert_eq!((shown.matches, shown.current), (3, 3));
    });
}
