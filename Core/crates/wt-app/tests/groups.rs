//! Groups (041): Sites in folders, one level deep. Through the contract,
//! against the scripted engine: making and naming a Group, putting a tab in
//! one and taking it out, where things sit in the row, what a shut Group
//! hides, what removing one leaves, and what comes back after a quit.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::collections::BTreeMap;
use std::path::Path;

use wt_proto::v1::{CreateSpace, StepTab, SwitchSpace, host_request, request, response, step_tab};
use wt_store::{SavedGroup, Store};
use wt_testkit::{FakeShell, run};

const MAIL: &str = "https://mail.example.com/";
const BOARD: &str = "https://board.example.com/";
const CARD: &str = "https://board.example.com/card/3";
const NEWS: &str = "https://news.example.com/";
const STORY: &str = "https://news.example.com/story";

fn web(shell: &mut FakeShell) {
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(BOARD, "Board");
    shell.engine.page(CARD, "Card 3");
    shell.engine.page(NEWS, "News");
    shell.engine.page(STORY, "A story");
}

/// The tabs a window is told to draw, in order: each by its name, `S` when
/// it is a Site, `z` asleep, and `@` the name of the Group it is in.
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
                "{name}{}{}{group}",
                if tab.kept { " S" } else { "" },
                if tab.asleep { " z" } else { "" },
            )
        })
        .collect()
}

/// The Groups a window is told of, in order: each by its name, its colour
/// when it has one, `shut` when it is, and how many Sites it holds.
fn groups(shell: &FakeShell, subscription: u64) -> Vec<String> {
    shell
        .window(subscription)
        .groups
        .into_iter()
        .map(|group| {
            format!(
                "{}{}{}{} ({})",
                group.name,
                if group.colour.is_empty() { "" } else { " " },
                group.colour,
                if group.shut { " shut" } else { "" },
                group.sites
            )
        })
        .collect()
}

/// The tab in front, as `row` says a tab.
fn front(shell: &FakeShell, subscription: u64) -> String {
    let tab = shell.window(subscription).tab.expect("a tab");
    if tab.title.is_empty() {
        "(empty)".to_owned()
    } else {
        tab.title
    }
}

/// What the shell was asked to load and how many web views to destroy:
/// what no change to a Group may add to.
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

/// Makes a Group in the row a window shows: its id.
async fn group(shell: &mut FakeShell, window: u64, name: &str) -> u64 {
    match shell.make_group(window, name).await {
        response::Kind::GroupMade(made) => made.group_id,
        other => panic!("expected a Group, got {other:?}"),
    }
}

async fn step(shell: &mut FakeShell, window: u64, to: step_tab::To, place: u32) {
    let answer = shell
        .request(request::Kind::StepTab(StepTab {
            window_id: window,
            to: to.into(),
            place,
        }))
        .await;
    acked(&answer);
}

/// A window with four tabs, Mail, Board, News and A story, the last in
/// front, and two Groups with nothing in them: Work and House.
async fn four_tabs(shell: &mut FakeShell) -> (u64, u64, [u64; 4], [u64; 2]) {
    web(shell);
    let (window, mail, subscription) = shell.open_window().await;
    shell.submit(mail, MAIL).await;
    let board = shell.open_tab(window, BOARD).await;
    let news = shell.open_tab(window, NEWS).await;
    let story = shell.open_tab(window, STORY).await;
    let work = group(shell, window, "Work").await;
    let house = group(shell, window, "House").await;
    (
        window,
        subscription,
        [mail, board, news, story],
        [work, house],
    )
}

