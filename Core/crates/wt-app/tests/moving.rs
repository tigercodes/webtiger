//! Moving a tab, and a Group, in the row (045). Through the contract, against
//! the scripted engine: a tab is put in its place among those of its part; one
//! moved to another part becomes of it; what is refused moves nothing; a step
//! stays within its part; a Group is moved whole; a move is undone; and the
//! order outlives a quit.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::collections::BTreeMap;
use std::path::Path;

use wt_proto::v1::{MoveTab, RowPart, RowStep, host_request, move_tab::To, request, response};
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

/// The row as a window is told of it: each tab by its name, `F` a Favourite,
/// `S` a Site, `z` asleep, `@` its Group; and after them the Groups, each
/// with whether it is shut.
fn row(shell: &FakeShell, subscription: u64) -> Vec<String> {
    let window = shell.window(subscription);
    let named: BTreeMap<u64, String> = window
        .groups
        .iter()
        .map(|group| (group.group_id, group.name.clone()))
        .collect();
    let mut said: Vec<String> = window
        .tabs
        .into_iter()
        .map(|tab| {
            let name = if tab.title.is_empty() {
                "(empty)".to_owned()
            } else {
                tab.title
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
        .collect();
    for group in window.groups {
        said.push(format!(
            "[{}{}]",
            group.name,
            if group.shut { " shut" } else { "" }
        ));
    }
    said
}

/// What an undo would undo in the row a window shows, as the window is told.
fn undoes(shell: &FakeShell, subscription: u64) -> String {
    shell.window(subscription).undo_arrangement
}

/// The name of the tab in front.
fn front(shell: &FakeShell, subscription: u64) -> String {
    shell
        .window(subscription)
        .tab
        .expect("a tab in front")
        .title
}

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

/// What a request was refused with, and the field of it that was at fault.
fn refusal(answer: response::Kind) -> (String, String) {
    match answer {
        response::Kind::Error(error) => (error.code, error.field),
        other => panic!("expected an error, got {other:?}"),
    }
}

fn said(code: &str, field: &str) -> (String, String) {
    (code.to_owned(), field.to_owned())
}

fn acked(answer: &response::Kind) {
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

async fn group(shell: &mut FakeShell, window: u64, name: &str) -> u64 {
    match shell.make_group(window, name).await {
        response::Kind::GroupMade(made) => made.group_id,
        other => panic!("expected a Group, got {other:?}"),
    }
}

fn before(tab: u64) -> To {
    To::BeforeTabId(tab)
}

fn end_of(part: RowPart) -> To {
    To::EndOf(part.into())
}

fn onto(group: u64) -> To {
    To::EndOfGroupId(group)
}

fn by(step: RowStep) -> To {
    To::By(step.into())
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

#[test]
fn a_tab_is_put_in_its_place_among_those_of_its_part() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story]) = four_tabs(&mut shell).await;
        let work_before = work_done(&shell);

        // Tabs that are no Sites: before another; before the first; to the end.
        acked(&shell.move_tab(story, before(board)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "A story", "Board", "News"]
        );
        acked(&shell.move_tab(news, before(mail)).await);
        assert_eq!(
            row(&shell, subscription),
            ["News", "Mail", "A story", "Board"]
        );
        acked(&shell.move_tab(news, end_of(RowPart::Tabs)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "A story", "Board", "News"]
        );
        // The tab in front is in front, wherever it is put.
        assert_eq!(front(&shell, subscription), "A story");

        // Sites that are in no Group.
        for tab in [mail, board, news] {
            acked(&shell.keep_tab(tab).await);
        }
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "News S", "A story"]
        );
        acked(&shell.move_tab(news, before(mail)).await);
        assert_eq!(
            row(&shell, subscription),
            ["News S", "Mail S", "Board S", "A story"]
        );
        acked(&shell.move_tab(news, end_of(RowPart::Sites)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "News S", "A story"]
        );

        // A Group's Sites.
        let work = group(&mut shell, window, "Work").await;
        for tab in [mail, board, news] {
            acked(&shell.move_to_group(tab, work).await);
        }
        acked(&shell.move_tab(news, before(board)).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "Mail S @Work",
                "News S @Work",
                "Board S @Work",
                "A story",
                "[Work]"
            ]
        );
        acked(&shell.move_tab(mail, onto(work)).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "News S @Work",
                "Board S @Work",
                "Mail S @Work",
                "A story",
                "[Work]"
            ]
        );

        // The Favourites.
        for tab in [mail, board, news] {
            acked(&shell.set_favourite(tab, true).await);
        }
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S", "Board F S", "News F S", "A story", "[Work]"]
        );
        acked(&shell.move_tab(news, before(mail)).await);
        assert_eq!(
            row(&shell, subscription),
            ["News F S", "Mail F S", "Board F S", "A story", "[Work]"]
        );
        acked(&shell.move_tab(news, end_of(RowPart::Favourites)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail F S", "Board F S", "News F S", "A story", "[Work]"]
        );

        // None of it loaded a page or destroyed a web view.
        assert_eq!(work_done(&shell), work_before);
        assert_eq!(front(&shell, subscription), "A story");
    });
}

