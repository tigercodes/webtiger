//! Places offered as the address field is typed in (011), through the
//! contract, against the scripted engine: what the core offers of what was
//! visited, and in what order.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_app::SUGGESTIONS;
use wt_proto::v1::{AddressSuggestion, SuggestAddresses, TabView, request, response};
use wt_testkit::{FakeEngine, FakeShell, run};

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

async fn suggest(
    shell: &mut FakeShell,
    window: u64,
    text: &str,
    limit: u32,
) -> Vec<AddressSuggestion> {
    let answer = shell
        .request(request::Kind::SuggestAddresses(SuggestAddresses {
            window_id: window,
            text: text.to_owned(),
            limit,
        }))
        .await;
    match answer {
        response::Kind::AddressSuggestions(offered) => offered.suggestions,
        other => panic!("SuggestAddresses answers AddressSuggestions, not {other:?}"),
    }
}

/// The addresses offered for what is typed.
async fn offered(shell: &mut FakeShell, window: u64, text: &str) -> Vec<String> {
    suggest(shell, window, text, 0)
        .await
        .into_iter()
        .map(|place| place.url)
        .collect()
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
    shell.engine.page("https://rust-lang.org/", "Rust");
    let (window, tab, subscription) = shell.open_window().await;
    (shell, window, tab, subscription)
}

#[test]
fn nothing_visited_nothing_offered() {
    run(async {
        let (mut shell, window, _, _) = browser().await;
        assert_eq!(offered(&mut shell, window, "example").await, [""; 0]);
    });
}

#[test]
fn a_place_visited_is_offered_by_a_word_of_its_title_or_its_address() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;
        shell.submit(tab, "doc.rust-lang.org/book/").await;

        let by_title = suggest(&mut shell, window, "programm", 0).await;
        assert_eq!(by_title.len(), 1);
        assert_eq!(by_title[0].url, "https://doc.rust-lang.org/book/");
        assert_eq!(by_title[0].title, "The Rust Programming Language");
        assert_eq!(
            by_title[0].display, "doc.rust-lang.org/book/",
            "as the address field shows it"
        );
        assert_eq!(
            offered(&mut shell, window, "exam").await,
            ["https://example.com/"],
            "by the start of a word of its address"
        );
        assert_eq!(offered(&mut shell, window, "zebra").await, [""; 0]);
        assert_eq!(offered(&mut shell, window, "").await, [""; 0]);
    });
}

#[test]
fn the_site_named_so_first_then_the_more_visited_then_the_later() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        // The book twice, the site named "rust…" once and first.
        shell.submit(tab, "rust-lang.org").await;
        shell.submit(tab, "doc.rust-lang.org/book/").await;
        shell.submit(tab, "example.com").await;
        shell.submit(tab, "doc.rust-lang.org/book/").await;

        assert_eq!(
            offered(&mut shell, window, "rust").await,
            ["https://rust-lang.org/", "https://doc.rust-lang.org/book/"],
            "the site that begins with what is typed, though it was visited less and earlier"
        );
        // Each once, however often it was visited.
        assert_eq!(
            offered(&mut shell, window, "book").await,
            ["https://doc.rust-lang.org/book/"]
        );
    });
}

#[test]
fn six_are_offered_unless_the_shell_says_and_never_more_than_twenty() {
    run(async {
        let (mut shell, window, tab, subscription) = browser().await;
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        for n in 0..30 {
            shell.report(FakeEngine::address_changed(
                web_view,
                &format!("https://example.com/page/{n}"),
            ));
        }
        shell.settle().await;

        assert_eq!(
            suggest(&mut shell, window, "page", 0).await.len(),
            SUGGESTIONS as usize
        );
        assert_eq!(suggest(&mut shell, window, "page", 3).await.len(), 3);
        assert_eq!(suggest(&mut shell, window, "page", 1_000).await.len(), 20);
        // The latest first, among places each seen once.
        assert_eq!(
            suggest(&mut shell, window, "page", 1).await[0].url,
            "https://example.com/page/29"
        );
    });
}

#[test]
fn what_was_forgotten_is_not_offered() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;
        assert_eq!(offered(&mut shell, window, "example").await.len(), 1);

        shell
            .request(request::Kind::ForgetHistory(wt_proto::v1::ForgetHistory {
                window_id: window,
                what: Some(wt_proto::v1::forget_history::What::Everything(true)),
            }))
            .await;

        assert_eq!(offered(&mut shell, window, "example").await, [""; 0]);
    });
}

#[test]
fn a_window_there_is_not_is_offered_nothing_but_a_code() {
    run(async {
        let (mut shell, window, tab, _) = browser().await;
        shell.submit(tab, "example.com").await;
        let answer = shell
            .request(request::Kind::SuggestAddresses(SuggestAddresses {
                window_id: window + 1_000,
                text: "example".to_owned(),
                limit: 0,
            }))
            .await;
        match answer {
            response::Kind::Error(error) => assert_eq!(error.code, "window.unknown"),
            other => panic!("expected an error, got {other:?}"),
        }
    });
}