#[test]
fn a_group_is_made_in_the_row_a_window_shows_and_named_and_coloured() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        assert_eq!(groups(&shell, subscription), [] as [&str; 0]);
        let before = work_done(&shell);

        // Made, a Group is after the Groups there are, with its name tidied as a Space's
        // is, no colour, open, and nothing in it. It changes no tab.
        let work = group(&mut shell, window, "Work").await;
        let house = group(&mut shell, window, "  The \n house  ").await;
        assert_ne!(work, house);
        assert_eq!(groups(&shell, subscription), ["Work (0)", "The house (0)"]);
        assert_eq!(row(&shell, subscription), ["Mail"]);

        // Two Groups may have one name: they are two Groups.
        let twin = group(&mut shell, window, "Work").await;
        assert_eq!(
            groups(&shell, subscription),
            ["Work (0)", "The house (0)", "Work (0)"]
        );
        acked(&shell.remove_group(twin).await);

        // Named again; and given a colour, another, and none.
        acked(&shell.rename_group(house, " House ").await);
        acked(&shell.colour_group(work, "blue").await);
        assert_eq!(groups(&shell, subscription), ["Work blue (0)", "House (0)"]);
        acked(&shell.colour_group(work, "green").await);
        acked(&shell.colour_group(house, "grey").await);
        assert_eq!(
            groups(&shell, subscription),
            ["Work green (0)", "House grey (0)"]
        );
        acked(&shell.colour_group(work, "").await);
        assert_eq!(groups(&shell, subscription), ["Work (0)", "House grey (0)"]);

        // What is refused, each by its reason, and none of it changes anything.
        assert_eq!(
            code(shell.make_group(window, " \t ").await),
            "group.name_empty"
        );
        assert_eq!(
            code(shell.make_group(window + 100, "Play").await),
            "window.unknown"
        );
        assert_eq!(code(shell.rename_group(work, "").await), "group.name_empty");
        assert_eq!(
            code(shell.rename_group(house + 100, "Play").await),
            "group.unknown"
        );
        assert_eq!(
            code(shell.colour_group(work, "mauve").await),
            "group.colour_unknown"
        );
        assert_eq!(
            code(shell.colour_group(work, "Blue").await),
            "group.colour_unknown"
        );
        assert_eq!(
            code(shell.colour_group(house + 100, "blue").await),
            "group.unknown"
        );
        assert_eq!(
            code(shell.shut_group(house + 100, true).await),
            "group.unknown"
        );
        assert_eq!(code(shell.remove_group(house + 100).await), "group.unknown");
        // A Group's id is no tab's and no window's.
        assert_eq!(
            code(shell.rename_group(mail, "Play").await),
            "group.unknown"
        );
        assert_eq!(code(shell.remove_group(window).await), "group.unknown");
        assert_eq!(groups(&shell, subscription), ["Work (0)", "House grey (0)"]);

        // A private window has no Groups: nothing of one is kept.
        let (private, _, private_subscription) = shell.open_private_window().await;
        assert_eq!(
            code(shell.make_group(private, "Play").await),
            "window.private"
        );
        assert_eq!(groups(&shell, private_subscription), [] as [&str; 0]);

        // None of it loaded a page or destroyed a web view.
        assert_eq!(work_done(&shell), before);
    });
}

#[test]
fn a_row_has_no_more_than_its_most_groups() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription) = shell.open_window().await;
        for number in 0..64 {
            group(&mut shell, window, &format!("Group {number}")).await;
        }
        assert_eq!(
            code(shell.make_group(window, "One more").await),
            "group.too_many"
        );
        assert_eq!(shell.window(subscription).groups.len(), 64);
        // With one taken away there is room for one.
        let first = shell.window(subscription).groups[0].group_id;
        acked(&shell.remove_group(first).await);
        group(&mut shell, window, "One more").await;
        assert_eq!(shell.window(subscription).groups.len(), 64);
    });
}

