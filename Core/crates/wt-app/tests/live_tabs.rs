//! No more live tabs than the device can hold (054), through the contract,
//! against the scripted engine: what the limit is on each kind of device,
//! which tabs sleep when it is passed and in what order, and which are
//! passed over.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    CoreConfig, CreateSpace, DeviceKind, EngineKind, FakeOptions, OpenTab, SwitchSpace, TabSummary,
    TabView, host_request, request, response,
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
    for number in 1..=40 {
        shell.engine.page(&at(number), &named(number));
    }
}

/// A shell in fake mode whose suite gave a limit of live tabs.
fn limited(live_tabs: u32) -> FakeShell {
    let mut config = FakeShell::config(EngineKind::Webkit);
    config.fake = Some(FakeOptions {
        live_tabs,
        ..FakeOptions::default()
    });
    let mut shell = FakeShell::start(&config).unwrap();
    web(&mut shell);
    shell
}

/// A live configuration, in `folder`, of a shell that says what it runs on.
fn on_a(folder: &std::path::Path, kind: i32, memory_megabytes: u32) -> CoreConfig {
    CoreConfig {
        device_kind: kind,
        memory_megabytes,
        ..FakeShell::live_config(folder)
    }
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

/// The titles of the row's tabs that are asleep.
fn sleepers(shell: &FakeShell, subscription: u64) -> Vec<String> {
    row(shell, subscription)
        .into_iter()
        .filter(|tab| tab.asleep)
        .map(|tab| tab.title)
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
/// window, its tabs in the row's order, and the subscription.
async fn a_row_of(shell: &mut FakeShell, count: usize) -> (u64, Vec<u64>, u64) {
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, &at(1)).await;
    let mut tabs = vec![first];
    for number in 2..=count {
        tabs.push(shell.open_tab(window, &at(number)).await);
    }
    (window, tabs, subscription)
}

/// Asks for a tab at a page behind the one in front, without waiting.
fn open_behind(shell: &mut FakeShell, window: u64, number: usize) -> u64 {
    shell.send(request::Kind::OpenTab(OpenTab {
        window_id: window,
        text: at(number),
        background: true,
        after_tab_id: 0,
    }))
}

#[test]
fn a_phone_holds_three_live_tabs_a_tablet_six_and_a_desktop_as_its_memory_allows() {
    run(async {
        let (phone, tablet, desktop) = (
            DeviceKind::Phone as i32,
            DeviceKind::Tablet as i32,
            DeviceKind::Desktop as i32,
        );
        // The device, its memory in megabytes, and the most tabs alive on it. A phone's and a
        // tablet's are theirs whatever the memory; a desktop has one for each half gigabyte,
        // and no fewer than eight. A shell that does not say what it runs on, or says a kind
        // this core has not heard of, is taken for a desktop.
        let devices = [
            (phone, 8_192, 3),
            (phone, 0, 3),
            (tablet, 16_384, 6),
            (desktop, 8_192, 16),
            (desktop, 16_384, 32),
            (desktop, 6_500, 12),
            (desktop, 2_048, 8),
            (desktop, 0, 8),
            (DeviceKind::Unspecified as i32, 0, 8),
            (DeviceKind::Unspecified as i32, 5_120, 10),
            (99, 5_120, 10),
        ];
        for (kind, memory, limit) in devices {
            let said = format!("kind {kind} with {memory} megabytes");
            let folder = tempfile::tempdir().unwrap();
            let mut shell = FakeShell::start(&on_a(folder.path(), kind, memory)).unwrap();
            web(&mut shell);
            // As many as it holds: all are alive.
            let (window, _, subscription) = a_row_of(&mut shell, limit).await;
            assert_eq!(sleepers(&shell, subscription), [] as [&str; 0], "{said}");
            assert_eq!(shell.engine.web_views.len(), limit, "{said}");
            // One more: the first, which has been out of sight the longest, sleeps.
            shell.open_tab(window, &at(limit + 1)).await;
            assert_eq!(sleepers(&shell, subscription), ["Site 1"], "{said}");
            assert_eq!(shell.engine.web_views.len(), limit, "{said}");
        }
    });
}

#[test]
fn fake_mode_has_no_limit_but_the_one_a_suite_gives() {
    run(async {
        // A phone, in fake mode: its suites open what they like.
        let mut config = FakeShell::config(EngineKind::Webkit);
        config.device_kind = DeviceKind::Phone as i32;
        let mut shell = FakeShell::start(&config).unwrap();
        web(&mut shell);
        let (_, _, subscription) = a_row_of(&mut shell, 12).await;
        assert_eq!(sleepers(&shell, subscription), [] as [&str; 0]);
        assert_eq!(shell.engine.web_views.len(), 12);

        // A suite's limit is the limit, on whatever it runs.
        let mut shell = limited(2);
        let (_, _, subscription) = a_row_of(&mut shell, 3).await;
        assert_eq!(asleep(&shell, subscription), [true, false, false]);
    });
}

#[test]
fn the_tab_longest_out_of_sight_sleeps_as_the_time_would_put_it_to_sleep_and_wakes_as_it_was() {
    run(async {
        let mut shell = limited(3);
        let (window, tabs, subscription) = a_row_of(&mut shell, 3).await;
        // The second has a way back, and is left behind; then the first is looked at again.
        shell.activate_tab(tabs[1]).await;
        shell.submit(tabs[1], &at(9)).await;
        let second_view = front(&shell, subscription).web_view_id;
        let before = shell.engine.web_views.get(&second_view).cloned().unwrap();
        shell.activate_tab(tabs[2]).await;
        shell.activate_tab(tabs[0]).await;
        assert_eq!(asleep(&shell, subscription), [false, false, false]);

        // A fourth: of the two out of sight, the second left the front before the third.
        // Its state is asked for and then its web view let go, in that order; it is where
        // it was in its row, with the name and the address it had.
        shell.open_tab(window, &at(4)).await;
        assert_eq!(asleep(&shell, subscription), [false, true, false, false]);
        assert_eq!(shell.engine.states_asked.last(), Some(&second_view));
        let lets_go = format!("destroy {second_view}");
        assert_eq!(
            asked(&shell)
                .iter()
                .filter(|asked| asked.starts_with("destroy"))
                .collect::<Vec<_>>(),
            [&lets_go]
        );
        assert_eq!(shell.engine.web_views.len(), 3);
        let behind = &row(&shell, subscription)[1];
        assert_eq!(
            (behind.title.as_str(), behind.host.as_str()),
            ("Site 9", "site9.example.com")
        );

        // Pressed, it wakes from its state, where it was and with the list it had. That is a
        // tab come alive: the third, which is now the longest out of sight, sleeps.
        shell.activate_tab(tabs[1]).await;
        assert!(asked(&shell).contains(&format!("restore {}", at(9))));
        let woken = front(&shell, subscription);
        assert_eq!((woken.title.as_str(), woken.can_go_back), ("Site 9", true));
        let view = shell.engine.web_views.get(&woken.web_view_id).unwrap();
        assert_eq!((&view.history, view.index), (&before.history, before.index));
        assert_eq!(asleep(&shell, subscription), [false, false, true, false]);
        assert_eq!(shell.engine.web_views.len(), 3);
    });
}

#[test]
fn a_tab_that_has_just_left_the_front_sleeps_when_it_is_the_only_one_that_can() {
    run(async {
        let mut shell = limited(1);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, &at(1)).await;
        let first_view = front(&shell, subscription).web_view_id;
        // A second in front of it: the first sleeps at once. It was asked for its state as it
        // left the front, and once more as it sleeps; then its web view was let go.
        let second = shell.open_tab(window, &at(2)).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(shell.engine.states_asked, [first_view, first_view]);
        assert_eq!(
            asked(&shell).last().unwrap(),
            &format!("destroy {first_view}")
        );
        // And back: each wakes from its state as the other sleeps.
        shell.activate_tab(first).await;
        assert_eq!(asleep(&shell, subscription), [false, true]);
        assert!(asked(&shell).contains(&format!("restore {}", at(1))));
        shell.activate_tab(second).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert!(asked(&shell).contains(&format!("restore {}", at(2))));
        assert_eq!(shell.engine.web_views.len(), 1);
    });
}

