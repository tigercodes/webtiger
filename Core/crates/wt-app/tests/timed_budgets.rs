//! The plan's budgets that are times, for the core's part (000 §7, 060):
//! measurements that fail at the budget. A time depends on the machine and
//! on what else it is doing, so these are not among the tests that run
//! every time: `Tools/test.sh budgets` runs them, on a release build, and
//! each says what it measured and on what.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "a measurement fails the way a test does, and prints what it measured"
)]

use std::time::{Duration, Instant};

use prost::Message;
use wt_core::{Timestamp, Uuid};
use wt_proto::v1::{
    ActivateTab, QueryPictures, SleepOthers, WindowSnapshot, host_request, request, response,
};
use wt_store::{SavedGroup, SavedTab, Store};
use wt_testkit::{FakeShell, run};

/// What a measurement ran on, for whoever reads its numbers.
fn machine() -> String {
    format!(
        "{} {}, {} cores, {} build",
        std::env::consts::OS,
        std::env::consts::ARCH,
        std::thread::available_parallelism().map_or(0, std::num::NonZero::get),
        if cfg!(debug_assertions) {
            "a debug"
        } else {
            "a release"
        }
    )
}

/// The time under which ninety-five in a hundred fall; and says the spread.
fn p95(what: &str, mut times: Vec<Duration>) -> Duration {
    times.sort();
    let at = times[times.len() * 95 / 100];
    println!(
        "{what}: median {:?}, p95 {at:?}, worst {:?} ({})",
        times[times.len() / 2],
        times[times.len() - 1],
        machine()
    );
    at
}

/// A row as the session keeps it: of every six tabs, five are Sites, spread
/// over ten Groups and none; the last tab is the one in front.
fn row(rows: usize) -> Vec<SavedTab> {
    (0..rows)
        .map(|n| {
            let site = n % 6 != 5;
            let url = format!("https://site{n}.example.com/a/page/of/some/length?item={n}");
            SavedTab {
                title: format!("The title of page number {n}, as long as titles are"),
                front: n + 1 == rows,
                home: if site { url.clone() } else { String::new() },
                name: if site {
                    format!("Site {n}")
                } else {
                    String::new()
                },
                group: if site {
                    u32::try_from(n % 11).unwrap()
                } else {
                    0
                },
                favourite: false,
                shown_at: 0,
                state: 0,
                url,
            }
        })
        .collect()
}