#[test]
fn a_tab_put_in_a_group_is_kept_and_the_row_is_sites_then_groups_then_tabs() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story], [work, house]) =
            four_tabs(&mut shell).await;
        assert_eq!(
            row(&shell, subscription),
            ["Mail", "Board", "News", "A story"]
        );
        let before = work_done(&shell);

        // Put in a Group, a tab that is no Site is kept: a Site, of that Group, and
        // before the tabs that are no Sites. It stays in front.
        acked(&shell.move_to_group(story, house).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story S @House", "Mail", "Board", "News"]
        );
        assert_eq!(front(&shell, subscription), "A story");
        assert_eq!(groups(&shell, subscription), ["Work (0)", "House (1)"]);

        // A Site that is in no Group is before every Group's.
        acked(&shell.keep_tab(mail).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "A story S @House", "Board", "News"]
        );

        // The Groups' Sites are Group by Group in the Groups' order, whichever was
        // filled first; and a tab put in a Group is at its end.
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.move_to_group(news, work).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "Mail S",
                "Board S @Work",
                "News S @Work",
                "A story S @House"
            ]
        );
        assert_eq!(groups(&shell, subscription), ["Work (2)", "House (1)"]);

        // A Site is put in a Group as a tab is, and from one Group into another: it is
        // in one Group or in none.
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, house).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "News S @Work",
                "Mail S @Work",
                "A story S @House",
                "Board S @House"
            ]
        );
        assert_eq!(groups(&shell, subscription), ["Work (2)", "House (2)"]);
        // Put in the Group it is in, it goes to that Group's end.
        acked(&shell.move_to_group(news, work).await);
        assert_eq!(
            row(&shell, subscription)[..2],
            ["Mail S @Work", "News S @Work"]
        );

        // Taken out, a Site is still a Site, after the Sites that are in no Group.
        acked(&shell.move_to_group(story, 0).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "A story S",
                "Mail S @Work",
                "News S @Work",
                "Board S @House"
            ]
        );
        acked(&shell.move_to_group(mail, 0).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story S", "Mail S", "News S @Work", "Board S @House"]
        );
        assert_eq!(groups(&shell, subscription), ["Work (1)", "House (1)"]);
        // Taken out of no Group, a Site is as it was; and a tab that is no Site is not
        // kept by it.
        acked(&shell.move_to_group(story, 0).await);
        let plain = shell.open_tab(window, CARD).await;
        acked(&shell.move_to_group(plain, 0).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "A story S",
                "Mail S",
                "News S @Work",
                "Board S @House",
                "Card 3"
            ]
        );

        // But for the one tab opened since, none of it loaded a page or destroyed a web
        // view: the tabs are the tabs they were.
        assert_eq!(work_done(&shell), (before.0 + 1, before.1));
    });
}

#[test]
fn what_cannot_be_put_in_a_group() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, _, _, story], [work, _]) = four_tabs(&mut shell).await;
        let as_it_was = row(&shell, subscription);

        // A tab there is not; a Group there is not; a tab with no page to keep.
        assert_eq!(
            code(shell.move_to_group(story + 100, work).await),
            "tab.unknown"
        );
        assert_eq!(
            code(shell.move_to_group(story + 100, 0).await),
            "tab.unknown"
        );
        assert_eq!(
            code(shell.move_to_group(mail, work + 100).await),
            "group.unknown"
        );
        let empty = shell.open_tab(window, "").await;
        assert_eq!(code(shell.move_to_group(empty, work).await), "tab.no_page");
        acked(&shell.close_tab(empty).await);

        // A Group of another row: another Space's, in the same window.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Play".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(play) = made else {
            panic!("expected a Space, got {made:?}");
        };
        let switch = |space: u64| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: space,
            })
        };
        let home = shell.window(subscription).space_id;
        acked(&shell.request(switch(play.space_id)).await);
        let theirs = group(&mut shell, window, "Games").await;
        assert_eq!(
            code(shell.move_to_group(mail, theirs).await),
            "group.unknown"
        );
        acked(&shell.request(switch(home)).await);
        assert_eq!(
            code(shell.move_to_group(mail, theirs).await),
            "group.unknown"
        );

        // And another window's: its row is its own, though the Space is the same.
        let (other, other_tab, other_subscription) = shell.open_window().await;
        shell.submit(other_tab, NEWS).await;
        assert_eq!(groups(&shell, other_subscription), [] as [&str; 0]);
        assert_eq!(
            code(shell.move_to_group(other_tab, work).await),
            "group.unknown"
        );
        let other_group = group(&mut shell, other, "Reading").await;
        assert_eq!(
            code(shell.move_to_group(mail, other_group).await),
            "group.unknown"
        );

        // A private window's tab is in no Group, whichever is named.
        let (_, private, private_subscription) = shell.open_private_window().await;
        shell.submit(private, BOARD).await;
        assert_eq!(
            code(shell.move_to_group(private, work).await),
            "window.private"
        );
        acked(&shell.move_to_group(private, 0).await);
        assert_eq!(row(&shell, private_subscription), ["Board"]);

        // None of it kept a tab, or put one anywhere.
        assert_eq!(row(&shell, subscription), as_it_was);
        assert_eq!(groups(&shell, subscription), ["Work (0)", "House (0)"]);
        assert_eq!(row(&shell, other_subscription), ["News"]);
    });
}

