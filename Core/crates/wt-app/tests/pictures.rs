//! A sleeping tab's picture (059), through the contract, against the
//! scripted engine: when a picture is asked for, which is kept and where,
//! when a tab is shown with it and until when, and what takes it away.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{
    ActivateTab, ForgetHistory, LoadState, MemoryLevel, MemoryShort, PagePicture, SubmitAddress,
    TabSummary, TabView, forget_history, host_request, request, response,
};
use wt_store::{KeptPicture, Store};
use wt_testkit::{FakeShell, picture_of, run};

const HOME: &str = "https://example.com/";
const NEWS: &str = "https://news.example.com/";
const MAIL: &str = "https://mail.example.com/";
const WIKI: &str = "https://wiki.example.com/";

/// The minutes a tab is out of sight before it sleeps (053 D1).
const SLEEP_AFTER: usize = 30;

/// The most bytes a picture is, and the most pixels wide (059 D4).
const MOST_BYTES: usize = 96 * 1024;
const MOST_WIDE: u32 = 800;

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

/// The same, with the first tab asleep by its time.
async fn one_asleep(shell: &mut FakeShell) -> (u64, u64, u64, u64, u64) {
    let tabs = two_tabs(shell).await;
    minutes(shell, SLEEP_AFTER).await;
    assert_eq!(asleep(shell, tabs.3), [true, false]);
    shell.settle().await;
    tabs
}

/// Brings a tab in front with the shell slow to begin anything: the tab is
/// woken, and its page has not arrived.
async fn wake_slowly(shell: &mut FakeShell, tab: u64) {
    shell.hold_host = true;
    shell.send(request::Kind::ActivateTab(ActivateTab {
        tab_id: tab,
        chosen_while_typing: false,
    }));
    shell.settle().await;
}

/// The shell does what it was asked: the page arrives.
async fn arrive(shell: &mut FakeShell) {
    shell.release_host();
    shell.settle().await;
}

fn store(folder: &Path) -> Store {
    Store::open(&folder.join("webtiger.sqlite")).unwrap()
}

/// The pictures the store has, by the ids of their states: of the first ten.
fn pictured(folder: &Path) -> Vec<(i64, KeptPicture)> {
    let store = store(folder);
    (1..=10)
        .filter_map(|id| Some((id, store.kept_picture(id).unwrap()?)))
        .collect()
}

/// The states the store has, by their ids: of the first ten.
fn kept(folder: &Path) -> Vec<i64> {
    let store = store(folder);
    (1..=10)
        .filter(|id| store.kept_state(*id).unwrap().is_some())
        .collect()
}

fn of(url: &str, width: u32, height: u32) -> KeptPicture {
    KeptPicture {
        jpeg: picture_of(url),
        width,
        height,
    }
}

/// Whether a file of the store's holds these words.
fn holds(folder: &Path, needle: &str) -> bool {
    std::fs::read_dir(folder)
        .unwrap()
        .map(|entry| std::fs::read(entry.unwrap().path()).unwrap())
        .any(|bytes| {
            bytes
                .windows(needle.len())
                .any(|window| window == needle.as_bytes())
        })
}

#[test]
fn a_tab_put_to_sleep_is_asked_for_a_picture_which_is_kept_with_its_state() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, _, _, subscription, first_view) = two_tabs(&mut shell).await;
        shell.settle().await;
        // Leaving the front, a tab is asked for its state (051), and for no picture: most
        // tabs that leave the front come back before they sleep.
        assert_eq!(shell.engine.states_asked, [first_view]);
        assert_eq!(shell.engine.pictures_asked, []);
        assert_eq!(pictured(folder.path()), []);
        minutes(&mut shell, SLEEP_AFTER - 1).await;
        assert_eq!(shell.engine.pictures_asked, []);

        // As it sleeps: asked once, of that web view, for a picture no wider and no larger
        // than the most. It was asked before the web view was let go: a web view that is
        // gone has no picture, and this one gave its own.
        minutes(&mut shell, 1).await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        shell.settle().await;
        assert_eq!(
            shell.engine.pictures_asked,
            [(first_view, MOST_WIDE, u32::try_from(MOST_BYTES).unwrap())]
        );
        assert!(!shell.engine.web_views.contains_key(&first_view));
        // It is with the state the tab sleeps on (2, in the place of the one from when it
        // left the front), and nowhere else.
        assert_eq!(kept(folder.path()), [2]);
        assert_eq!(pictured(folder.path()), [(2, of(HOME, 800, 600))]);
        // The sleeping tab is shown as it was: nothing of the picture is in the row.
        assert_eq!(front(&shell, subscription).picture_id, 0);
        // Nor is it asked again, however long the tab sleeps.
        minutes(&mut shell, 2 * SLEEP_AFTER).await;
        assert_eq!(shell.engine.pictures_asked.len(), 1);
    });
}