#[test]
fn a_tab_moved_to_another_part_becomes_of_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story]) = four_tabs(&mut shell).await;
        acked(&shell.keep_tab(mail).await);
        let work_before = work_done(&shell);

        // A tab put before a Site is a Site, there; and at the end of the Sites, one.
        acked(&shell.move_tab(story, before(mail)).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story S", "Mail S", "Board", "News"]
        );
        acked(&shell.move_tab(board, end_of(RowPart::Sites)).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story S", "Mail S", "Board S", "News"]
        );
        // It goes both ways: a Site put before a tab that is none is a Site no longer; and
        // at the end of the tabs, none.
        acked(&shell.move_tab(story, before(news)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "A story", "News"]
        );
        acked(&shell.move_tab(mail, end_of(RowPart::Tabs)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board S", "A story", "News", "Mail"]
        );

        // Onto a Group, a tab is a Site of it; before a Site of a Group, of that Group, there.
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_tab(news, onto(work)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board S", "News S @Work", "A story", "Mail", "[Work]"]
        );
        acked(&shell.move_tab(mail, before(news)).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "Board S",
                "Mail S @Work",
                "News S @Work",
                "A story",
                "[Work]"
            ]
        );
        // A Site of no Group into the Group; and out of it, among the Sites that are in none.
        acked(&shell.move_tab(board, before(news)).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "Mail S @Work",
                "Board S @Work",
                "News S @Work",
                "A story",
                "[Work]"
            ]
        );
        acked(&shell.move_tab(news, end_of(RowPart::Sites)).await);
        acked(&shell.move_tab(mail, before(news)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "News S", "Board S @Work", "A story", "[Work]"]
        );

        // Among the Favourites, a Favourite: a tab that was no Site, and a Site of a Group.
        acked(&shell.move_tab(story, end_of(RowPart::Favourites)).await);
        acked(&shell.move_tab(board, before(story)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board F S", "A story F S", "Mail S", "News S", "[Work]"]
        );
        // A Favourite put in a Group, or among the Sites, is a Favourite no longer.
        acked(&shell.move_tab(story, onto(work)).await);
        acked(&shell.move_tab(board, before(news)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "News S", "A story S @Work", "[Work]"]
        );

        assert_eq!(work_done(&shell), work_before);
        assert_eq!(front(&shell, subscription), "A story");
    });
}

