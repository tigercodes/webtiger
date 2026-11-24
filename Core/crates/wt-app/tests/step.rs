//! From tab to tab by where they are in the row (035): the next, the one
//! before, the last, the tab at a place. Through the contract, against the
//! scripted engine.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{
    CreateSpace, StepTab, SwitchSpace, host_request, request, response, step_tab::To,
};
use wt_testkit::{FakeShell, run};

const FIRST: &str = "https://example.com/";
const SECOND: &str = "https://example.org/";
const THIRD: &str = "https://example.net/";

fn web(shell: &mut FakeShell) {
    shell.engine.page(FIRST, "One");
    shell.engine.page(SECOND, "Two");
    shell.engine.page(THIRD, "Three");
}

/// The title of the tab in front.
fn front(shell: &FakeShell, subscription: u64) -> String {
    shell
        .window(subscription)
        .tab
        .expect("a window has a tab")
        .title
}

async fn step(shell: &mut FakeShell, window: u64, to: To, place: u32) -> response::Kind {
    shell
        .request(request::Kind::StepTab(StepTab {
            window_id: window,
            to: to.into(),
            place,
        }))
        .await
}

/// Steps, and says which tab is then in front. The step is answered with Ack.
async fn stepped(
    shell: &mut FakeShell,
    window: u64,
    subscription: u64,
    to: To,
    place: u32,
) -> String {
    let answer = step(shell, window, to, place).await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
    front(shell, subscription)
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// A window with three tabs, One, Two and Three, and Three in front.
async fn three_tabs(shell: &mut FakeShell) -> (u64, u64) {
    web(shell);
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, FIRST).await;
    shell.open_tab(window, SECOND).await;
    shell.open_tab(window, THIRD).await;
    assert_eq!(front(shell, subscription), "Three");
    (window, subscription)
}

#[test]
fn the_next_tab_and_the_one_before_go_round_at_the_ends() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription) = three_tabs(&mut shell).await;

        // From the last, the next is the first; and on along the row.
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Next, 0).await,
            "One"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Next, 0).await,
            "Two"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Next, 0).await,
            "Three"
        );
        // The one before; and from the first, the last.
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Previous, 0).await,
            "Two"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Previous, 0).await,
            "One"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Previous, 0).await,
            "Three"
        );
        // The row is as it was: only which tab is in front changed.
        let titles: Vec<String> = shell
            .window(subscription)
            .tabs
            .into_iter()
            .map(|tab| tab.title)
            .collect();
        assert_eq!(titles, ["One", "Two", "Three"]);
    });
}

#[test]
fn the_tab_at_a_place_and_the_last() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription) = three_tabs(&mut shell).await;

        // Places are counted from one.
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Place, 1).await,
            "One"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Place, 3).await,
            "Three"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Place, 2).await,
            "Two"
        );
        // The tab already in front: nothing changes, and nothing is refused.
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Place, 2).await,
            "Two"
        );
        // A place the row does not reach, and no place at all: nothing happens.
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Place, 4).await,
            "Two"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Place, 0).await,
            "Two"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Place, u32::MAX).await,
            "Two"
        );
        // The last, wherever it is asked from; and again from the last.
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Last, 0).await,
            "Three"
        );
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Last, 0).await,
            "Three"
        );
    });
}

#[test]
fn with_one_tab_nothing_changes_and_nothing_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, FIRST).await;
        let asked = shell.host_requests.len();
        for (to, place) in [
            (To::Next, 0),
            (To::Previous, 0),
            (To::Last, 0),
            (To::Place, 1),
            (To::Place, 2),
        ] {
            assert_eq!(
                stepped(&mut shell, window, subscription, to, place).await,
                "One"
            );
        }
        assert_eq!(
            shell.host_requests.len(),
            asked,
            "the shell is asked for nothing"
        );
    });
}

#[test]
fn a_window_there_is_not_and_a_step_with_no_way_to_go_are_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription) = three_tabs(&mut shell).await;

        assert_eq!(
            error_code(step(&mut shell, window, To::Unspecified, 0).await),
            "tab.step"
        );
        assert_eq!(
            error_code(step(&mut shell, window + 100, To::Next, 0).await),
            "window.unknown"
        );
        // A window there is not is refused first, whatever else is wrong.
        assert_eq!(
            error_code(step(&mut shell, window + 100, To::Unspecified, 0).await),
            "window.unknown"
        );
        // A way to go that a later contract may have, and this core does not know.
        let unknown = shell
            .request(request::Kind::StepTab(StepTab {
                window_id: window,
                to: 99,
                place: 0,
            }))
            .await;
        assert_eq!(error_code(unknown), "tab.step");
        assert_eq!(front(&shell, subscription), "Three", "nothing changed");
    });
}

#[test]
fn the_row_is_the_one_of_the_space_the_window_shows() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription) = three_tabs(&mut shell).await;
        let home = shell.window(subscription).space_id;
        // Another Space, with a row of its own: one tab, empty.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(created) = made else {
            panic!("expected a Space, got {made:?}");
        };
        let work = created.space_id;
        shell
            .request(request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: work,
            }))
            .await;
        assert_eq!(shell.window(subscription).tabs.len(), 1);

        // In the Space the window shows there is one tab: nothing to step to, though the
        // window has three more in a row that is not shown.
        let only = shell.window(subscription).tab.expect("a tab").tab_id;
        for (to, place) in [
            (To::Next, 0),
            (To::Previous, 0),
            (To::Last, 0),
            (To::Place, 2),
        ] {
            let answer = step(&mut shell, window, to, place).await;
            assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
            assert_eq!(shell.window(subscription).tab.expect("a tab").tab_id, only);
        }

        // And the row that is not shown is as it was: back in its Space, the tab that was
        // in front there still is.
        shell
            .request(request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: home,
            }))
            .await;
        assert_eq!(front(&shell, subscription), "Three");
    });
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
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

#[test]
fn a_tab_that_came_back_asleep_is_loaded_when_it_is_stepped_to() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, FIRST).await;
        shell.open_tab(window, SECOND).await;
        shell.open_tab(window, THIRD).await;
        drop(shell);

        // After a quit the tabs are back (009): the one that was in front is loaded, the
        // others sleep until they come in front.
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(loads(&shell), [THIRD]);

        assert_eq!(
            stepped(&mut shell, window, subscription, To::Previous, 0).await,
            "Two"
        );
        assert_eq!(loads(&shell), [THIRD, SECOND], "stepped to, it is loaded");
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Place, 1).await,
            "One"
        );
        assert_eq!(loads(&shell), [THIRD, SECOND, FIRST]);
        // And one that is awake is not loaded again.
        assert_eq!(
            stepped(&mut shell, window, subscription, To::Last, 0).await,
            "Three"
        );
        assert_eq!(loads(&shell), [THIRD, SECOND, FIRST]);
    });
}
