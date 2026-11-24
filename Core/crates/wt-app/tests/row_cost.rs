//! What a long row costs the core (000 §7: "Sidebar with 5,000 rows", "a
//! contract round trip for a 200-row sidebar page"). A measurement, not a
//! gate: run it with
//! `cargo test -p wt-app --release --test row_cost -- --ignored --nocapture`.
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
use wt_proto::v1::WindowSnapshot;
use wt_store::{SavedGroup, SavedTab, Store};
use wt_testkit::{FakeShell, run};

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

fn spread(what: &str, mut times: Vec<Duration>) {
    times.sort();
    println!(
        "  {what}: median {:?}, p95 {:?}, worst {:?}",
        times[times.len() / 2],
        times[times.len() * 95 / 100],
        times[times.len() - 1]
    );
}

#[test]
#[ignore = "a measurement: it takes seconds and asserts nothing about time"]
fn measure_telling_a_window_of_a_long_row() {
    for rows in [240_usize, 1_000, 5_000] {
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
            store.save_tabs(space, &row(rows)).unwrap();
        }
        run(async {
            let mut shell = FakeShell::live(folder.path());
            let opening = Instant::now();
            let (_, _, subscription) = shell.open_window().await;
            println!(
                "{rows} rows: the window opened with its row in {:?}",
                opening.elapsed()
            );
            let told = shell.window(subscription);
            assert_eq!(told.tabs.len(), rows);
            assert_eq!(told.groups.len(), 10);
            let bytes = told.encode_to_vec();
            println!(
                "  what the window is told: {} bytes as the contract carries it",
                bytes.len()
            );

            // A change that tells the window again and asks nothing of an engine: a Group
            // shut and opened. Shut, its Sites are left out: the open ones are measured.
            let group = told.groups[0].group_id;
            let mut changes = Vec::new();
            for n in 0..200 {
                let asked = Instant::now();
                shell.shut_group(group, n % 2 == 0).await;
                if n % 2 == 1 {
                    changes.push(asked.elapsed());
                }
            }
            spread("a change, and the window told of it", changes);

            // What crossing the contract adds: the bytes written, and read again.
            let mut written = Vec::new();
            let mut read = Vec::new();
            for _ in 0..200 {
                let started = Instant::now();
                let bytes = told.encode_to_vec();
                written.push(started.elapsed());
                let started = Instant::now();
                let again = WindowSnapshot::decode(bytes.as_slice()).unwrap();
                read.push(started.elapsed());
                assert_eq!(again.tabs.len(), rows);
            }
            spread("written as the contract carries it", written);
            spread("read again", read);
        });
    }
}