#[test]
fn a_tab_with_a_reason_to_be_awake_is_passed_over_and_the_limit_is_passed_when_all_have_one() {
    run(async {
        let mut shell = limited(2);
        let (window, tabs, subscription) = a_row_of(&mut shell, 2).await;
        let second_view = front(&shell, subscription).web_view_id;
        shell.activate_tab(tabs[0]).await;
        let first_view = front(&shell, subscription).web_view_id;
        shell.activate_tab(tabs[1]).await;
        // The first plays a sound; the second has been typed into.
        shell.report(FakeEngine::page_activity(first_view, true, false, false));
        shell.report(FakeEngine::page_activity(second_view, false, false, true));
        shell.settle().await;

        // A third: there is nobody to put to sleep, and three are alive on a device that
        // holds two. The limit is a wish, and no reason to stop a song or lose a draft.
        shell.open_tab(window, &at(3)).await;
        assert_eq!(asleep(&shell, subscription), [false, false, false]);
        assert_eq!(shell.engine.web_views.len(), 3);

        // A fourth: the third sleeps, though the others have been out of sight longer.
        shell.open_tab(window, &at(4)).await;
        assert_eq!(asleep(&shell, subscription), [false, false, true, false]);

        // The song stops: the limit is passed still, and the tab that played it sleeps at
        // once, with no tab come alive.
        shell.report(FakeEngine::page_activity(first_view, false, false, false));
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false, true, false]);
        assert_eq!(shell.engine.web_views.len(), 2);
        assert_eq!(row(&shell, subscription)[0].title, "Site 1");
    });
}

