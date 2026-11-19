//! History (008), through the contract, against the scripted engine: what is
//! written down as a visit, with which title, and what a shell is told and
//! may forget.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_app::HISTORY_PAGE;
use wt_proto::v1::{
    ForgetHistory, HistoryEntry, HistoryPage, NavigationCommand, QueryHistory, TabCommand, TabView,
    forget_history, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

/// Fake mode's first reading of the clock: 2026-01-01T12:00:00Z. It moves on
/// a second with each visit.
const FAKE_NOW_MS: i64 = 1_767_268_800_000;

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

async fn ask(shell: &mut FakeShell, query: QueryHistory) -> HistoryPage {
    match shell.request(request::Kind::QueryHistory(query)).await {
        response::Kind::HistoryPage(page) => page,
        other => panic!("QueryHistory answers HistoryPage, not {other:?}"),
    }
}

/// A window's whole history, newest first.
async fn history(shell: &mut FakeShell, window: u64) -> Vec<HistoryEntry> {
    ask(
        shell,
        QueryHistory {
            window_id: window,
            ..Default::default()
        },
    )
    .await
    .entries
}

/// The visits found by words, as their titles.
async fn found(shell: &mut FakeShell, window: u64, text: &str) -> Vec<String> {
    ask(
        shell,
        QueryHistory {
            window_id: window,
            text: text.to_owned(),
            ..Default::default()
        },
    )
    .await
    .entries
    .into_iter()
    .map(|entry| entry.title)
    .collect()
}

fn urls(entries: &[HistoryEntry]) -> Vec<&str> {
    entries.iter().map(|entry| entry.url.as_str()).collect()
}

async fn forget(shell: &mut FakeShell, window: u64, what: forget_history::What) -> response::Kind {
    shell
        .request(request::Kind::ForgetHistory(ForgetHistory {
            window_id: window,
            what: Some(what),
        }))
        .await
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// A shell whose web has a few pages. Its window, first tab and subscription.
async fn browser() -> (FakeShell, u64, u64, u64) {
    let mut shell = FakeShell::fake();
    shell.engine.page("https://example.com/", "Example Domain");
    shell.engine.page("https://example.org/", "Another Place");
    shell.engine.page(
        "https://doc.rust-lang.org/book/",
        "The Rust Programming Language",
    );
    shell
        .engine
        .redirect("https://example.net/old", "https://example.net/new");
    shell
        .engine
        .page("https://example.net/new", "Where It Moved To");
    let (window, tab, subscription) = shell.open_window().await;
    (shell, window, tab, subscription)
}

// What a visit is

#[test]
fn a_page_that_loads_is_a_visit() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        assert_eq!(history(&mut shell, window).await, [], "nothing seen yet");

        shell.submit(tab, "example.com").await;

        let entries = history(&mut shell, window).await;
        assert_eq!(entries.len(), 1);
        let visit = &entries[0];
        assert_eq!(visit.url, "https://example.com/");
        assert_eq!(visit.title, "Example Domain");
        assert_eq!(
            visit.display, "example.com",
            "as the address field shows it"
        );
        assert_eq!(visit.visited_at_ms, FAKE_NOW_MS);
        assert_ne!(visit.visit_id, 0);
    });
}

#[test]
fn visits_are_newest_first_and_later_ones_have_later_times_and_greater_ids() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;
        shell.submit(tab, "example.org").await;
        // In another tab: history is the window's Space's, not a tab's.
        shell.open_tab(window, "doc.rust-lang.org/book/").await;

        let entries = history(&mut shell, window).await;
        assert_eq!(
            urls(&entries),
            [
                "https://doc.rust-lang.org/book/",
                "https://example.org/",
                "https://example.com/"
            ]
        );
        assert!(entries[0].visit_id > entries[1].visit_id);
        assert!(entries[1].visit_id > entries[2].visit_id);
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.visited_at_ms - FAKE_NOW_MS)
                .collect::<Vec<_>>(),
            [2_000, 1_000, 0]
        );
    });
}