#[test]
fn a_shut_group_hides_its_sites_but_for_the_one_in_front_and_changes_none_of_them() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story], [work, house]) =
            four_tabs(&mut shell).await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.move_to_group(news, house).await);
        let open = ["Mail S @Work", "Board S @Work", "News S @House", "A story"];
        assert_eq!(row(&shell, subscription), open);
        let before = work_done(&shell);

        // Shut, a Group's Sites are not among the tabs the window is told to draw. The
        // Group is, and says how many it holds.
        acked(&shell.shut_group(work, true).await);
        assert_eq!(row(&shell, subscription), ["News S @House", "A story"]);
        assert_eq!(groups(&shell, subscription), ["Work shut (2)", "House (1)"]);
        // Shut again it is as it was; opened, its Sites are where they were, awake as
        // they were.
        acked(&shell.shut_group(work, true).await);
        assert_eq!(row(&shell, subscription), ["News S @House", "A story"]);
        acked(&shell.shut_group(work, false).await);
        acked(&shell.shut_group(work, false).await);
        assert_eq!(row(&shell, subscription), open);
        assert_eq!(groups(&shell, subscription), ["Work (2)", "House (1)"]);

        // A Site that is in front is shown though its Group is shut: the tab in front
        // always is. The other of its Group is not.
        acked(&shell.activate_tab(board).await);
        acked(&shell.shut_group(work, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board S @Work", "News S @House", "A story"]
        );
        assert_eq!(front(&shell, subscription), "Board");
        // When another tab is in front it is out of the way with the rest.
        acked(&shell.activate_tab(story).await);
        assert_eq!(row(&shell, subscription), ["News S @House", "A story"]);

        // A tab of a shut Group that is brought in front some other way, as one chosen
        // while typing is, is shown; and the Group stays shut.
        acked(&shell.choose_tab(mail).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S @Work", "News S @House", "A story"]
        );
        assert_eq!(groups(&shell, subscription), ["Work shut (2)", "House (1)"]);

        // An empty Group can be shut, and a tab put in a Group that is shut is out of
        // the way at once, unless it is in front.
        acked(&shell.shut_group(house, true).await);
        assert_eq!(row(&shell, subscription), ["Mail S @Work", "A story"]);
        acked(&shell.move_to_group(story, house).await);
        assert_eq!(row(&shell, subscription), ["Mail S @Work"]);
        assert_eq!(
            groups(&shell, subscription),
            ["Work shut (2)", "House shut (2)"]
        );

        // Shutting and opening changed no tab: none was loaded, none destroyed, none
        // put to sleep.
        assert_eq!(work_done(&shell), before);
        acked(&shell.shut_group(work, false).await);
        acked(&shell.shut_group(house, false).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "Mail S @Work",
                "Board S @Work",
                "News S @House",
                "A story S @House"
            ]
        );
        let _ = window;
    });
}