#[test]
fn what_is_refused_moves_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, _, _, story]) = four_tabs(&mut shell).await;
        let (_, elsewhere, _) = shell.open_window().await;
        shell.submit(elsewhere, NEWS).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.keep_tab(mail).await);
        let was = row(&shell, subscription);
        let newest = undoes(&shell, subscription);

        // A tab there is not: refused first, whatever else was said.
        assert_eq!(
            code(shell.move_tab(story + 100, before(mail)).await),
            "tab.unknown"
        );
        let nowhere = |tab: u64| {
            request::Kind::MoveTab(MoveTab {
                tab_id: tab,
                to: None,
            })
        };
        assert_eq!(
            code(shell.request(nowhere(story + 100)).await),
            "tab.unknown"
        );
        // No place named; and a part, and a step, this core does not know.
        assert_eq!(code(shell.request(nowhere(story)).await), "move.nowhere");
        assert_eq!(
            code(shell.move_tab(story, To::EndOf(0)).await),
            "move.nowhere"
        );
        assert_eq!(
            code(shell.move_tab(story, To::EndOf(9)).await),
            "move.nowhere"
        );
        assert_eq!(code(shell.move_tab(story, To::By(0)).await), "move.nowhere");
        assert_eq!(code(shell.move_tab(story, To::By(9)).await), "move.nowhere");
        // Before a tab of another window's row, and one there is not.
        assert_eq!(
            code(shell.move_tab(story, before(elsewhere)).await),
            "move.other_row"
        );
        assert_eq!(
            code(shell.move_tab(story, before(story + 100)).await),
            "move.other_row"
        );
        // Onto a Group there is not.
        assert_eq!(
            code(shell.move_tab(story, onto(work + 100)).await),
            "group.unknown"
        );
        assert_eq!(row(&shell, subscription), was);
        // Each names the field of the request that was at fault.
        assert_eq!(
            refusal(shell.move_tab(story + 100, before(mail)).await),
            said("tab.unknown", "tab_id")
        );
        assert_eq!(
            refusal(shell.request(nowhere(story)).await),
            said("move.nowhere", "to")
        );
        assert_eq!(
            refusal(shell.move_tab(story, before(elsewhere)).await),
            said("move.other_row", "before_tab_id")
        );
        assert_eq!(
            refusal(shell.move_tab(story, onto(work + 100)).await),
            said("group.unknown", "end_of_group_id")
        );
        assert_eq!(
            refusal(shell.move_tab(story + 100, by(RowStep::Back)).await),
            said("tab.unknown", "tab_id")
        );

        // A tab with no page of the web's cannot be a Site: it is not put among them, nor
        // in a Group, nor among the Favourites; and it is where it was.
        let empty = shell.open_tab(window, "").await;
        let with_the_empty_tab = row(&shell, subscription);
        assert!(with_the_empty_tab.contains(&"(empty)".to_owned()));
        assert_eq!(
            code(shell.move_tab(empty, before(mail)).await),
            "tab.no_page"
        );
        assert_eq!(
            code(shell.move_tab(empty, end_of(RowPart::Sites)).await),
            "tab.no_page"
        );
        assert_eq!(code(shell.move_tab(empty, onto(work)).await), "tab.no_page");
        assert_eq!(
            code(shell.move_tab(empty, end_of(RowPart::Favourites)).await),
            "tab.no_page"
        );
        assert_eq!(row(&shell, subscription), with_the_empty_tab);
        // Among the tabs that are no Sites it moves as any tab.
        acked(&shell.move_tab(empty, before(story)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board", "News", "(empty)", "A story", "[Work]"]
        );
        acked(&shell.undo_arrangement(window).await);
        // Nothing that was refused was kept to undo.
        assert_eq!(undoes(&shell, subscription), newest);

        // A private window's tabs move among themselves, which keeps nothing; and nothing
        // of one is kept by a move.
        let (private, first, private_subscription) = shell.open_private_window().await;
        shell.submit(first, BOARD).await;
        let second = shell.open_tab(private, NEWS).await;
        acked(&shell.move_tab(second, before(first)).await);
        assert_eq!(row(&shell, private_subscription), ["News", "Board"]);
        acked(&shell.move_tab(second, by(RowStep::Forward)).await);
        assert_eq!(row(&shell, private_subscription), ["Board", "News"]);
        assert_eq!(
            code(shell.move_tab(second, end_of(RowPart::Sites)).await),
            "window.private"
        );
        assert_eq!(
            code(shell.move_tab(second, end_of(RowPart::Favourites)).await),
            "window.private"
        );
        assert_eq!(row(&shell, private_subscription), ["Board", "News"]);
        assert_eq!(undoes(&shell, private_subscription), "tab.moved");
    });
}

#[test]
fn a_full_row_of_favourites_takes_no_more_by_a_move() {
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
        for tab in &tabs[..12] {
            acked(&shell.set_favourite(*tab, true).await);
        }
        let was = row(&shell, subscription);
        assert_eq!(was.last().map(String::as_str), Some("Site 12"));
        // The thirteenth is refused, before a Favourite and at their end; it is not kept
        // by the asking, and is where it was.
        assert_eq!(
            code(shell.move_tab(tabs[12], before(tabs[3])).await),
            "favourite.too_many"
        );
        assert_eq!(
            code(shell.move_tab(tabs[12], end_of(RowPart::Favourites)).await),
            "favourite.too_many"
        );
        assert_eq!(row(&shell, subscription), was);
        // The twelve move among themselves.
        acked(&shell.move_tab(tabs[11], before(tabs[0])).await);
        assert_eq!(row(&shell, subscription)[0], "Site 11 F S");
        assert_eq!(row(&shell, subscription)[12], "Site 12");
    });
}

