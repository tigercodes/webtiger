//! A tab sleeps by itself (053), through the contract, against the scripted
//! engine, with the minutes passed by the test: which tabs sleep and when,
//! what stops the count, what a sleeping tab keeps and how it wakes.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{
    CreateSpace, PermissionKind, SwitchSpace, TabSummary, TabView, host_request, request, response,
};
use wt_store::Store;
use wt_testkit::{FakeEngine, FakeShell, run};

const HOME: &str = "https://example.com/";
const NEWS: &str = "https://news.example.com/";
const MAIL: &str = "https://mail.example.com/";
const WIKI: &str = "https://wiki.example.com/";
const NOWHERE: &str = "https://nowhere.example.com/";

/// The minutes a tab is out of sight before it sleeps (053 D1).
const SLEEP_AFTER: usize = 30;

fn web(shell: &mut FakeShell) {
    shell.engine.page(HOME, "Home");
    shell.engine.page(NEWS, "News");
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(WIKI, "Wiki");
}

fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
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

async fn minutes(shell: &mut FakeShell, count: usize) {
    for _ in 0..count {
        shell.a_minute_passes().await;
    }
}

/// What the shell was asked of web views' pages, in order: a load, or a wake
/// from a state, each by its address; and a web view let go.
fn asked(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewLoad(load) => Some(format!("load {}", load.url)),
            host_request::Kind::WebViewRestore(restore) => Some(format!("restore {}", restore.url)),
            host_request::Kind::WebViewDestroy(destroy) => {
                Some(format!("destroy {}", destroy.web_view_id))
            }
            _ => None,
        })
        .collect()
}

/// A window with two tabs: Home behind, with a way back from News, and Mail
/// in front. The window, the two tabs, the subscription, and the first tab's
/// web view.
async fn two_tabs(shell: &mut FakeShell) -> (u64, u64, u64, u64, u64) {
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, NEWS).await;
    shell.submit(first, HOME).await;
    let first_view = front(shell, subscription).web_view_id;
    let second = shell.open_tab(window, MAIL).await;
    (window, first, second, subscription, first_view)
}

fn store(folder: &Path) -> Store {
    Store::open(&folder.join("webtiger.sqlite")).unwrap()
}

/// The states the store has, by the ids it gave them: the first ten.
fn kept(folder: &Path) -> Vec<i64> {
    let store = store(folder);
    (1..=10)
        .filter(|id| store.kept_state(*id).unwrap().is_some())
        .collect()
}

#[test]
fn a_tab_out_of_sight_for_half_an_hour_sleeps_and_the_tab_in_front_does_not() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, _, _, subscription, first_view) = two_tabs(&mut shell).await;
        assert_eq!(
            shell.engine.states_asked,
            [first_view],
            "as it left the front"
        );

        // A minute short of the time: both are awake, and nothing was asked.
        minutes(&mut shell, SLEEP_AFTER - 1).await;
        assert_eq!(asleep(&shell, subscription), [false, false]);
        assert_eq!(shell.engine.web_views.len(), 2);

        // The minute that makes it half an hour: the tab behind is asleep. Its state was
        // asked for once more, and then its web view let go, in that order; the tab is where
        // it was in its row, with the name and the address it had.
        minutes(&mut shell, 1).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(shell.engine.states_asked, [first_view, first_view]);
        assert_eq!(
            asked(&shell).last().unwrap(),
            &format!("destroy {first_view}")
        );
        assert!(!shell.engine.web_views.contains_key(&first_view));
        assert_eq!(shell.engine.web_views.len(), 1);
        let behind = &row(&shell, subscription)[0];
        assert_eq!(
            (behind.title.as_str(), behind.host.as_str()),
            ("Home", "example.com")
        );
        assert_eq!(behind.keeps_awake, [] as [i32; 0]);
        // What an engine says late of the web view that is gone is nothing of the tab's: it
        // has no page to be doing anything, and nobody is told.
        let told = shell.snapshots(subscription).len();
        shell.report(FakeEngine::page_activity(first_view, true, false, true));
        shell.report(FakeEngine::title(first_view, "Late"));
        shell.settle().await;
        assert_eq!(shell.snapshots(subscription).len(), told);
        assert_eq!(row(&shell, subscription)[0].title, "Home");

        // The tab in front is awake however long it is in front.
        minutes(&mut shell, 3 * SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(shell.engine.web_views.len(), 1);
        assert_eq!(front(&shell, subscription).title, "Mail");
    });
}