#[test]
fn a_step_along_the_row_does_not_go_into_a_shut_group() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story], [work, _]) =
            four_tabs(&mut shell).await;
        // Mail is a Site in no Group; Board and News are in Work; A story is a tab.
        acked(&shell.keep_tab(mail).await);
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.move_to_group(news, work).await);
        acked(&shell.shut_group(work, true).await);
        assert_eq!(row(&shell, subscription), ["Mail S", "A story"]);
        assert_eq!(front(&shell, subscription), "A story");

        // The next and the one before are among the tabs that are shown, round the ends.
        step(&mut shell, window, step_tab::To::Previous, 0).await;
        assert_eq!(front(&shell, subscription), "Mail");
        step(&mut shell, window, step_tab::To::Next, 0).await;
        assert_eq!(front(&shell, subscription), "A story");
        step(&mut shell, window, step_tab::To::Next, 0).await;
        assert_eq!(front(&shell, subscription), "Mail");
        // A place is counted among them too: the second is the tab, not a hidden Site.
        step(&mut shell, window, step_tab::To::Place, 2).await;
        assert_eq!(front(&shell, subscription), "A story");

        // From a Site of the shut Group that is in front, a step goes to its neighbours
        // among what is shown, and it is then out of the way again.
        acked(&shell.choose_tab(news).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "News S @Work", "A story"]
        );
        step(&mut shell, window, step_tab::To::Next, 0).await;
        assert_eq!(front(&shell, subscription), "A story");
        assert_eq!(row(&shell, subscription), ["Mail S", "A story"]);

        // Open again, a step goes through the Group.
        acked(&shell.shut_group(work, false).await);
        step(&mut shell, window, step_tab::To::Previous, 0).await;
        assert_eq!(front(&shell, subscription), "News");
        let _ = story;
    });
}

#[test]
fn closing_a_tab_brings_no_site_out_of_a_shut_group() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, [mail, board, news, story], [work, _]) = four_tabs(&mut shell).await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.shut_group(work, true).await);
        assert_eq!(row(&shell, subscription), ["News", "A story"]);
        let before = work_done(&shell);

        // The tab in front is closed: the one that comes forward is one that is shown.
        acked(&shell.close_tab(story).await);
        assert_eq!(front(&shell, subscription), "News");
        // And with no tab shown at all, an empty tab is opened: the Sites of the shut
        // Group stay out of the way, awake as they were.
        acked(&shell.close_tab(news).await);
        assert_eq!(row(&shell, subscription), ["(empty)"]);
        assert_eq!(front(&shell, subscription), "(empty)");
        assert_eq!(groups(&shell, subscription), ["Work shut (2)", "House (0)"]);
        assert_eq!(work_done(&shell).0, before.0, "nothing was loaded");

        // A Site of the shut Group that is in front and is closed goes to sleep, as a
        // Site does, and is out of the way; what comes forward is a tab that is shown.
        acked(&shell.activate_tab(board).await);
        assert_eq!(row(&shell, subscription), ["Board S @Work", "(empty)"]);
        acked(&shell.close_tab(board).await);
        assert_eq!(row(&shell, subscription), ["(empty)"]);
        assert_eq!(front(&shell, subscription), "(empty)");
        // And when no tab is shown but the Site itself (chosen while typing in the empty
        // tab, which that closes, 038), it is not its neighbour in the Group that comes
        // forward, awake though that is: an empty tab is opened.
        acked(&shell.choose_tab(board).await);
        assert_eq!(row(&shell, subscription), ["Board S @Work"]);
        acked(&shell.close_tab(board).await);
        assert_eq!(row(&shell, subscription), ["(empty)"]);
        assert_eq!(front(&shell, subscription), "(empty)");
        // Opened, the Group has both: the one that was closed is asleep.
        acked(&shell.shut_group(work, false).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S @Work", "Board S z @Work", "(empty)"]
        );
    });
}

