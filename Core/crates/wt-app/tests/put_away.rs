//! A Space that is not shown is put away (061), through the contract,
//! against the scripted engine, on a store that keeps the session: when a
//! Space is put away and when it is not, that it comes back as it was, and
//! what goes with it. Nothing is said to a shell of a Space put away: it is
//! told by what follows from it. A row made again has other numbers for
//! its tabs, and nothing of its arrangement to undo.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::collections::BTreeSet;
use std::path::Path;
use std::rc::Rc;

use wt_core::{Clock, ManualClock, Timestamp};
use wt_proto::v1::{
    ActivateTab, CreateSpace, QueryAwake, SleepOthers, SwitchSpace, Undo, WindowSnapshot,
    host_request, request, response,
};
use wt_testkit::{FakeShell, run};

/// The minutes a tab is out of sight before it sleeps (053 D1).
const SLEEP_AFTER: usize = 30;

fn page(name: &str) -> String {
    format!("https://{name}.example.com/")
}

const PAGES: [&str; 10] = [
    "home", "news", "mail", "wiki", "board", "forum", "maps", "music", "notes", "shop",
];

fn web(shell: &mut FakeShell) {
    for name in PAGES {
        shell.engine.page(&page(name), name);
    }
}

fn live(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

async fn acked(shell: &mut FakeShell, kind: request::Kind) {
    let answer = shell.request(kind).await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

/// Makes a Space, and says its number.
async fn make_space(shell: &mut FakeShell, name: &str) -> u64 {
    let answer = shell
        .request(request::Kind::CreateSpace(CreateSpace {
            name: name.to_owned(),
            ..CreateSpace::default()
        }))
        .await;
    let response::Kind::SpaceCreated(made) = answer else {
        panic!("a Space, not {answer:?}");
    };
    made.space_id
}

async fn show(shell: &mut FakeShell, window: u64, space: u64) {
    acked(
        shell,
        request::Kind::SwitchSpace(SwitchSpace {
            window_id: window,
            space_id: space,
        }),
    )
    .await;
}

async fn sleep_others(shell: &mut FakeShell, window: u64) {
    acked(
        shell,
        request::Kind::SleepOthers(SleepOthers { window_id: window }),
    )
    .await;
    shell.settle().await;
}

/// A row as a window is told it, without the numbers that are a run's: each
/// tab by its name, its site, what it is, whether it sleeps and the name of
/// the Group it is in; the Groups; and the name of the tab in front.
fn told(window: &WindowSnapshot) -> (Vec<String>, Vec<String>, String) {
    let group = |id: u64| {
        window
            .groups
            .iter()
            .find(|group| group.group_id == id)
            .map_or(String::new(), |group| group.name.clone())
    };
    let tabs = window
        .tabs
        .iter()
        .map(|tab| {
            format!(
                "{}|{}|site:{}|name:{}|favourite:{}|asleep:{}|group:{}",
                tab.title,
                tab.host,
                tab.kept,
                tab.site_name,
                tab.favourite,
                tab.asleep,
                group(tab.group_id)
            )
        })
        .collect();
    let groups = window
        .groups
        .iter()
        .map(|group| {
            format!(
                "{}|{}|shut:{}|{}",
                group.name, group.colour, group.shut, group.sites
            )
        })
        .collect();
    let front = window.tab.as_ref().map_or(0, |tab| tab.tab_id);
    let front = window
        .tabs
        .iter()
        .find(|tab| tab.tab_id == front)
        .map_or(String::new(), |tab| tab.title.clone());
    (tabs, groups, front)
}

fn ids(window: &WindowSnapshot) -> BTreeSet<u64> {
    window.tabs.iter().map(|tab| tab.tab_id).collect()
}

/// A second Space, "Work", arranged: a Favourite, two Sites in a Group that
/// is shut, a Site in none, and three tabs, of which the second is in front.
/// The window is left showing it. The Space's number.
async fn work(shell: &mut FakeShell, window: u64, subscription: u64) -> u64 {
    let space = make_space(shell, "Work").await;
    show(shell, window, space).await;
    let first = shell.window(subscription).tab.unwrap().tab_id;
    shell.submit(first, &page("mail")).await;
    let mut sites = vec![first];
    for name in ["wiki", "board", "forum"] {
        sites.push(shell.open_tab(window, &page(name)).await);
    }
    for id in &sites {
        acked_kind(shell.keep_tab(*id).await);
    }
    acked_kind(shell.set_favourite(sites[0], true).await);
    let response::Kind::GroupMade(group) = shell.make_group(window, "Reading").await else {
        panic!("a Group");
    };
    for id in &sites[1..3] {
        acked_kind(shell.move_to_group(*id, group.group_id).await);
    }
    acked_kind(shell.shut_group(group.group_id, true).await);
    let mut tabs = Vec::new();
    for name in ["maps", "music", "notes"] {
        tabs.push(shell.open_tab(window, &page(name)).await);
    }
    acked_kind(shell.activate_tab(tabs[1]).await);
    shell.settle().await;
    space
}

fn acked_kind(answer: response::Kind) {
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

/// What the shell was asked to make web views from: the addresses, in order.
fn restored(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|kind| match kind {
            host_request::Kind::WebViewRestore(restore) => Some(restore.url.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_space_left_with_nothing_alive_is_put_away_and_comes_back_as_it_was() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        shell.submit(first, &page("home")).await;
        let space = work(&mut shell, window, subscription).await;
        // In Work, with all but the tab in front asleep: this is how it is left.
        sleep_others(&mut shell, window).await;
        let before = shell.window(subscription);
        // (Seven tabs, of which the window is told five: its Group is shut, and holds two.)
        assert_eq!(before.tabs.len(), 5);
        assert_eq!(before.groups[0].sites, 2);
        assert_eq!(
            told(&before).2,
            "music",
            "the second of its tabs is in front"
        );
        assert!(
            !before.undo_arrangement.is_empty(),
            "its arrangement can be undone"
        );

        // The window shows the first Space, and Work's last page is put to sleep: nothing of
        // Work is alive, and it is put away.
        show(&mut shell, window, home).await;
        sleep_others(&mut shell, window).await;
        assert_eq!(
            shell.engine.web_views.len(),
            1,
            "one page alive: the first Space's"
        );

        // Shown again, it is as it was: the same tabs in the same order, each what it was,
        // the same Groups, the same tab in front, which wakes from its state.
        let asked_before = restored(&shell).len();
        show(&mut shell, window, space).await;
        shell.settle().await;
        let after = shell.window(subscription);
        assert_eq!(told(&after), told(&before));
        assert_eq!(
            restored(&shell)[asked_before..],
            [page("music")],
            "its front tab wakes from its state"
        );
        // It was made again from what the session keeps: its tabs have other numbers, and
        // there is nothing of its arrangement to undo.
        assert!(
            ids(&after).is_disjoint(&ids(&before)),
            "a row that was put away is made again: {:?} and {:?}",
            ids(&after),
            ids(&before)
        );
        assert_eq!(
            after.undo_arrangement, "",
            "its line of things to undo went with it"
        );
        assert!(after.groups[0].group_id != before.groups[0].group_id);

        // And the store has it as it is: after a start it is the same again.
        drop(shell);
        let mut shell = live(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        shell.settle().await;
        assert_eq!(shell.window(subscription).space_id, space);
        assert_eq!(told(&shell.window(subscription)), told(&before));
    });
}

#[test]
fn a_space_with_a_page_alive_or_a_state_awaited_is_not_put_away() {
    run(async {
        // Left with its front tab's page alive: it is as it was when the window comes back,
        // by the same numbers, with its arrangement to undo. When that tab has slept by its
        // time, the Space is put away.
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        shell.submit(first, &page("home")).await;
        let space = work(&mut shell, window, subscription).await;
        sleep_others(&mut shell, window).await;
        let before = shell.window(subscription);
        show(&mut shell, window, home).await;
        shell.settle().await;
        assert_eq!(shell.engine.web_views.len(), 2, "Work's front tab is alive");
        show(&mut shell, window, space).await;
        let back = shell.window(subscription);
        assert_eq!(ids(&back), ids(&before), "the same row, not one made again");
        assert!(!back.undo_arrangement.is_empty());

        show(&mut shell, window, home).await;
        for _ in 0..SLEEP_AFTER {
            shell.a_minute_passes().await;
        }
        assert_eq!(shell.engine.web_views.len(), 1, "it slept by its time");
        show(&mut shell, window, space).await;
        shell.settle().await;
        let again = shell.window(subscription);
        assert!(
            ids(&again).is_disjoint(&ids(&before)),
            "put away once nothing of it was alive"
        );
        assert_eq!(told(&again), told(&before));
    });
    run(async {
        // Its last tab is put to sleep, and the shell is slow to say that tab's state: the
        // Space is not put away while the state is awaited, since the tab that waits for it
        // would be gone. When the state has come, it is; and the tab wakes from that state.
        let folder = tempfile::tempdir().unwrap();
        let patient = std::time::Duration::from_secs(3_600);
        let mut shell =
            FakeShell::start_with(&FakeShell::live_config(folder.path()), patient).unwrap();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        shell.submit(first, &page("home")).await;
        let space = work(&mut shell, window, subscription).await;
        sleep_others(&mut shell, window).await;
        let before = shell.window(subscription);
        show(&mut shell, window, home).await;
        shell.settle().await;
        shell.hold_states = true;
        sleep_others(&mut shell, window).await;
        assert_eq!(
            shell.engine.web_views.len(),
            1,
            "Work's front tab was let go"
        );
        show(&mut shell, window, space).await;
        shell.settle().await;
        assert_eq!(
            ids(&shell.window(subscription)),
            ids(&before),
            "not put away while a state is awaited"
        );
        // (Shown, its front tab woke, and has no state to wait for any more: it is put to
        // sleep again, and this time its state comes.)
        show(&mut shell, window, home).await;
        shell.release_states();
        shell.settle().await;
        sleep_others(&mut shell, window).await;
        let asked_before = restored(&shell).len();
        show(&mut shell, window, space).await;
        shell.settle().await;
        let again = shell.window(subscription);
        assert!(ids(&again).is_disjoint(&ids(&before)));
        assert_eq!(told(&again), told(&before));
        assert_eq!(restored(&shell)[asked_before..], [page("music")]);
    });
}

#[test]
fn the_space_a_window_shows_and_another_windows_spaces_are_not_put_away() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        shell.submit(first, &page("home")).await;
        let space = work(&mut shell, window, subscription).await;
        // The Space the window shows, with every page of it asleep but none alive at all:
        // its tab in front is an empty one. It stays: it is shown.
        let empty = shell.open_tab(window, "").await;
        sleep_others(&mut shell, window).await;
        let shown = shell.window(subscription);
        assert_eq!(shown.tab.as_ref().unwrap().tab_id, empty);
        assert_eq!(shell.engine.web_views.len(), 0, "no page is alive anywhere");
        shell.a_minute_passes().await;
        assert_eq!(ids(&shell.window(subscription)), ids(&shown));
        assert!(!shell.window(subscription).undo_arrangement.is_empty());

        // A second window is not the one the session keeps (009): nothing keeps its rows
        // but the model, and they are not put away. It shows Work, then the first Space,
        // with nothing of its Work alive; and Work again, which is as it left it.
        let (second, second_tab, second_subscription) = shell.open_window().await;
        show(&mut shell, second, space).await;
        let work_tab = shell.window(second_subscription).tab.unwrap().tab_id;
        shell.submit(work_tab, &page("shop")).await;
        let its_work = shell.window(second_subscription);
        show(&mut shell, second, home).await;
        sleep_others(&mut shell, second).await;
        show(&mut shell, second, space).await;
        shell.settle().await;
        assert_eq!(ids(&shell.window(second_subscription)), ids(&its_work));
        assert_eq!(told(&shell.window(second_subscription)).2, "shop");
        let _ = second_tab;
        // And the first window, which shows Work all this while, has its row still: another
        // tab is opened in it, beside the ones it had.
        let another = shell.open_tab(window, "").await;
        let mut with_another = ids(&shown);
        with_another.insert(another);
        assert_eq!(ids(&shell.window(subscription)), with_another);
    });
}

#[test]
fn what_a_space_kept_is_there_when_it_comes_back() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        shell.submit(first, &page("home")).await;
        let space = work(&mut shell, window, subscription).await;
        // A tab of Work is closed: it can be brought back by undo (033).
        let notes = shell
            .window(subscription)
            .tabs
            .iter()
            .find(|tab| tab.title == "notes")
            .unwrap()
            .tab_id;
        shell.close_tab(notes).await;
        sleep_others(&mut shell, window).await;
        let before = shell.window(subscription);
        assert!(!before.undo.is_empty(), "a closed tab to bring back");
        // (Work's five that sleep, and the first Space's one.)
        assert_eq!(asleep(&mut shell, window).await, 6);

        show(&mut shell, window, home).await;
        sleep_others(&mut shell, window).await;
        // How many tabs are asleep is counted the same whether a Space's tabs are the
        // model's or the session's (056): Work's six, all asleep now, and it is put away.
        assert_eq!(asleep(&mut shell, window).await, 6);

        show(&mut shell, window, space).await;
        shell.settle().await;
        let after = shell.window(subscription);
        assert_eq!(told(&after), told(&before));
        // The closed tab is the store's to keep, by its Space: undo brings it back.
        assert!(!after.undo.is_empty());
        let undone = shell
            .request(request::Kind::Undo(Undo { window_id: window }))
            .await;
        assert!(matches!(undone, response::Kind::TabOpened(_)), "{undone:?}");
        shell.settle().await;
        assert!(
            shell
                .window(subscription)
                .tabs
                .iter()
                .any(|tab| tab.title == "notes"),
            "the closed tab is back"
        );
        // A tab that slept before the Space was put away wakes from the state it slept on,
        // with its picture named while it is on its way (059).
        let maps = shell
            .window(subscription)
            .tabs
            .iter()
            .find(|tab| tab.title == "maps")
            .unwrap()
            .tab_id;
        let asked_before = restored(&shell).len();
        shell.hold_host = true;
        shell.send(request::Kind::ActivateTab(ActivateTab {
            tab_id: maps,
            chosen_while_typing: false,
        }));
        shell.settle().await;
        assert_ne!(shell.window(subscription).tab.unwrap().picture_id, 0);
        shell.release_host();
        shell.settle().await;
        assert_eq!(restored(&shell)[asked_before..], [page("maps")]);
    });
}