#[test]
fn a_tab_that_slept_wakes_where_it_was_with_its_way_back() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, first_view) = two_tabs(&mut shell).await;
        let before = shell.engine.web_views.get(&first_view).cloned().unwrap();
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);

        // Pressed, it wakes: a web view is made from its state, with its address to fall
        // back on, and it is where it was, with the list it had.
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {HOME}"));
        let woken = front(&shell, subscription);
        assert_eq!((woken.title.as_str(), woken.can_go_back), ("Home", true));
        let view = shell.engine.web_views.get(&woken.web_view_id).unwrap();
        assert_eq!((&view.history, view.index), (&before.history, before.index));
        assert_eq!(asleep(&shell, subscription), [false, false]);
        // And the tab that was in front is out of sight from now: its own half hour begins.
        minutes(&mut shell, SLEEP_AFTER - 1).await;
        assert_eq!(asleep(&shell, subscription), [false, false]);
        minutes(&mut shell, 1).await;
        assert_eq!(asleep(&shell, subscription), [false, true]);
    });
}

#[test]
fn a_tab_that_comes_in_front_starts_its_time_again() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, second, subscription, _) = two_tabs(&mut shell).await;
        // Twenty minutes behind; in front for a moment; behind again.
        minutes(&mut shell, 20).await;
        shell.activate_tab(first).await;
        shell.activate_tab(second).await;
        minutes(&mut shell, 20).await;
        assert_eq!(
            asleep(&shell, subscription),
            [false, false],
            "forty minutes in all, twenty since it was seen"
        );
        minutes(&mut shell, 10).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
    });
}

#[test]
fn a_tab_with_a_reason_to_be_awake_is_passed_over_and_starts_its_time_when_the_reason_ends() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, _, _, subscription, first_view) = two_tabs(&mut shell).await;
        // A third tab, in front. Twenty minutes on, the first begins to play a sound, and the
        // second is typed into (by a script of the engine's own telling, as far as this goes).
        let second_view = front(&shell, subscription).web_view_id;
        shell.open_tab(window, WIKI).await;
        minutes(&mut shell, 20).await;
        shell.report(FakeEngine::page_activity(first_view, true, false, false));
        shell.report(FakeEngine::page_activity(second_view, false, false, true));
        shell.settle().await;

        // Two hours: neither sleeps.
        minutes(&mut shell, 4 * SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [false, false, false]);

        // The song stops. Half an hour from then, and not a minute sooner, the tab sleeps: the
        // twenty minutes before the song are not counted. The one that was typed into is
        // awake for as long as its page is its page.
        shell.report(FakeEngine::page_activity(first_view, false, false, false));
        shell.settle().await;
        minutes(&mut shell, SLEEP_AFTER - 1).await;
        assert_eq!(asleep(&shell, subscription), [false, false, false]);
        minutes(&mut shell, 1).await;
        assert_eq!(asleep(&shell, subscription), [true, false, false]);
        minutes(&mut shell, 4 * SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false, false]);
    });
}

#[test]
fn a_question_and_a_camera_keep_a_tab_from_sleeping_too() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, _, _, subscription, first_view) = two_tabs(&mut shell).await;
        let second_view = front(&shell, subscription).web_view_id;
        shell.open_tab(window, WIKI).await;
        // The first asks for the camera, behind as it is: the question waits in it. The
        // second holds the microphone.
        shell.report(FakeEngine::permission_requested(
            first_view,
            1,
            "https://example.com",
            &[PermissionKind::Camera],
        ));
        shell.report(FakeEngine::page_activity(second_view, false, true, false));
        shell.settle().await;
        minutes(&mut shell, 2 * SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [false, false, false]);
    });
}