#[test]
fn a_redirect_is_one_visit_where_it_landed() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.net/old").await;
        let entries = history(&mut shell, window).await;
        assert_eq!(urls(&entries), ["https://example.net/new"]);
        assert_eq!(entries[0].title, "Where It Moved To");
    });
}

#[test]
fn a_load_that_never_commits_is_no_visit() {
    run(async {
        let (mut shell, window, tab, subscription) = browser().await;
        // A name that does not resolve.
        shell.submit(tab, "nowhere.example.com").await;
        // A load that starts and is stopped, and one that is refused.
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        shell.report(FakeEngine::navigation_started(
            web_view,
            "https://example.org/",
        ));
        shell.report(FakeEngine::navigation_failed(
            web_view,
            "net.cancelled",
            "https://example.org/",
        ));
        shell.report(FakeEngine::navigation_refused(
            web_view,
            "mailto:ada@example.com",
        ));
        shell.settle().await;

        assert_eq!(
            urls(&history(&mut shell, window).await),
            ["https://example.com/"],
            "only the page that was seen"
        );
    });
}

#[test]
fn a_page_that_changes_its_own_address_is_a_visit() {
    run(async {
        let (mut shell, window, tab, subscription) = browser().await;
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;

        // A one-page site moves: the History API, then its new title.
        shell.report(FakeEngine::address_changed(
            web_view,
            "https://example.com/inbox",
        ));
        shell.report(FakeEngine::title(web_view, "Inbox"));
        shell.settle().await;

        let entries = history(&mut shell, window).await;
        assert_eq!(
            urls(&entries),
            ["https://example.com/inbox", "https://example.com/"]
        );
        assert_eq!(entries[0].title, "Inbox");
        assert_eq!(
            entries[1].title, "Example Domain",
            "the page it left keeps its own"
        );
    });
}

#[test]
fn the_same_address_again_straight_after_is_the_same_visit() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;

        // A reload, and the address typed again.
        shell
            .request(request::Kind::TabCommand(TabCommand {
                tab_id: tab,
                command: NavigationCommand::Reload as i32,
            }))
            .await;
        shell.submit(tab, "example.com").await;

        assert_eq!(history(&mut shell, window).await.len(), 1);
    });
}

#[test]
fn going_back_is_a_visit_like_any_other() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;
        shell.submit(tab, "example.org").await;

        shell
            .request(request::Kind::TabCommand(TabCommand {
                tab_id: tab,
                command: NavigationCommand::Back as i32,
            }))
            .await;

        assert_eq!(
            urls(&history(&mut shell, window).await),
            [
                "https://example.com/",
                "https://example.org/",
                "https://example.com/"
            ],
            "the page was seen again"
        );
    });
}

#[test]
fn a_page_that_is_not_the_webs_is_no_visit() {
    run(async {
        let (mut shell, window, tab, subscription) = browser().await;
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;

        // The page goes to a blank document of its own.
        for report in [
            FakeEngine::navigation_started(web_view, "about:blank"),
            FakeEngine::navigation_committed(web_view, "about:blank"),
            FakeEngine::navigation_finished(web_view),
        ] {
            shell.report(report);
        }
        shell.settle().await;

        assert_eq!(
            urls(&history(&mut shell, window).await),
            ["https://example.com/"]
        );
    });
}

// The title

