//! Tidying temporary tabs, and the Archive they go to (049), through the
//! contract, against the scripted engine, on a clock the test moves: which
//! tabs are tidied and when, which never are, what the Archive lists and how
//! a tab is brought back from it or forgotten, and what a quit keeps.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use wt_core::{Clock, ManualClock, Timestamp};
use wt_proto::v1::{CreateSpace, QuerySpaces, SpacesPage, SwitchSpace, request, response};
use wt_testkit::{FakeShell, run};

const HOME: &str = "https://example.com/";
const NEWS: &str = "https://news.example.com/";
const MAIL: &str = "https://mail.example.com/";
const BOARD: &str = "https://board.example.com/";
const WIKI: &str = "https://wiki.example.com/";
const FORUM: &str = "https://forum.example.com/";

const HOUR: i64 = 3_600_000;
const HALF_A_DAY: u32 = 43_200;

fn web(shell: &mut FakeShell) {
    shell.engine.page(HOME, "Home");
    shell.engine.page(NEWS, "News");
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(BOARD, "Board");
    shell.engine.page(WIKI, "Wiki");
    shell.engine.page(FORUM, "Forum");
}

fn clock() -> Rc<ManualClock> {
    Rc::new(ManualClock::new(Timestamp(1_700_000_000_000)))
}

fn shell_at(clock: &Rc<ManualClock>) -> FakeShell {
    let mut shell = FakeShell::fake_at(Rc::clone(clock) as Rc<dyn Clock>);
    web(&mut shell);
    shell
}

/// The names of a window's tabs, in the row's order.
fn row(shell: &FakeShell, subscription: u64) -> Vec<String> {
    shell
        .window(subscription)
        .tabs
        .iter()
        .map(|tab| tab.title.clone())
        .collect()
}

fn acked(answer: &response::Kind) {
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

fn code(answer: &response::Kind) -> (String, String) {
    match answer {
        response::Kind::Error(error) => (error.code.clone(), error.field.clone()),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn said(code: &str, field: &str) -> (String, String) {
    (code.to_owned(), field.to_owned())
}

/// The names the Archive of the Space a window shows lists, newest first.
async fn archived(shell: &mut FakeShell, window: u64) -> Vec<String> {
    shell
        .archive(window, 0, 0)
        .await
        .0
        .into_iter()
        .map(|entry| entry.title)
        .collect()
}

async fn spaces(shell: &mut FakeShell) -> SpacesPage {
    match shell
        .request(request::Kind::QuerySpaces(QuerySpaces {}))
        .await
    {
        response::Kind::SpacesPage(page) => page,
        other => panic!("QuerySpaces answers SpacesPage, not {other:?}"),
    }
}

/// A window with three tabs, "Home", "News" and "Mail", the last in front.
async fn three_tabs(shell: &mut FakeShell) -> (u64, u64, u64) {
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, HOME).await;
    shell.open_tab(window, NEWS).await;
    shell.open_tab(window, MAIL).await;
    assert_eq!(row(shell, subscription), ["Home", "News", "Mail"]);
    (window, first, subscription)
}

#[test]
fn a_tab_that_has_not_been_in_front_for_its_spaces_time_is_tidied_into_the_archive() {
    run(async {
        let clock = clock();
        let mut shell = shell_at(&clock);
        let (window, _, subscription) = three_tabs(&mut shell).await;
        let space = shell.window(subscription).space_id;
        assert_eq!(shell.engine.web_views.len(), 3);

        // The Space is given twelve hours: the tabs' times start now.
        acked(&shell.set_tidying(space, HALF_A_DAY).await);
        assert_eq!(row(&shell, subscription), ["Home", "News", "Mail"]);

        // A minute short of twelve hours: nothing has gone.
        clock.advance_millis(12 * HOUR - 60_000);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Home", "News", "Mail"]);
        assert_eq!(archived(&mut shell, window).await, [] as [&str; 0]);

        // Twelve hours: the two that were not in front are tidied. The window is told, as
        // when a tab is closed; their pages are let go; and the one in front stays.
        clock.advance_millis(60_000);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Mail"]);
        assert_eq!(
            shell.engine.web_views.len(),
            1,
            "the tidied tabs' web views went"
        );
        let (entries, more) = shell.archive(window, 0, 0).await;
        assert!(!more);
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.title.as_str(), entry.url.as_str()))
                .collect::<Vec<_>>(),
            [("News", NEWS), ("Home", HOME)],
            "newest first: the row's order, the last of it first"
        );
        // Each with its address as the address field would show it.
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.display.as_str())
                .collect::<Vec<_>>(),
            ["news.example.com", "example.com"]
        );
        assert!(entries.iter().all(|entry| entry.tidied_at == clock.now().0));
        assert!(entries[0].id > entries[1].id);

        // The tab in front is never tidied, however long it has been there.
        clock.advance_millis(30 * 24 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Mail"]);
        assert_eq!(archived(&mut shell, window).await.len(), 2);
    });
}

