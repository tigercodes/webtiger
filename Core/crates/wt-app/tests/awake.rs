//! What is awake, and "sleep all others" (056), through the contract, against
//! the scripted engine: what is listed and in what order, why each tab is
//! awake, what a private window keeps to itself, how many are asleep, and
//! what the command puts to sleep.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    AwakePage, AwakeTab, CreateSpace, KeepAwake, OpenTab, QueryAwake, SleepOthers, SubmitAddress,
    SwitchSpace, TabSummary, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

/// The address of the page a test calls by a number, and its title.
fn at(number: usize) -> String {
    format!("https://site{number}.example.com/")
}

fn named(number: usize) -> String {
    format!("Site {number}")
}

fn web(shell: &mut FakeShell) {
    for number in 1..=30 {
        shell.engine.page(&at(number), &named(number));
    }
}

fn fake() -> FakeShell {
    let mut shell = FakeShell::fake();
    web(&mut shell);
    shell
}

fn row(shell: &FakeShell, subscription: u64) -> Vec<TabSummary> {
    shell.window(subscription).tabs
}

/// Which tabs of the row are asleep, in the row's order.
fn asleep(shell: &FakeShell, subscription: u64) -> Vec<bool> {
    row(shell, subscription)
        .iter()
        .map(|tab| tab.asleep)
        .collect()
}

/// A window (one that is open already, or a new one) with its first tab at
/// the first of `pages` and a tab opened in front for each of the rest: the
/// last is in front. The window, its tabs in the row's order, and the
/// subscription.
async fn a_window_of(shell: &mut FakeShell, pages: &[usize]) -> (u64, Vec<u64>, u64) {
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, &at(pages[0])).await;
    let mut tabs = vec![first];
    for number in &pages[1..] {
        tabs.push(shell.open_tab(window, &at(*number)).await);
    }
    (window, tabs, subscription)
}

/// What is awake, as a window is told.
async fn awake(shell: &mut FakeShell, window: u64) -> AwakePage {
    let answer = shell
        .request(request::Kind::QueryAwake(QueryAwake { window_id: window }))
        .await;
    let response::Kind::AwakePage(page) = answer else {
        panic!("QueryAwake answers AwakePage, not {answer:?}");
    };
    page
}

/// The titles of the tabs listed, in order.
fn titles(page: &AwakePage) -> Vec<String> {
    page.tabs.iter().map(|tab| tab.title.clone()).collect()
}

fn names(numbers: &[usize]) -> Vec<String> {
    numbers.iter().map(|number| named(*number)).collect()
}

async fn sleep_others(shell: &mut FakeShell, window: u64) {
    let answer = shell
        .request(request::Kind::SleepOthers(SleepOthers {
            window_id: window,
        }))
        .await;
    assert!(
        matches!(answer, response::Kind::Ack(_)),
        "SleepOthers answers Ack, not {answer:?}"
    );
}

/// Makes a Space and shows it in a window: its id, and the tab the window
/// then has in front.
async fn a_space(shell: &mut FakeShell, window: u64, subscription: u64, name: &str) -> (u64, u64) {
    let made = shell
        .request(request::Kind::CreateSpace(CreateSpace {
            name: name.to_owned(),
            ..CreateSpace::default()
        }))
        .await;
    let response::Kind::SpaceCreated(space) = made else {
        panic!("expected a Space, got {made:?}");
    };
    show(shell, window, space.space_id).await;
    let front = shell.window(subscription).tab.expect("a tab").tab_id;
    (space.space_id, front)
}

async fn show(shell: &mut FakeShell, window: u64, space: u64) {
    shell
        .request(request::Kind::SwitchSpace(SwitchSpace {
            window_id: window,
            space_id: space,
        }))
        .await;
}