#[test]
fn a_visit_takes_its_pages_first_title_and_the_one_it_is_left_with() {
    run(async {
        let (mut shell, window, tab, subscription) = browser().await;
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        assert_eq!(history(&mut shell, window).await[0].title, "Example Domain");

        // The page changes its title, again and again: a counter of unread
        // letters. None of it is written while the page is there.
        for unread in 1..=10 {
            shell.report(FakeEngine::title(
                web_view,
                &format!("({unread}) Example Domain"),
            ));
        }
        shell.settle().await;
        assert_eq!(history(&mut shell, window).await[0].title, "Example Domain");

        // The tab goes on: the visit takes the title its page ended with.
        shell.submit(tab, "example.org").await;
        let entries = history(&mut shell, window).await;
        assert_eq!(entries[0].title, "Another Place");
        assert_eq!(entries[1].title, "(10) Example Domain");
        // And it is found by that title's words.
        assert_eq!(
            found(&mut shell, window, "10").await,
            ["(10) Example Domain"]
        );
    });
}

#[test]
fn a_page_with_no_title_is_shown_by_its_address() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.engine.page("https://example.com/plain.txt", "");
        shell.submit(tab, "example.com/plain.txt").await;
        let entries = history(&mut shell, window).await;
        assert_eq!(entries[0].title, "");
        assert_eq!(entries[0].display, "example.com/plain.txt");
    });
}

// Asking

#[test]
fn history_comes_a_page_at_a_time() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        for page in ["example.com", "example.org", "doc.rust-lang.org/book/"] {
            shell.submit(tab, page).await;
        }

        let first = ask(
            &mut shell,
            QueryHistory {
                window_id: window,
                limit: 2,
                ..Default::default()
            },
        )
        .await;
        assert_eq!(
            urls(&first.entries),
            ["https://doc.rust-lang.org/book/", "https://example.org/"]
        );
        assert!(first.more);

        let second = ask(
            &mut shell,
            QueryHistory {
                window_id: window,
                limit: 2,
                before_visit_id: first.entries[1].visit_id,
                ..Default::default()
            },
        )
        .await;
        assert_eq!(urls(&second.entries), ["https://example.com/"]);
        assert!(!second.more);
    });
}

#[test]
fn a_page_is_a_hundred_visits_when_the_shell_does_not_say() {
    run(async {
        let (mut shell, window, tab, subscription) = browser().await;
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        for n in 0..HISTORY_PAGE + 5 {
            shell.report(FakeEngine::address_changed(
                web_view,
                &format!("https://example.com/{n}"),
            ));
        }
        shell.settle().await;

        let page = ask(
            &mut shell,
            QueryHistory {
                window_id: window,
                ..Default::default()
            },
        )
        .await;
        assert_eq!(page.entries.len(), HISTORY_PAGE as usize);
        assert!(page.more);
        // An id no visit has is "from the newest", not an error.
        let from_the_newest = ask(
            &mut shell,
            QueryHistory {
                window_id: window,
                before_visit_id: u64::MAX,
                limit: 1,
                ..Default::default()
            },
        )
        .await;
        assert_eq!(
            from_the_newest.entries[0].visit_id,
            page.entries[0].visit_id
        );
    });
}

#[test]
fn visits_are_found_by_the_words_of_their_titles_and_addresses() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        for page in ["example.com", "example.org", "doc.rust-lang.org/book/"] {
            shell.submit(tab, page).await;
        }

        assert_eq!(
            found(&mut shell, window, "rust").await,
            ["The Rust Programming Language"]
        );
        assert_eq!(
            found(&mut shell, window, "exam").await,
            ["Another Place", "Example Domain"],
            "the start of a word, in an address or a title, newest first"
        );
        assert_eq!(
            found(&mut shell, window, "book LANG").await,
            ["The Rust Programming Language"]
        );
        assert_eq!(found(&mut shell, window, "rust OR place").await, [""; 0]);
        assert_eq!(found(&mut shell, window, "zebra").await, [""; 0]);
    });
}

// Forgetting

