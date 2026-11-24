//! When the system is short of memory (055), through the contract, against
//! the scripted engine: who sleeps when a shell says so, at each of the two
//! levels, who stays, how each wakes, and that nothing lasts.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::time::Duration;

use wt_proto::v1::{
    CreateSpace, EngineKind, Envelope, FakeOptions, KeepAwake, MemoryLevel, MemoryShort, OpenTab,
    PermissionKind, Request, SubmitAddress, SwitchSpace, TabSummary, TabView, envelope,
    host_request, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const SHORT: i32 = MemoryLevel::Short as i32;
const VERY_SHORT: i32 = MemoryLevel::VeryShort as i32;

/// The address of the page a test calls by a number, and its title.
fn at(number: usize) -> String {
    format!("https://site{number}.example.com/")
}

fn named(number: usize) -> String {
    format!("Site {number}")
}

fn web(shell: &mut FakeShell) {
    for number in 1..=20 {
        shell.engine.page(&at(number), &named(number));
    }
}

fn fake() -> FakeShell {
    let mut shell = FakeShell::fake();
    web(&mut shell);
    shell
}

/// A shell in fake mode with a suite's knobs.
fn with(options: FakeOptions) -> FakeShell {
    let mut config = FakeShell::config(EngineKind::Webkit);
    config.fake = Some(options);
    let mut shell = FakeShell::start(&config).unwrap();
    web(&mut shell);
    shell
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

/// A window whose first tab is at page 1 and whose others, opened in front
/// one after another, are at pages 2 to `count`: the last is in front. The
/// window, its tabs in the row's order with each one's web view, and the
/// subscription.
async fn a_row_of(shell: &mut FakeShell, count: usize) -> (u64, Vec<(u64, u64)>, u64) {
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, &at(1)).await;
    let mut tabs = vec![(first, front(shell, subscription).web_view_id)];
    for number in 2..=count {
        let tab = shell.open_tab(window, &at(number)).await;
        tabs.push((tab, front(shell, subscription).web_view_id));
    }
    (window, tabs, subscription)
}

/// The shell says the system is short of memory, so short, and is answered.
async fn told(shell: &mut FakeShell, level: i32) {
    let answer = shell
        .request(request::Kind::MemoryShort(MemoryShort { level }))
        .await;
    assert!(
        matches!(answer, response::Kind::Ack(_)),
        "MemoryShort answers Ack, not {answer:?}"
    );
}

#[test]
fn short_of_memory_every_tab_out_of_sight_sleeps_at_once_and_the_tab_in_sight_stays() {
    run(async {
        let mut shell = fake();
        let (_, tabs, subscription) = a_row_of(&mut shell, 4).await;
        let views: Vec<u64> = tabs.iter().map(|(_, view)| *view).collect();
        // Each of the first three was asked for its state as it left the front.
        assert_eq!(shell.engine.states_asked, views[..3]);
        assert_eq!(asleep(&shell, subscription), [false; 4]);
        let told_before = shell.snapshots(subscription).len();

        // No minute has passed. The system says it is short: the three behind are asleep, each
        // asked for its state once more and its web view let go; the one in sight is awake.
        told(&mut shell, SHORT).await;
        assert_eq!(asleep(&shell, subscription), [true, true, true, false]);
        assert_eq!(
            shell.engine.states_asked[3..],
            views[..3],
            "each asked once more"
        );
        let let_go: Vec<String> = views[..3]
            .iter()
            .map(|view| format!("destroy {view}"))
            .collect();
        assert_eq!(asked(&shell)[asked(&shell).len() - 3..], let_go);
        assert_eq!(
            shell.engine.web_views.keys().copied().collect::<Vec<_>>(),
            [views[3]]
        );
        // All three went together: nobody was shown a row with one or two of them asleep.
        let seen: Vec<usize> = shell.snapshots(subscription)[told_before..]
            .iter()
            .map(|window| window.tabs.iter().filter(|tab| tab.asleep).count())
            .collect();
        assert!(seen.iter().all(|count| *count == 3), "{seen:?}");
        // Each is where it was, with the name and the address it had, and no reason.
        for (number, tab) in row(&shell, subscription)[..3].iter().enumerate() {
            assert_eq!(tab.title, named(number + 1));
            assert_eq!(tab.host, format!("site{}.example.com", number + 1));
            assert_eq!(tab.keeps_awake, [] as [i32; 0]);
        }
        assert_eq!(front(&shell, subscription).title, named(4));

        // The first is woken, from its state, where it was.
        shell.activate_tab(tabs[0].0).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {}", at(1)));
        assert_eq!(front(&shell, subscription).title, named(1));
        // Nothing lasts: the tab it was woken over is behind now and awake, and stays so a
        // minute on. The report was acted on once.
        assert_eq!(asleep(&shell, subscription), [false, true, true, false]);
        shell.a_minute_passes().await;
        assert_eq!(asleep(&shell, subscription), [false, true, true, false]);
        // Said again, it is acted on again.
        told(&mut shell, SHORT).await;
        assert_eq!(asleep(&shell, subscription), [false, true, true, true]);
    });
}

#[test]
fn with_nothing_out_of_sight_a_report_is_answered_and_changes_nothing() {
    run(async {
        let mut shell = fake();
        let (_, tab, subscription) = shell.open_window().await;
        // An empty tab alone, and then a tab with a page alone.
        told(&mut shell, VERY_SHORT).await;
        shell.submit(tab, &at(1)).await;
        let told_before = shell.snapshots(subscription).len();
        told(&mut shell, SHORT).await;
        told(&mut shell, VERY_SHORT).await;
        assert_eq!(asleep(&shell, subscription), [false]);
        assert_eq!(shell.engine.web_views.len(), 1);
        assert_eq!(
            shell.snapshots(subscription).len(),
            told_before,
            "nobody is told"
        );
    });
}

#[test]
fn a_report_sent_as_a_notification_is_acted_on_and_gets_no_answer() {
    run(async {
        let mut shell = fake();
        let (_, _, subscription) = a_row_of(&mut shell, 2).await;
        let responses = shell
            .order
            .iter()
            .filter(|kind| **kind == "response")
            .count();
        shell.send_raw(Envelope {
            id: 0,
            body: Some(envelope::Body::Request(Request {
                kind: Some(request::Kind::MemoryShort(MemoryShort { level: SHORT })),
            })),
        });
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(
            shell
                .order
                .iter()
                .filter(|kind| **kind == "response")
                .count(),
            responses
        );
    });
}

/// A tab behind is sent somewhere, with the shell slow to begin: it is on its
/// way, shown as going there. (The shell holds what it is asked from now.)
async fn sent_on_its_way(shell: &mut FakeShell, tab: u64, number: usize) {
    shell.hold_host = true;
    shell.send(request::Kind::SubmitAddress(SubmitAddress {
        tab_id: tab,
        text: at(number),
    }));
    shell.settle().await;
}

#[test]
fn a_level_that_is_not_said_or_not_known_is_taken_for_short() {
    run(async {
        for level in [0, 3, 99, -1] {
            let mut shell = fake();
            let (_, tabs, subscription) = a_row_of(&mut shell, 3).await;
            sent_on_its_way(&mut shell, tabs[0].0, 9).await;
            // The tab at rest sleeps, as when short; the one on its way does not, as it would
            // were the system very short.
            told(&mut shell, level).await;
            assert_eq!(
                asleep(&shell, subscription),
                [false, true, false],
                "level {level}"
            );
        }
    });
}

#[test]
fn a_tab_on_its_way_sleeps_only_when_the_system_is_very_short_and_wakes_where_it_was_going() {
    run(async {
        let mut shell = fake();
        let (_, tabs, subscription) = a_row_of(&mut shell, 2).await;
        let (first, first_view) = tabs[0];
        sent_on_its_way(&mut shell, first, 9).await;
        assert!(row(&shell, subscription)[0].loading);
        assert_eq!(
            row(&shell, subscription)[0].keeps_awake,
            [KeepAwake::Loading as i32]
        );

        // Short: it is on its way, and stays.
        told(&mut shell, SHORT).await;
        assert_eq!(asleep(&shell, subscription), [false, false]);

        // Very short: it sleeps, asked for no state (its web view is where it was, not where
        // the tab is shown as going), and says where it was going.
        let states = shell.engine.states_asked.len();
        told(&mut shell, VERY_SHORT).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(shell.engine.states_asked.len(), states);
        assert_eq!(row(&shell, subscription)[0].host, "site9.example.com");
        assert!(!row(&shell, subscription)[0].loading);

        // The shell gets round to what it was asked: the load, in a web view that is then let
        // go. What the engine says of it is nothing of the tab's.
        shell.release_host();
        shell.settle().await;
        assert!(!shell.engine.web_views.contains_key(&first_view));
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(row(&shell, subscription)[0].host, "site9.example.com");

        // Woken, it goes where it was going.
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("load {}", at(9)));
        assert_eq!(front(&shell, subscription).title, named(9));
    });
}

