//! Zoom (012), through the contract, against the scripted engine: what is
//! asked of which web views, what a tab's snapshot says, and what the store
//! keeps for a site.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_core::ZOOM_LEVELS;
use wt_proto::v1::{TabView, ZoomChange, ZoomPage, ZoomView, request, response};
use wt_testkit::{FakeEngine, FakeShell, run};

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// The zoom of the tab in front: its level, and whether there is one above
/// and one below.
fn zoom_of(shell: &FakeShell, subscription: u64) -> Option<(u32, bool, bool)> {
    front(shell, subscription).zoom.map(
        |ZoomView {
             percent,
             can_zoom_in,
             can_zoom_out,
         }| (percent, can_zoom_in, can_zoom_out),
    )
}

async fn zoom(shell: &mut FakeShell, tab: u64, change: ZoomChange) -> response::Kind {
    shell
        .request(request::Kind::ZoomPage(ZoomPage {
            tab_id: tab,
            change: change as i32,
        }))
        .await
}

/// Zooms, and expects to be told it was done.
async fn zoomed(shell: &mut FakeShell, tab: u64, change: ZoomChange) {
    let answer = zoom(shell, tab, change).await;
    assert!(
        matches!(answer, response::Kind::Ack(_)),
        "expected Ack, got {answer:?}"
    );
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// The web a shell browses in these tests.
fn pages(shell: &mut FakeShell) {
    for (url, title) in [
        ("https://example.com/", "Example"),
        ("https://www.example.com/about", "About Example"),
        ("http://shop.example.com:8080/", "Example's shop"),
        ("https://example.org/", "Another"),
        ("http://127.0.0.1:8080/", "A machine"),
    ] {
        shell.engine.page(url, title);
    }
}

/// A window whose one tab shows example.com. Its window, tab, subscription
/// and the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    pages(shell);
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

#[test]
fn bigger_smaller_and_actual_size_go_to_the_web_view_and_the_snapshot() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        assert_eq!(
            zoom_of(&shell, subscription),
            Some((100, true, true)),
            "a page nobody zoomed is at actual size"
        );
        assert_eq!(shell.engine.zooms, [], "and its web view was told nothing");

        zoomed(&mut shell, tab, ZoomChange::In).await;
        assert_eq!(zoom_of(&shell, subscription), Some((110, true, true)));
        zoomed(&mut shell, tab, ZoomChange::In).await;
        zoomed(&mut shell, tab, ZoomChange::Out).await;
        zoomed(&mut shell, tab, ZoomChange::Out).await;
        zoomed(&mut shell, tab, ZoomChange::Out).await;
        assert_eq!(zoom_of(&shell, subscription), Some((90, true, true)));
        zoomed(&mut shell, tab, ZoomChange::Reset).await;

        assert_eq!(
            shell.engine.zooms,
            [110, 125, 110, 100, 90, 100].map(|percent| (web_view, percent)),
            "one level at a time, each asked of the tab's web view"
        );
        assert_eq!(zoom_of(&shell, subscription), Some((100, true, true)));
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn the_levels_end_and_the_snapshot_says_so() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        // Up from actual size to the largest, and once more.
        let above: Vec<u32> = ZOOM_LEVELS
            .into_iter()
            .filter(|level| *level > 100)
            .collect();
        for _ in &above {
            zoomed(&mut shell, tab, ZoomChange::In).await;
        }
        assert_eq!(
            shell.engine.zooms,
            above
                .iter()
                .map(|percent| (web_view, *percent))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            zoom_of(&shell, subscription),
            Some((300, false, true)),
            "nothing above the largest"
        );
        let asked = shell.engine.zooms.len();
        zoomed(&mut shell, tab, ZoomChange::In).await;
        assert_eq!(
            shell.engine.zooms.len(),
            asked,
            "no further level: done, and nothing asked"
        );
        assert_eq!(zoom_of(&shell, subscription), Some((300, false, true)));

        // Down to the smallest, and once more.
        for _ in 1..ZOOM_LEVELS.len() {
            zoomed(&mut shell, tab, ZoomChange::Out).await;
        }
        assert_eq!(zoom_of(&shell, subscription), Some((50, true, false)));
        let asked = shell.engine.zooms.len();
        zoomed(&mut shell, tab, ZoomChange::Out).await;
        assert_eq!(shell.engine.zooms.len(), asked);
        assert_eq!(shell.engine.zooms.last(), Some(&(web_view, 50)));
    });
}