#[test]
fn a_tab_on_its_way_is_passed_over_and_sleeps_where_it_arrives() {
    run(async {
        let mut shell = limited(1);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, &at(1)).await;
        let first_view = front(&shell, subscription).web_view_id;
        // It starts on its way somewhere by itself, and another tab comes in front of it.
        shell.report(FakeEngine::navigation_started(first_view, &at(5)));
        shell.settle().await;
        shell.open_tab(window, &at(2)).await;
        assert_eq!(
            asleep(&shell, subscription),
            [false, false],
            "it is on its way"
        );

        // It arrives, and sleeps there: woken, it is at the page it arrived at.
        for report in shell.engine.goes_to(first_view, &at(5)) {
            shell.report(report);
        }
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(row(&shell, subscription)[0].title, "Site 5");
        shell.activate_tab(first).await;
        assert_eq!(front(&shell, subscription).title, "Site 5");
        assert!(asked(&shell).contains(&format!("restore {}", at(5))));
    });
}

#[test]
fn a_tab_that_comes_alive_behind_is_not_put_to_sleep_by_its_own_arrival() {
    run(async {
        let mut shell = limited(1);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, &at(1)).await;
        // A tab opened behind the one being read: two are alive where one is held. The one
        // in front is in sight; the other has only just come, and stays, loaded and with no
        // reason to be awake.
        open_behind(&mut shell, window, 2);
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [false, false]);
        assert_eq!(row(&shell, subscription)[1].title, "Site 2");
        assert_eq!(row(&shell, subscription)[1].keeps_awake, [] as [i32; 0]);
        // Whatever else happens, it stays until another comes alive.
        let view = front(&shell, subscription).web_view_id;
        shell.report(FakeEngine::title(view, "Site 1, read"));
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [false, false]);

        // Another behind: the one before it sleeps now, and this one stays.
        open_behind(&mut shell, window, 3);
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [false, true, false]);
    });
}

#[test]
fn tabs_opened_behind_sleep_in_the_order_they_came_alive() {
    run(async {
        let mut shell = limited(3);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, &at(1)).await;
        // Three behind the one being read: at the third, the first of them sleeps; at a
        // fourth, the second. None of them was ever in front.
        for number in 2..=4 {
            open_behind(&mut shell, window, number);
            shell.settle().await;
        }
        assert_eq!(sleepers(&shell, subscription), ["Site 2"]);
        open_behind(&mut shell, window, 5);
        shell.settle().await;
        assert_eq!(sleepers(&shell, subscription), ["Site 2", "Site 3"]);
        assert_eq!(front(&shell, subscription).title, "Site 1");
        assert_eq!(shell.engine.web_views.len(), 3);
    });
}