#[test]
fn a_tab_that_wakes_is_shown_with_its_picture_until_its_page_arrives() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, first, second, subscription, _) = one_asleep(&mut shell).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);

        // Pressed, with a shell that is slow to begin: the tab is in front, on its way to
        // its page, and says which picture is to stand where the page will be.
        wake_slowly(&mut shell, first).await;
        let waking = front(&shell, subscription);
        assert_eq!(waking.tab_id, first);
        assert_eq!(waking.load_state, LoadState::Loading as i32);
        assert_eq!(waking.picture_id, 2, "its state's number");
        // The address is the one it slept at, and is not yet a page's: it has no mark of a
        // page that arrived.
        assert_eq!(waking.address.as_ref().unwrap().url, HOME);
        // The state is the store's no longer, and the picture went with it: it is held for
        // the tab that is shown with it, and asked for by its number. (The state the store
        // has now, 3, is of the tab that has just left the front.)
        assert_eq!(kept(folder.path()), [3]);
        assert_eq!(pictured(folder.path()), []);
        let answer = shell.send(request::Kind::QueryPictures(wt_proto::v1::QueryPictures {
            picture_ids: vec![2],
        }));
        shell.settle().await;
        let Some(response::Kind::PicturesPage(page)) = shell.take_response(answer) else {
            panic!("pictures are answered while the shell is busy with the page");
        };
        assert_eq!(
            page.pictures,
            [PagePicture {
                picture_id: 2,
                jpeg: picture_of(HOME),
                width: 800,
                height: 600,
            }]
        );
        // Only the tab in front says a picture: the row does not.
        assert_eq!(asleep(&shell, subscription), [false, false]);

        // The page arrives: the tab says no picture, and there is none to ask for.
        arrive(&mut shell).await;
        let woken = front(&shell, subscription);
        assert_eq!(
            (woken.title.as_str(), woken.load_state, woken.picture_id),
            ("Home", LoadState::Idle as i32, 0)
        );
        assert!(woken.can_go_back, "woken from its state");
        assert_eq!(shell.query_pictures(&[2]).await, []);

        // Behind and in front again, awake: no picture. And asleep again, it has another,
        // by another number.
        shell.activate_tab(second).await;
        shell.activate_tab(first).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
        shell.activate_tab(second).await;
        minutes(&mut shell, SLEEP_AFTER).await;
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        let again = pictured(folder.path());
        assert_eq!(again.len(), 1);
        assert!(again[0].0 > 2, "a number is not used twice");
        assert_eq!(again[0].1, of(HOME, 800, 600));
        wake_slowly(&mut shell, first).await;
        assert_eq!(
            front(&shell, subscription).picture_id,
            u64::try_from(again[0].0).unwrap()
        );
        arrive(&mut shell).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
    });
}

#[test]
fn a_tab_that_did_not_sleep_or_slept_with_no_picture_is_shown_with_none() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        shell.engine.gives_no_picture = true;
        let (window, first, _, subscription, first_view) = one_asleep(&mut shell).await;
        // Asked, and given none: the tab wakes from its state with nothing before its page.
        assert_eq!(shell.engine.pictures_asked.len(), 1);
        assert_eq!(shell.engine.pictures_asked[0].0, first_view);
        wake_slowly(&mut shell, first).await;
        let waking = front(&shell, subscription);
        assert_eq!(
            (waking.tab_id, waking.load_state, waking.picture_id),
            (first, LoadState::Loading as i32, 0)
        );
        arrive(&mut shell).await;
        assert!(
            front(&shell, subscription).can_go_back,
            "from its state all the same"
        );

        // A tab that is sent somewhere, and a new one, are on their way with no picture.
        shell.engine.gives_no_picture = false;
        shell.hold_host = true;
        shell.send(request::Kind::SubmitAddress(SubmitAddress {
            tab_id: first,
            text: WIKI.to_owned(),
        }));
        shell.settle().await;
        assert_eq!(
            (
                front(&shell, subscription).load_state,
                front(&shell, subscription).picture_id
            ),
            (LoadState::Loading as i32, 0)
        );
        arrive(&mut shell).await;
        shell.open_tab(window, NEWS).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
    });
}