#[test]
fn a_tabs_time_starts_again_when_it_has_been_in_front() {
    run(async {
        let clock = clock();
        let mut shell = shell_at(&clock);
        let (_, home, subscription) = three_tabs(&mut shell).await;
        let space = shell.window(subscription).space_id;
        let mail = shell.window(subscription).tabs[2].tab_id;
        acked(&shell.set_tidying(space, HALF_A_DAY).await);

        // Eleven hours on, "Home" is brought in front: it is looked at. "Mail" has left the
        // front, and its time starts from then.
        clock.advance_millis(11 * HOUR);
        acked(&shell.activate_tab(home).await);
        shell.a_minute_passes().await;
        // Two hours more: "News" has been out of sight for thirteen, and goes. "Mail" has
        // for two; "Home" is in front.
        clock.advance_millis(2 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Home", "Mail"]);

        // "Mail" in front again: now "Home" has left the front.
        acked(&shell.activate_tab(mail).await);
        shell.a_minute_passes().await;
        clock.advance_millis(11 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(
            row(&shell, subscription),
            ["Home", "Mail"],
            "eleven hours since it was in front"
        );
        clock.advance_millis(2 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Mail"], "and thirteen");
    });
}

#[test]
fn what_is_kept_is_never_tidied_nor_an_empty_tab_nor_a_private_windows() {
    run(async {
        let clock = clock();
        let mut shell = shell_at(&clock);
        let (window, first, subscription) = shell.open_window().await;
        // A Site, a Favourite, a Group's Site, an empty tab, a tab, and the one in front.
        shell.submit(first, HOME).await;
        acked(&shell.keep_tab(first).await);
        let favourite = shell.open_tab(window, NEWS).await;
        acked(&shell.set_favourite(favourite, true).await);
        let grouped = shell.open_tab(window, BOARD).await;
        let made = shell.make_group(window, "Work").await;
        let response::Kind::GroupMade(group) = made else {
            panic!("expected a Group, got {made:?}");
        };
        acked(&shell.move_to_group(grouped, group.group_id).await);
        shell.open_tab(window, "").await;
        shell.open_tab(window, WIKI).await;
        shell.open_tab(window, FORUM).await;
        let before = row(&shell, subscription);
        assert_eq!(before.len(), 6);
        // A private window, with a tab behind the one in front.
        let (private_window, private_first, private_subscription) =
            shell.open_private_window().await;
        shell.submit(private_first, MAIL).await;
        shell.open_tab(private_window, HOME).await;
        let private_space = shell.window(private_subscription).space_id;

        let space = shell.window(subscription).space_id;
        acked(&shell.set_tidying(space, HALF_A_DAY).await);
        clock.advance_millis(13 * HOUR);
        shell.a_minute_passes().await;

        // Only the tab that is no Site, has a page and is not in front has gone.
        let after = row(&shell, subscription);
        assert_eq!(after.len(), 5);
        assert!(!after.contains(&"Wiki".to_owned()), "{after:?}");
        assert_eq!(archived(&mut shell, window).await, ["Wiki"]);
        // Nothing of a private window is tidied, or written: it has no Archive, and its
        // row is no Space to be given a time.
        assert_eq!(row(&shell, private_subscription), ["Mail", "Home"]);
        assert_eq!(
            code(&shell.set_tidying(private_space, HALF_A_DAY).await),
            said("space.unknown", "space_id")
        );
        assert_eq!(
            shell.archive(private_window, 0, 0).await,
            (Vec::new(), false)
        );
        clock.advance_millis(30 * 24 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, private_subscription), ["Mail", "Home"]);
        assert_eq!(
            row(&shell, subscription).len(),
            5,
            "and what is kept is kept"
        );
    });
}

#[test]
fn a_space_has_no_time_until_it_is_given_one_of_the_times_there_are() {
    run(async {
        let clock = clock();
        let mut shell = shell_at(&clock);
        let (window, _, subscription) = three_tabs(&mut shell).await;
        let space = shell.window(subscription).space_id;

        // The times a Space can have, never first; and a Space is made with none.
        let page = spaces(&mut shell).await;
        assert_eq!(page.tidy_times, [0, 43_200, 86_400, 604_800, 2_592_000]);
        assert_eq!(page.spaces[0].tidy_after_seconds, 0);
        // With no time, nothing is tidied, however long.
        clock.advance_millis(60 * 24 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription).len(), 3);

        // A time that is none of them, and a Space there is not, are refused, and nothing
        // is changed.
        assert_eq!(
            code(&shell.set_tidying(space, 3_600).await),
            said("tidy.time_unknown", "after_seconds")
        );
        assert_eq!(
            code(&shell.set_tidying(space + 1000, HALF_A_DAY).await),
            said("space.unknown", "space_id")
        );
        assert_eq!(spaces(&mut shell).await.spaces[0].tidy_after_seconds, 0);

        // A day: the Space says so. A second Space has none still, and its tabs stay
        // while the first's go.
        acked(&shell.set_tidying(space, 86_400).await);
        assert_eq!(
            spaces(&mut shell).await.spaces[0].tidy_after_seconds,
            86_400
        );
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(work) = made else {
            panic!("expected a Space, got {made:?}");
        };
        let switch = |space_id| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id,
            })
        };
        acked(&shell.request(switch(work.space_id)).await);
        let work_first = shell.window(subscription).tab.unwrap().tab_id;
        shell.submit(work_first, BOARD).await;
        shell.open_tab(window, WIKI).await;
        assert_eq!(row(&shell, subscription), ["Board", "Wiki"]);
        // (Looked at once, so that a tab's time would have begun, were its Space tidied.)
        shell.a_minute_passes().await;
        clock.advance_millis(25 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(
            row(&shell, subscription),
            ["Board", "Wiki"],
            "a Space with no time"
        );
        assert_eq!(archived(&mut shell, window).await, [] as [&str; 0]);
        // The first Space's tabs went while it was out of sight; the one that was in front
        // of its row is there when the Space is shown again.
        acked(&shell.request(switch(space)).await);
        assert_eq!(row(&shell, subscription), ["Mail"]);
        assert_eq!(archived(&mut shell, window).await, ["News", "Home"]);

        // Never again: what is there stays, and the Space says so.
        shell.open_tab(window, FORUM).await;
        shell.open_tab(window, HOME).await;
        acked(&shell.set_tidying(space, 0).await);
        assert_eq!(spaces(&mut shell).await.spaces[0].tidy_after_seconds, 0);
        clock.advance_millis(60 * 24 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Mail", "Forum", "Home"]);
    });
}