#[test]
fn one_visit_is_forgotten() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;
        shell.submit(tab, "example.org").await;
        let entries = history(&mut shell, window).await;
        let org = entries[0].visit_id;

        let answer = forget(&mut shell, window, forget_history::What::VisitId(org)).await;

        assert!(matches!(answer, response::Kind::Ack(_)));
        assert_eq!(
            urls(&history(&mut shell, window).await),
            ["https://example.com/"]
        );
        assert_eq!(found(&mut shell, window, "another").await, [""; 0]);
        // Forgetting what is not there is not an error.
        for gone in [org, 9_999_999, u64::MAX] {
            let again = forget(&mut shell, window, forget_history::What::VisitId(gone)).await;
            assert!(matches!(again, response::Kind::Ack(_)));
        }
        assert_eq!(history(&mut shell, window).await.len(), 1);
    });
}

#[test]
fn visits_since_a_time_are_forgotten() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        for page in ["example.com", "example.org", "doc.rust-lang.org/book/"] {
            shell.submit(tab, page).await;
        }

        // At or after the second visit's time.
        forget(
            &mut shell,
            window,
            forget_history::What::SinceMs(FAKE_NOW_MS + 1_000),
        )
        .await;

        assert_eq!(
            urls(&history(&mut shell, window).await),
            ["https://example.com/"]
        );
    });
}

#[test]
fn everything_is_forgotten_and_stays_forgotten() {
    run(async {
        let (mut shell, window, tab, subscription) = browser().await;
        shell.submit(tab, "example.com").await;
        shell.submit(tab, "example.org").await;
        let web_view = front(&shell, subscription).web_view_id;

        let answer = forget(&mut shell, window, forget_history::What::Everything(true)).await;
        assert!(matches!(answer, response::Kind::Ack(_)));
        assert_eq!(history(&mut shell, window).await, []);

        // The page still on screen says a new title, and the tab goes on:
        // the visit that was forgotten is not written to again.
        shell.report(FakeEngine::title(web_view, "A Title Said Afterwards"));
        shell.settle().await;
        shell.submit(tab, "doc.rust-lang.org/book/").await;
        let entries = history(&mut shell, window).await;
        assert_eq!(urls(&entries), ["https://doc.rust-lang.org/book/"]);
        assert_eq!(found(&mut shell, window, "afterwards").await, [""; 0]);
        assert_eq!(found(&mut shell, window, "another").await, [""; 0]);
    });
}

#[test]
fn a_page_seen_again_after_forgetting_is_a_visit_again() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;
        forget(&mut shell, window, forget_history::What::Everything(true)).await;

        // The same address, straight after: it is not "the visit that is
        // there already" any more.
        shell
            .request(request::Kind::TabCommand(TabCommand {
                tab_id: tab,
                command: NavigationCommand::Reload as i32,
            }))
            .await;

        let entries = history(&mut shell, window).await;
        assert_eq!(urls(&entries), ["https://example.com/"]);
        assert_eq!(entries[0].title, "Example Domain");
    });
}

#[test]
fn forgetting_must_say_what() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;

        let nothing = shell
            .request(request::Kind::ForgetHistory(ForgetHistory {
                window_id: window,
                what: None,
            }))
            .await;
        assert_eq!(error_code(nothing), "history.nothing_named");
        // "Everything: false" is not everything.
        let not_everything =
            forget(&mut shell, window, forget_history::What::Everything(false)).await;
        assert_eq!(error_code(not_everything), "history.nothing_named");

        assert_eq!(
            history(&mut shell, window).await.len(),
            1,
            "nothing was forgotten"
        );
    });
}

#[test]
fn a_window_there_is_not_has_no_history() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;

        let asked = shell
            .request(request::Kind::QueryHistory(QueryHistory {
                window_id: window + 1_000,
                ..Default::default()
            }))
            .await;
        assert_eq!(error_code(asked), "window.unknown");
        let forgotten = forget(
            &mut shell,
            window + 1_000,
            forget_history::What::Everything(true),
        )
        .await;
        assert_eq!(error_code(forgotten), "window.unknown");
        assert_eq!(history(&mut shell, window).await.len(), 1);
    });
}