#[test]
fn a_tab_whose_page_is_going_somewhere_by_itself_sleeps_when_very_short_and_wakes_where_it_was() {
    run(async {
        let mut shell = fake();
        let (_, tabs, subscription) = a_row_of(&mut shell, 2).await;
        let (first, first_view) = tabs[0];
        // The page behind starts for another by itself. Where it is going is not the tab's
        // to say until it arrives: the tab says the page it is at.
        shell.report(FakeEngine::navigation_started(first_view, &at(9)));
        shell.settle().await;
        assert!(row(&shell, subscription)[0].loading);
        told(&mut shell, SHORT).await;
        assert_eq!(asleep(&shell, subscription), [false, false]);

        // Very short: it sleeps at the page it is at, asked for its state as a tab at rest
        // is, and wakes there.
        let states = shell.engine.states_asked.len();
        told(&mut shell, VERY_SHORT).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(shell.engine.states_asked[states..], [first_view]);
        let behind = &row(&shell, subscription)[0];
        assert_eq!(
            (behind.title.clone(), behind.host.as_str()),
            (named(1), "site1.example.com")
        );
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {}", at(1)));
    });
}

#[test]
fn what_is_the_persons_stays_however_short_the_system_is() {
    run(async {
        let mut shell = fake();
        // Seven behind and one in front. The first plays a sound, the second holds the
        // camera, the third was typed into, the fourth has a question waiting, the fifth
        // plays a sound and is on its way besides; the sixth does nothing; the seventh has a
        // download coming, which the person chose from its page's menu.
        let (_, tabs, subscription) = a_row_of(&mut shell, 8).await;
        shell.report(FakeEngine::download_chosen(
            tabs[6].1,
            7,
            "https://site7.example.com/report.pdf",
            "report.pdf",
        ));
        shell.report(FakeEngine::page_activity(tabs[0].1, true, false, false));
        shell.report(FakeEngine::page_activity(tabs[1].1, false, true, false));
        shell.report(FakeEngine::page_activity(tabs[2].1, false, false, true));
        shell.report(FakeEngine::permission_requested(
            tabs[3].1,
            1,
            "https://site4.example.com",
            &[PermissionKind::Camera],
        ));
        shell.report(FakeEngine::page_activity(tabs[4].1, true, false, false));
        shell.report(FakeEngine::navigation_started(tabs[4].1, &at(9)));
        shell.settle().await;
        let reasons: Vec<Vec<i32>> = row(&shell, subscription)
            .into_iter()
            .map(|tab| tab.keeps_awake)
            .collect();
        assert_eq!(
            reasons,
            [
                vec![KeepAwake::Sound as i32],
                vec![KeepAwake::Capture as i32],
                vec![KeepAwake::Typed as i32],
                vec![KeepAwake::Question as i32],
                vec![KeepAwake::Sound as i32, KeepAwake::Loading as i32],
                vec![],
                vec![KeepAwake::Download as i32],
                vec![],
            ]
        );

        // Short, and very short: only the one that does nothing sleeps.
        let only_the_sixth = [false, false, false, false, false, true, false, false];
        told(&mut shell, SHORT).await;
        assert_eq!(asleep(&shell, subscription), only_the_sixth);
        told(&mut shell, VERY_SHORT).await;
        assert_eq!(asleep(&shell, subscription), only_the_sixth);

        // The first one's song stops. It sleeps when the system next says it is short, and
        // not before: a report is acted on when it comes.
        shell.report(FakeEngine::page_activity(tabs[0].1, false, false, false));
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), only_the_sixth);
        told(&mut shell, SHORT).await;
        assert_eq!(
            asleep(&shell, subscription),
            [true, false, false, false, false, true, false, false]
        );
    });
}