#[test]
fn the_tabs_of_a_space_no_window_shows_sleep_and_its_front_wakes_when_it_is_shown() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, _, _, subscription, _) = two_tabs(&mut shell).await;
        let home = shell.window(subscription).space_id;
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(work) = made else {
            panic!("expected a Space, got {made:?}");
        };
        let show = |space: u64| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: space,
            })
        };
        shell.request(show(work.space_id)).await;
        let work_tab = front(&shell, subscription).tab_id;
        shell.submit(work_tab, WIKI).await;
        assert_eq!(shell.engine.web_views.len(), 3);

        // Half an hour in the other Space: both tabs of the first are asleep, the one that
        // was in front of its row among them. The tab in sight is awake.
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(shell.engine.web_views.len(), 1);
        assert_eq!(asleep(&shell, subscription), [false]);

        // The first Space is shown again: its row is as it was, the tab that was in front in
        // front, and waking; the other asleep.
        shell.request(show(home)).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(front(&shell, subscription).title, "Mail");
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {MAIL}"));
    });
}

#[test]
fn a_site_sleeps_at_its_home_with_no_state_and_stays_where_it_is() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, _, subscription, first_view) = two_tabs(&mut shell).await;
        // The first is kept: a Site, whose home is Home. It wanders to News, and is left
        // behind there.
        shell.keep_tab(first).await;
        shell.activate_tab(first).await;
        shell.submit(first, NEWS).await;
        let second = row(&shell, subscription)[1].tab_id;
        shell.activate_tab(second).await;
        let asked_before = shell.engine.states_asked.len();

        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert!(!shell.engine.web_views.contains_key(&first_view));
        assert_eq!(
            shell.engine.states_asked.len(),
            asked_before,
            "a Site is asked for no state"
        );
        let site = &row(&shell, subscription)[0];
        assert!(site.kept);
        assert_eq!(
            site.host, "example.com",
            "at its home, not where it wandered"
        );
        // Woken, it is loaded at its home.
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("load {HOME}"));
        let _ = window;
    });
}

#[test]
fn a_private_windows_tabs_do_not_sleep() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        // An ordinary window with a tab behind, which sleeps; and a private one with a tab
        // behind, which does not: nothing of a private window is written, a state least of
        // all, so its tab would wake with nothing of where it was.
        let (_, _, _, subscription, _) = two_tabs(&mut shell).await;
        let (window, first, private) = shell.open_private_window().await;
        shell.submit(first, HOME).await;
        shell.open_tab(window, MAIL).await;
        let asked_before = shell.engine.states_asked.len();
        minutes(&mut shell, 3 * SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(asleep(&shell, private), [false, false]);
        assert_eq!(shell.engine.web_views.len(), 3);
        // The one state asked for since is the ordinary tab's, as it slept.
        assert_eq!(shell.engine.states_asked.len(), asked_before + 1);
    });
}

#[test]
fn a_tab_a_page_opened_that_has_nowhere_to_wake_is_left_as_it_is() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let opener = front(&shell, subscription).web_view_id;
        // The page asks for a window, which is a tab with a web view of its own and no page
        // yet: nothing has arrived in it. Another tab comes in front of both.
        let (opened, asks) = shell
            .engine
            .ask_for_window(opener, "https://example.com/popup", true);
        shell.report(asks);
        shell.settle().await;
        shell.open_tab(window, MAIL).await;
        assert_eq!(row(&shell, subscription).len(), 3);

        // Half an hour: the first sleeps. The tab the page opened shows no page of the
        // web's to wake to, and keeps its web view.
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false, false]);
        assert!(shell.engine.web_views.contains_key(&opened));
    });
}

