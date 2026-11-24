//! What a sleeping tab keeps (051), through the contract, against the
//! scripted engine, which hands out a web view's state and makes one from
//! it: when a state is asked for, which tabs are never asked, what is kept
//! and dropped, and that a tab that comes back at a start wakes from its
//! state, where it was.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{
    CloseWindow, CreateSpace, EngineInfo, EngineKind, ForgetHistory, NavigationCommand,
    SubmitAddress, SwitchSpace, TabCommand, TabView, forget_history, host_request, request,
    response,
};
use wt_store::Store;
use wt_testkit::{FakeEngine, FakeShell, run};

const HOME: &str = "https://example.com/";
const NEWS: &str = "https://news.example.com/";
const MAIL: &str = "https://mail.example.com/";
const WIKI: &str = "https://wiki.example.com/";
const NOWHERE: &str = "https://nowhere.example.com/";
const INBOX: &str = "https://mail.example.com/#inbox";

fn web(shell: &mut FakeShell) {
    shell.engine.page(HOME, "Home");
    shell.engine.page(NEWS, "News");
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(WIKI, "Wiki");
}

/// The app started in `folder`: a store on disk, and a session kept.
fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the shell was asked of web views' pages, in order: a load, or a wake
/// from a state, each by its address.
fn asked(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewLoad(load) => Some(format!("load {}", load.url)),
            host_request::Kind::WebViewRestore(restore) => Some(format!("restore {}", restore.url)),
            _ => None,
        })
        .collect()
}

async fn back(shell: &mut FakeShell, tab: u64) {
    let answer = shell
        .request(request::Kind::TabCommand(TabCommand {
            tab_id: tab,
            command: NavigationCommand::Back as i32,
        }))
        .await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

/// Forgets every visit of the logins of the Space the window shows.
async fn forget_everything(shell: &mut FakeShell, window: u64) {
    let answer = shell
        .request(request::Kind::ForgetHistory(ForgetHistory {
            window_id: window,
            what: Some(forget_history::What::Everything(true)),
        }))
        .await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

async fn close_window(shell: &mut FakeShell, window: u64) {
    let answer = shell
        .request(request::Kind::CloseWindow(CloseWindow {
            window_id: window,
        }))
        .await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

/// Makes a Space with logins of its own, and has the window show it.
async fn show_a_new_space(shell: &mut FakeShell, window: u64, name: &str) -> u64 {
    let made = shell
        .request(request::Kind::CreateSpace(CreateSpace {
            name: name.to_owned(),
            ..CreateSpace::default()
        }))
        .await;
    let response::Kind::SpaceCreated(space) = made else {
        panic!("expected a Space, got {made:?}");
    };
    let answer = shell
        .request(request::Kind::SwitchSpace(SwitchSpace {
            window_id: window,
            space_id: space.space_id,
        }))
        .await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
    space.space_id
}

/// Whether any of the store's files in `folder` has those bytes in it, as the files are
/// now: the database, and what is beside it.
fn holds(folder: &Path, needle: &str) -> bool {
    std::fs::read_dir(folder)
        .unwrap()
        .filter_map(|entry| std::fs::read(entry.unwrap().path()).ok())
        .any(|bytes| {
            bytes
                .windows(needle.len())
                .any(|window| window == needle.as_bytes())
        })
}

/// The store the app left in `folder`, once the app has gone.
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
fn a_tab_that_leaves_the_front_is_asked_for_its_state_and_a_tab_in_front_is_not() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let first_view = front(&shell, subscription).web_view_id;
        // In front, however long and wherever it goes: nothing is asked.
        shell.submit(first, NEWS).await;
        assert_eq!(shell.engine.states_asked, [] as [u64; 0]);

        // Another tab comes in front: the one that left it is asked, once.
        let second = shell.open_tab(window, MAIL).await;
        let second_view = front(&shell, subscription).web_view_id;
        assert_eq!(shell.engine.states_asked, [first_view]);
        // Things happen in the tab in front, and behind it: nobody is asked.
        shell.submit(second, WIKI).await;
        assert_eq!(shell.engine.states_asked, [first_view]);

        // The first in front again: the second has left it.
        shell.activate_tab(first).await;
        assert_eq!(shell.engine.states_asked, [first_view, second_view]);

        // The window shows another Space: the tab it showed is out of sight.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(work) = made else {
            panic!("expected a Space, got {made:?}");
        };
        shell
            .request(request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: work.space_id,
            }))
            .await;
        assert_eq!(
            shell.engine.states_asked,
            [first_view, second_view, first_view]
        );
    });
}

