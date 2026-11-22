//! Favourites (042): the few Sites that are always first. Through the
//! contract, against the scripted engine: marking a tab and unmarking it,
//! where a Favourite sits in the row, what a Group and a Favourite are to one
//! another, how many a row may have, and what comes back after a quit.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::collections::BTreeMap;
use std::path::Path;

use wt_proto::v1::{host_request, response};
use wt_store::Store;
use wt_testkit::{FakeShell, run};

const MAIL: &str = "https://mail.example.com/";
const INBOX: &str = "https://mail.example.com/inbox/7";
const BOARD: &str = "https://board.example.com/";
const NEWS: &str = "https://news.example.com/";
const STORY: &str = "https://news.example.com/story";

fn web(shell: &mut FakeShell) {
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(INBOX, "Re: lunch");
    shell.engine.page(BOARD, "Board");
    shell.engine.page(NEWS, "News");
    shell.engine.page(STORY, "A story");
}

/// The tabs a window is told to draw, in order: each by its name, `F` when
/// it is a Favourite, `S` a Site, `z` asleep, and `@` the name of its Group.
fn row(shell: &FakeShell, subscription: u64) -> Vec<String> {
    let window = shell.window(subscription);
    let named: BTreeMap<u64, String> = window
        .groups
        .iter()
        .map(|group| (group.group_id, group.name.clone()))
        .collect();
    window
        .tabs
        .into_iter()
        .map(|tab| {
            let name = match (tab.title.is_empty(), tab.host.is_empty()) {
                (false, _) => tab.title,
                (true, false) => tab.host,
                (true, true) => "(empty)".to_owned(),
            };
            let group = match tab.group_id {
                0 => String::new(),
                id => format!(
                    " @{}",
                    named
                        .get(&id)
                        .map_or("a Group there is not", String::as_str)
                ),
            };
            format!(
                "{name}{}{}{}{group}",
                if tab.favourite { " F" } else { "" },
                if tab.kept { " S" } else { "" },
                if tab.asleep { " z" } else { "" },
            )
        })
        .collect()
}

/// The tab in front: its title, and whether it says it is a Favourite.
fn front(shell: &FakeShell, subscription: u64) -> (String, bool) {
    let tab = shell.window(subscription).tab.expect("a tab");
    (tab.title, tab.favourite)
}

/// How many pages the shell was asked to load and web views to destroy.
fn work_done(shell: &FakeShell) -> (usize, usize) {
    let count = |wanted: fn(&host_request::Kind) -> bool| {
        shell
            .host_requests
            .iter()
            .filter(|request| wanted(request))
            .count()
    };
    (
        count(|request| matches!(request, host_request::Kind::WebViewLoad(_))),
        count(|request| matches!(request, host_request::Kind::WebViewDestroy(_))),
    )
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

/// A window with four tabs, Mail, Board, News and A story, the last in front.
async fn four_tabs(shell: &mut FakeShell) -> (u64, u64, [u64; 4]) {
    web(shell);
    let (window, mail, subscription) = shell.open_window().await;
    shell.submit(mail, MAIL).await;
    let board = shell.open_tab(window, BOARD).await;
    let news = shell.open_tab(window, NEWS).await;
    let story = shell.open_tab(window, STORY).await;
    (window, subscription, [mail, board, news, story])
}

async fn group(shell: &mut FakeShell, window: u64, name: &str) -> u64 {
    match shell.make_group(window, name).await {
        response::Kind::GroupMade(made) => made.group_id,
        other => panic!("expected a Group, got {other:?}"),
    }
}

#[test]
fn a_tab_marked_is_kept_and_favourites_are_the_first_of_the_row_in_the_order_they_were_made() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, [mail, board, news, story]) = four_tabs(&mut shell).await;
        let before = work_done(&shell);

        // Marked, a tab that is no Site is kept: a Site, a Favourite, the first of the
        // row. It is still in front, and says so.
        acked(&shell.set_favourite(story, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story F S", "Mail", "Board", "News"]
        );
        assert_eq!(front(&shell, subscription), ("A story".to_owned(), true));

        // A Site that is in no Group is after the Favourites; one marked is the last of
        // the Favourites, wherever it was.
        acked(&shell.keep_tab(board).await);
        acked(&shell.set_favourite(mail, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story F S", "Mail F S", "Board S", "News"]
        );
        // A Site is marked as a tab is.
        acked(&shell.set_favourite(board, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story F S", "Mail F S", "Board F S", "News"]
        );
        // Marked again, a Favourite is where it was.
        acked(&shell.set_favourite(story, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story F S", "Mail F S", "Board F S", "News"]
        );

        // Unmarked, it is a Site still: the last of those that are in no Group.
        acked(&shell.keep_tab(news).await);
        acked(&shell.set_favourite(story, false).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S", "Board F S", "News S", "A story S"]
        );
        assert_eq!(front(&shell, subscription), ("A story".to_owned(), false));
        // Unmarked again, it is where it was; and marked again, the last of the Favourites.
        acked(&shell.set_favourite(story, false).await);
        acked(&shell.set_favourite(story, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S", "Board F S", "A story F S", "News S"]
        );

        // None of it loaded a page or destroyed a web view.
        assert_eq!(work_done(&shell), before);
    });
}

#[test]
fn what_cannot_be_a_favourite() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, _, _, story]) = four_tabs(&mut shell).await;
        let as_it_was = row(&shell, subscription);

        // A tab there is not, either way; a tab with no page to keep.
        assert_eq!(
            code(shell.set_favourite(story + 100, true).await),
            "tab.unknown"
        );
        assert_eq!(
            code(shell.set_favourite(story + 100, false).await),
            "tab.unknown"
        );
        let empty = shell.open_tab(window, "").await;
        assert_eq!(code(shell.set_favourite(empty, true).await), "tab.no_page");
        // Unmarking a tab that is none is not refused, and keeps nothing.
        acked(&shell.set_favourite(empty, false).await);
        acked(&shell.set_favourite(mail, false).await);
        acked(&shell.close_tab(empty).await);
        assert_eq!(row(&shell, subscription), as_it_was);

        // A private window's tab: nothing of one is kept.
        let (_, private, private_subscription) = shell.open_private_window().await;
        shell.submit(private, BOARD).await;
        assert_eq!(
            code(shell.set_favourite(private, true).await),
            "window.private"
        );
        acked(&shell.set_favourite(private, false).await);
        assert_eq!(row(&shell, private_subscription), ["Board"]);
    });
}

