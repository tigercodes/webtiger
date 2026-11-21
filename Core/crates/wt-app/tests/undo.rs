//! A closed tab comes back (033): undo, through the contract, against the
//! scripted engine. What is written before a tab is closed, what an undo
//! brings back and where, what the window is told, and what forgetting
//! takes with it. Over a store in memory, and over one on disk for what
//! must outlive a quit.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{
    CreateSpace, ForgetHistory, LoadState, SwitchSpace, TabView, Undo, address_view::Security,
    forget_history, host_request, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const FIRST: &str = "https://example.com/";
const SECOND: &str = "https://example.org/";
const THIRD: &str = "https://example.net/";

fn web(shell: &mut FakeShell) {
    shell.engine.page(FIRST, "Example");
    shell.engine.page(SECOND, "Another Place");
    shell.engine.page(THIRD, "A Third");
}

/// The strip: each tab's title.
fn strip(shell: &FakeShell, subscription: u64) -> Vec<String> {
    shell
        .window(subscription)
        .tabs
        .into_iter()
        .map(|tab| tab.title)
        .collect()
}

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the window is told an undo would undo.
fn undoable(shell: &FakeShell, subscription: u64) -> String {
    shell.window(subscription).undo
}

/// The addresses the shell was asked to load, in order.
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

fn asked(window: u64) -> request::Kind {
    request::Kind::Undo(Undo { window_id: window })
}

async fn undo(shell: &mut FakeShell, window: u64) -> response::Kind {
    shell.request(asked(window)).await
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// A window with three tabs, each at its page, the first in front.
/// The window, its subscription, and the tabs.
async fn three_tabs(shell: &mut FakeShell) -> (u64, u64, [u64; 3]) {
    web(shell);
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, FIRST).await;
    let second = shell.open_tab(window, SECOND).await;
    let third = shell.open_tab(window, THIRD).await;
    shell.activate_tab(first).await;
    assert_eq!(
        strip(shell, subscription),
        ["Example", "Another Place", "A Third"]
    );
    (window, subscription, [first, second, third])
}

#[test]
fn a_closed_tab_comes_back_at_its_place_in_front_loading_its_address() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [first, second, _]) = three_tabs(&mut shell).await;
        assert_eq!(undoable(&shell, subscription), "", "nothing was closed yet");

        shell.close_tab(second).await;
        assert_eq!(strip(&shell, subscription), ["Example", "A Third"]);
        assert_eq!(undoable(&shell, subscription), "tab.closed");
        let loaded = loads(&shell).len();

        // The shell has been asked for the page and has not answered yet:
        // how the tab that came back stands before its page has.
        shell.hold_host = true;
        let request = shell.send(asked(window));
        shell.settle().await;
        let back = front(&shell, subscription);
        assert_eq!(
            strip(&shell, subscription),
            ["Example", "Another Place", "A Third"],
            "where it was, with the title it had"
        );
        assert_ne!(
            back.tab_id, second,
            "a tab of its own: the closed one is gone"
        );
        assert_ne!(back.tab_id, first);
        assert_eq!(back.load_state(), LoadState::Loading);
        let shown = back.address.clone().unwrap();
        assert_eq!(
            shown.url, SECOND,
            "the field says where it goes from the start"
        );
        assert_eq!(
            shown.security(),
            Security::Unspecified,
            "and the mark waits"
        );
        shell.release_host();
        shell.settle().await;

        // Its page arrived: loaded once, as the person's own.
        let response::Kind::TabOpened(opened) = shell.take_response(request).unwrap() else {
            panic!("expected the tab that came back");
        };
        let arrived = front(&shell, subscription);
        assert_eq!(arrived.tab_id, opened.tab_id);
        assert_eq!(arrived.load_state(), LoadState::Idle);
        assert_eq!(arrived.title, "Another Place");
        assert_eq!(arrived.address.unwrap().security(), Security::Secure);
        assert_eq!(&loads(&shell)[loaded..], [SECOND]);
        // And there is nothing more to undo: what was undone is not undone twice.
        assert_eq!(undoable(&shell, subscription), "");
        assert_eq!(error_code(undo(&mut shell, window).await), "undo.nothing");
        assert_eq!(strip(&shell, subscription).len(), 3);
    });
}