#[test]
fn a_tab_that_comes_alive_behind_has_been_out_of_sight_since_it_came() {
    run(async {
        let mut shell = limited(3);
        // Two tabs, and the first looked at again: the second has left the front.
        let (window, tabs, subscription) = a_row_of(&mut shell, 2).await;
        shell.activate_tab(tabs[0]).await;
        // A third comes alive behind, after that; and then a fourth. Of the two that can be
        // put to sleep, the second has been out of sight the longer: the third came later,
        // though it was never looked at.
        open_behind(&mut shell, window, 3);
        shell.settle().await;
        assert_eq!(sleepers(&shell, subscription), [] as [&str; 0]);
        open_behind(&mut shell, window, 4);
        shell.settle().await;
        assert_eq!(sleepers(&shell, subscription), ["Site 2"]);
    });
}

#[test]
fn tabs_that_come_alive_together_are_counted_together() {
    run(async {
        let mut shell = limited(3);
        let (window, _, subscription) = a_row_of(&mut shell, 3).await;
        // Two asked for behind, in the same breath: five are alive when the core looks.
        // Neither of the two is put to sleep, and the two longest out of sight are, at once.
        open_behind(&mut shell, window, 4);
        open_behind(&mut shell, window, 5);
        shell.settle().await;
        assert_eq!(sleepers(&shell, subscription), ["Site 1", "Site 2"]);
        assert_eq!(front(&shell, subscription).title, "Site 3");
        assert_eq!(shell.engine.web_views.len(), 3);
        // At once: the window was never told of one asleep and the other not.
        let one_asleep = shell
            .snapshots(subscription)
            .iter()
            .filter(|window| window.tabs.iter().filter(|tab| tab.asleep).count() == 1)
            .count();
        assert_eq!(one_asleep, 0);
    });
}

#[test]
fn a_private_windows_tabs_are_counted_and_are_never_the_ones_put_to_sleep() {
    run(async {
        let mut shell = limited(3);
        // A private window with a tab behind and one in front: two alive.
        let (private_window, first, private) = shell.open_private_window().await;
        shell.submit(first, &at(1)).await;
        shell.open_tab(private_window, &at(2)).await;
        // An ordinary window: a third, and a fourth. The private window's tab behind has
        // been out of sight the longest, and is passed over: it could keep nothing to wake
        // with. The ordinary window's sleeps; with the private tabs not counted it would
        // not, there being only two.
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, &at(3)).await;
        shell.open_tab(window, &at(4)).await;
        assert_eq!(asleep(&shell, private), [false, false]);
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(shell.engine.web_views.len(), 3);

        // Another private tab: nobody can be put to sleep, and four are alive.
        shell.open_tab(private_window, &at(5)).await;
        assert_eq!(asleep(&shell, private), [false, false, false]);
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(shell.engine.web_views.len(), 4);
    });
}

#[test]
fn a_site_sleeps_at_its_home_with_no_state() {
    run(async {
        let mut shell = limited(2);
        let (window, tabs, subscription) = a_row_of(&mut shell, 2).await;
        // The first is kept: a Site, whose home is page 1. It wanders, and is left behind.
        shell.keep_tab(tabs[0]).await;
        shell.activate_tab(tabs[0]).await;
        shell.submit(tabs[0], &at(7)).await;
        shell.activate_tab(tabs[1]).await;
        let asked_before = shell.engine.states_asked.len();

        // A third: the Site sleeps, at its home and not where it wandered, asked for no
        // state. (The tab that left the front is asked for its own, as any is.)
        shell.open_tab(window, &at(3)).await;
        assert_eq!(asleep(&shell, subscription), [true, false, false]);
        let second_view = shell.engine.states_asked[asked_before..].to_vec();
        assert_eq!(second_view.len(), 1, "only the tab that left the front");
        let site = &row(&shell, subscription)[0];
        assert!(site.kept);
        assert_eq!(site.host, "site1.example.com");
        // Woken, it is loaded at its home.
        shell.activate_tab(tabs[0]).await;
        assert!(asked(&shell).contains(&format!("load {}", at(1))));
        assert_eq!(
            asked(&shell)
                .iter()
                .filter(|a| **a == format!("load {}", at(1)))
                .count(),
            2
        );
    });
}