#[test]
fn every_tab_that_is_alive_is_listed_in_its_rows_order_with_why_it_is_awake() {
    run(async {
        let mut shell = fake();
        let (window, tabs, subscription) = a_window_of(&mut shell, &[1, 2, 3]).await;
        let space = shell.window(subscription).space_id;
        let views: Vec<u64> = shell.engine.web_views.keys().copied().collect();

        // Three alive, in the row's order; the third is the one in sight; none has a reason.
        let page = awake(&mut shell, window).await;
        let expected: Vec<AwakeTab> = (0..3)
            .map(|index| AwakeTab {
                tab_id: tabs[index],
                window_id: window,
                space_id: space,
                // The Space the app made itself has no name of its own: a shell has a word.
                space_name: String::new(),
                title: named(index + 1),
                host: format!("site{}.example.com", index + 1),
                in_sight: index == 2,
                keeps_awake: vec![],
                kept: false,
                memory_bytes: 0,
                private_window: false,
            })
            .collect();
        assert_eq!(page.tabs, expected);
        assert_eq!((page.private_tabs, page.asleep), (0, 0));

        // The first plays a sound and the second sets off somewhere: each says why. The
        // first is kept, too: a Site.
        shell.report(FakeEngine::page_activity(views[0], true, false, false));
        shell.report(FakeEngine::navigation_started(views[1], &at(9)));
        shell.settle().await;
        shell.keep_tab(tabs[0]).await;
        let page = awake(&mut shell, window).await;
        let why: Vec<(Vec<i32>, bool, bool)> = page
            .tabs
            .iter()
            .map(|tab| (tab.keeps_awake.clone(), tab.in_sight, tab.kept))
            .collect();
        assert_eq!(
            why,
            [
                (vec![KeepAwake::Sound as i32], false, true),
                (vec![KeepAwake::Loading as i32], false, false),
                (vec![], true, false),
            ]
        );

        // An empty tab comes in front: it has no web view, and is not listed. Nothing that
        // is listed is in sight now.
        shell
            .request(request::Kind::OpenTab(OpenTab {
                window_id: window,
                ..OpenTab::default()
            }))
            .await;
        let page = awake(&mut shell, window).await;
        assert_eq!(page.tabs.len(), 3);
        assert!(page.tabs.iter().all(|tab| !tab.in_sight));
        assert_eq!(page.asleep, 0, "an empty tab is not asleep");
    });
}

#[test]
fn sleep_all_others_takes_what_can_sleep_off_the_list_and_counts_it_asleep() {
    run(async {
        let mut shell = fake();
        let (window, tabs, subscription) = a_window_of(&mut shell, &[1, 2, 3, 4]).await;
        let views: Vec<u64> = shell.engine.web_views.keys().copied().collect();
        // The second was typed into: it is the person's.
        shell.report(FakeEngine::page_activity(views[1], false, false, true));
        shell.settle().await;

        sleep_others(&mut shell, window).await;
        assert_eq!(asleep(&shell, subscription), [true, false, true, false]);
        assert_eq!(shell.engine.web_views.len(), 2);
        let page = awake(&mut shell, window).await;
        assert_eq!(titles(&page), names(&[2, 4]));
        assert_eq!(page.asleep, 2);

        // Nothing lasts: the first is woken, the fourth is behind and awake, and stays so.
        shell.activate_tab(tabs[0]).await;
        shell.a_minute_passes().await;
        assert_eq!(asleep(&shell, subscription), [false, false, true, false]);
        let page = awake(&mut shell, window).await;
        assert_eq!(titles(&page), names(&[1, 2, 4]));
        assert_eq!(page.asleep, 1);
        assert_eq!(
            page.tabs.iter().map(|tab| tab.in_sight).collect::<Vec<_>>(),
            [true, false, false]
        );
        // Asked again, the command is done again.
        sleep_others(&mut shell, window).await;
        assert_eq!(asleep(&shell, subscription), [false, false, true, true]);
    });
}