#[test]
fn a_picture_that_is_too_large_or_too_wide_or_says_no_size_is_not_kept() {
    // What the engine gives beyond the words of its picture, the size it says, and whether
    // it is kept.
    let words = picture_of(HOME).len();
    let cases: [(usize, (u32, u32), bool); 8] = [
        (0, (800, 600), true),
        (MOST_BYTES - words, (800, 600), true),
        (MOST_BYTES - words + 1, (800, 600), false),
        (0, (801, 600), false),
        (0, (0, 600), false),
        (0, (800, 0), false),
        (0, (1, 1), true),
        (0, (800, 20_000), true),
    ];
    for (padding, size, is_kept) in cases {
        run(async move {
            let folder = tempfile::tempdir().unwrap();
            let mut shell = FakeShell::live(folder.path());
            web(&mut shell);
            shell.engine.picture_padding = padding;
            shell.engine.picture_size = size;
            let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
            let case = format!("{padding} bytes more, {size:?}");
            assert_eq!(shell.engine.pictures_asked.len(), 1, "{case}");
            // Its state is kept either way: the tab wakes from it.
            assert_eq!(kept(folder.path()), [2], "{case}");
            let has = pictured(folder.path());
            assert_eq!(has.len(), usize::from(is_kept), "{case}");
            if let Some((state, picture)) = has.first() {
                assert_eq!(*state, 2, "{case}");
                assert_eq!(picture.jpeg.len(), words + padding, "{case}");
                assert_eq!((picture.width, picture.height), size, "{case}");
            }
            wake_slowly(&mut shell, first).await;
            assert_eq!(
                front(&shell, subscription).picture_id,
                if is_kept { 2 } else { 0 },
                "{case}"
            );
            arrive(&mut shell).await;
            assert!(front(&shell, subscription).can_go_back, "{case}");
        });
    }
}

#[test]
fn a_tab_with_no_state_to_sleep_on_keeps_no_picture_and_one_with_an_older_state_keeps_it_there() {
    run(async {
        // An engine that gives no state: the tab sleeps on none. Its picture is asked for
        // and has nowhere to be kept; the tab wakes at its address with nothing before it.
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        shell.engine.gives_no_state = true;
        let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
        assert_eq!(shell.engine.pictures_asked.len(), 1);
        assert_eq!(kept(folder.path()), [] as [i64; 0]);
        assert_eq!(pictured(folder.path()), []);
        assert!(!holds(folder.path(), "scripted engine's picture"));
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
        arrive(&mut shell).await;
        assert!(!front(&shell, subscription).can_go_back, "at its address");
    });
    run(async {
        // A state too large to keep as the tab sleeps is none; so is none given. The state
        // from when the tab left the front (1) is of the same page, and is its still: the
        // picture is kept with that one.
        for too_large in [false, true] {
            let folder = tempfile::tempdir().unwrap();
            let mut shell = FakeShell::live(folder.path());
            web(&mut shell);
            let (_, first, _, subscription, _) = two_tabs(&mut shell).await;
            shell.settle().await;
            assert_eq!(kept(folder.path()), [1]);
            if too_large {
                shell.engine.state_padding = 132_000;
            } else {
                shell.engine.gives_no_state = true;
            }
            minutes(&mut shell, SLEEP_AFTER).await;
            shell.settle().await;
            assert_eq!(asleep(&shell, subscription), [true, false]);
            assert_eq!(kept(folder.path()), [1], "too large: {too_large}");
            assert_eq!(pictured(folder.path()), [(1, of(HOME, 800, 600))]);
            wake_slowly(&mut shell, first).await;
            assert_eq!(front(&shell, subscription).picture_id, 1);
            arrive(&mut shell).await;
            assert_eq!(front(&shell, subscription).picture_id, 0);
        }
    });
}

