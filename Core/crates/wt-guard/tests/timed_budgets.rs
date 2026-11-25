//! Guard's budget that is a time (000 §7: "A Guard decision: ≤ 25 µs at p95
//! on the phone"; 064): a measurement that fails at the budget. A time
//! depends on the machine, so this is not among the tests that run every
//! time: `Tools/test.sh budgets` runs it, on a release build, and it says
//! what it measured and on what. The budget is the phone's; what this runs
//! on is whatever it runs on, and it says so.
//!
//! The rules are made up for the measure: 100,000 of the shapes the public
//! lists have. They are not the public lists.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "a measurement fails the way a test does, and prints what it measured"
)]

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use wt_guard::{Guard, Kind, Verdict};

/// 000 §7.
const A_GUARD_DECISION: Duration = Duration::from_micros(25);

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

/// So many made-up rules, of five ordinary shapes.
fn rules(count: usize) -> String {
    let mut text = String::new();
    for n in 0..count {
        match n % 5 {
            0 => writeln!(text, "||ads{n}.tracker{}.example.com^", n % 977),
            1 => writeln!(text, "/banner/{n}/*$image,third-party"),
            2 => writeln!(text, "||metrics{n}.example.net^$script,third-party"),
            3 => writeln!(text, "@@||good{n}.example.com^$script"),
            _ => writeln!(text, "-ad-unit-{n}."),
        }
        .unwrap();
    }
    text
}

/// Requests as a page makes them: most match no rule; three in ten do.
fn requests() -> Vec<(String, Kind)> {
    (0..2_000)
        .map(|n| match n % 10 {
            0 => (
                format!(
                    "https://ads{}.tracker{}.example.com/pixel.gif?id={n}",
                    n * 5,
                    (n * 5) % 977
                ),
                Kind::Image,
            ),
            1 => (
                format!("https://metrics{}.example.net/collect.js", n * 5 + 2),
                Kind::Script,
            ),
            2 => (
                format!("https://cdn.example.org/banner/{}/top.png", n * 5 + 1),
                Kind::Image,
            ),
            _ => (
                format!("https://static{n}.example.org/assets/app.{n}.js?v=3"),
                Kind::Script,
            ),
        })
        .collect()
}

const PAGE: &str =
    "https://news.example.com/world/2026/11/a-story-with-a-path-of-an-ordinary-length";

fn spread(what: &str, mut times: Vec<Duration>) -> Duration {
    times.sort();
    let p95 = times[times.len() * 95 / 100];
    println!(
        "{what}: median {:?}, p95 {p95:?}, p99 {:?}, worst {:?} ({})",
        times[times.len() / 2],
        times[times.len() * 99 / 100],
        times[times.len() - 1],
        machine()
    );
    p95
}

#[test]
#[ignore = "a measurement: Tools/test.sh budgets runs it, on a release build"]
fn a_guard_decision_is_within_25_microseconds_at_p95() {
    let guard = Guard::with_rules([rules(100_000).as_str()]);
    let asked = requests();
    let blocked = asked
        .iter()
        .filter(|(url, kind)| guard.decide(url, PAGE, *kind) == Verdict::Block)
        .count();
    assert_eq!(blocked, 600, "three in ten of the requests match a rule");

    // One thread: an address parsed, the page's address parsed, and the rules looked up.
    let mut times = Vec::new();
    for _ in 0..20 {
        for (url, kind) in &asked {
            let began = Instant::now();
            std::hint::black_box(guard.decide(url, PAGE, *kind));
            times.push(began.elapsed());
        }
    }
    let alone = spread(
        "a Guard decision among 100,000 made-up rules, one thread",
        times,
    );
    assert!(
        alone <= A_GUARD_DECISION,
        "a decision took {alone:?} at p95, and may take {A_GUARD_DECISION:?}"
    );

    // Four threads at once, each with other work between its requests, as a page's
    // loading has: the blocker makes its decisions one at a time.
    let guard = std::sync::Arc::new(guard);
    let asked = std::sync::Arc::new(asked);
    let threads: Vec<_> = (0..4)
        .map(|_| {
            let (guard, asked) = (std::sync::Arc::clone(&guard), std::sync::Arc::clone(&asked));
            std::thread::spawn(move || {
                let mut times = Vec::new();
                for _ in 0..5 {
                    for (url, kind) in asked.iter() {
                        let began = Instant::now();
                        std::hint::black_box(guard.decide(url, PAGE, *kind));
                        times.push(began.elapsed());
                        let since = Instant::now();
                        while since.elapsed() < Duration::from_micros(50) {
                            std::hint::spin_loop();
                        }
                    }
                }
                times
            })
        })
        .collect();
    let together: Vec<Duration> = threads
        .into_iter()
        .flat_map(|thread| thread.join().unwrap())
        .collect();
    let together = spread(
        "the same, four threads at once with 50 µs between a thread's requests",
        together,
    );
    assert!(
        together <= A_GUARD_DECISION,
        "a decision took {together:?} at p95 with four threads asking, and may take {A_GUARD_DECISION:?}"
    );
}
