//! Undo for what is done to the tree (044). Through the contract, against
//! the scripted engine: each thing that arranges a row is undone, the newest
//! first; a row is put back as it was though tabs have come and gone since;
//! what is no arranging is not kept; and a closed tab's way back is apart.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::collections::BTreeMap;
use std::path::Path;

use wt_proto::v1::{
    Layout, RowStep, StepTab, Undo, host_request, move_tab, request, response, step_tab,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const MAIL: &str = "https://mail.example.com/";
const BOARD: &str = "https://board.example.com/";
const NEWS: &str = "https://news.example.com/";
const STORY: &str = "https://news.example.com/story";
const CARD: &str = "https://board.example.com/card/3";

fn web(shell: &mut FakeShell) {
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(BOARD, "Board");
    shell.engine.page(NEWS, "News");
    shell.engine.page(STORY, "A story");
    shell.engine.page(CARD, "Card 3");
}

/// The row as a window is told of it: each tab by its name, `F` a Favourite,
/// `S` a Site, `z` asleep, `@` its Group; and after a line the Groups, each
/// with its colour and whether it is shut.
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
            "[{}{}{}{}]",
            group.name,
            if group.colour.is_empty() { "" } else { " " },
            group.colour,
            if group.shut { " shut" } else { "" }
        ));
    }
    said
}

/// What an undo would undo in the row a window shows, as the window is told.
fn undoes(shell: &FakeShell, subscription: u64) -> String {
    shell.window(subscription).undo_arrangement
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

fn acked(answer: &response::Kind) {
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

async fn group(shell: &mut FakeShell, window: u64, name: &str) -> u64 {
    match shell.make_group(window, name).await {
        response::Kind::GroupMade(made) => made.group_id,
        other => panic!("expected a Group, got {other:?}"),
    }
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
fn each_thing_that_arranges_a_row_is_undone_the_newest_first() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, _]) = four_tabs(&mut shell).await;
        let before = work_done(&shell);
        // Nothing done yet: nothing to undo, and the window is told so.
        assert_eq!(undoes(&shell, subscription), "");
        assert_eq!(code(shell.undo_arrangement(window).await), "undo.nothing");
        assert_eq!(
            code(shell.undo_arrangement(window + 100).await),
            "window.unknown"
        );

        // Nine things done, the window told after each what an undo would undo; and the
        // row as it was before each is kept here, to see that it comes back.
        let mut steps: Vec<(Vec<String>, &str)> = Vec::new();
        macro_rules! done {
            ($kind:literal, $doing:expr) => {{
                let was = row(&shell, subscription);
                $doing;
                assert_eq!(undoes(&shell, subscription), $kind);
                steps.push((was, $kind));
            }};
        }
        done!("site.kept", acked(&shell.keep_tab(mail).await));
        let work = {
            let was = row(&shell, subscription);
            let made = group(&mut shell, window, "Work").await;
            assert_eq!(undoes(&shell, subscription), "group.made");
            steps.push((was, "group.made"));
            made
        };
        done!(
            "group.moved",
            acked(&shell.move_to_group(board, work).await)
        );
        done!(
            "group.renamed",
            acked(&shell.rename_group(work, "Office").await)
        );
        done!(
            "group.coloured",
            acked(&shell.colour_group(work, "blue").await)
        );
        done!(
            "favourite.added",
            acked(&shell.set_favourite(news, true).await)
        );
        done!(
            "favourite.removed",
            acked(&shell.set_favourite(news, false).await)
        );
        done!("site.let_go", acked(&shell.let_go_site(mail).await));
        done!("group.removed", acked(&shell.remove_group(work).await));
        assert_eq!(
            row(&shell, subscription),
            ["News S", "Board S", "Mail", "A story"]
        );

        // Undone one by one, the newest first: each time the row is as it was before that
        // thing, and the window is told what the next would undo.
        while let Some((was, kind)) = steps.pop() {
            assert_eq!(undoes(&shell, subscription), kind);
            acked(&shell.undo_arrangement(window).await);
            assert_eq!(row(&shell, subscription), was, "after undoing {kind}");
        }
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "Board", "News", "A story"]
        );
        assert_eq!(undoes(&shell, subscription), "");
        assert_eq!(code(shell.undo_arrangement(window).await), "undo.nothing");

        // None of it, done or undone, loaded a page or destroyed a web view.
        assert_eq!(work_done(&shell), before);
    });
}