#[test]
fn a_picture_that_comes_late_is_kept_for_a_tab_still_asleep_and_not_for_one_that_was_woken() {
    run(async {
        // The shell is slow to draw. The tab sleeps on its state; its picture has not come.
        // (A core that waits an hour for a shell: an answer slower than the core waits is
        // no picture, which is not what is tried here.)
        let folder = tempfile::tempdir().unwrap();
        let patient = std::time::Duration::from_secs(3_600);
        let mut shell =
            FakeShell::start_with(&FakeShell::live_config(folder.path()), patient).unwrap();
        web(&mut shell);
        shell.hold_pictures = true;
        let (_, first, second, subscription, _) = one_asleep(&mut shell).await;
        assert_eq!(kept(folder.path()), [2]);
        assert_eq!(pictured(folder.path()), []);
        // It comes while the tab is asleep still: kept, of the page as it was when asked.
        shell.release_pictures();
        shell.settle().await;
        assert_eq!(pictured(folder.path()), [(2, of(HOME, 800, 600))]);
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 2);
        arrive(&mut shell).await;

        // Asleep again, and woken before its picture comes: it wakes with none, and the one
        // that comes afterwards is nobody's. It is not kept, and the tab is not shown with
        // it.
        shell.activate_tab(second).await;
        shell.hold_pictures = true;
        minutes(&mut shell, SLEEP_AFTER).await;
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        let state = *kept(folder.path()).last().unwrap();
        assert_eq!(pictured(folder.path()), []);
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
        shell.release_pictures();
        shell.settle().await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
        assert_eq!(pictured(folder.path()), []);
        assert_eq!(
            shell.query_pictures(&[u64::try_from(state).unwrap()]).await,
            []
        );
        arrive(&mut shell).await;
        assert_eq!(pictured(folder.path()), []);
    });
}

#[test]
fn a_site_is_asked_for_no_picture_and_whatever_puts_a_tab_to_sleep_asks_for_one() {
    run(async {
        // A Site sleeps at its home, with no state (051 D4): a picture of where it had
        // wandered would be of another page.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, _) = two_tabs(&mut shell).await;
        shell.keep_tab(first).await;
        minutes(&mut shell, SLEEP_AFTER).await;
        shell.settle().await;
        assert!(row(&shell, subscription)[0].asleep);
        assert_eq!(shell.engine.pictures_asked, []);
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
        arrive(&mut shell).await;
    });
    run(async {
        // The system short of memory (055) puts a tab to sleep as the time does: with its
        // picture.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, first_view) = two_tabs(&mut shell).await;
        shell
            .request(request::Kind::MemoryShort(MemoryShort {
                level: MemoryLevel::Short as i32,
            }))
            .await;
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        assert_eq!(shell.engine.pictures_asked.len(), 1);
        assert_eq!(shell.engine.pictures_asked[0].0, first_view);
        wake_slowly(&mut shell, first).await;
        assert_ne!(front(&shell, subscription).picture_id, 0);
        arrive(&mut shell).await;
    });
    run(async {
        // Very short of memory, a tab is asked for its state and for no picture: drawing one
        // takes memory, and time before the web view is let go. It wakes from its state with
        // nothing before its page. (And "Sleep All Others", which is the person's and not
        // the system's, takes pictures: 056.)
        for (very, others) in [(true, false), (false, true)] {
            let mut shell = FakeShell::fake();
            web(&mut shell);
            let (window, first, _, subscription, first_view) = two_tabs(&mut shell).await;
            let asked_before = shell.engine.states_asked.len();
            if others {
                shell
                    .request(request::Kind::SleepOthers(wt_proto::v1::SleepOthers {
                        window_id: window,
                    }))
                    .await;
            } else {
                shell
                    .request(request::Kind::MemoryShort(MemoryShort {
                        level: MemoryLevel::VeryShort as i32,
                    }))
                    .await;
            }
            shell.settle().await;
            assert_eq!(asleep(&shell, subscription), [true, false]);
            assert_eq!(shell.engine.states_asked.len(), asked_before + 1);
            assert_eq!(shell.engine.states_asked.last(), Some(&first_view));
            assert_eq!(shell.engine.pictures_asked.len(), usize::from(!very));
            wake_slowly(&mut shell, first).await;
            assert_eq!(front(&shell, subscription).picture_id == 0, very);
            arrive(&mut shell).await;
            assert!(front(&shell, subscription).can_go_back, "from its state");
        }
    });
    run(async {
        // The limit of live tabs (054) puts a tab to sleep as the time does: with its
        // picture. Two may be alive; a third arrives, and the first goes.
        let mut config = FakeShell::config(wt_proto::v1::EngineKind::Webkit);
        config.fake = Some(wt_proto::v1::FakeOptions {
            live_tabs: 2,
            ..wt_proto::v1::FakeOptions::default()
        });
        let mut shell = FakeShell::start(&config).unwrap();
        web(&mut shell);
        let (window, first, _, subscription, first_view) = two_tabs(&mut shell).await;
        assert_eq!(shell.engine.pictures_asked, []);
        shell.open_tab(window, WIKI).await;
        shell.settle().await;
        assert_eq!(asleep(&shell, subscription), [true, false, false]);
        assert_eq!(shell.engine.pictures_asked.len(), 1);
        assert_eq!(shell.engine.pictures_asked[0].0, first_view);
        wake_slowly(&mut shell, first).await;
        assert_ne!(front(&shell, subscription).picture_id, 0);
        arrive(&mut shell).await;
    });
    run(async {
        // A private window's tabs do not sleep (037), and are asked for nothing.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, _) = shell.open_private_window().await;
        shell.submit(first, HOME).await;
        shell.open_tab(window, NEWS).await;
        minutes(&mut shell, 2 * SLEEP_AFTER).await;
        assert_eq!(shell.engine.pictures_asked, []);
        assert_eq!(shell.engine.states_asked, [] as [u64; 0]);
    });
}