#[test]
fn a_tab_that_has_just_come_alive_behind_is_not_spared_as_the_limit_spares_it() {
    run(async {
        // A device that holds two. A third tab is opened behind the one in front: the limit
        // puts the first to sleep, and spares the one that has just come alive (054).
        let mut shell = with(FakeOptions {
            live_tabs: 2,
            ..FakeOptions::default()
        });
        let (window, _, subscription) = a_row_of(&mut shell, 2).await;
        shell
            .request(request::Kind::OpenTab(OpenTab {
                window_id: window,
                text: at(3),
                background: true,
                after_tab_id: 0,
            }))
            .await;
        let third = row(&shell, subscription)
            .iter()
            .position(|tab| tab.title == named(3))
            .expect("the tab opened behind");
        let mut expected = vec![true, false, false];
        assert_eq!(asleep(&shell, subscription), expected);

        // The system is short: memory is wanted now, and the newcomer sleeps too.
        told(&mut shell, SHORT).await;
        expected[third] = true;
        assert_eq!(asleep(&shell, subscription), expected);
        assert_eq!(shell.engine.web_views.len(), 1);
    });
}

#[test]
fn a_private_windows_tabs_stay_at_both_levels() {
    run(async {
        let mut shell = fake();
        // An ordinary window with a tab behind, and a private one with two behind, one of
        // them on its way.
        let (_, _, subscription) = a_row_of(&mut shell, 2).await;
        let (window, first, private) = shell.open_private_window().await;
        shell.submit(first, &at(5)).await;
        let second = shell.open_tab(window, &at(6)).await;
        shell.open_tab(window, &at(7)).await;
        let states = shell.engine.states_asked.len();
        sent_on_its_way(&mut shell, second, 9).await;

        told(&mut shell, SHORT).await;
        told(&mut shell, VERY_SHORT).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(asleep(&shell, private), [false, false, false]);
        // The one state asked for is the ordinary tab's, as it slept.
        assert_eq!(shell.engine.states_asked.len(), states + 1);
    });
}