#[test]
fn what_changed_nothing_and_what_is_no_arranging_is_not_kept() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, _, story]) = four_tabs(&mut shell).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.undo_arrangement(window).await);
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(undoes(&shell, subscription), "");
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.keep_tab(mail).await);
        assert_eq!(undoes(&shell, subscription), "site.kept");

        // Things that change nothing: a Site kept again; a tab that is none let go, taken
        // out of no Group, unmarked; a Group given the name and the colour it has.
        acked(&shell.keep_tab(mail).await);
        acked(&shell.let_go_site(board).await);
        acked(&shell.move_to_group(board, 0).await);
        acked(&shell.set_favourite(board, false).await);
        acked(&shell.rename_group(work, "Work").await);
        acked(&shell.colour_group(work, "").await);
        // Things that are refused.
        assert_eq!(code(shell.keep_tab(story + 100).await), "tab.unknown");
        assert_eq!(
            code(shell.rename_group(work, " ").await),
            "group.name_empty"
        );
        assert_eq!(
            code(shell.colour_group(work, "mauve").await),
            "group.colour_unknown"
        );
        assert_eq!(
            code(shell.move_to_group(board, work + 100).await),
            "group.unknown"
        );
        assert_eq!(code(shell.remove_group(work + 100).await), "group.unknown");
        // Things that are how the row is looked at: a Group shut and opened, the layout
        // switched, a tab brought in front.
        acked(&shell.shut_group(work, true).await);
        acked(&shell.shut_group(work, false).await);
        acked(&shell.set_layout(window, Layout::Sidebar).await);
        acked(&shell.activate_tab(board).await);
        // The newest is still the tab that was kept; and there are two things to undo, the
        // tab kept and the Group made, and no more.
        assert_eq!(undoes(&shell, subscription), "site.kept");
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(undoes(&shell, subscription), "group.made");
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(code(shell.undo_arrangement(window).await), "undo.nothing");
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "Board", "News", "A story"]
        );

        // A private window has nothing to arrange, and nothing to undo.
        let (private, tab, private_subscription) = shell.open_private_window().await;
        shell.submit(tab, BOARD).await;
        assert_eq!(code(shell.keep_tab(tab).await), "window.private");
        assert_eq!(undoes(&shell, private_subscription), "");
        assert_eq!(code(shell.undo_arrangement(private).await), "undo.nothing");
    });
}