#[test]
fn the_tabs_of_a_space_no_window_shows_are_counted_and_sleep_with_the_rest() {
    run(async {
        let mut shell = limited(2);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, &at(1)).await;
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
        // The other Space is shown, and has two tabs: three are alive, one of them in front
        // of a row no window shows. It has been out of sight the longest, and sleeps.
        shell.request(show(work.space_id)).await;
        let work_tab = front(&shell, subscription).tab_id;
        shell.submit(work_tab, &at(2)).await;
        assert_eq!(shell.engine.web_views.len(), 2);
        shell.open_tab(window, &at(3)).await;
        assert_eq!(
            asleep(&shell, subscription),
            [false, false],
            "the Space shown"
        );
        assert_eq!(shell.engine.web_views.len(), 2);

        // Its Space is shown again: it is in front of its row, and wakes from its state.
        // That is a tab come alive: of the other Space's two, now out of sight, the one that
        // was behind sleeps.
        shell.request(show(home)).await;
        assert_eq!(asleep(&shell, subscription), [false]);
        assert!(asked(&shell).contains(&format!("restore {}", at(1))));
        assert_eq!(shell.engine.web_views.len(), 2);
        shell.request(show(work.space_id)).await;
        assert_eq!(front(&shell, subscription).title, "Site 3");
        assert_eq!(sleepers(&shell, subscription), ["Site 2"]);
    });
}

#[test]
fn a_tab_with_nowhere_to_wake_is_passed_over_for_the_next() {
    run(async {
        let mut shell = limited(2);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, &at(1)).await;
        let opener = front(&shell, subscription).web_view_id;
        // The page asks for a window, which is a tab with a web view of its own and no page
        // yet: nothing has arrived in it. The first is looked at again, so the tab the page
        // opened is the longer out of sight.
        let (opened, asks) =
            shell
                .engine
                .ask_for_window(opener, "https://site1.example.com/popup", true);
        shell.report(asks);
        shell.settle().await;
        shell.activate_tab(first).await;
        assert_eq!(asleep(&shell, subscription), [false, false]);

        // A third: the tab the page opened shows no page of the web's to wake to, and keeps
        // its web view; the first sleeps in its place.
        shell.open_tab(window, &at(3)).await;
        assert_eq!(asleep(&shell, subscription), [true, false, false]);
        assert!(shell.engine.web_views.contains_key(&opened));
        assert_eq!(shell.engine.web_views.len(), 2);
    });
}

#[test]
fn a_tab_the_limit_put_to_sleep_is_asleep_with_its_state_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let phone = on_a(folder.path(), DeviceKind::Phone as i32, 6_144);
        let mut shell = FakeShell::start(&phone).unwrap();
        web(&mut shell);
        let (_, tabs, subscription) = a_row_of(&mut shell, 4).await;
        assert_eq!(asleep(&shell, subscription), [true, false, false, false]);
        // The first had a way back before it slept.
        shell.activate_tab(tabs[0]).await;
        shell.submit(tabs[0], &at(8)).await;
        shell.activate_tab(tabs[3]).await;
        assert_eq!(asleep(&shell, subscription), [false, true, false, false]);
        shell.activate_tab(tabs[2]).await;
        shell.activate_tab(tabs[1]).await;
        assert_eq!(asleep(&shell, subscription), [true, false, false, false]);
        shell.settle().await;
        drop(shell);

        // Started again: the session's tabs are asleep but the one in front, as they always
        // are, and the first wakes from the state it slept with.
        let mut shell = FakeShell::start(&phone).unwrap();
        web(&mut shell);
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(asleep(&shell, subscription), [true, false, true, true]);
        let first = row(&shell, subscription)[0].tab_id;
        shell.activate_tab(first).await;
        assert!(asked(&shell).contains(&format!("restore {}", at(8))));
        assert!(front(&shell, subscription).can_go_back);
        // Tabs that wake are tabs come alive: a phone holds three of them.
        let others: Vec<u64> = row(&shell, subscription)[2..]
            .iter()
            .map(|tab| tab.tab_id)
            .collect();
        for tab in others {
            shell.activate_tab(tab).await;
        }
        assert_eq!(asleep(&shell, subscription), [false, true, false, false]);
        assert_eq!(shell.engine.web_views.len(), 3);
    });
}