#[test]
fn a_picture_is_kept_over_a_start_with_its_state() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        one_asleep(&mut shell).await;
        drop(shell);
        assert_eq!(pictured(folder.path()), [(2, of(HOME, 800, 600))]);

        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(asleep(&shell, subscription), [true, false]);
        // A start drops what no tab names, and this is named.
        assert_eq!(pictured(folder.path()), [(2, of(HOME, 800, 600))]);
        let first = row(&shell, subscription)[0].tab_id;
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 2);
        let answer = shell.send(request::Kind::QueryPictures(wt_proto::v1::QueryPictures {
            picture_ids: vec![2],
        }));
        shell.settle().await;
        let Some(response::Kind::PicturesPage(page)) = shell.take_response(answer) else {
            panic!("pictures are answered");
        };
        assert_eq!(page.pictures.len(), 1);
        assert_eq!(page.pictures[0].jpeg, picture_of(HOME));
        arrive(&mut shell).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
        assert!(front(&shell, subscription).can_go_back);
    });
}

async fn forget_everything(shell: &mut FakeShell, window: u64) {
    let asked = shell.send(request::Kind::ForgetHistory(ForgetHistory {
        window_id: window,
        what: Some(forget_history::What::Everything(true)),
    }));
    shell.settle().await;
    let answer = shell.take_response(asked);
    assert!(matches!(answer, Some(response::Kind::Ack(_))), "{answer:?}");
}

#[test]
fn forgetting_history_takes_a_sleeping_tabs_picture_from_the_files_and_from_a_waking_tab() {
    run(async {
        // Asleep with its picture: forgotten, the picture is gone with the state, from the
        // store's files and not only from its table; the tab wakes at its address with
        // nothing before its page.
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (window, first, _, subscription, _) = one_asleep(&mut shell).await;
        assert!(holds(folder.path(), "scripted engine's picture"));
        forget_everything(&mut shell, window).await;
        assert_eq!(pictured(folder.path()), []);
        assert_eq!(kept(folder.path()), [] as [i64; 0]);
        assert!(!holds(folder.path(), "scripted engine's picture"));
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
        arrive(&mut shell).await;
    });
    run(async {
        // On its way, shown with its picture: forgotten, the tab says no picture, at once,
        // and the picture is asked for in vain. It was in no file by then.
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (window, first, _, subscription, _) = one_asleep(&mut shell).await;
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 2);
        // Forgotten, and asked for in the same breath: nothing is between the two.
        let forgetting = shell.send(request::Kind::ForgetHistory(ForgetHistory {
            window_id: window,
            what: Some(forget_history::What::Everything(true)),
        }));
        let asked = shell.send(request::Kind::QueryPictures(wt_proto::v1::QueryPictures {
            picture_ids: vec![2],
        }));
        shell.settle().await;
        assert!(matches!(
            shell.take_response(forgetting),
            Some(response::Kind::Ack(_))
        ));
        assert_eq!(
            shell.take_response(asked),
            Some(response::Kind::PicturesPage(wt_proto::v1::PicturesPage {
                pictures: Vec::new()
            }))
        );
        assert_eq!(front(&shell, subscription).picture_id, 0);
        assert!(!holds(folder.path(), "scripted engine's picture"));
        arrive(&mut shell).await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
    });
}