#[test]
fn a_tab_with_nothing_to_keep_is_not_asked() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;

        // An empty tab has no page: another comes in front, and nothing is asked.
        let second = shell.open_tab(window, HOME).await;
        assert_eq!(shell.engine.states_asked, [] as [u64; 0]);

        // A Site sleeps at its home (039): it has nothing of where it was to keep.
        shell.keep_tab(second).await;
        shell.activate_tab(first).await;
        assert_eq!(shell.engine.states_asked, [] as [u64; 0], "a Site");

        // A tab whose load failed is shown where it was going, and its web view is where it
        // was: its state would wake it at the wrong page.
        let third = shell.open_tab(window, NEWS).await;
        shell.engine.fail(MAIL, "net.offline");
        shell.submit(third, MAIL).await;
        assert_eq!(
            front(&shell, subscription).load_state().as_str_name(),
            "LOAD_STATE_FAILED"
        );
        shell.activate_tab(first).await;
        assert_eq!(shell.engine.states_asked, [] as [u64; 0], "a failed load");

        // And one that is on its way somewhere, whose load the shell has not begun.
        let fourth = shell.open_tab(window, WIKI).await;
        shell.hold_host = true;
        shell.send(request::Kind::SubmitAddress(SubmitAddress {
            tab_id: fourth,
            text: NEWS.to_owned(),
        }));
        shell.settle().await;
        shell.hold_host = false;
        shell.release_host();
        shell.settle().await;
        // (It has arrived by now; while it was going it was in front, and was asked nothing.)
        assert_eq!(shell.engine.states_asked, [] as [u64; 0]);

        // A private window's tab: nothing of it is written (037).
        let (private_window, private_first, _) = shell.open_private_window().await;
        shell.submit(private_first, HOME).await;
        shell.open_tab(private_window, NEWS).await;
        assert_eq!(
            shell.engine.states_asked,
            [] as [u64; 0],
            "a private window"
        );
    });
}

#[test]
fn a_tab_that_comes_back_at_a_start_wakes_from_its_state_where_it_was() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        // Home, then News, then back to Home: the list has a page to go forward to.
        shell.submit(first, HOME).await;
        shell.submit(first, NEWS).await;
        back(&mut shell, first).await;
        assert!(front(&shell, subscription).can_go_forward);
        let first_view = front(&shell, subscription).web_view_id;
        // A second tab in front: the first leaves it, and its state is kept.
        shell.open_tab(window, MAIL).await;
        assert_eq!(shell.engine.states_asked, [first_view]);
        let given = shell.engine.web_views.get(&first_view).cloned().unwrap();
        drop(shell);

        // The state is in the store, with the engine it is of, and the first tab names it.
        assert_eq!(kept(folder.path()), [1]);
        {
            let store = store(folder.path());
            let (engine, state) = store.kept_state(1).unwrap().unwrap();
            assert_eq!(engine, EngineKind::Webkit as i32);
            assert!(
                String::from_utf8_lossy(&state).contains(NEWS),
                "the engine's own bytes"
            );
            let space = store.spaces().unwrap()[0].id;
            let tabs = store.saved_tabs(space).unwrap();
            assert_eq!(
                tabs.iter()
                    .map(|tab| (tab.url.as_str(), tab.state))
                    .collect::<Vec<_>>(),
                [(HOME, 1), (MAIL, 0)],
                "the tab that was in front at the quit has none"
            );
        }

        // The app starts again. The tab in front was never out of sight: it is loaded.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(asked(&shell), [format!("load {MAIL}")]);
        assert_eq!(shell.engine.restores.len(), 0);

        // The first is brought in front: its web view is made from its state, with its
        // address to fall back on. It arrives where it was, and its list is the one it had.
        let first = shell.window(subscription).tabs[0].tab_id;
        shell.activate_tab(first).await;
        assert_eq!(
            asked(&shell),
            [format!("load {MAIL}"), format!("restore {HOME}")]
        );
        let (restored_view, state, url) = shell.engine.restores[0].clone();
        assert_eq!(url, HOME);
        assert!(String::from_utf8_lossy(&state).contains(NEWS));
        let woken = front(&shell, subscription);
        assert_eq!(woken.web_view_id, restored_view);
        assert_eq!(woken.title, "Home");
        assert_eq!(woken.address.unwrap().url, HOME);
        assert!(!woken.can_go_back);
        assert!(
            woken.can_go_forward,
            "a load would have had nowhere to go forward to"
        );
        let view = shell.engine.web_views.get(&restored_view).unwrap();
        assert_eq!((&view.history, view.index), (&given.history, given.index));
        // Once: brought in front again, it is where it is.
        let second = shell.window(subscription).tabs[1].tab_id;
        shell.activate_tab(second).await;
        shell.activate_tab(first).await;
        assert_eq!(shell.engine.restores.len(), 1);
    });
}