#[test]
fn a_row_has_no_more_than_twelve_favourites() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription) = shell.open_window().await;
        let mut tabs = vec![first];
        for number in 0..13 {
            let address = format!("https://site{number}.example.com/");
            shell.engine.page(&address, &format!("Site {number}"));
            if number == 0 {
                shell.submit(first, &address).await;
            } else {
                tabs.push(shell.open_tab(window, &address).await);
            }
        }
        let full =
            |shell: &FakeShell, subscription: u64| shell.window(subscription).favourites_full;
        for tab in &tabs[..11] {
            acked(&shell.set_favourite(*tab, true).await);
        }
        assert!(!full(&shell, subscription), "eleven: there is room for one");
        acked(&shell.set_favourite(tabs[11], true).await);
        // The window is told the row is full: a shell offers no more, and needs no number.
        assert!(full(&shell, subscription), "twelve: the row is full");
        // The thirteenth is refused, and is not kept by the asking.
        assert_eq!(
            code(shell.set_favourite(tabs[12], true).await),
            "favourite.too_many"
        );
        let said = row(&shell, subscription);
        assert_eq!(said.iter().filter(|tab| tab.contains(" F")).count(), 12);
        assert_eq!(said[12], "Site 12");
        // One that is a Favourite already is not counted again; and with one unmarked
        // there is room for one.
        acked(&shell.set_favourite(tabs[3], true).await);
        acked(&shell.set_favourite(tabs[0], false).await);
        assert!(!full(&shell, subscription), "eleven again");
        acked(&shell.set_favourite(tabs[12], true).await);
        assert!(full(&shell, subscription));
        let said = row(&shell, subscription);
        assert_eq!(said.iter().filter(|tab| tab.contains(" F")).count(), 12);
        assert_eq!(said[11], "Site 12 F S");
        assert_eq!(said[12], "Site 0 S");

        // The count is a row's own: another window has its twelve.
        let (other, other_tab, other_subscription) = shell.open_window().await;
        shell.submit(other_tab, "https://site0.example.com/").await;
        acked(&shell.set_favourite(other_tab, true).await);
        assert_eq!(row(&shell, other_subscription), ["Site 0 F S"]);
        assert!(!full(&shell, other_subscription), "its row has one");
        assert!(
            full(&shell, subscription),
            "and the first window's is full still"
        );
        let _ = other;
    });
}