#[test]
fn removing_a_group_leaves_its_sites_in_no_group() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news, story], [work, house]) =
            four_tabs(&mut shell).await;
        let card = shell.open_tab(window, CARD).await;
        acked(&shell.keep_tab(mail).await);
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.move_to_group(news, work).await);
        acked(&shell.move_to_group(story, house).await);
        acked(&shell.shut_group(work, true).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "A story S @House", "Card 3"]
        );
        let before = work_done(&shell);

        // The Group is gone; its Sites are Sites still, after those that were in no
        // Group, in the order they had; and being in no shut Group, they are shown.
        acked(&shell.remove_group(work).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "News S", "A story S @House", "Card 3"]
        );
        assert_eq!(groups(&shell, subscription), ["House (1)"]);
        assert_eq!(work_done(&shell), before, "no tab was closed or loaded");
        // It is gone: nothing can be done with it, or put in it.
        assert_eq!(code(shell.remove_group(work).await), "group.unknown");
        assert_eq!(code(shell.move_to_group(card, work).await), "group.unknown");

        // A Group with nothing in it is removed as any other.
        acked(&shell.move_to_group(story, 0).await);
        acked(&shell.remove_group(house).await);
        assert_eq!(groups(&shell, subscription), [] as [&str; 0]);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S", "News S", "A story S", "Card 3"]
        );
    });
}

#[test]
fn a_site_let_go_is_in_no_group_and_one_asleep_is_in_the_group_it_was_in() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, [mail, board, news, _], [work, _]) = four_tabs(&mut shell).await;
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, work).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S @Work", "Board S @Work", "News", "A story"]
        );

        // Closed, a Site of a Group sleeps where it is, in its Group; woken, the same.
        acked(&shell.activate_tab(board).await);
        acked(&shell.close_tab(board).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S @Work", "Board S z @Work", "News", "A story"]
        );
        acked(&shell.activate_tab(board).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S @Work", "Board S @Work", "News", "A story"]
        );

        // Let go, a Site is a tab like any other: in no Group, the first of those that
        // are no Sites. The Group holds one fewer.
        acked(&shell.let_go_site(mail).await);
        assert_eq!(
            row(&shell, subscription),
            ["Board S @Work", "Mail", "News", "A story"]
        );
        assert_eq!(groups(&shell, subscription), ["Work (1)", "House (0)"]);
        // Kept again it is a Site in no Group: it does not go back by itself.
        acked(&shell.keep_tab(mail).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S @Work", "News", "A story"]
        );
        let _ = news;
    });
}

#[test]
fn each_row_has_its_own_groups() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, _, _, _], [work, _]) = four_tabs(&mut shell).await;
        acked(&shell.move_to_group(mail, work).await);
        let here = row(&shell, subscription);
        let home = shell.window(subscription).space_id;

        // Another Space in the window: its row has no Groups, and one made there is its
        // own.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Play".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(play) = made else {
            panic!("expected a Space, got {made:?}");
        };
        let switch = |space: u64| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: space,
            })
        };
        acked(&shell.request(switch(play.space_id)).await);
        assert_eq!(groups(&shell, subscription), [] as [&str; 0]);
        let games = group(&mut shell, window, "Games").await;
        assert_eq!(groups(&shell, subscription), ["Games (0)"]);

        // Back in the first, its Groups are as they were, and a Group of the row that
        // is not shown can still be named and shut.
        acked(&shell.request(switch(home)).await);
        assert_eq!(groups(&shell, subscription), ["Work (1)", "House (0)"]);
        assert_eq!(row(&shell, subscription), here);
        acked(&shell.rename_group(games, "Games and such").await);
        acked(&shell.shut_group(games, true).await);
        assert_eq!(groups(&shell, subscription), ["Work (1)", "House (0)"]);
        acked(&shell.request(switch(play.space_id)).await);
        assert_eq!(groups(&shell, subscription), ["Games and such shut (0)"]);
    });
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