#[test]
fn an_engine_that_will_not_take_the_state_loads_the_address_and_another_engines_state_is_not_handed_over()
 {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, HOME).await;
        shell.submit(first, NEWS).await;
        shell.open_tab(window, MAIL).await;
        drop(shell);

        // The engine will not take what it is handed: the tab wakes all the same, at its
        // address, with the list of a tab that has been nowhere else.
        let mut shell = start(folder.path());
        shell.engine.refuses_states = true;
        let (_, _, subscription) = shell.open_window().await;
        let first = shell.window(subscription).tabs[0].tab_id;
        shell.activate_tab(first).await;
        assert_eq!(shell.engine.restores.len(), 1, "it was asked to");
        let woken = front(&shell, subscription);
        assert_eq!((woken.title.as_str(), woken.can_go_back), ("News", false));
        // It arrived: the state it woke from is its no longer, and it is asked again when
        // it leaves the front.
        let second = shell.window(subscription).tabs[1].tab_id;
        shell.activate_tab(second).await;
        drop(shell);

        // Another engine: the state is WebKit's, and is not handed to it. The tab is loaded.
        let mut config = FakeShell::live_config(folder.path());
        config.engines = vec![EngineInfo {
            kind: EngineKind::Chromium as i32,
            version: "fake".to_owned(),
        }];
        let mut shell = FakeShell::start(&config).unwrap();
        web(&mut shell);
        shell.engine.engines = vec![EngineKind::Chromium];
        let (_, _, subscription) = shell.open_window().await;
        let first = shell.window(subscription).tabs[0].tab_id;
        shell.activate_tab(first).await;
        assert_eq!(shell.engine.restores.len(), 0);
        assert_eq!(asked(&shell).last().unwrap(), &format!("load {NEWS}"));
        assert_eq!(front(&shell, subscription).title, "News");
    });
}

#[test]
fn a_state_is_of_the_page_it_was_asked_on_and_goes_when_the_tab_goes_elsewhere() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let second = shell.open_tab(window, MAIL).await;
        shell.settle().await;
        drop(shell);
        assert_eq!(kept(folder.path()), [1], "the first tab's");

        // It wakes from its state, and arrives: the state is its no longer, and is dropped.
        let mut shell = start(folder.path());
        let (_, _, subscription_again) = shell.open_window().await;
        let first = shell.window(subscription_again).tabs[0].tab_id;
        shell.activate_tab(first).await;
        assert_eq!(shell.engine.restores.len(), 1);
        // In front, it goes on to another page, and the app is quit with it there.
        shell.submit(first, WIKI).await;
        shell.settle().await;
        drop(shell);
        // The second tab left the front when the first woke: it has a state now. The first
        // has none: it wakes at its address.
        assert_eq!(kept(folder.path()), [2]);
        let mut shell = start(folder.path());
        shell.open_window().await;
        assert_eq!(asked(&shell), [format!("load {WIKI}")]);
        let _ = (second, subscription);
    });
}