#[test]
fn a_step_moves_a_tab_one_place_within_its_part_and_never_out_of_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, _, story]) = four_tabs(&mut shell).await;
        acked(&shell.keep_tab(mail).await);
        acked(&shell.keep_tab(board).await);
        let start = ["Mail S", "Board S", "News", "A story"];
        assert_eq!(row(&shell, subscription), start);

        // Among the Sites: forward one; at their end it stays, and is a Site; back one; at
        // the row's start it stays.
        acked(&shell.move_tab(mail, by(RowStep::Forward)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board S", "Mail S", "News", "A story"]
        );
        acked(&shell.move_tab(mail, by(RowStep::Forward)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board S", "Mail S", "News", "A story"]
        );
        acked(&shell.move_tab(mail, by(RowStep::Back)).await);
        assert_eq!(row(&shell, subscription), start);
        acked(&shell.move_tab(mail, by(RowStep::Back)).await);
        assert_eq!(row(&shell, subscription), start);

        // Among the tabs: back one; with the Sites before it, it stays, and is no Site;
        // forward one; at the row's end it stays.
        acked(&shell.move_tab(story, by(RowStep::Back)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "A story", "News"]
        );
        acked(&shell.move_tab(story, by(RowStep::Back)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "A story", "News"]
        );
        acked(&shell.move_tab(story, by(RowStep::Forward)).await);
        assert_eq!(row(&shell, subscription), start);
        acked(&shell.move_tab(story, by(RowStep::Forward)).await);
        assert_eq!(row(&shell, subscription), start);

        // The four steps that moved a tab are kept to undo, after the two tabs kept; the
        // four that moved nothing are not.
        let mut kinds = Vec::new();
        while !undoes(&shell, subscription).is_empty() {
            kinds.push(undoes(&shell, subscription));
            acked(&shell.undo_arrangement(window).await);
            assert!(kinds.len() <= 8, "more was undone than was done");
        }
        assert_eq!(
            kinds,
            [
                "tab.moved",
                "tab.moved",
                "tab.moved",
                "tab.moved",
                "site.kept",
                "site.kept"
            ]
        );
    });
}

#[test]
fn a_group_is_moved_whole_among_the_groups() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story]) = four_tabs(&mut shell).await;
        let work = group(&mut shell, window, "Work").await;
        let house = group(&mut shell, window, "House").await;
        let play = group(&mut shell, window, "Play").await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, house).await);
        acked(&shell.move_to_group(news, play).await);
        let made = [
            "Mail S @Work",
            "Board S @House",
            "News S @Play",
            "A story",
            "[Work]",
            "[House]",
            "[Play]",
        ];
        assert_eq!(row(&shell, subscription), made);
        let (other, other_tab, _) = shell.open_window().await;
        shell.submit(other_tab, NEWS).await;
        let theirs = group(&mut shell, other, "Theirs").await;
        let work_before = work_done(&shell);

        // The last before the first: its Site goes with it. And after the last again.
        acked(&shell.move_group(play, work).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "News S @Play",
                "Mail S @Work",
                "Board S @House",
                "A story",
                "[Play]",
                "[Work]",
                "[House]"
            ]
        );
        assert_eq!(undoes(&shell, subscription), "group.reordered");
        acked(&shell.move_group(play, 0).await);
        assert_eq!(row(&shell, subscription), made);
        // The first before the last: between the others.
        acked(&shell.move_group(work, play).await);
        let moved = [
            "Board S @House",
            "Mail S @Work",
            "News S @Play",
            "A story",
            "[House]",
            "[Work]",
            "[Play]",
        ];
        assert_eq!(row(&shell, subscription), moved);

        // Before itself, and where it is already: nothing, and nothing kept to undo. Three
        // moves were: three are undone, and then the tabs put in their Groups.
        acked(&shell.move_group(work, work).await);
        acked(&shell.move_group(work, play).await);
        acked(&shell.move_group(play, 0).await);
        assert_eq!(row(&shell, subscription), moved);
        // What is refused: a Group there is not, either way; and one of another row.
        assert_eq!(code(shell.move_group(play + 100, 0).await), "group.unknown");
        assert_eq!(
            code(shell.move_group(play, play + 100).await),
            "group.unknown"
        );
        assert_eq!(code(shell.move_group(play, theirs).await), "group.unknown");
        assert_eq!(
            refusal(shell.move_group(play + 100, 0).await),
            said("group.unknown", "group_id")
        );
        assert_eq!(
            refusal(shell.move_group(play, play + 100).await),
            said("group.unknown", "before_group_id")
        );
        assert_eq!(row(&shell, subscription), moved);
        for _ in 0..3 {
            assert_eq!(undoes(&shell, subscription), "group.reordered");
            acked(&shell.undo_arrangement(window).await);
        }
        assert_eq!(row(&shell, subscription), made);
        assert_eq!(undoes(&shell, subscription), "group.moved");

        // A shut Group is moved shut; and the Sites that are in no Group stay before
        // every Group.
        acked(&shell.shut_group(work, true).await);
        acked(&shell.move_group(work, 0).await);
        acked(&shell.keep_tab(story).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "A story S",
                "Board S @House",
                "News S @Play",
                "[House]",
                "[Play]",
                "[Work shut]"
            ]
        );
        assert_eq!(work_done(&shell), work_before);
    });
}