#[test]
fn a_tab_whose_load_failed_sleeps_with_no_state_and_an_empty_tab_has_nothing_to_sleep() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        // Sent somewhere that does not load: it is shown as going there.
        shell.engine.fail(NOWHERE, "net.offline");
        shell.submit(first, NOWHERE).await;
        // An empty tab behind it too, and a tab in front of both.
        shell
            .request(request::Kind::OpenTab(wt_proto::v1::OpenTab {
                window_id: window,
                ..Default::default()
            }))
            .await;
        shell.open_tab(window, MAIL).await;
        let asked_before = shell.engine.states_asked.len();
        minutes(&mut shell, SLEEP_AFTER).await;

        // The failed tab sleeps, asked for no state: its web view was where it had been,
        // not where the tab is shown as going. The empty tab has no web view to let go.
        assert_eq!(asleep(&shell, subscription), [true, false, false]);
        assert_eq!(shell.engine.states_asked.len(), asked_before);
        // Woken, it goes where it was going.
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("load {NOWHERE}"));
    });
}

#[test]
fn a_state_that_arrives_after_the_tab_is_asleep_is_kept_and_one_for_a_tab_already_woken_is_not() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        // A shell that is given an hour to answer, and not half a minute: a state that is
        // slower than the core waits for a shell is no state, which is not what is tried here.
        let patient = std::time::Duration::from_secs(3_600);
        let mut shell =
            FakeShell::start_with(&FakeShell::live_config(folder.path()), patient).unwrap();
        web(&mut shell);
        let (_, first, second, subscription, first_view) = two_tabs(&mut shell).await;
        shell.settle().await;
        // The first tab's page goes on behind: the state from when it left the front (1) is
        // its no longer, and the one it sleeps with is the only one it will have.
        for report in shell.engine.goes_to(first_view, WIKI) {
            shell.report(report);
        }
        shell.settle().await;
        // The engine says at once from now, and its word is slow to arrive. The tab sleeps;
        // its state has not come, and it has none.
        shell.slow_states = true;
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        // The state arrives: it is the sleeping tab's.
        shell.release_states();
        shell.settle().await;
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {WIKI}"));
        shell.settle().await;

        // The second sleeps in its turn, its state slow again; and is woken before the state
        // has come. It wakes from the one it had from when it left the front; what arrives
        // after is of a web view that is gone, for a tab that is awake, and is not kept.
        shell.slow_states = true;
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [false, true]);
        shell.activate_tab(second).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {MAIL}"));
        shell.release_states();
        shell.settle().await;
        drop(shell);
        // In the store: the first tab's state from leaving the front as the second woke, and
        // nothing of the second's, which is in front.
        let states = kept(folder.path());
        assert_eq!(states.len(), 1, "{states:?}");
    });
}

#[test]
fn a_tab_that_slept_is_asleep_with_its_state_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, _, _, subscription, _) = two_tabs(&mut shell).await;
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        shell.settle().await;
        drop(shell);
        // One state: the one asked for as it slept (2), in the place of the one from when it
        // left the front (1).
        assert_eq!(kept(folder.path()), [2]);

        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        let first = row(&shell, subscription)[0].tab_id;
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {HOME}"));
        assert!(front(&shell, subscription).can_go_back);
    });
}

#[test]
fn a_suite_can_have_a_tab_sleep_in_seconds() {
    run(async {
        let mut config = FakeShell::config(wt_proto::v1::EngineKind::Webkit);
        config.fake = Some(wt_proto::v1::FakeOptions {
            sleep_after_seconds: 3,
            ..wt_proto::v1::FakeOptions::default()
        });
        let mut shell = FakeShell::start(&config).unwrap();
        web(&mut shell);
        let (_, _, _, subscription, _) = two_tabs(&mut shell).await;
        // Looked for every second: asleep after three, and not after two.
        for _ in 0..2 {
            tokio::time::sleep(std::time::Duration::from_millis(1_010)).await;
            shell.settle().await;
        }
        assert_eq!(asleep(&shell, subscription), [false, false]);
        tokio::time::sleep(std::time::Duration::from_millis(1_010)).await;
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
    });
}