#[test]
fn actual_size_at_actual_size_asks_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, _, _) = a_page(&mut shell).await;
        zoomed(&mut shell, tab, ZoomChange::Reset).await;
        assert_eq!(shell.engine.zooms, []);
    });
}

#[test]
fn there_is_nothing_to_zoom_without_a_page_with_a_site() {
    run(async {
        let mut shell = FakeShell::fake();
        pages(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        assert_eq!(
            zoom_of(&shell, subscription),
            None,
            "an empty tab has no zoom"
        );
        assert_eq!(
            error_code(zoom(&mut shell, tab, ZoomChange::In).await),
            "tab.no_page"
        );
        assert_eq!(
            error_code(zoom(&mut shell, tab + 1_000, ZoomChange::In).await),
            "tab.unknown"
        );

        shell.submit(tab, "example.com").await;
        assert_eq!(
            error_code(zoom(&mut shell, tab, ZoomChange::Unspecified).await),
            "tab.bad_command",
            "a change that is none of the three"
        );

        // A page with no host has no site: nothing to remember a level for.
        let web_view = front(&shell, subscription).web_view_id;
        shell.report(FakeEngine::navigation_started(web_view, "about:blank"));
        shell.report(FakeEngine::navigation_committed(web_view, "about:blank"));
        shell.report(FakeEngine::navigation_finished(web_view));
        shell.settle().await;
        assert_eq!(zoom_of(&shell, subscription), None);
        assert_eq!(
            error_code(zoom(&mut shell, tab, ZoomChange::In).await),
            "tab.no_page"
        );
        assert_eq!(shell.engine.zooms, []);
    });
}

#[test]
fn every_tab_of_the_site_is_told_and_no_other() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_view) = a_page(&mut shell).await;
        let elsewhere = shell.open_tab(window, "example.org").await;
        let elsewhere_view = front(&shell, subscription).web_view_id;
        let second = shell.open_tab(window, "www.example.com/about").await;
        let second_view = front(&shell, subscription).web_view_id;
        assert_eq!(shell.engine.zooms, []);

        // From the tab in front: the other tab of the site is told too, in
        // the order the tabs were opened; the tab of another site is not.
        zoomed(&mut shell, second, ZoomChange::In).await;

        assert_eq!(shell.engine.zooms, [(first_view, 110), (second_view, 110)]);
        assert_eq!(zoom_of(&shell, subscription), Some((110, true, true)));
        // Each tab's snapshot says its own page's level, and coming in
        // front asks nothing more.
        shell.activate_tab(elsewhere).await;
        assert_eq!(front(&shell, subscription).web_view_id, elsewhere_view);
        assert_eq!(zoom_of(&shell, subscription), Some((100, true, true)));
        shell.activate_tab(first).await;
        assert_eq!(zoom_of(&shell, subscription), Some((110, true, true)));
        assert_eq!(shell.engine.zooms.len(), 2);

        // And from a tab behind's site: one level on from where the site is.
        zoomed(&mut shell, first, ZoomChange::In).await;
        assert_eq!(
            shell.engine.zooms[2..],
            [(first_view, 125), (second_view, 125)]
        );
    });
}