#[test]
fn a_row_is_put_back_though_tabs_have_come_and_gone_since() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story]) = four_tabs(&mut shell).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.keep_tab(news).await);
        acked(&shell.shut_group(work, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["News S", "A story", "[Work shut]"]
        );
        // The Group is removed: its Sites are in no Group, and shown.
        acked(&shell.remove_group(work).await);
        assert_eq!(
            row(&shell, subscription),
            ["News S", "Mail S", "Board S", "A story"]
        );

        // Since then: a tab closed, a tab opened, and a Site of the Group that was put to
        // sleep.
        acked(&shell.close_tab(story).await);
        let card = shell.open_tab(window, CARD).await;
        acked(&shell.activate_tab(board).await);
        acked(&shell.close_tab(board).await);
        acked(&shell.activate_tab(card).await);
        assert_eq!(
            row(&shell, subscription),
            ["News S", "Mail S", "Board S z", "Card 3"]
        );
        let before = work_done(&shell);

        // Undone: the Group is back, as it was, with the Sites it held that are still here,
        // the sleeping one asleep still; the tab that was closed stays closed, and the tab
        // that came since stays. A Group that comes back was shut when it was removed: it
        // is as it was, and hides its Sites.
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(
            row(&shell, subscription),
            ["News S", "Card 3", "[Work shut]"]
        );
        // The tab that was closed is not in the row in any way: the second place of the
        // row is the tab that came since, and not where a closed tab was.
        acked(&shell.activate_tab(news).await);
        let answer = shell
            .request(request::Kind::StepTab(StepTab {
                window_id: window,
                to: step_tab::To::Place.into(),
                place: 2,
            }))
            .await;
        acked(&answer);
        assert_eq!(
            shell.window(subscription).tab.expect("a tab").title,
            "Card 3"
        );
        acked(
            &shell
                .shut_group(shell.window(subscription).groups[0].group_id, false)
                .await,
        );
        assert_eq!(
            row(&shell, subscription),
            [
                "News S",
                "Mail S @Work",
                "Board S z @Work",
                "Card 3",
                "[Work]"
            ]
        );
        assert_eq!(
            work_done(&shell),
            before,
            "undoing loads nothing and closes nothing"
        );

        // The closed tab has its own way back, apart: it was not the arrangement's to bring.
        let answer = shell
            .request(request::Kind::Undo(Undo { window_id: window }))
            .await;
        assert!(matches!(answer, response::Kind::TabOpened(_)), "{answer:?}");
        assert_eq!(
            row(&shell, subscription).last().map(String::as_str),
            Some("[Work]")
        );
        assert!(row(&shell, subscription).contains(&"A story".to_owned()));
        // And bringing a closed tab back is no arranging: the next undo is the Site kept.
        assert_eq!(undoes(&shell, subscription), "site.kept");
    });
}

#[test]
fn a_group_that_is_still_there_is_shut_or_open_as_it_is_now() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, _, _, _]) = four_tabs(&mut shell).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_to_group(mail, work).await);
        // Named again while open; then shut. Undoing the name leaves it shut: whether a
        // Group is shut is how the row is looked at, and is not undone.
        acked(&shell.rename_group(work, "Office").await);
        acked(&shell.shut_group(work, true).await);
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board", "News", "A story", "[Work shut]"]
        );
    });
}

#[test]
fn each_row_has_its_own_line_fifty_deep() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, _, _, _]) = four_tabs(&mut shell).await;
        let (other, other_tab, other_subscription) = shell.open_window().await;
        shell.submit(other_tab, NEWS).await;

        // What is done in one window's row is that row's to undo.
        acked(&shell.keep_tab(mail).await);
        assert_eq!(undoes(&shell, subscription), "site.kept");
        assert_eq!(undoes(&shell, other_subscription), "");
        assert_eq!(code(shell.undo_arrangement(other).await), "undo.nothing");
        acked(&shell.keep_tab(other_tab).await);
        acked(&shell.undo_arrangement(other).await);
        assert_eq!(row(&shell, other_subscription), ["News"]);
        assert_eq!(undoes(&shell, subscription), "site.kept");

        // Fifty are kept: of sixty things done after the first, the newest fifty can be
        // undone, and the first is past undoing.
        for _ in 0..30 {
            acked(&shell.set_favourite(mail, true).await);
            acked(&shell.set_favourite(mail, false).await);
        }
        let mut undone = 0;
        while !undoes(&shell, subscription).is_empty() {
            acked(&shell.undo_arrangement(window).await);
            undone += 1;
            assert!(undone <= 60, "more was undone than is kept");
        }
        assert_eq!(undone, 50);
        assert_eq!(code(shell.undo_arrangement(window).await), "undo.nothing");
        assert!(
            row(&shell, subscription)[0].starts_with("Mail"),
            "and the tab that was kept first is a Site still: {:?}",
            row(&shell, subscription)
        );
        assert_eq!(
            row(&shell, subscription)[0],
            "Mail S",
            "as it was after the eleventh thing done: a Site, and no Favourite"
        );
    });
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

#[test]
fn what_is_undone_is_what_is_kept_across_a_quit_and_the_line_is_not() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        shell.open_tab(window, BOARD).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.remove_group(work).await);
        // The Group's removal undone: the row with its Group is what is written.
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S @Work", "Board", "[Work]"]
        );
        drop(shell);

        // After a quit the row is as it was left, the undoing included; and there is
        // nothing of the arrangement to undo: the line is the run's.
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(
            row(&shell, subscription),
            ["Mail S z @Work", "Board", "[Work]"]
        );
        assert_eq!(undoes(&shell, subscription), "");
        assert_eq!(code(shell.undo_arrangement(window).await), "undo.nothing");
    });
}

