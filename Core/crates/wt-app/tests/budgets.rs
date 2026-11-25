//! The plan's budgets that are counts (000 §7, 060): things that are the
//! same on every machine, held at the budget itself. Bytes on disk, pages
//! loaded at a start. (What the core holds in memory is counted in
//! `wt-ffi`'s `held` test, under an allocator that counts.)
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{SleepOthers, host_request, request};
use wt_store::Store;
use wt_testkit::{
    FakeShell, SITES_A_SPACE, SPACES, TABS_A_SPACE, run, standard_pages, standard_session,
};

fn store(folder: &Path) -> Store {
    Store::open(&folder.join("webtiger.sqlite")).unwrap()
}

/// The bytes of every file in a folder: a store, and whatever it keeps
/// beside itself.
fn bytes_in(folder: &Path) -> u64 {
    std::fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().metadata().unwrap().len())
        .sum()
}

/// What the shell was asked of web views, by kind: made, loaded (by an
/// address or from a state), let go.
fn asked(shell: &FakeShell) -> (usize, usize, usize) {
    let count = |wanted: fn(&host_request::Kind) -> bool| {
        shell
            .host_requests
            .iter()
            .filter(|kind| wanted(kind))
            .count()
    };
    (
        count(|kind| matches!(kind, host_request::Kind::WebViewCreate(_))),
        count(|kind| {
            matches!(
                kind,
                host_request::Kind::WebViewLoad(_) | host_request::Kind::WebViewRestore(_)
            )
        }),
        count(|kind| matches!(kind, host_request::Kind::WebViewDestroy(_))),
    )
}

#[test]
fn the_standard_session_is_five_spaces_two_hundred_sites_forty_tabs_and_one_live_page() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        let session = standard_session(&mut shell).await;
        assert_eq!(session.spaces.len(), SPACES);
        assert_eq!(shell.engine.web_views.len(), 1, "one live page");
        let window = shell.window(session.subscription);
        assert_eq!(
            window.space_id, session.spaces[0],
            "the first Space is shown"
        );
        assert_eq!(window.tabs.len(), SITES_A_SPACE + TABS_A_SPACE);
        assert_eq!(
            window.tabs.iter().filter(|tab| !tab.asleep).count(),
            1,
            "every tab but the one in front is asleep"
        );
        drop(shell);

        // As the store keeps it: five Spaces, each with forty Sites and eight tabs.
        let store = store(folder.path());
        let spaces = store.spaces().unwrap();
        assert_eq!(spaces.len(), 5);
        let (mut sites, mut tabs, mut favourites, mut grouped) = (0, 0, 0, 0);
        for space in &spaces {
            let saved = store.saved_tabs(space.id).unwrap();
            assert_eq!(saved.len(), 48, "a Space's row");
            sites += saved.iter().filter(|tab| !tab.home.is_empty()).count();
            tabs += saved.iter().filter(|tab| tab.home.is_empty()).count();
            favourites += saved.iter().filter(|tab| tab.favourite).count();
            grouped += saved.iter().filter(|tab| tab.group != 0).count();
            assert_eq!(store.saved_groups(space.id).unwrap().len(), 2);
            assert!(saved.iter().all(|tab| !tab.url.is_empty()), "no empty tab");
        }
        assert_eq!((sites, tabs), (200, 40));
        assert_eq!((favourites, grouped), (20, 80));
    });
}

#[test]
fn a_start_on_the_standard_session_loads_exactly_one_page() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        standard_session(&mut shell).await;
        drop(shell);

        // The app starts again on what it kept, and its window opens.
        let mut shell = FakeShell::live(folder.path());
        standard_pages(&mut shell, SPACES);
        let (_, _, subscription) = shell.open_window().await;
        shell.settle().await;
        assert_eq!(
            asked(&shell),
            (1, 1, 0),
            "one web view made and one page loaded: the tab in front, and no other"
        );
        assert_eq!(shell.engine.web_views.len(), 1);
        let window = shell.window(subscription);
        assert_eq!(window.tabs.len(), SITES_A_SPACE + TABS_A_SPACE);
        assert_eq!(window.tabs.iter().filter(|tab| !tab.asleep).count(), 1);
        // And however long it then stands: nothing more is loaded.
        for _ in 0..90 {
            shell.a_minute_passes().await;
        }
        assert_eq!(asked(&shell), (1, 1, 0));
    });
}

#[test]
fn a_sleeping_tab_is_no_more_than_250_kb_on_disk() {
    /// The budget (000 §7).
    const MOST_ON_DISK: u64 = 250 * 1024;
    /// How many tabs are put to sleep with all a sleeping tab may keep.
    const TABS: u64 = 20;
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        let page = |number: u64| {
            format!(
                "https://www.example.com/articles/a-page-with-a-path-of-an-ordinary-length-{number}"
            )
        };
        for number in 0..=TABS {
            shell
                .engine
                .page(&page(number), "A page with a title of an ordinary length");
        }
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, &page(0)).await;
        shell.settle().await;
        drop(shell);
        let before = bytes_in(folder.path());

        // Twenty tabs, each asleep with the largest state the core keeps (051: 128 KB) and
        // the largest picture (059: 96 KB): what the engine gives is padded to a byte under
        // each limit, whatever its own words come to.
        let mut shell = FakeShell::live(folder.path());
        for number in 0..=TABS {
            shell
                .engine
                .page(&page(number), "A page with a title of an ordinary length");
        }
        let (window_again, _, _) = shell.open_window().await;
        assert_eq!(window_again, window);
        shell.engine.state_padding = 128 * 1024 - 400;
        shell.engine.picture_padding = 96 * 1024 - 200;
        for number in 1..=TABS {
            shell.open_tab(window, &page(number)).await;
        }
        shell
            .request(request::Kind::SleepOthers(SleepOthers {
                window_id: window,
            }))
            .await;
        shell.settle().await;
        drop(shell);

        // Each sleeps on a state and a picture close to the most there can be.
        let store = store(folder.path());
        let kept: Vec<(usize, usize)> = (1..=200)
            .filter_map(|id| {
                let state = store.kept_state(id).unwrap()?.1.len();
                let picture = store
                    .kept_picture(id)
                    .unwrap()
                    .map_or(0, |picture| picture.jpeg.len());
                Some((state, picture))
            })
            .collect();
        assert_eq!(
            kept.len(),
            usize::try_from(TABS).unwrap(),
            "a state for each sleeping tab"
        );
        for (state, picture) in &kept {
            assert!(
                (127 * 1024..=128 * 1024).contains(state),
                "a state of {state} bytes"
            );
            assert!(
                (95 * 1024..=96 * 1024).contains(picture),
                "a picture of {picture} bytes"
            );
        }
        drop(store);
        let each = (bytes_in(folder.path()) - before) / TABS;
        println!("a sleeping tab with the largest state and picture: {each} bytes on disk");
        assert!(
            each <= MOST_ON_DISK,
            "a sleeping tab is {each} bytes on disk, and may be {MOST_ON_DISK}"
        );
        // And not so far under that the test is of nothing: its state and its picture are there.
        assert!(
            each >= 223 * 1024,
            "{each} bytes for a state and a picture of 224 KB"
        );
    });
}