#[test]
fn a_tab_is_brought_back_from_the_archive_or_forgotten() {
    run(async {
        let clock = clock();
        let mut shell = shell_at(&clock);
        let (window, _, subscription) = three_tabs(&mut shell).await;
        shell.open_tab(window, BOARD).await;
        shell.open_tab(window, WIKI).await;
        let space = shell.window(subscription).space_id;
        acked(&shell.set_tidying(space, HALF_A_DAY).await);
        clock.advance_millis(13 * HOUR);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Wiki"]);
        assert_eq!(
            archived(&mut shell, window).await,
            ["Board", "Mail", "News", "Home"]
        );

        // A page at a time: two, then the two older, and then no more.
        let (first, more) = shell.archive(window, 0, 2).await;
        assert_eq!(
            first.iter().map(|e| e.title.as_str()).collect::<Vec<_>>(),
            ["Board", "Mail"]
        );
        assert!(more);
        let (second, more) = shell.archive(window, first[1].id, 2).await;
        assert_eq!(
            second.iter().map(|e| e.title.as_str()).collect::<Vec<_>>(),
            ["News", "Home"]
        );
        assert!(!more);

        // Brought back: a tab again, the last of the row, in front, at its address; and the
        // Archive lists it no longer. It cannot be brought back twice.
        let mail = first[1].id;
        let answer = shell.restore_archived(window, mail).await;
        let response::Kind::TabOpened(opened) = answer else {
            panic!("expected a tab, got {answer:?}");
        };
        assert_eq!(row(&shell, subscription), ["Wiki", "Mail"]);
        let front = shell.window(subscription).tab.unwrap();
        assert_eq!(front.tab_id, opened.tab_id);
        assert_eq!(front.address.unwrap().url, MAIL);
        assert_eq!(
            archived(&mut shell, window).await,
            ["Board", "News", "Home"]
        );
        assert_eq!(
            code(&shell.restore_archived(window, mail).await),
            said("archive.unknown", "entry_id")
        );
        assert_eq!(row(&shell, subscription), ["Wiki", "Mail"]);

        // Forgotten: one, and it is gone; again, and nothing is; then all of them.
        acked(&shell.forget_archived(window, Some(second[0].id)).await);
        assert_eq!(archived(&mut shell, window).await, ["Board", "Home"]);
        acked(&shell.forget_archived(window, Some(second[0].id)).await);
        assert_eq!(archived(&mut shell, window).await, ["Board", "Home"]);
        acked(&shell.forget_archived(window, None).await);
        assert_eq!(archived(&mut shell, window).await, [] as [&str; 0]);

        // A window there is not has no Archive to ask, bring back from or forget.
        let nowhere = window + 1000;
        let asked = shell
            .request(request::Kind::QueryArchive(wt_proto::v1::QueryArchive {
                window_id: nowhere,
                ..Default::default()
            }))
            .await;
        assert_eq!(code(&asked), said("window.unknown", "window_id"));
        assert_eq!(
            code(&shell.restore_archived(nowhere, 1).await),
            said("window.unknown", "window_id")
        );
        assert_eq!(
            code(&shell.forget_archived(nowhere, None).await),
            said("window.unknown", "window_id")
        );
        // A private window has none: nothing is brought back there, and nothing forgotten.
        let (private_window, _, _) = shell.open_private_window().await;
        assert_eq!(
            code(&shell.restore_archived(private_window, 1).await),
            said("archive.unknown", "entry_id")
        );
        acked(&shell.forget_archived(private_window, None).await);
    });
}