/// How many tabs the list of what is awake says are asleep (056).
async fn asleep(shell: &mut FakeShell, window: u64) -> u32 {
    let answer = shell
        .request(request::Kind::QueryAwake(QueryAwake { window_id: window }))
        .await;
    let response::Kind::AwakePage(page) = answer else {
        panic!("what is awake, not {answer:?}");
    };
    page.asleep
}

#[test]
fn a_space_that_was_never_used_is_put_away_and_comes_back_as_one_empty_tab() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (window, first, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        shell.submit(first, &page("home")).await;
        let space = make_space(&mut shell, "Spare").await;
        show(&mut shell, window, space).await;
        let before = shell.window(subscription);
        assert_eq!(before.tabs.len(), 1);
        show(&mut shell, window, home).await;
        shell.settle().await;
        show(&mut shell, window, space).await;
        shell.settle().await;
        let after = shell.window(subscription);
        assert_eq!(told(&after), told(&before), "one empty tab");
        assert_eq!(after.tabs.len(), 1);
        assert_ne!(
            after.tabs[0].tab_id, before.tabs[0].tab_id,
            "it was put away, and made again"
        );
        assert!(!after.tabs[0].asleep);
        assert_eq!(
            shell.engine.web_views.len(),
            1,
            "an empty tab asks for no web view"
        );
    });
}