/// "A contract round trip for a 200-row sidebar page: ≤ 2 ms at p95."
#[test]
#[ignore = "timed (060): `Tools/test.sh budgets` runs it, on a release build; it fails at the plan's 2 ms at p95"]
fn a_change_to_a_row_of_200_is_told_across_the_contract_within_2_ms_at_p95() {
    const BUDGET: Duration = Duration::from_millis(2);
    let folder = tempfile::tempdir().unwrap();
    {
        let mut store = Store::open(&folder.path().join("webtiger.sqlite")).unwrap();
        let space = store
            .default_space(Uuid::from_random_bytes([0; 16]), Timestamp(1))
            .unwrap()
            .id;
        let groups: Vec<SavedGroup> = (0..10)
            .map(|n| SavedGroup {
                name: format!("Group {n}"),
                colour: String::new(),
                shut: false,
            })
            .collect();
        store.save_groups(space, &groups).unwrap();
        store.save_tabs(space, &row(200)).unwrap();
    }
    run(async {
        let mut shell = FakeShell::live(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let told = shell.window(subscription);
        assert_eq!(told.tabs.len(), 200);
        let group = told.groups[0].group_id;
        // A change that tells the window again and asks nothing of an engine: a Group shut
        // and opened. The round trip is the request, the model changed, the session written,
        // the window told, and what it is told written as the contract carries it and read
        // again. The Group opened is the one measured: shut, its Sites are left out.
        let mut round_trips = Vec::new();
        for n in 0..400 {
            // (The shell that watches keeps every snapshot it was ever sent, and looks
            // through them all for the last: its notes are thrown away first, and taking
            // the snapshot out of them is not counted. A real shell reads the bytes.)
            shell.forget();
            let asked = Instant::now();
            shell.shut_group(group, n % 2 == 0).await;
            let changed = asked.elapsed();
            let now = shell.window(subscription);
            let crossing = Instant::now();
            let bytes = now.encode_to_vec();
            let again = WindowSnapshot::decode(bytes.as_slice()).unwrap();
            let took = changed + crossing.elapsed();
            if n % 2 == 1 {
                assert_eq!(again.tabs.len(), 200);
                round_trips.push(took);
            }
        }
        let at = p95(
            "a change to a row of 200, told across the contract",
            round_trips,
        );
        assert!(at <= BUDGET, "{at:?} at p95, and it may be {BUDGET:?}");
    });
}

/// "Wake a sleeping tab: snapshot within 50 ms; the engine loading within
/// 100 ms": the core's part of each. From the request that brings the tab
/// in front to the window told which picture stands for its page and the
/// picture's bytes answered; and to the load asked of the shell. A tab that
/// sleeps on the largest state and the largest picture the core keeps.
#[test]
#[ignore = "timed (060): `Tools/test.sh budgets` runs it, on a release build; it fails at the plan's 50 and 100 ms"]
fn a_sleeping_tab_wakes_with_its_picture_within_50_ms_and_its_load_asked_within_100_ms() {
    const PICTURE: Duration = Duration::from_millis(50);
    const LOAD: Duration = Duration::from_millis(100);
    const TABS: usize = 60;
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        let page = |number: usize| {
            format!(
                "https://www.example.com/articles/a-page-with-a-path-of-an-ordinary-length-{number}"
            )
        };
        for number in 0..=TABS {
            shell
                .engine
                .page(&page(number), "A page with a title of an ordinary length");
        }
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, &page(0)).await;
        shell.engine.state_padding = 128 * 1024 - 400;
        shell.engine.picture_padding = 96 * 1024 - 200;
        let mut tabs = Vec::new();
        for number in 1..=TABS {
            tabs.push(shell.open_tab(window, &page(number)).await);
        }
        shell
            .request(request::Kind::SleepOthers(SleepOthers {
                window_id: window,
            }))
            .await;
        shell.settle().await;

        let (mut pictured, mut loading) = (Vec::new(), Vec::new());
        // Each in turn, but the one that was left in front.
        for tab in &tabs[..TABS - 1] {
            // The shell is slow to begin the page: what is measured is the core's part.
            shell.hold_host = true;
            let asked = Instant::now();
            shell.send(request::Kind::ActivateTab(ActivateTab {
                tab_id: *tab,
                chosen_while_typing: false,
            }));
            shell.settle().await;
            let loads = shell
                .host_requests
                .iter()
                .filter(|kind| matches!(kind, host_request::Kind::WebViewRestore(_)))
                .count();
            let load_asked = asked.elapsed();
            let number = shell.window(subscription).tab.unwrap().picture_id;
            assert_ne!(number, 0, "the tab that wakes names its picture");
            let answer = shell.send(request::Kind::QueryPictures(QueryPictures {
                picture_ids: vec![number],
            }));
            shell.settle().await;
            let Some(response::Kind::PicturesPage(answered)) = shell.take_response(answer) else {
                panic!("the picture is answered");
            };
            let picture_there = asked.elapsed();
            assert_eq!(answered.pictures.len(), 1);
            assert!(
                answered.pictures[0].jpeg.len() > 95 * 1024,
                "the largest a picture is"
            );
            assert!(loads >= 1, "the load was asked of the shell");
            pictured.push(picture_there);
            loading.push(load_asked);
            // The page comes, and the next is woken over it.
            shell.release_host();
            shell.settle().await;
            shell.forget();
        }
        let picture = p95(
            "a sleeping tab woken, to its picture's bytes answered",
            pictured,
        );
        let load = p95(
            "a sleeping tab woken, to its load asked of the shell",
            loading,
        );
        assert!(
            picture <= PICTURE,
            "{picture:?} at p95, and it may be {PICTURE:?}"
        );
        assert!(load <= LOAD, "{load:?} at p95, and it may be {LOAD:?}");
    });
}