#[test]
fn closed_twice_they_come_back_newest_first_each_where_it_was() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [first, second, third]) = three_tabs(&mut shell).await;

        // The last in the row, then the first.
        shell.close_tab(third).await;
        shell.close_tab(first).await;
        assert_eq!(strip(&shell, subscription), ["Another Place"]);

        // The first was closed last: it is back first, at the front of the row.
        undo(&mut shell, window).await;
        assert_eq!(strip(&shell, subscription), ["Example", "Another Place"]);
        assert_eq!(front(&shell, subscription).title, "Example");
        assert_eq!(undoable(&shell, subscription), "tab.closed", "one more");

        // Then the one closed before it, at the end, where it was.
        undo(&mut shell, window).await;
        assert_eq!(
            strip(&shell, subscription),
            ["Example", "Another Place", "A Third"]
        );
        assert_eq!(front(&shell, subscription).title, "A Third");
        assert_eq!(undoable(&shell, subscription), "");

        // A row that is shorter now than the place the tab had: at the end.
        shell.close_tab(second).await;
        let now = shell.window(subscription).tabs;
        shell.close_tab(now[1].tab_id).await;
        shell.close_tab(now[0].tab_id).await;
        // What is left is an empty tab, which makes way for the last closed
        // (the row's first); then the one before it, which was second of
        // two and is second again; then the one that was the middle of
        // three, at the place it had.
        for _ in 0..3 {
            undo(&mut shell, window).await;
        }
        assert_eq!(
            strip(&shell, subscription),
            ["Example", "Another Place", "A Third"]
        );
    });
}

#[test]
fn the_only_tab_closed_and_brought_back_leaves_no_empty_tab() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, only, subscription) = shell.open_window().await;
        shell.submit(only, FIRST).await;

        // Closing the row's only tab leaves an empty one in its place (005).
        shell.close_tab(only).await;
        let left = shell.window(subscription);
        assert_eq!(left.tabs.len(), 1);
        assert_eq!(left.tabs[0].title, "");
        assert_eq!(left.tab.unwrap().web_view_id, 0, "an empty tab");

        undo(&mut shell, window).await;

        // The tab is back, and the empty one that stood in for it is gone.
        assert_eq!(strip(&shell, subscription), ["Example"]);
        assert_eq!(front(&shell, subscription).address.unwrap().url, FIRST);

        // An empty tab the person opened is theirs: it stays, though it is
        // the row's only tab when the closed one comes back.
        let empty = shell.open_tab(window, "").await;
        let page = shell.window(subscription).tabs[0].tab_id;
        shell.close_tab(page).await;
        assert_eq!(strip(&shell, subscription), [""]);
        undo(&mut shell, window).await;
        assert_eq!(strip(&shell, subscription), ["Example", ""]);
        assert!(
            shell
                .window(subscription)
                .tabs
                .iter()
                .any(|tab| tab.tab_id == empty)
        );
        assert_eq!(front(&shell, subscription).title, "Example");

        // The stand-in the person has since used is no stand-in: closing the
        // only tab again leaves one, they go somewhere in it, and what was
        // closed comes back beside it.
        shell.close_tab(empty).await;
        let page = shell.window(subscription).tabs[0].tab_id;
        shell.close_tab(page).await;
        let stand_in = shell.window(subscription).tabs[0].tab_id;
        shell.submit(stand_in, SECOND).await;
        undo(&mut shell, window).await;
        assert_eq!(strip(&shell, subscription), ["Example", "Another Place"]);
    });
}

#[test]
fn an_empty_tab_closed_is_nothing_to_undo_and_nothing_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, FIRST).await;
        let loaded = loads(&shell).len();

        // An empty tab has nothing to bring back.
        let empty = shell.open_tab(window, "").await;
        shell.close_tab(empty).await;
        assert_eq!(undoable(&shell, subscription), "");
        assert_eq!(error_code(undo(&mut shell, window).await), "undo.nothing");

        // A window there is not is refused, and that is said first.
        assert_eq!(
            error_code(undo(&mut shell, window + 999).await),
            "window.unknown"
        );
        assert_eq!(strip(&shell, subscription), ["Example"]);
        assert_eq!(loads(&shell).len(), loaded, "nothing was loaded");
    });
}