#[test]
fn groups_come_back_after_a_quit_with_what_is_in_them_and_whether_they_were_shut() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let board = shell.open_tab(window, BOARD).await;
        let news = shell.open_tab(window, NEWS).await;
        let story = shell.open_tab(window, STORY).await;
        let work = group(&mut shell, window, "Work").await;
        let empty = group(&mut shell, window, "Nothing yet").await;
        let house = group(&mut shell, window, "House").await;
        acked(&shell.colour_group(work, "teal").await);
        acked(&shell.keep_tab(mail).await);
        acked(&shell.move_to_group(news, house).await);
        acked(&shell.move_to_group(board, house).await);
        acked(&shell.move_to_group(story, work).await);
        acked(&shell.shut_group(house, true).await);
        // A story, in Work, is in front at the quit.
        assert_eq!(row(&shell, subscription), ["Mail S", "A story S @Work"]);
        let kept = ["Work teal (1)", "Nothing yet (0)", "House shut (2)"];
        assert_eq!(groups(&shell, subscription), kept);
        let _ = empty;
        drop(shell);

        // They are back: their names, colours and order, the one with nothing in it,
        // the one that was shut still shut, and each Site in the Group it was in.
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(groups(&shell, subscription), kept);
        assert_eq!(row(&shell, subscription), ["Mail S z", "A story S @Work"]);
        let again = shell.window(subscription).groups;
        let (work, nothing, house) = (again[0].group_id, again[1].group_id, again[2].group_id);
        acked(&shell.shut_group(house, false).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "Mail S z",
                "A story S @Work",
                "News S z @House",
                "Board S z @House"
            ]
        );
        // Opening a Group woke nothing: the one page loaded is the one in front.
        let loads = |shell: &FakeShell| work_done(shell).0;
        assert_eq!(loads(&shell), 1);

        // Changed and quit again: a Group removed, one renamed, a Site moved, and a
        // Site of a shut Group in front.
        acked(&shell.remove_group(nothing).await);
        acked(&shell.rename_group(work, "Office").await);
        acked(&shell.colour_group(work, "").await);
        let ids: Vec<u64> = shell
            .window(subscription)
            .tabs
            .iter()
            .map(|tab| tab.tab_id)
            .collect();
        acked(&shell.move_to_group(ids[0], house).await);
        acked(&shell.shut_group(house, true).await);
        acked(&shell.choose_tab(ids[2]).await);
        assert_eq!(
            row(&shell, subscription),
            ["A story S @Office", "News S @House"]
        );
        let _ = window;
        drop(shell);

        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(
            groups(&shell, subscription),
            ["Office (1)", "House shut (3)"]
        );
        // The Site that was in front is in front, shown though its Group is shut.
        assert_eq!(
            row(&shell, subscription),
            ["A story S z @Office", "News S @House"]
        );
        assert_eq!(front(&shell, subscription), "News");
        let house = shell.window(subscription).groups[1].group_id;
        acked(&shell.shut_group(house, false).await);
        assert_eq!(
            row(&shell, subscription),
            [
                "A story S z @Office",
                "News S @House",
                "Board S z @House",
                "Mail S z @House"
            ]
        );
        drop(shell);

        // Quit with nothing changed, they are as they were.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(groups(&shell, subscription), ["Office (1)", "House (3)"]);
        assert_eq!(
            row(&shell, subscription),
            [
                "A story S z @Office",
                "News S @House",
                "Board S z @House",
                "Mail S z @House"
            ]
        );
    });
}

#[test]
fn each_spaces_groups_come_back_with_it() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_to_group(mail, work).await);
        let home = shell.window(subscription).space_id;
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Play".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(play) = made else {
            panic!("expected a Space, got {made:?}");
        };
        let switch = |window: u64, space: u64| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: space,
            })
        };
        acked(&shell.request(switch(window, play.space_id)).await);
        let front = shell.window(subscription).tab.expect("a tab").tab_id;
        shell.submit(front, BOARD).await;
        let games = group(&mut shell, window, "Games").await;
        acked(&shell.move_to_group(front, games).await);
        acked(&shell.colour_group(games, "pink").await);
        drop(shell);

        // The window opens in the Space it showed, with that Space's Groups; the other
        // Space has its own, and neither has the other's.
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(groups(&shell, subscription), ["Games pink (1)"]);
        assert_eq!(row(&shell, subscription), ["Board S @Games"]);
        acked(&shell.request(switch(window, home)).await);
        assert_eq!(groups(&shell, subscription), ["Work (1)"]);
        assert_eq!(row(&shell, subscription), ["Mail S @Work"]);
    });
}