#[test]
fn the_picture_goes_when_the_page_fails_or_the_tab_is_sent_elsewhere_or_closed() {
    run(async {
        // The page does not come: what is shown is the failure, and no picture.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
        // (An engine that will not take the state loads the address, which fails.)
        shell.engine.refuses_states = true;
        shell.engine.fail(HOME, "net.dns");
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 2);
        arrive(&mut shell).await;
        let failed = front(&shell, subscription);
        assert_eq!(
            (
                failed.load_state,
                failed.error_code.as_str(),
                failed.picture_id
            ),
            (LoadState::Failed as i32, "net.dns", 0)
        );
        assert_eq!(shell.query_pictures(&[2]).await, []);
    });
    run(async {
        // Sent somewhere else on its way: the picture is of where it was, and goes at once.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 2);
        shell.send(request::Kind::SubmitAddress(SubmitAddress {
            tab_id: first,
            text: WIKI.to_owned(),
        }));
        shell.settle().await;
        let sent = front(&shell, subscription);
        assert_eq!(
            (sent.load_state, sent.picture_id),
            (LoadState::Loading as i32, 0)
        );
        arrive(&mut shell).await;
        assert_eq!(shell.query_pictures(&[2]).await, []);
        assert_eq!(front(&shell, subscription).title, "Wiki");
    });
    run(async {
        // The load stopped: the picture stood in for a page that is not coming.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
        wake_slowly(&mut shell, first).await;
        let waking = front(&shell, subscription);
        assert_eq!(waking.picture_id, 2);
        shell.report(wt_testkit::FakeEngine::navigation_failed(
            waking.web_view_id,
            "net.cancelled",
            HOME,
        ));
        shell.settle().await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
    });
    run(async {
        // The page arrives, and is loading still: the picture goes when the page is there,
        // and not when the page is done.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
        wake_slowly(&mut shell, first).await;
        let waking = front(&shell, subscription);
        assert_eq!(waking.picture_id, 2);
        shell.report(wt_testkit::FakeEngine::navigation_started(
            waking.web_view_id,
            HOME,
        ));
        shell.report(wt_testkit::FakeEngine::navigation_committed(
            waking.web_view_id,
            HOME,
        ));
        shell.settle().await;
        let arrived = front(&shell, subscription);
        assert_eq!(
            (arrived.load_state, arrived.picture_id),
            (LoadState::Loading as i32, 0)
        );
    });
    run(async {
        // What was drawing the page is gone: what is shown is that.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
        wake_slowly(&mut shell, first).await;
        let waking = front(&shell, subscription);
        assert_eq!(waking.picture_id, 2);
        shell.report(wt_testkit::FakeEngine::renderer_gone(waking.web_view_id));
        shell.settle().await;
        let gone = front(&shell, subscription);
        assert_eq!(
            (gone.load_state, gone.picture_id),
            (LoadState::Failed as i32, 0)
        );
    });
    run(async {
        // The load ended and no page came of it: nothing more is coming that the picture
        // could stand in for.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
        wake_slowly(&mut shell, first).await;
        let waking = front(&shell, subscription);
        assert_eq!(waking.picture_id, 2);
        // What an engine says of a load that goes on is nothing to the picture.
        shell.report(wt_testkit::FakeEngine::navigation_started(
            waking.web_view_id,
            HOME,
        ));
        shell.report(wt_testkit::FakeEngine::progress(waking.web_view_id, 0.5));
        shell.report(wt_testkit::FakeEngine::title(waking.web_view_id, "Coming"));
        shell.settle().await;
        assert_eq!(front(&shell, subscription).picture_id, 2);
        shell.report(wt_testkit::FakeEngine::navigation_finished(
            waking.web_view_id,
        ));
        shell.settle().await;
        assert_eq!(front(&shell, subscription).picture_id, 0);
    });
    run(async {
        // Behind while it is on its way, and in front again: on its way still, with its
        // picture still. Closed on its way: there is none to ask for.
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, second, subscription, _) = one_asleep(&mut shell).await;
        wake_slowly(&mut shell, first).await;
        for tab in [second, first] {
            shell.send(request::Kind::ActivateTab(ActivateTab {
                tab_id: tab,
                chosen_while_typing: false,
            }));
            shell.settle().await;
        }
        assert_eq!(front(&shell, subscription).tab_id, first);
        assert_eq!(front(&shell, subscription).picture_id, 2);
        let asked = shell.send(request::Kind::QueryPictures(wt_proto::v1::QueryPictures {
            picture_ids: vec![2],
        }));
        shell.settle().await;
        let Some(response::Kind::PicturesPage(page)) = shell.take_response(asked) else {
            panic!("pictures are answered");
        };
        assert_eq!(page.pictures.len(), 1);
        shell.send(request::Kind::CloseTab(wt_proto::v1::CloseTab {
            tab_id: first,
        }));
        shell.settle().await;
        let asked = shell.send(request::Kind::QueryPictures(wt_proto::v1::QueryPictures {
            picture_ids: vec![2],
        }));
        shell.settle().await;
        let Some(response::Kind::PicturesPage(page)) = shell.take_response(asked) else {
            panic!("pictures are answered");
        };
        assert_eq!(page.pictures, []);
        arrive(&mut shell).await;
    });
}