#[test]
fn a_move_is_undone_and_one_that_moved_nothing_is_not_kept() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story]) = four_tabs(&mut shell).await;
        let start = ["Mail", "Board", "News", "A story"];

        acked(&shell.move_tab(story, before(mail)).await);
        assert_eq!(undoes(&shell, subscription), "tab.moved");
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(row(&shell, subscription), start);
        assert_eq!(undoes(&shell, subscription), "");

        // A Site that a move unmade is a Site again when the move is undone, where it was.
        acked(&shell.keep_tab(board).await);
        acked(&shell.move_tab(board, end_of(RowPart::Tabs)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "News", "A story", "Board"]
        );
        assert_eq!(undoes(&shell, subscription), "tab.moved");
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board S", "Mail", "News", "A story"]
        );
        assert_eq!(undoes(&shell, subscription), "site.kept");

        // Moves that move nothing are not kept: before itself; before the tab it is before
        // already; to the end it is at; a step at its part's end.
        acked(&shell.move_tab(news, before(news)).await);
        acked(&shell.move_tab(news, before(story)).await);
        acked(&shell.move_tab(story, end_of(RowPart::Tabs)).await);
        acked(&shell.move_tab(story, by(RowStep::Forward)).await);
        acked(&shell.move_tab(board, end_of(RowPart::Sites)).await);
        acked(&shell.move_tab(board, by(RowStep::Back)).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board S", "Mail", "News", "A story"]
        );
        assert_eq!(undoes(&shell, subscription), "site.kept");
    });
}

#[test]
fn a_tab_moved_into_a_shut_group_is_out_of_the_way_unless_it_is_in_front() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, _, story]) = four_tabs(&mut shell).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.shut_group(work, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board", "News", "A story", "[Work shut]"]
        );
        let work_before = work_done(&shell);

        // A tab moved onto the shut Group is its Site, and out of the way with the others.
        acked(&shell.move_tab(board, onto(work)).await);
        assert_eq!(
            row(&shell, subscription),
            ["News", "A story", "[Work shut]"]
        );
        // The tab in front, moved onto it, is shown still, and in front.
        acked(&shell.move_tab(story, onto(work)).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story S @Work", "News", "[Work shut]"]
        );
        assert_eq!(front(&shell, subscription), "A story");
        // Opened, the Group has them in the order they were put in it; none was put to
        // sleep, and nothing was loaded.
        acked(&shell.shut_group(work, false).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "Mail S @Work",
                "Board S @Work",
                "A story S @Work",
                "News",
                "[Work]"
            ]
        );
        assert_eq!(work_done(&shell), work_before);
    });
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

#[test]
fn the_order_outlives_a_quit() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let board = shell.open_tab(window, BOARD).await;
        let news = shell.open_tab(window, NEWS).await;
        let story = shell.open_tab(window, STORY).await;
        let work = group(&mut shell, window, "Work").await;
        let house = group(&mut shell, window, "House").await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, house).await);
        // The Groups the other way round, and the tabs.
        acked(&shell.move_group(house, work).await);
        acked(&shell.move_tab(story, before(news)).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "Board S @House",
                "Mail S @Work",
                "A story",
                "News",
                "[House]",
                "[Work]"
            ]
        );
        drop(shell);

        // After a quit the row is in the order it was left in: the tab that was in front
        // loaded, the others asleep.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(
            row(&shell, subscription),
            [
                "Board S z @House",
                "Mail S z @Work",
                "A story",
                "News z",
                "[House]",
                "[Work]"
            ]
        );
    });
}