#[test]
fn a_tab_on_its_way_is_not_put_to_sleep_until_it_has_arrived() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, _, _, subscription, first_view) = two_tabs(&mut shell).await;
        // The tab behind starts on its way somewhere by itself, and is a long time about it.
        shell.report(FakeEngine::navigation_started(first_view, WIKI));
        shell.settle().await;
        assert!(row(&shell, subscription)[0].loading);
        minutes(&mut shell, 2 * SLEEP_AFTER).await;
        assert_eq!(
            asleep(&shell, subscription),
            [false, false],
            "it is on its way"
        );

        // It arrives: half an hour from then, it sleeps.
        for report in shell.engine.goes_to(first_view, WIKI) {
            shell.report(report);
        }
        shell.settle().await;
        minutes(&mut shell, SLEEP_AFTER - 1).await;
        assert_eq!(asleep(&shell, subscription), [false, false]);
        minutes(&mut shell, 1).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(row(&shell, subscription)[0].title, "Wiki");
    });
}

#[test]
fn a_tab_whose_history_was_forgotten_sleeps_with_no_state_and_is_asked_again_once_it_has_woken() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, second, subscription, first_view) = two_tabs(&mut shell).await;
        assert_eq!(shell.engine.states_asked, [first_view]);
        // History is forgotten: the web views that are open hold lists of what was visited,
        // and are asked for no state (051 D6), not even to sleep.
        let forgotten = shell
            .request(request::Kind::ForgetHistory(wt_proto::v1::ForgetHistory {
                window_id: window,
                what: Some(wt_proto::v1::forget_history::What::Everything(true)),
            }))
            .await;
        assert!(matches!(forgotten, response::Kind::Ack(_)), "{forgotten:?}");
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(
            shell.engine.states_asked,
            [first_view],
            "asked for nothing more"
        );

        // Woken, it is loaded, in a web view that has seen nothing that was forgotten: and
        // that one is asked for its state as any is, when it leaves the front and to sleep.
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("load {HOME}"));
        let woken_view = front(&shell, subscription).web_view_id;
        shell.activate_tab(second).await;
        assert_eq!(shell.engine.states_asked, [first_view, woken_view]);
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(
            shell.engine.states_asked,
            [first_view, woken_view, woken_view]
        );
    });
}

#[test]
fn a_tab_whose_page_went_on_behind_is_asked_as_it_sleeps_before_its_web_view_is_let_go() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, first_view) = two_tabs(&mut shell).await;
        // Behind, the first tab's page goes on to another by itself: the state it had from
        // when it left the front is of a page it has left, and is its no longer (051).
        for report in shell.engine.goes_to(first_view, WIKI) {
            shell.report(report);
        }
        shell.settle().await;
        let before = shell.engine.web_views.get(&first_view).cloned().unwrap();
        assert_eq!(before.history.last().map(String::as_str), Some(WIKI));

        // It sleeps. The only state it can wake from is the one asked for now: asked while
        // the web view was still there to be asked.
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {WIKI}"));
        let woken = front(&shell, subscription);
        assert_eq!((woken.title.as_str(), woken.can_go_back), ("Wiki", true));
        let view = shell.engine.web_views.get(&woken.web_view_id).unwrap();
        assert_eq!((&view.history, view.index), (&before.history, before.index));
    });
}

#[test]
fn with_no_state_to_give_as_a_tab_sleeps_or_one_too_large_the_one_it_had_is_its_still() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, first, _, subscription, _) = two_tabs(&mut shell).await;
        shell.settle().await;
        // The first has a state from when it left the front (1). As it sleeps, the engine
        // has none to give: the one it had is of the same page, and is its still.
        shell.engine.gives_no_state = true;
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        shell.settle().await;
        shell.engine.gives_no_state = false;
        // Woken, it is made from that one.
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {HOME}"));
        shell.settle().await;

        // The second tab left the front as the first woke, with a state of the usual size
        // (2). As it sleeps the engine gives one just over the limit of 128 KB, which is not
        // kept: the one it had is its still.
        shell.engine.state_padding = 132_000;
        minutes(&mut shell, SLEEP_AFTER).await;
        assert_eq!(asleep(&shell, subscription), [false, true]);
        shell.settle().await;
        drop(shell);
        assert_eq!(kept(folder.path()), [2]);
    });
}