#[test]
fn a_site_sleeps_at_its_home_with_no_state() {
    run(async {
        let mut shell = fake();
        let (_, tabs, subscription) = a_row_of(&mut shell, 2).await;
        let (first, first_view) = tabs[0];
        // The first is kept: a Site, whose home is page 1. It wanders to page 8, and is left
        // behind there.
        shell.keep_tab(first).await;
        shell.activate_tab(first).await;
        shell.submit(first, &at(8)).await;
        shell.activate_tab(tabs[1].0).await;
        let states = shell.engine.states_asked.len();

        told(&mut shell, SHORT).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert!(!shell.engine.web_views.contains_key(&first_view));
        assert_eq!(
            shell.engine.states_asked.len(),
            states,
            "a Site is asked for no state"
        );
        let site = &row(&shell, subscription)[0];
        assert!(site.kept);
        assert_eq!(
            site.host, "site1.example.com",
            "at its home, not where it wandered"
        );
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("load {}", at(1)));
    });
}

#[test]
fn the_tabs_of_a_space_no_window_shows_sleep_the_one_in_front_of_its_row_among_them() {
    run(async {
        let mut shell = fake();
        let (window, _, subscription) = a_row_of(&mut shell, 2).await;
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
        shell.submit(work_tab, &at(5)).await;
        assert_eq!(shell.engine.web_views.len(), 3);

        told(&mut shell, SHORT).await;
        assert_eq!(shell.engine.web_views.len(), 1);
        assert_eq!(asleep(&shell, subscription), [false]);

        // The first Space is shown again: the tab that was in front of its row wakes, and
        // the other is asleep.
        shell.request(show(home)).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {}", at(2)));
    });
}

