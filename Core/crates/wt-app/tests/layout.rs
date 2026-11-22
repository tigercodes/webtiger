//! A window's layout (043): the tabs along the top, or down the side. Through
//! the contract, against the scripted engine: that switching it changes
//! nothing else, that each window has its own, that a new window has the one
//! last chosen, and that the choice outlives a quit.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{Layout, SetLayout, WindowSnapshot, request, response};
use wt_testkit::{FakeShell, run};

const MAIL: &str = "https://mail.example.com/";
const BOARD: &str = "https://board.example.com/";
const NEWS: &str = "https://news.example.com/";
const STORY: &str = "https://news.example.com/story";

fn web(shell: &mut FakeShell) {
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(BOARD, "Board");
    shell.engine.page(NEWS, "News");
    shell.engine.page(STORY, "A story");
}

fn layout(shell: &FakeShell, subscription: u64) -> Layout {
    Layout::try_from(shell.window(subscription).layout).expect("a layout the contract has")
}

/// A window as it is told to its shell, with the layout taken out: what a
/// switch must leave as it was.
fn but_for_its_layout(shell: &FakeShell, subscription: u64) -> WindowSnapshot {
    let mut window = shell.window(subscription);
    window.layout = 0;
    window
}

fn code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

fn acked(answer: &response::Kind) {
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

/// A window with a row of every kind of thing: a Favourite, a Site in no
/// Group, an open Group with a Site asleep, a shut Group with a Site, and a
/// tab in front.
async fn a_full_row(shell: &mut FakeShell) -> (u64, u64) {
    web(shell);
    let (window, mail, subscription) = shell.open_window().await;
    shell.submit(mail, MAIL).await;
    let board = shell.open_tab(window, BOARD).await;
    let news = shell.open_tab(window, NEWS).await;
    let story = shell.open_tab(window, STORY).await;
    let other = shell.open_tab(window, MAIL).await;
    let made = |answer: response::Kind| match answer {
        response::Kind::GroupMade(made) => made.group_id,
        other => panic!("expected a Group, got {other:?}"),
    };
    let work = made(shell.make_group(window, "Work").await);
    let house = made(shell.make_group(window, "House").await);
    acked(&shell.set_favourite(mail, true).await);
    acked(&shell.keep_tab(other).await);
    acked(&shell.move_to_group(board, work).await);
    acked(&shell.move_to_group(news, house).await);
    acked(&shell.activate_tab(board).await);
    acked(&shell.close_tab(board).await);
    acked(&shell.shut_group(house, true).await);
    acked(&shell.activate_tab(story).await);
    (window, subscription)
}

#[test]
fn switching_the_layout_changes_nothing_else() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription) = a_full_row(&mut shell).await;
        // A window starts with its tabs along the top.
        assert_eq!(layout(&shell, subscription), Layout::Strip);
        let before = but_for_its_layout(&shell, subscription);
        assert_eq!(before.tabs.len(), 4, "a shut Group's Site is left out");
        assert_eq!(before.groups.len(), 2);
        let asked = shell.host_requests.len();

        // To the side: the window says so, and everything else it says is what it said.
        // The shell was asked for nothing: no tab was loaded, moved or closed.
        acked(&shell.set_layout(window, Layout::Sidebar).await);
        assert_eq!(layout(&shell, subscription), Layout::Sidebar);
        assert_eq!(but_for_its_layout(&shell, subscription), before);
        assert_eq!(shell.host_requests.len(), asked);
        // Asked for again, it is as it is; and back along the top, the same again.
        acked(&shell.set_layout(window, Layout::Sidebar).await);
        assert_eq!(layout(&shell, subscription), Layout::Sidebar);
        acked(&shell.set_layout(window, Layout::Strip).await);
        assert_eq!(layout(&shell, subscription), Layout::Strip);
        assert_eq!(but_for_its_layout(&shell, subscription), before);
        assert_eq!(shell.host_requests.len(), asked);

        // What is done in one layout is done as in the other: a tab closed at the side
        // is closed.
        acked(&shell.set_layout(window, Layout::Sidebar).await);
        let front = shell.window(subscription).tab.expect("a tab").tab_id;
        acked(&shell.close_tab(front).await);
        assert_eq!(shell.window(subscription).tabs.len(), 3);
        assert_eq!(layout(&shell, subscription), Layout::Sidebar);
    });
}