#[test]
fn a_spaces_archive_is_its_own() {
    run(async {
        let clock = clock();
        let mut shell = shell_at(&clock);
        let (window, _, subscription) = three_tabs(&mut shell).await;
        let space = shell.window(subscription).space_id;
        acked(&shell.set_tidying(space, HALF_A_DAY).await);
        clock.advance_millis(13 * HOUR);
        shell.a_minute_passes().await;
        let (entries, _) = shell.archive(window, 0, 0).await;
        assert_eq!(entries.len(), 2);

        // Another Space, shown in the window: its Archive is empty, and the first Space's
        // tabs are not its to bring back or to forget.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(work) = made else {
            panic!("expected a Space, got {made:?}");
        };
        acked(
            &shell
                .request(request::Kind::SwitchSpace(SwitchSpace {
                    window_id: window,
                    space_id: work.space_id,
                }))
                .await,
        );
        assert_eq!(archived(&mut shell, window).await, [] as [&str; 0]);
        assert_eq!(
            code(&shell.restore_archived(window, entries[0].id).await),
            said("archive.unknown", "entry_id")
        );
        acked(&shell.forget_archived(window, None).await);
        // Back in the first Space, they are all there.
        acked(
            &shell
                .request(request::Kind::SwitchSpace(SwitchSpace {
                    window_id: window,
                    space_id: space,
                }))
                .await,
        );
        assert_eq!(archived(&mut shell, window).await, ["News", "Home"]);
    });
}

#[test]
fn a_quit_does_not_start_a_tabs_time_again_and_what_is_past_it_is_tidied_before_a_window_is_told() {
    run(async {
        let clock = clock();
        let folder = tempfile::tempdir().unwrap();
        let start = |clock: &Rc<ManualClock>| {
            let mut shell = FakeShell::start_at(
                &FakeShell::live_config(folder.path()),
                Rc::clone(clock) as Rc<dyn Clock>,
            )
            .unwrap();
            web(&mut shell);
            shell
        };

        // The first run: three tabs, and the Space given twelve hours.
        let mut shell = start(&clock);
        let (_, _, subscription) = three_tabs(&mut shell).await;
        let space = shell.window(subscription).space_id;
        acked(&shell.set_tidying(space, HALF_A_DAY).await);
        shell.settle().await;
        drop(shell);

        // Six hours on, the app starts again: the Space has its time still, and the tabs
        // are all there.
        clock.advance_millis(6 * HOUR);
        let mut shell = start(&clock);
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(row(&shell, subscription), ["Home", "News", "Mail"]);
        assert_eq!(
            spaces(&mut shell).await.spaces[0].tidy_after_seconds,
            HALF_A_DAY
        );
        shell.settle().await;
        drop(shell);

        // Seven more, and it starts a third time: thirteen hours since the tabs' time
        // began, which the two quits did not begin again. The window is first told of a row
        // without them: they are in the Archive, and never showed.
        clock.advance_millis(7 * HOUR);
        let mut shell = start(&clock);
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(
            shell
                .snapshots(subscription)
                .iter()
                .map(|window| window.tabs.len())
                .collect::<Vec<_>>()
                .first(),
            Some(&1),
            "the first the window is told"
        );
        assert_eq!(row(&shell, subscription), ["Mail"]);
        assert_eq!(archived(&mut shell, window).await, ["News", "Home"]);
    });
}