/// Opens a link of the page in front in a tab behind it, beside it in the row.
async fn link_beside_the_front(shell: &mut FakeShell, subscription: u64, url: &str) {
    let web_view = shell.window(subscription).tab.expect("a tab").web_view_id;
    shell.report(FakeEngine::link_open_requested(web_view, url, true, true));
    shell.settle().await;
}

/// A step keeps what the thing done changed (062), and undoing it puts that back and
/// nothing else: a thing that moved no tab moves none when it is undone, whatever has come
/// since.
#[test]
fn a_thing_that_moved_nothing_moves_nothing_when_it_is_undone() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, ..]) = four_tabs(&mut shell).await;
        // A Group is made, named again and coloured: none of them moves a tab.
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.rename_group(work, "Play").await);
        acked(&shell.colour_group(work, "green").await);
        // Since then: a link of the first tab's page, opened in a tab beside it.
        acked(&shell.activate_tab(mail).await);
        link_beside_the_front(&mut shell, subscription, CARD).await;
        let beside = ["Mail", "Card 3", "Board", "News", "A story"];
        assert_eq!(row(&shell, subscription)[..5], beside);

        // Each undone in turn: the Group is what it was, and every tab is where it is.
        for (undone, groups) in [
            ("group.coloured", vec!["[Play]"]),
            ("group.renamed", vec!["[Work]"]),
            ("group.made", vec![]),
        ] {
            assert_eq!(undoes(&shell, subscription), undone);
            acked(&shell.undo_arrangement(window).await);
            let now = row(&shell, subscription);
            assert_eq!(now[..5], beside, "after undoing {undone}");
            assert_eq!(now[5..], groups, "after undoing {undone}");
        }
        assert_eq!(undoes(&shell, subscription), "");
    });
}

/// Where the thing changed what a tab is and not the order, the row is still in a row's
/// order once it is undone (062): a tab that came since is not left among the Sites.
#[test]
fn a_site_let_go_is_a_site_again_before_a_tab_that_came_since() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, ..]) = four_tabs(&mut shell).await;
        acked(&shell.keep_tab(mail).await);
        acked(&shell.keep_tab(board).await);
        // The last of the Sites is let go: it is the first of the tabs, where it was.
        acked(&shell.let_go_site(board).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board", "News", "A story"]
        );
        // Since then a link was opened beside the Site in front: a tab, so after the Sites.
        acked(&shell.activate_tab(mail).await);
        link_beside_the_front(&mut shell, subscription, CARD).await;
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Card 3", "Board", "News", "A story"]
        );

        // Undone: the Site is a Site again, with the Sites; the tab that came since is
        // after it, and before the tabs it was before.
        acked(&shell.undo_arrangement(window).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "Card 3", "News", "A story"]
        );
    });
}

/// Where the thing changed the order, the tabs that are still there are in the order they
/// were, a tab that came since is after them, and a tab that has gone since is not in the
/// row in any way (044's rule, as it was).
#[test]
fn a_tab_moved_is_back_where_it_was_and_a_tab_that_came_since_is_after_the_others() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, _, story]) = four_tabs(&mut shell).await;
        acked(
            &shell
                .move_tab(story, move_tab::To::By(RowStep::Back.into()))
                .await,
        );
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "Board", "A story", "News"]
        );
        acked(&shell.activate_tab(mail).await);
        link_beside_the_front(&mut shell, subscription, CARD).await;
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "Card 3", "Board", "A story", "News"]
        );
        // And a tab was closed.
        acked(&shell.close_tab(board).await);

        acked(&shell.undo_arrangement(window).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "News", "A story", "Card 3"]
        );
        // The row is those four, and no place is kept in it for the tab that has gone.
        assert_eq!(shell.window(subscription).row_tabs, 4);
    });
}