#[test]
fn a_space_that_is_away_is_tidied_as_it_is_shown_again() {
    run(async {
        // On a clock the test moves (049): a Space whose tabs are tidied after half a day.
        let folder = tempfile::tempdir().unwrap();
        let clock = Rc::new(ManualClock::new(Timestamp(1_700_000_000_000)));
        let mut shell = FakeShell::start_at(
            &FakeShell::live_config(folder.path()),
            Rc::clone(&clock) as Rc<dyn Clock>,
        )
        .unwrap();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        shell.submit(first, &page("home")).await;
        let space = work(&mut shell, window, subscription).await;
        acked_kind(shell.set_tidying(space, 43_200).await);
        // A look, so that each tab's time has begun; then the Space is left and put away.
        shell.a_minute_passes().await;
        sleep_others(&mut shell, window).await;
        let names = |shell: &FakeShell| -> Vec<String> {
            shell
                .window(subscription)
                .tabs
                .iter()
                .map(|tab| tab.title.clone())
                .collect()
        };
        assert_eq!(
            names(&shell),
            ["mail", "forum", "maps", "music", "notes"],
            "the Group is shut"
        );
        show(&mut shell, window, home).await;
        sleep_others(&mut shell, window).await;

        // A day on. The Space was away, and nothing looked at its tabs: none is in its
        // Archive yet.
        clock.advance_millis(86_400_000);
        for _ in 0..3 {
            shell.a_minute_passes().await;
        }
        // Shown again: its tabs that are past its time are tidied before the window is told
        // of its row. The Sites stay, and so does the tab in front.
        show(&mut shell, window, space).await;
        shell.settle().await;
        assert_eq!(
            names(&shell),
            ["mail", "forum", "music"],
            "no tab showed and then went"
        );
        let (entries, _) = shell.archive(window, 0, 0).await;
        let mut archived: Vec<String> = entries.iter().map(|entry| entry.title.clone()).collect();
        archived.sort();
        assert_eq!(archived, ["maps", "notes"]);
    });
}