#[test]
fn what_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(
            code(shell.set_layout(window + 100, Layout::Sidebar).await),
            "window.unknown"
        );
        // No layout named, and one this core does not have.
        assert_eq!(
            code(shell.set_layout(window, Layout::Unspecified).await),
            "layout.unknown"
        );
        let later = shell
            .request(request::Kind::SetLayout(SetLayout {
                window_id: window,
                layout: 7,
            }))
            .await;
        assert_eq!(code(later), "layout.unknown");
        assert_eq!(layout(&shell, subscription), Layout::Strip);
        // None of it was taken for a choice: the next window has the strip.
        let (_, _, next) = shell.open_window().await;
        assert_eq!(layout(&shell, next), Layout::Strip);
    });
}

#[test]
fn each_window_has_its_own_and_a_new_one_has_the_one_last_chosen() {
    run(async {
        let mut shell = FakeShell::fake();
        let (first, _, first_subscription) = shell.open_window().await;
        let (second, _, second_subscription) = shell.open_window().await;

        // Switching one does not switch the other.
        acked(&shell.set_layout(first, Layout::Sidebar).await);
        assert_eq!(layout(&shell, first_subscription), Layout::Sidebar);
        assert_eq!(layout(&shell, second_subscription), Layout::Strip);
        // A window opened now has the one last chosen.
        let (_, _, third) = shell.open_window().await;
        assert_eq!(layout(&shell, third), Layout::Sidebar);
        // Chosen again in another window: the last choice is the next window's, and the
        // windows that are open keep theirs.
        acked(&shell.set_layout(second, Layout::Strip).await);
        let (_, _, fourth) = shell.open_window().await;
        assert_eq!(layout(&shell, fourth), Layout::Strip);
        assert_eq!(layout(&shell, first_subscription), Layout::Sidebar);
        assert_eq!(layout(&shell, third), Layout::Sidebar);

        // A private window has the layout last chosen, as any window; but what is chosen
        // in one is its own, and is not the next window's (037).
        acked(&shell.set_layout(first, Layout::Sidebar).await);
        let (private, _, private_subscription) = shell.open_private_window().await;
        assert_eq!(layout(&shell, private_subscription), Layout::Sidebar);
        acked(&shell.set_layout(private, Layout::Strip).await);
        assert_eq!(layout(&shell, private_subscription), Layout::Strip);
        let (_, _, after) = shell.open_window().await;
        assert_eq!(layout(&shell, after), Layout::Sidebar);
    });
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

#[test]
fn the_choice_outlives_a_quit_and_a_private_windows_does_not() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, MAIL).await;
        assert_eq!(
            layout(&shell, subscription),
            Layout::Strip,
            "a new installation has the strip"
        );
        acked(&shell.set_layout(window, Layout::Sidebar).await);
        drop(shell);

        // The first window after a quit has it, with its tabs as they were.
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(layout(&shell, subscription), Layout::Sidebar);
        assert_eq!(shell.window(subscription).tabs.len(), 1);
        // A private window's choice is not written.
        let (private, _, _) = shell.open_private_window().await;
        acked(&shell.set_layout(private, Layout::Strip).await);
        let _ = window;
        drop(shell);

        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(layout(&shell, subscription), Layout::Sidebar);
        // Back along the top, and quit: the strip.
        acked(&shell.set_layout(window, Layout::Strip).await);
        drop(shell);
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(layout(&shell, subscription), Layout::Strip);
    });
}

#[test]
fn a_store_that_names_a_layout_nobody_has_is_the_strip() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        drop(start(folder.path()));
        let mut store = wt_store::Store::open(&folder.path().join("webtiger.sqlite")).unwrap();
        store.set_setting("window.layout", "carousel").unwrap();
        drop(store);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(layout(&shell, subscription), Layout::Strip);
    });
}