#[test]
fn a_tab_with_nowhere_to_wake_is_left_as_it_is() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, &at(1)).await;
        let opener = front(&shell, subscription).web_view_id;
        // The page asks for a window, which is a tab with a web view of its own and no page
        // yet. Another tab comes in front of both.
        let (opened, asks) =
            shell
                .engine
                .ask_for_window(opener, "https://site1.example.com/popup", true);
        shell.report(asks);
        shell.settle().await;
        shell.open_tab(window, &at(2)).await;
        shell.open_tab(window, &at(3)).await;
        assert_eq!(row(&shell, subscription).len(), 4);

        // The tab the page opened shows no page of the web's to wake to, and keeps its web
        // view. The tabs on either side of it sleep: it stops nobody after it.
        told(&mut shell, VERY_SHORT).await;
        assert_eq!(asleep(&shell, subscription), [true, false, true, false]);
        assert!(shell.engine.web_views.contains_key(&opened));
        assert_eq!(shell.engine.web_views.len(), 2);
    });
}

#[test]
fn the_limit_of_live_tabs_is_what_it_was_after_a_report() {
    run(async {
        // A device that holds three, with three alive. The system is short: two sleep.
        let mut shell = with(FakeOptions {
            live_tabs: 3,
            ..FakeOptions::default()
        });
        let (window, tabs, subscription) = a_row_of(&mut shell, 3).await;
        told(&mut shell, SHORT).await;
        assert_eq!(asleep(&shell, subscription), [true, true, false]);

        // The first is woken, and a fourth opened: three alive, as the device holds, and
        // nobody sleeps. The limit was not made less by the report.
        shell.activate_tab(tabs[0].0).await;
        shell.open_tab(window, &at(4)).await;
        assert_eq!(asleep(&shell, subscription), [false, true, false, false]);
        // A fifth: four alive. The one longest out of sight sleeps, which is the third: it
        // left the front before the first did.
        shell.open_tab(window, &at(5)).await;
        assert_eq!(
            asleep(&shell, subscription),
            [false, true, true, false, false]
        );
    });
}

#[test]
fn a_tab_that_slept_for_memory_is_asleep_with_its_state_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, first, subscription) = shell.open_window().await;
        shell.submit(first, &at(1)).await;
        shell.submit(first, &at(2)).await;
        let window = shell.window(subscription).window_id;
        shell.open_tab(window, &at(3)).await;
        told(&mut shell, SHORT).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        shell.settle().await;
        drop(shell);

        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        let first = row(&shell, subscription)[0].tab_id;
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell).last().unwrap(), &format!("restore {}", at(2)));
        assert!(front(&shell, subscription).can_go_back);
    });
}

#[test]
fn a_suite_can_have_the_core_told_every_few_seconds() {
    run(async {
        let mut shell = with(FakeOptions {
            memory_short_every_seconds: 2,
            ..FakeOptions::default()
        });
        let (_, tabs, subscription) = a_row_of(&mut shell, 3).await;
        // The second's page starts for another by itself, and is a long time about it.
        shell.report(FakeEngine::navigation_started(tabs[1].1, &at(9)));
        shell.settle().await;
        // Not after a second; after two. Short is what the core is told, and not very short:
        // the tab on its way stays.
        tokio::time::sleep(Duration::from_millis(1_010)).await;
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [false, false, false]);
        tokio::time::sleep(Duration::from_millis(1_010)).await;
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false, false]);

        // And again every two: the first is woken, and the one it was woken over sleeps. The
        // core says so itself, with nothing asked of it: three seconds in which the shell
        // only listens, and in which no other look of the core's comes round.
        shell.activate_tab(tabs[0].0).await;
        assert_eq!(asleep(&shell, subscription), [false, false, false]);
        shell.seconds_pass(3).await;
        assert_eq!(asleep(&shell, subscription), [false, false, true]);
    });
}

#[test]
fn outside_fake_mode_the_suites_knob_is_nothing() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut config = FakeShell::live_config(folder.path());
        config.fake = Some(FakeOptions {
            memory_short_every_seconds: 1,
            ..FakeOptions::default()
        });
        let mut shell = FakeShell::start(&config).unwrap();
        web(&mut shell);
        let (_, _, subscription) = a_row_of(&mut shell, 2).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [false, false]);
    });
}