#[test]
fn pictures_are_answered_in_the_order_asked_and_no_more_than_an_answer_carries() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, first, _, subscription, _) = one_asleep(&mut shell).await;
        wake_slowly(&mut shell, first).await;
        assert_eq!(front(&shell, subscription).picture_id, 2);
        let mut ask = async |numbers: Vec<u64>| {
            let asked = shell.send(request::Kind::QueryPictures(wt_proto::v1::QueryPictures {
                picture_ids: numbers,
            }));
            shell.settle().await;
            let Some(response::Kind::PicturesPage(page)) = shell.take_response(asked) else {
                panic!("pictures are answered");
            };
            page.pictures
                .iter()
                .map(|picture| picture.picture_id)
                .collect::<Vec<_>>()
        };
        // None asked, none answered; a number the core has not is left out; one asked twice
        // is answered twice.
        assert_eq!(ask(Vec::new()).await, [] as [u64; 0]);
        assert_eq!(ask(vec![0, 1, 3, 99]).await, [] as [u64; 0]);
        assert_eq!(ask(vec![7, 2, 9, 2]).await, [2, 2]);
        // The first eight asked are looked at, and no more.
        assert_eq!(ask(vec![9, 9, 9, 9, 9, 9, 9, 2]).await, [2]);
        assert_eq!(ask(vec![9, 9, 9, 9, 9, 9, 9, 9, 2]).await, [] as [u64; 0]);
    });
}

/// What the shell was asked of web views, in order, by the kind of each: the
/// state and the picture are the scripted engine's to answer and are listed
/// by it.
#[test]
fn a_sleeping_tabs_web_view_is_let_go_as_before_and_the_row_is_as_it_was() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, _, _, subscription, first_view) = one_asleep(&mut shell).await;
        let destroyed: Vec<u64> = shell
            .host_requests
            .iter()
            .filter_map(|request| match request {
                host_request::Kind::WebViewDestroy(destroy) => Some(destroy.web_view_id),
                _ => None,
            })
            .collect();
        assert_eq!(destroyed, [first_view]);
        let behind = &row(&shell, subscription)[0];
        assert_eq!(
            (behind.title.as_str(), behind.host.as_str(), behind.asleep),
            ("Home", "example.com", true)
        );
    });
}