#[test]
fn the_asking_window_is_first_then_its_shown_space_then_the_others_in_the_spaces_order() {
    run(async {
        let mut shell = fake();
        // A window whose first Space has pages 1 and 2; then a Space "Work" with page 3 and
        // a Space "Play" with page 4; and it shows Work again.
        let (first, _, first_seen) = a_window_of(&mut shell, &[1, 2]).await;
        let home = shell.window(first_seen).space_id;
        let (work, work_tab) = a_space(&mut shell, first, first_seen, "Work").await;
        shell.submit(work_tab, &at(3)).await;
        let (play, play_tab) = a_space(&mut shell, first, first_seen, "Play").await;
        shell.submit(play_tab, &at(4)).await;
        show(&mut shell, first, work).await;
        // And another window, with page 5.
        let (second, _, _) = a_window_of(&mut shell, &[5]).await;

        // Asked from the first: its own tabs first, the Space it shows before the others,
        // which follow in the Spaces' own order; then the other window's.
        let page = awake(&mut shell, first).await;
        assert_eq!(titles(&page), names(&[3, 1, 2, 4, 5]));
        let where_each_is: Vec<(u64, u64, &str, bool)> = page
            .tabs
            .iter()
            .map(|tab| {
                (
                    tab.window_id,
                    tab.space_id,
                    tab.space_name.as_str(),
                    tab.in_sight,
                )
            })
            .collect();
        assert_eq!(
            where_each_is,
            [
                (first, work, "Work", true),
                (first, home, "", false),
                // The tab in front of a row that is not shown is not in sight.
                (first, home, "", false),
                (first, play, "Play", false),
                (second, home, "", true),
            ]
        );

        // Asked from the second: its own first.
        let page = awake(&mut shell, second).await;
        assert_eq!(titles(&page), names(&[5, 3, 1, 2, 4]));

        // The first shows Play: Play is first, then the rest in the Spaces' order.
        show(&mut shell, first, play).await;
        let page = awake(&mut shell, first).await;
        assert_eq!(titles(&page), names(&[4, 1, 2, 3, 5]));
    });
}

#[test]
fn a_private_windows_tabs_are_named_to_that_window_alone_and_counted_for_the_others() {
    run(async {
        let mut shell = fake();
        let (ordinary, _, _) = a_window_of(&mut shell, &[1]).await;
        // A private window with pages 2 and 3, and another with page 4.
        let (private, tab, _) = shell.open_private_window().await;
        shell.submit(tab, &at(2)).await;
        shell.open_tab(private, &at(3)).await;
        let (other, tab, _) = shell.open_private_window().await;
        shell.submit(tab, &at(4)).await;

        // The ordinary window is told its own tab, and that three more are alive in
        // private windows: not their names, nor where they are.
        let page = awake(&mut shell, ordinary).await;
        assert_eq!(titles(&page), names(&[1]));
        assert_eq!(page.private_tabs, 3);
        assert!(!page.tabs[0].private_window);

        // A private window is told its own, marked as its own and with no Space's name, and
        // the ordinary window's; the other private window's is counted.
        let page = awake(&mut shell, private).await;
        assert_eq!(titles(&page), names(&[2, 3, 1]));
        assert_eq!(page.private_tabs, 1);
        let own: Vec<(bool, &str, u64)> = page
            .tabs
            .iter()
            .map(|tab| (tab.private_window, tab.space_name.as_str(), tab.window_id))
            .collect();
        assert_eq!(
            own,
            [
                (true, "", private),
                (true, "", private),
                (false, "", ordinary)
            ]
        );
        let page = awake(&mut shell, other).await;
        assert_eq!(titles(&page), names(&[4, 1]));
        assert_eq!(page.private_tabs, 2);
    });
}