#[test]
fn a_state_that_is_too_large_is_not_kept_and_the_one_before_it_is_dropped() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let second = shell.open_tab(window, MAIL).await;
        shell.settle().await;
        // The first has a state, of the usual size (1). One of four hundred thousand bytes
        // is kept as well: the limit is above it. It is the second's (2).
        shell.engine.state_padding = 400_000;
        shell.activate_tab(first).await;
        shell.settle().await;
        // And the first's again, of the usual size (3), in the place of the one it had.
        shell.engine.state_padding = 0;
        shell.activate_tab(second).await;
        shell.settle().await;
        drop(shell);
        assert_eq!(kept(folder.path()), [2, 3]);
        let large = store(folder.path()).kept_state(2).unwrap().unwrap().1.len();
        assert!((400_000..=512 * 1024).contains(&large), "{large} bytes");

        // Each wakes from its own, and has a state again when it leaves the front (4, 5).
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let tabs = shell.window(subscription).tabs.clone();
        let (first, second) = (tabs[0].tab_id, tabs[1].tab_id);
        shell.activate_tab(first).await;
        shell.settle().await;
        shell.activate_tab(second).await;
        shell.settle().await;
        assert_eq!(shell.engine.restores.len(), 2);
        // Then each leaves the front with a state of half a megabyte and more: it is not
        // kept, and the one the tab had is dropped. An old state for a page that has
        // changed so would wake the tab as it no longer is.
        shell.engine.state_padding = 600_000;
        shell.activate_tab(first).await;
        shell.activate_tab(second).await;
        shell.settle().await;
        assert_eq!(shell.engine.states_asked.len(), 4);
        drop(shell);
        assert_eq!(kept(folder.path()), [] as [i64; 0]);

        // An engine that has none to give is asked, and nothing is kept either: the one
        // the tab had (6) goes.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let tabs = shell.window(subscription).tabs.clone();
        let (first, second) = (tabs[0].tab_id, tabs[1].tab_id);
        shell.activate_tab(first).await;
        shell.settle().await;
        shell.engine.gives_no_state = true;
        shell.activate_tab(second).await;
        shell.activate_tab(first).await;
        shell.settle().await;
        assert_eq!(shell.engine.states_asked.len(), 3);
        // Neither tab had a state to wake from: each was loaded.
        assert_eq!(
            asked(&shell),
            [format!("load {MAIL}"), format!("load {HOME}")]
        );
        drop(shell);
        assert_eq!(kept(folder.path()), [] as [i64; 0]);
    });
}

#[test]
fn a_closed_tabs_state_goes_with_it() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, HOME).await;
        shell.open_tab(window, MAIL).await;
        let third = shell.open_tab(window, WIKI).await;
        shell.settle().await;
        // The first and the second have each left the front: two states.
        assert_eq!(shell.engine.states_asked.len(), 2);
        // The first is closed, behind: its state is dropped. The second keeps its own.
        shell.close_tab(first).await;
        shell.settle().await;
        let _ = third;
        drop(shell);
        assert_eq!(kept(folder.path()), [2]);
    });
}

#[test]
fn a_state_no_saved_tab_names_goes_when_the_app_starts() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, HOME).await;
        shell.open_tab(window, MAIL).await;
        shell.settle().await;
        drop(shell);
        // The first tab's state; and one that nobody names, as a run that was killed
        // between keeping a state and writing its session leaves.
        assert_eq!(kept(folder.path()), [1]);
        let orphan = store(folder.path()).keep_state(1, b"nobody's").unwrap();
        assert_eq!(kept(folder.path()), [1, orphan]);

        let shell = start(folder.path());
        drop(shell);
        assert_eq!(kept(folder.path()), [1], "the one a tab names is kept");
    });
}

#[test]
fn a_tab_that_becomes_a_site_has_no_state() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, HOME).await;
        shell.open_tab(window, MAIL).await;
        shell.settle().await;
        assert_eq!(shell.engine.states_asked.len(), 1, "the first has a state");
        // It is kept, behind as it is: a Site sleeps at its home (039), and the state it
        // had as a tab is dropped.
        shell.keep_tab(first).await;
        shell.settle().await;
        drop(shell);
        assert_eq!(kept(folder.path()), [] as [i64; 0]);

        // It comes back a Site, and wakes by being loaded.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let site = shell.window(subscription).tabs[0].tab_id;
        shell.activate_tab(site).await;
        assert_eq!(shell.engine.restores.len(), 0);
        assert_eq!(
            asked(&shell),
            [format!("load {MAIL}"), format!("load {HOME}")]
        );
    });
}