#[test]
fn a_page_that_arrives_is_given_its_sites_level() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        zoomed(&mut shell, tab, ZoomChange::In).await;
        zoomed(&mut shell, tab, ZoomChange::In).await;
        assert_eq!(shell.engine.zooms, [(web_view, 110), (web_view, 125)]);

        // On to a site nobody zoomed: its web view is put back.
        shell.submit(tab, "example.org").await;
        assert_eq!(shell.engine.zooms.last(), Some(&(web_view, 100)));
        assert_eq!(zoom_of(&shell, subscription), Some((100, true, true)));

        // And back: the site's level again, unasked.
        shell.submit(tab, "example.com").await;
        assert_eq!(shell.engine.zooms.last(), Some(&(web_view, 125)));
        assert_eq!(zoom_of(&shell, subscription), Some((125, true, true)));
        assert_eq!(shell.engine.zooms.len(), 4);

        // A new tab's web view is at actual size until its page arrives.
        shell.open_tab(window, "www.example.com/about").await;
        let new_view = front(&shell, subscription).web_view_id;
        assert_eq!(shell.engine.zooms.last(), Some(&(new_view, 125)));
        assert_eq!(zoom_of(&shell, subscription), Some((125, true, true)));

        // One whose site is at actual size is told nothing.
        shell.open_tab(window, "example.org").await;
        assert_eq!(shell.engine.zooms.len(), 5);
        assert_eq!(zoom_of(&shell, subscription), Some((100, true, true)));
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn a_site_is_what_was_registered_whatever_the_scheme_and_the_port() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        zoomed(&mut shell, tab, ZoomChange::In).await;

        // Another host of the same site, by another scheme and port: the
        // web view has the site's level already, and is told nothing.
        shell.submit(tab, "http://shop.example.com:8080/").await;
        assert_eq!(shell.engine.zooms, [(web_view, 110)]);
        assert_eq!(zoom_of(&shell, subscription), Some((110, true, true)));

        // A host with nothing registered is its own site.
        shell.submit(tab, "http://127.0.0.1:8080/").await;
        assert_eq!(shell.engine.zooms.last(), Some(&(web_view, 100)));
        zoomed(&mut shell, tab, ZoomChange::Out).await;
        assert_eq!(zoom_of(&shell, subscription), Some((90, true, true)));
        shell.submit(tab, "www.example.com/about").await;
        assert_eq!(
            zoom_of(&shell, subscription),
            Some((110, true, true)),
            "and leaves the other site's level alone"
        );
        shell.submit(tab, "http://127.0.0.1:8080/").await;
        assert_eq!(zoom_of(&shell, subscription), Some((90, true, true)));
    });
}

#[test]
fn a_page_changing_its_own_address_keeps_its_zoom() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        zoomed(&mut shell, tab, ZoomChange::In).await;

        shell.report(FakeEngine::address_changed(
            web_view,
            "https://example.com/#part",
        ));
        shell.settle().await;

        assert_eq!(
            shell.engine.zooms,
            [(web_view, 110)],
            "nothing more is asked"
        );
        assert_eq!(zoom_of(&shell, subscription), Some((110, true, true)));
    });
}

/// Starts the app in `folder`, with the web it browses.
fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    pages(&mut shell);
    shell
}

#[test]
fn the_level_is_still_the_sites_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (_, tab, _) = shell.open_window().await;
        shell.submit(tab, "example.com").await;
        zoomed(&mut shell, tab, ZoomChange::In).await;
        zoomed(&mut shell, tab, ZoomChange::In).await;
        // Ended without a word: nothing is said to the core.
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;

        // The tab that came back loads its page, and the page is given the
        // level its site was left at.
        let web_view = front(&shell, subscription).web_view_id;
        assert_eq!(shell.engine.zooms, [(web_view, 125)]);
        assert_eq!(zoom_of(&shell, subscription), Some((125, true, true)));
    });
}

#[test]
fn a_site_put_back_to_actual_size_is_not_remembered() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (_, tab, _) = shell.open_window().await;
        shell.submit(tab, "example.com").await;
        zoomed(&mut shell, tab, ZoomChange::Out).await;
        zoomed(&mut shell, tab, ZoomChange::Reset).await;
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;

        assert_eq!(shell.engine.zooms, [], "the page is told nothing");
        assert_eq!(zoom_of(&shell, subscription), Some((100, true, true)));
    });
}