#[test]
fn sleep_all_others_is_for_every_window_and_leaves_the_tabs_in_sight_and_a_private_windows() {
    run(async {
        let mut shell = fake();
        let (first, _, first_seen) = a_window_of(&mut shell, &[1, 2, 3]).await;
        let (second, _, second_seen) = a_window_of(&mut shell, &[4, 5]).await;
        let (private, tab, private_seen) = shell.open_private_window().await;
        shell.submit(tab, &at(6)).await;
        shell.open_tab(private, &at(7)).await;
        let states = shell.engine.states_asked.len();

        // Asked from the second window: the tabs behind in both ordinary windows sleep, each
        // asked for its state; the tab in front of each stays; the private window's stay.
        sleep_others(&mut shell, second).await;
        assert_eq!(asleep(&shell, first_seen), [true, true, false]);
        assert_eq!(asleep(&shell, second_seen), [true, false]);
        assert_eq!(asleep(&shell, private_seen), [false, false]);
        assert_eq!(shell.engine.states_asked.len(), states + 3);
        let page = awake(&mut shell, first).await;
        assert_eq!(titles(&page), names(&[3, 5]));
        assert_eq!((page.private_tabs, page.asleep), (2, 3));
    });
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn a_window_that_is_not_there_is_told_nothing_and_puts_nothing_to_sleep() {
    run(async {
        let mut shell = fake();
        let (window, _, subscription) = a_window_of(&mut shell, &[1, 2]).await;
        let nowhere = window + 1_000;
        let answer = shell
            .request(request::Kind::QueryAwake(QueryAwake { window_id: nowhere }))
            .await;
        assert_eq!(error_code(answer), "window.unknown");
        let answer = shell
            .request(request::Kind::SleepOthers(SleepOthers {
                window_id: nowhere,
            }))
            .await;
        assert_eq!(error_code(answer), "window.unknown");
        assert_eq!(asleep(&shell, subscription), [false, false]);
    });
}

#[test]
fn the_tabs_of_a_space_not_shown_since_the_app_started_are_counted_asleep() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        // The first Space has pages 1, 2 and 3, and an empty tab; "Work" has pages 4 and 5,
        // and is the one shown when the app is quit.
        let (window, _, subscription) = a_window_of(&mut shell, &[1, 2, 3]).await;
        shell
            .request(request::Kind::OpenTab(OpenTab {
                window_id: window,
                background: true,
                ..OpenTab::default()
            }))
            .await;
        let home = shell.window(subscription).space_id;
        let (_, work_tab) = a_space(&mut shell, window, subscription, "Work").await;
        shell.submit(work_tab, &at(4)).await;
        shell.open_tab(window, &at(5)).await;
        shell.settle().await;
        drop(shell);

        // Started again: the window shows Work, whose tab in front wakes. Its other tab is
        // asleep, and so are the first Space's three, which are nowhere but in the store
        // until that Space is shown. The empty tab there is not asleep: it has nothing.
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (window, _, _) = shell.open_window().await;
        let page = awake(&mut shell, window).await;
        assert_eq!(titles(&page), names(&[5]));
        assert_eq!(page.asleep, 4);

        // The first Space is shown: its tab in front wakes, and is listed first; the count
        // is of the tabs that are asleep still, each once.
        show(&mut shell, window, home).await;
        let page = awake(&mut shell, window).await;
        assert_eq!(titles(&page), names(&[3, 5]));
        assert_eq!(page.asleep, 3);
        assert_eq!(
            page.tabs
                .iter()
                .map(|tab| (tab.space_name.as_str(), tab.in_sight))
                .collect::<Vec<_>>(),
            [("", true), ("Work", false)]
        );
    });
}

#[test]
fn a_tab_on_its_way_that_the_person_sent_is_listed_by_where_it_is_going() {
    run(async {
        let mut shell = fake();
        let (window, tabs, _) = a_window_of(&mut shell, &[1, 2]).await;
        // The first, behind, is sent somewhere, with the shell slow to begin.
        shell.hold_host = true;
        shell.send(request::Kind::SubmitAddress(SubmitAddress {
            tab_id: tabs[0],
            text: at(9),
        }));
        shell.settle().await;
        let page = awake(&mut shell, window).await;
        assert_eq!(page.tabs[0].host, "site9.example.com");
        assert_eq!(page.tabs[0].keeps_awake, [KeepAwake::Loading as i32]);
        // Sleep all others leaves it: it is on its way.
        sleep_others(&mut shell, window).await;
        let page = awake(&mut shell, window).await;
        assert_eq!(page.tabs.len(), 2);
        assert_eq!(page.asleep, 0);
    });
}