/// A clock that counts how often it is read.
struct Counting {
    clock: ManualClock,
    read: AtomicU32,
}

impl Clock for Counting {
    fn now(&self) -> Timestamp {
        self.read.fetch_add(1, Ordering::SeqCst);
        self.clock.now()
    }
}

#[test]
fn the_clock_is_looked_at_once_a_minute_and_only_while_some_space_is_tidied() {
    run(async {
        let counting = Rc::new(Counting {
            clock: ManualClock::new(Timestamp(1_700_000_000_000)),
            read: AtomicU32::new(0),
        });
        let mut shell = FakeShell::fake_at(Rc::clone(&counting) as Rc<dyn Clock>);
        web(&mut shell);
        let (_, _, subscription) = three_tabs(&mut shell).await;
        let space = shell.window(subscription).space_id;
        shell.settle().await;

        // With no Space tidied, ten minutes pass and nothing reads the clock.
        let before = counting.read.load(Ordering::SeqCst);
        for _ in 0..10 {
            shell.a_minute_passes().await;
        }
        assert_eq!(counting.read.load(Ordering::SeqCst), before);

        // With one: once when it is given its time, and once a minute after.
        acked(&shell.set_tidying(space, HALF_A_DAY).await);
        let given = counting.read.load(Ordering::SeqCst);
        assert_eq!(given, before + 1);
        for _ in 0..10 {
            shell.a_minute_passes().await;
        }
        assert_eq!(counting.read.load(Ordering::SeqCst), given + 10);

        // Never again: the look stops, and the clock is left alone.
        acked(&shell.set_tidying(space, 0).await);
        shell.a_minute_passes().await;
        let stopped = counting.read.load(Ordering::SeqCst);
        for _ in 0..10 {
            shell.a_minute_passes().await;
        }
        assert_eq!(counting.read.load(Ordering::SeqCst), stopped);
    });
}

#[test]
fn in_fake_mode_a_suite_can_have_a_day_pass_each_time_the_clock_is_read() {
    run(async {
        // Fake mode as it is: a second a read. A minute on, nothing is near its time.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, _, subscription) = three_tabs(&mut shell).await;
        let space = shell.window(subscription).space_id;
        acked(&shell.set_tidying(space, HALF_A_DAY).await);
        shell.a_minute_passes().await;
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Home", "News", "Mail"]);
        assert_eq!(archived(&mut shell, window).await, [] as [&str; 0]);

        // With a day a read, the look after the Space is given its time finds the tabs a
        // day older: the rule is the same one, and only the clock is quicker.
        let mut config = FakeShell::config(wt_proto::v1::EngineKind::Webkit);
        config.fake = Some(wt_proto::v1::FakeOptions {
            seconds_a_read: 86_400,
            ..Default::default()
        });
        let mut shell = FakeShell::start(&config).unwrap();
        web(&mut shell);
        let (window, _, subscription) = three_tabs(&mut shell).await;
        let space = shell.window(subscription).space_id;
        acked(&shell.set_tidying(space, HALF_A_DAY).await);
        assert_eq!(row(&shell, subscription), ["Home", "News", "Mail"]);
        shell.a_minute_passes().await;
        assert_eq!(row(&shell, subscription), ["Mail"]);
        let (entries, _) = shell.archive(window, 0, 0).await;
        assert_eq!(entries.len(), 2);
        // Whole days after fake mode's "now".
        assert!(
            entries
                .iter()
                .all(|entry| entry.tidied_at > 1_767_268_800_000
                    && (entry.tidied_at - 1_767_268_800_000) % 86_400_000 == 0)
        );
    });
}