#[test]
fn a_favourite_is_in_no_group_and_a_site_put_in_a_group_is_no_favourite() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story]) = four_tabs(&mut shell).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.keep_tab(news).await);
        assert_eq!(
            row(&shell, subscription),
            ["News S", "Mail S @Work", "Board S @Work", "A story"]
        );
        let groups = |shell: &FakeShell| shell.window(subscription).groups[0].sites;

        // Marked, a Site of a Group leaves it: the row is the Favourites, the Sites in
        // no Group, the Group's, the tabs.
        acked(&shell.set_favourite(mail, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S", "News S", "Board S @Work", "A story"]
        );
        assert_eq!(groups(&shell), 1, "the Group holds one fewer");

        // Put in a Group, a Favourite is one no longer; and a tab that is no Site, put
        // in a Group, is no Favourite either.
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(story, work).await);
        assert_eq!(
            row(&shell, subscription),
            ["News S", "Board S @Work", "Mail S @Work", "A story S @Work"]
        );
        // Taken out of no Group, a Favourite is as it was.
        acked(&shell.set_favourite(news, true).await);
        acked(&shell.move_to_group(news, 0).await);
        assert_eq!(row(&shell, subscription)[0], "News F S");

        // A Favourite is never out of the way: its Group is none, whichever is shut.
        acked(&shell.set_favourite(board, true).await);
        acked(&shell.shut_group(work, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["News F S", "Board F S", "A story S @Work"],
            "the tab in front is shown; the other of the Group is not"
        );
        // Removing the Group leaves the Favourites as they are.
        acked(&shell.remove_group(work).await);
        assert_eq!(
            row(&shell, subscription),
            ["News F S", "Board F S", "Mail S", "A story S"]
        );
    });
}

#[test]
fn a_favourite_is_a_site_closed_it_sleeps_and_let_go_it_is_a_tab() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, [mail, board, _, story]) = four_tabs(&mut shell).await;
        acked(&shell.set_favourite(mail, true).await);
        acked(&shell.set_favourite(board, true).await);
        // It wanders from its home.
        acked(&shell.activate_tab(mail).await);
        shell.submit(mail, INBOX).await;
        assert_eq!(
            row(&shell, subscription),
            ["Re: lunch F S", "Board F S", "News", "A story"]
        );

        // Closed, a Favourite sleeps where it is, at its home, by its name: a Favourite
        // still. Pressed, it wakes there.
        acked(&shell.close_tab(mail).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S z", "Board F S", "News", "A story"]
        );
        acked(&shell.activate_tab(mail).await);
        assert_eq!(front(&shell, subscription), ("Mail".to_owned(), true));
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S", "Board F S", "News", "A story"]
        );

        // Let go, it is a tab like any other, the first of those: no Site, no Favourite.
        acked(&shell.let_go_site(mail).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board F S", "Mail", "News", "A story"]
        );
        assert_eq!(front(&shell, subscription), ("Mail".to_owned(), false));
        // Kept again it is a Site, and does not go back among the Favourites by itself.
        acked(&shell.keep_tab(mail).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board F S", "Mail S", "News", "A story"]
        );
        let _ = story;
    });
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

#[test]
fn favourites_come_back_after_a_quit_in_their_order() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let board = shell.open_tab(window, BOARD).await;
        let news = shell.open_tab(window, NEWS).await;
        let story = shell.open_tab(window, STORY).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.set_favourite(news, true).await);
        acked(&shell.set_favourite(mail, true).await);
        acked(&shell.move_to_group(board, work).await);
        assert_eq!(
            row(&shell, subscription),
            ["News F S", "Mail F S", "Board S @Work", "A story"]
        );
        let _ = story;
        drop(shell);

        // They are back, in the order they were made, each a Site asleep at its home;
        // the Group's Site and the tab are as they were.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(
            row(&shell, subscription),
            ["News F S z", "Mail F S z", "Board S z @Work", "A story"]
        );
        // Unmarked and quit again: it comes back a Site that is no Favourite.
        let ids: Vec<u64> = shell
            .window(subscription)
            .tabs
            .iter()
            .map(|tab| tab.tab_id)
            .collect();
        acked(&shell.set_favourite(ids[0], false).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S z", "News S z", "Board S z @Work", "A story"]
        );
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S z", "News S z", "Board S z @Work", "A story"]
        );
    });
}

#[test]
fn only_a_site_is_a_favourite_and_none_is_in_a_group_whatever_the_store_says() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let board = shell.open_tab(window, BOARD).await;
        shell.open_tab(window, NEWS).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.keep_tab(mail).await);
        acked(&shell.move_to_group(board, work).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S @Work", "News"]
        );
        drop(shell);

        // The store is made to say what the core never writes: a Site of a Group that is
        // a Favourite too, and a tab that is no Site and a Favourite.
        let mut store = Store::open(&folder.path().join("webtiger.sqlite")).unwrap();
        let space = store.spaces().unwrap()[0].id;
        let mut tabs = store.saved_tabs(space).unwrap();
        assert_eq!(
            (tabs[1].group, tabs[1].favourite, tabs[2].home.as_str()),
            (1, false, "")
        );
        tabs[1].favourite = true;
        tabs[2].favourite = true;
        store.save_tabs(space, &tabs).unwrap();
        drop(store);

        // The Site is a Favourite, first in the row, and in no Group; the tab is a tab.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(
            row(&shell, subscription),
            ["Board F S z", "Mail S z", "News"]
        );
        assert_eq!(shell.window(subscription).groups[0].sites, 0);
    });
}