#[test]
fn only_the_window_with_the_session_has_its_groups() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let work = group(&mut shell, window, "Work").await;
        acked(&shell.move_to_group(mail, work).await);

        // A second window's row is its own: it has none of the first's Groups, as it
        // has none of its tabs.
        let (_, _, second) = shell.open_window().await;
        assert_eq!(groups(&shell, second), [] as [&str; 0]);
        assert_eq!(row(&shell, second), ["(empty)"]);
        assert_eq!(groups(&shell, subscription), ["Work (1)"]);
        drop(shell);

        // After a quit the first window opened has the session, and its Groups; the
        // next has neither.
        let mut shell = start(folder.path());
        let (_, _, first) = shell.open_window().await;
        let (_, _, second) = shell.open_window().await;
        assert_eq!(groups(&shell, first), ["Work (1)"]);
        assert_eq!(row(&shell, first), ["Mail S @Work"]);
        assert_eq!(groups(&shell, second), [] as [&str; 0]);
        assert_eq!(row(&shell, second), ["(empty)"]);
    });
}

#[test]
fn a_site_whose_group_the_store_has_lost_is_a_site_in_no_group() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let board = shell.open_tab(window, BOARD).await;
        let news = shell.open_tab(window, NEWS).await;
        shell.open_tab(window, STORY).await;
        let work = group(&mut shell, window, "Work").await;
        let house = group(&mut shell, window, "House").await;
        acked(&shell.keep_tab(mail).await);
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.move_to_group(news, house).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S @Work", "News S @House", "A story"]
        );
        drop(shell);

        // The store loses the second Group, as when its Groups were written and its
        // tabs were not: a Site still says it is in the second.
        let mut store = Store::open(&folder.path().join("webtiger.sqlite")).unwrap();
        let space = store.spaces().unwrap()[0].id;
        assert_eq!(store.saved_tabs(space).unwrap()[2].group, 2);
        store
            .save_groups(
                space,
                &[SavedGroup {
                    name: "Work".to_owned(),
                    colour: String::new(),
                    shut: false,
                }],
            )
            .unwrap();
        drop(store);

        // That Site is a Site, in no Group, and the row is in a row's order: it is with
        // the Sites that are in none, before the Group's.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(groups(&shell, subscription), ["Work (1)"]);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S z", "News S z", "Board S z @Work", "A story"]
        );
    });
}

#[test]
fn a_window_is_told_how_many_tabs_its_row_has_those_out_of_the_way_among_them() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, [mail, board, _, story], [work, _]) = four_tabs(&mut shell).await;
        let told = |shell: &FakeShell| {
            let window = shell.window(subscription);
            (window.tabs.len(), window.row_tabs)
        };
        assert_eq!(told(&shell), (4, 4));
        // Two of them in a Group, and the Group shut: two are shown, and the row has four.
        acked(&shell.move_to_group(mail, work).await);
        acked(&shell.move_to_group(board, work).await);
        acked(&shell.shut_group(work, true).await);
        assert_eq!(told(&shell), (2, 4));
        // A Site that is closed sleeps, and is a tab of the row still; a tab that is closed
        // is one fewer.
        acked(&shell.shut_group(work, false).await);
        acked(&shell.activate_tab(mail).await);
        acked(&shell.close_tab(mail).await);
        assert_eq!(told(&shell), (4, 4));
        acked(&shell.close_tab(story).await);
        assert_eq!(told(&shell), (3, 3));
        // A window with one tab has one.
        let (_, _, other) = shell.open_window().await;
        assert_eq!(shell.window(other).row_tabs, 1);
    });
}