#[test]
fn a_state_that_is_said_late_is_not_kept_for_a_tab_that_is_elsewhere_by_then() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, HOME).await;
        // The engine is slow to say: the first tab is asked when it leaves the front, and
        // has gone on to another page before the answer comes.
        shell.hold_states = true;
        let second = shell.open_tab(window, MAIL).await;
        shell.activate_tab(first).await;
        shell.submit(first, NEWS).await;
        assert_eq!(shell.engine.states_asked.len(), 0, "nothing is said yet");
        shell.release_states();
        shell.settle().await;
        // Two were asked: the first, which is elsewhere now, and the second, which left
        // the front after it and is where it was. Only the second's is kept.
        assert_eq!(shell.engine.states_asked.len(), 2);
        drop(shell);
        assert_eq!(kept(folder.path()), [2]);

        // One that became a Site before its answer came has none.
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        let third = shell.open_tab(window, WIKI).await;
        shell.settle().await;
        shell.hold_states = true;
        shell.open_tab(window, HOME).await;
        shell.keep_tab(third).await;
        shell.release_states();
        shell.settle().await;
        let _ = (second, subscription);
        drop(shell);
        // The second's own is still there: it was asleep, and nobody woke it. The first's
        // (3) was said at once, when the third tab came in front of it. The third's, which
        // the store numbered 4, is not there.
        assert_eq!(kept(folder.path()), [2, 3]);

        // And one whose history was forgotten before its answer came.
        let mut shell = start(folder.path());
        let (window, _, _) = shell.open_window().await;
        let fourth = shell.open_tab(window, WIKI).await;
        shell.settle().await;
        shell.hold_states = true;
        shell.open_tab(window, NEWS).await;
        forget_everything(&mut shell, window).await;
        shell.release_states();
        shell.settle().await;
        let _ = fourth;
        drop(shell);
        // Nor has any other tab of these logins the one it had.
        assert_eq!(kept(folder.path()), [] as [i64; 0]);
    });
}

#[test]
fn a_window_that_closes_takes_its_tabs_states_and_the_sessions_window_keeps_them() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        // The first window is the session's. Its first tab has left the front.
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, HOME).await;
        shell.open_tab(window, MAIL).await;
        // A second window, and the same in it.
        let (other, other_first, _) = shell.open_window().await;
        shell.submit(other_first, NEWS).await;
        shell.open_tab(other, WIKI).await;
        shell.settle().await;
        assert_eq!(shell.engine.states_asked.len(), 2);
        // The second window closes: its tabs are gone, and the state with them.
        close_window(&mut shell, other).await;
        shell.settle().await;
        drop(shell);
        assert_eq!(kept(folder.path()), [1]);

        // The session's window keeps its tabs when it closes (009), and they their states.
        let mut shell = start(folder.path());
        let (window, _, _) = shell.open_window().await;
        close_window(&mut shell, window).await;
        shell.settle().await;
        drop(shell);
        assert_eq!(kept(folder.path()), [1]);
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let first = shell.window(subscription).tabs[0].tab_id;
        shell.activate_tab(first).await;
        assert_eq!(shell.engine.restores.len(), 1, "it wakes from its state");
    });
}

#[test]
fn forgetting_history_takes_the_states_of_its_logins_tabs_and_leaves_the_others() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        // Another Space, with logins of its own: the first Space's tab is out of sight, and
        // has a state. In the new Space, a tab at Mail and then one at News in front of it.
        show_a_new_space(&mut shell, window, "Work").await;
        let work_first = front(&shell, subscription).tab_id;
        shell.submit(work_first, MAIL).await;
        let work_second = shell.open_tab(window, NEWS).await;
        shell.settle().await;
        assert_eq!(shell.engine.states_asked.len(), 2);

        // Work's history is forgotten. A state holds the list a tab goes back through:
        // the one its tab had is gone, and the tabs that were open are asked for no other,
        // whichever leaves the front: their web views still hold their lists.
        let of_work = format!("{MAIL}\n#");
        let of_home = format!("{HOME}\n#");
        assert!(holds(folder.path(), &of_work) && holds(folder.path(), &of_home));
        forget_everything(&mut shell, window).await;
        // Gone from the store's files with the answer, as the visits are (008): from the
        // database and from its write-ahead log. The other Space's is there.
        assert!(!holds(folder.path(), &of_work), "Work's tab's state");
        assert!(holds(folder.path(), &of_home));
        shell.activate_tab(work_first).await;
        shell.activate_tab(work_second).await;
        assert_eq!(shell.engine.states_asked.len(), 2);
        // A tab opened since has nothing forgotten in its list: it is asked as any is.
        let work_third = shell.open_tab(window, WIKI).await;
        let third_view = front(&shell, subscription).web_view_id;
        shell.activate_tab(work_first).await;
        shell.settle().await;
        assert_eq!(shell.engine.states_asked.len(), 3);
        assert_eq!(shell.engine.states_asked[2], third_view);
        drop(shell);
        // The other Space's tab keeps its own (1): it was not its history. Work's first
        // tab's (2) is gone; the new tab's (3) is there.
        assert_eq!(kept(folder.path()), [1, 3]);

        // After a start the tabs have new web views: one that was open when history was
        // forgotten is asked again. And a tab that is asleep with a state loses it when
        // history is forgotten, and wakes by being loaded.
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        let tabs = shell.window(subscription).tabs.clone();
        assert_eq!(tabs.len(), 3, "Work, as it was left");
        forget_everything(&mut shell, window).await;
        shell.activate_tab(tabs[2].tab_id).await;
        assert_eq!(shell.engine.restores.len(), 0);
        assert_eq!(asked(&shell).last().unwrap(), &format!("load {WIKI}"));
        // The tab that was in front at the start had a web view when history was forgotten:
        // it has left the front now, and was not asked. The one that woke since is.
        assert_eq!(shell.engine.states_asked.len(), 0);
        shell.activate_tab(tabs[1].tab_id).await;
        shell.settle().await;
        assert_eq!(shell.engine.states_asked.len(), 1);
        let _ = (work_third, work_second);
        drop(shell);
        assert_eq!(kept(folder.path()), [1, 4]);
    });
}