#[test]
fn each_space_has_its_own_closed_tabs() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [_, second, _]) = three_tabs(&mut shell).await;
        let home = shell.window(subscription).space_id;
        shell.close_tab(second).await;
        assert_eq!(undoable(&shell, subscription), "tab.closed");

        // Another Space, which the window now shows: nothing was closed in it.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                share_logins_with: 0,
                colour: String::new(),
            }))
            .await;
        let response::Kind::SpaceCreated(work) = made else {
            panic!("expected a Space, got {made:?}");
        };
        let switch = |space| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: space,
            })
        };
        shell.request(switch(work.space_id)).await;
        assert_eq!(
            undoable(&shell, subscription),
            "",
            "Work has none of its own"
        );
        assert_eq!(error_code(undo(&mut shell, window).await), "undo.nothing");

        // One is closed in Work, and brought back there.
        let here = shell.open_tab(window, THIRD).await;
        shell.close_tab(here).await;
        assert_eq!(undoable(&shell, subscription), "tab.closed");
        undo(&mut shell, window).await;
        assert_eq!(front(&shell, subscription).address.unwrap().url, THIRD);
        assert_eq!(undoable(&shell, subscription), "");

        // Home's closed tab is still Home's, and comes back in Home.
        shell.request(switch(home)).await;
        assert_eq!(undoable(&shell, subscription), "tab.closed");
        undo(&mut shell, window).await;
        assert_eq!(
            strip(&shell, subscription),
            ["Example", "Another Place", "A Third"]
        );
    });
}

#[test]
fn forgetting_history_forgets_the_closed_tabs_whatever_was_named() {
    run(async {
        for what in [
            forget_history::What::Everything(true),
            forget_history::What::SinceMs(0),
            // A visit there is not: nothing of history is forgotten, and the
            // closed tabs are, all the same.
            forget_history::What::VisitId(9_999),
        ] {
            let mut shell = FakeShell::fake();
            let (window, subscription, [_, second, third]) = three_tabs(&mut shell).await;
            shell.close_tab(second).await;
            shell.close_tab(third).await;
            assert_eq!(undoable(&shell, subscription), "tab.closed");

            let forgotten = shell
                .request(request::Kind::ForgetHistory(ForgetHistory {
                    window_id: window,
                    what: Some(what),
                }))
                .await;
            assert!(matches!(forgotten, response::Kind::Ack(_)), "{what:?}");

            // What was asked to be forgotten does not come back by undo.
            assert_eq!(undoable(&shell, subscription), "", "{what:?}");
            assert_eq!(error_code(undo(&mut shell, window).await), "undo.nothing");
            assert_eq!(strip(&shell, subscription), ["Example"]);
        }
    });
}

/// Starts the app in `folder`, with the web it browses.
fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

#[test]
fn a_tab_closed_before_a_quit_comes_back_after_it() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, FIRST).await;
        let second = shell.open_tab(window, SECOND).await;
        shell.close_tab(second).await;
        assert_eq!(undoable(&shell, subscription), "tab.closed");
        // Dropped without a word, as a quit or a crash leaves it.
        drop(shell);

        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        // The session's tab is back (009), and the window is told there is
        // a closed tab to bring back, from its first snapshot.
        assert_eq!(strip(&shell, subscription), ["Example"]);
        assert_eq!(undoable(&shell, subscription), "tab.closed");

        undo(&mut shell, window).await;

        assert_eq!(strip(&shell, subscription), ["Example", "Another Place"]);
        assert_eq!(front(&shell, subscription).address.unwrap().url, SECOND);
        assert_eq!(undoable(&shell, subscription), "");
        drop(shell);

        // And what was brought back is not brought back again after the next.
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(undoable(&shell, subscription), "");
        assert_eq!(error_code(undo(&mut shell, window).await), "undo.nothing");
    });
}

#[test]
fn a_tab_a_page_closed_itself_is_not_brought_back() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, FIRST).await;
        // A page opens a window, which is a tab (006), and then closes it.
        let opener = front(&shell, subscription).web_view_id;
        let (opened, asks) = shell.engine.ask_for_window(opener, SECOND, true);
        shell.report(asks);
        shell.settle().await;
        assert_eq!(shell.window(subscription).tabs.len(), 2);
        shell.report(FakeEngine::close_requested(opened));
        shell.settle().await;
        assert_eq!(shell.window(subscription).tabs.len(), 1);

        // The person did not close it: there is nothing of theirs to undo.
        assert_eq!(undoable(&shell, subscription), "");
        assert_eq!(error_code(undo(&mut shell, window).await), "undo.nothing");
    });
}