#[test]
fn a_state_that_is_said_late_is_not_kept_for_a_tab_whose_load_has_failed_since() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        // The first is asked as it leaves the front; before its engine says, it is in
        // front again and sent to a page that does not load. It is shown as going there,
        // and its web view is still at Home: the state would wake it at the wrong page.
        shell.hold_states = true;
        shell.open_tab(window, MAIL).await;
        shell.activate_tab(first).await;
        shell.engine.fail(NEWS, "net.offline");
        shell.submit(first, NEWS).await;
        assert_eq!(
            front(&shell, subscription).load_state().as_str_name(),
            "LOAD_STATE_FAILED"
        );
        shell.release_states();
        shell.settle().await;
        drop(shell);
        // The second tab's only, which the store numbered 2.
        assert_eq!(kept(folder.path()), [2]);

        // It comes back where it was going (009), by being loaded.
        let mut shell = start(folder.path());
        shell.open_window().await;
        assert_eq!(asked(&shell), [format!("load {NEWS}")]);
    });
}

#[test]
fn a_tab_that_goes_on_by_itself_or_is_sent_where_it_never_arrives_has_no_state() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let first_view = front(&shell, subscription).web_view_id;
        shell.open_tab(window, MAIL).await;
        let second_view = front(&shell, subscription).web_view_id;
        let third = shell.open_tab(window, WIKI).await;
        shell.open_tab(window, HOME).await;
        shell.settle().await;
        // Three tabs behind, each with a state: 1, 2 and 3.
        assert_eq!(shell.engine.states_asked.len(), 3);

        // The first's page goes on to another by itself, behind as it is: the state is of
        // a page the tab has left.
        for report in shell.engine.goes_to(first_view, NEWS) {
            shell.report(report);
        }
        // The second's page changes its address without a load: its list has a place more
        // than the state has.
        shell.report(FakeEngine::address_changed(second_view, INBOX));
        shell.settle().await;
        // The third is brought in front and sent to a page that does not load. It is shown
        // as going there, and nothing arrives to say its state is old: it went when the tab
        // was sent. (The fourth has left the front, and has a state: 4.)
        shell.activate_tab(third).await;
        shell.engine.fail(NOWHERE, "net.offline");
        shell.submit(third, NOWHERE).await;
        shell.settle().await;
        drop(shell);
        assert_eq!(kept(folder.path()), [4]);

        // Each comes back by being loaded, where it is or was going.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let tabs = shell.window(subscription).tabs.clone();
        shell.activate_tab(tabs[0].tab_id).await;
        shell.activate_tab(tabs[1].tab_id).await;
        assert_eq!(
            asked(&shell),
            [
                format!("load {NOWHERE}"),
                format!("load {NEWS}"),
                format!("load {INBOX}")
            ]
        );
        assert_eq!(shell.engine.restores.len(), 0);
    });
}
