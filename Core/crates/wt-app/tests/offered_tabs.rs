//! An open tab is offered while typing (038), through the contract, against
//! the scripted engine: which tabs, in what order, among the places of
//! history, and what choosing one does.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    AddressSuggestion, CreateSpace, SuggestAddresses, SwitchSpace, host_request, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const HOME: &str = "https://example.com/";
const BOOK: &str = "https://doc.rust-lang.org/book/";
const RUST: &str = "https://rust-lang.org/";
const CRATES: &str = "https://crates.io/";
const STD: &str = "https://doc.rust-lang.org/std/";
const FORUM: &str = "https://users.rust-lang.org/";

fn web(shell: &mut FakeShell) {
    shell.engine.page(HOME, "Example Domain");
    shell.engine.page(BOOK, "The Rust Programming Language");
    shell.engine.page(RUST, "Rust");
    shell
        .engine
        .page(CRATES, "crates.io: Rust Package Registry");
    shell.engine.page(STD, "std - Rust");
    shell
        .engine
        .page(FORUM, "The Rust Programming Language Forum");
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

/// What is offered, in order: each as the tab it is (not 0) and its address.
async fn offered(shell: &mut FakeShell, window: u64, text: &str) -> Vec<(u64, String)> {
    suggest(shell, window, text, 0)
        .await
        .into_iter()
        .map(|one| (one.tab_id, one.url))
        .collect()
}

/// The tab in front.
fn front(shell: &FakeShell, subscription: u64) -> u64 {
    shell.window(subscription).tab.expect("a tab").tab_id
}

/// The addresses the shell was asked to load, in order.
fn loads(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewLoad(load) => Some(load.url.clone()),
            _ => None,
        })
        .collect()
}

fn place(url: &str) -> (u64, String) {
    (0, url.to_owned())
}

fn tab(id: u64, url: &str) -> (u64, String) {
    (id, url.to_owned())
}

#[test]
fn an_open_tab_is_offered_first_and_its_place_not_a_second_time() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let book = shell.open_tab(window, BOOK).await;
        // The tab in front, where the typing is, has been to the forum and to Rust's site.
        let typing = shell.open_tab(window, FORUM).await;
        shell.submit(typing, RUST).await;

        // The book is open in another tab: offered first, as that tab, and not again as
        // the place it also is. Then the places of history, as they were ordered before
        // (011): the site named so first.
        let rust = suggest(&mut shell, window, "rust", 0).await;
        let seen: Vec<(u64, String)> = rust
            .iter()
            .map(|one| (one.tab_id, one.url.clone()))
            .collect();
        assert_eq!(seen, [tab(book, BOOK), place(RUST), place(FORUM)]);
        assert_eq!(rust[0].title, "The Rust Programming Language");
        assert_eq!(rust[0].display, "doc.rust-lang.org/book/");

        // The tab in front is where the person is: its page is a place, and no tab.
        assert!(seen.iter().all(|(id, _)| *id != typing));
        assert!(seen.contains(&place(RUST)));
        // A word that only the first tab's page has.
        assert_eq!(
            offered(&mut shell, window, "example").await,
            [tab(first, HOME)]
        );
        // And one that nothing has.
        assert_eq!(offered(&mut shell, window, "python").await, []);
    });
}

#[test]
fn every_word_typed_is_in_the_tabs_title_or_its_address() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, BOOK).await;
        let web_view = shell.window(subscription).tab.expect("a tab").web_view_id;
        shell.open_tab(window, HOME).await;
        let tabs = |offered: Vec<(u64, String)>| -> Vec<u64> {
            offered
                .into_iter()
                .map(|(id, _)| id)
                .filter(|id| *id != 0)
                .collect()
        };

        for typed in [
            "rust",
            "RUS",
            "prog lang",
            "book",
            "doc.rust-lang.org/book",
            "the book",
            "lang rust doc",
        ] {
            assert_eq!(
                tabs(offered(&mut shell, window, typed).await),
                [first],
                "{typed}"
            );
        }
        // The middle of a word; a word that is not there; nothing; no word at all.
        for typed in ["ust", "rust python", "", "   ", "-- /"] {
            assert_eq!(
                tabs(offered(&mut shell, window, typed).await),
                [],
                "{typed:?}"
            );
        }

        // The title is the one the page has now.
        shell.report(FakeEngine::title(web_view, "Ownership"));
        shell.settle().await;
        assert_eq!(tabs(offered(&mut shell, window, "owner").await), [first]);
        assert_eq!(tabs(offered(&mut shell, window, "programming").await), []);
    });
}

#[test]
fn a_tab_with_no_page_is_not_offered() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        shell
            .engine
            .fail("https://rust.example.com/", "net.unreachable");
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, BOOK).await;
        // An empty tab, and one whose page never arrived.
        shell.open_tab(window, "").await;
        shell.open_tab(window, "https://rust.example.com/").await;
        // A page that is not the web's, whatever it is called: it is no place either.
        shell.engine.page("about:blank", "Rust, on a blank page");
        let blank = shell.open_tab(window, "about:blank").await;
        let titles: Vec<String> = shell
            .window(subscription)
            .tabs
            .into_iter()
            .filter(|one| one.tab_id == blank)
            .map(|one| one.title)
            .collect();
        assert_eq!(
            titles,
            ["Rust, on a blank page"],
            "the blank page is shown, and so called"
        );
        // Typed in a third.
        shell.open_tab(window, HOME).await;

        assert_eq!(
            offered(&mut shell, window, "rust").await,
            [tab(first, BOOK)]
        );
    });
}

#[test]
fn at_most_three_tabs_in_the_rows_order_and_the_list_is_no_longer_than_it_was() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, BOOK).await;
        let second = shell.open_tab(window, CRATES).await;
        let third = shell.open_tab(window, STD).await;
        let fourth = shell.open_tab(window, FORUM).await;
        let fifth = shell.open_tab(window, RUST).await;
        // Typed in a sixth, which has been to two more places that match.
        let typing = shell.open_tab(window, HOME).await;
        assert_eq!(typing, typing.max(fifth), "the sixth tab");

        // Three tabs, the first three of the row that match; then places, without the
        // three the tabs show: six in all.
        let all = offered(&mut shell, window, "rust").await;
        assert_eq!(
            all.len(),
            5,
            "three tabs, and the two places no offered tab shows"
        );
        assert_eq!(
            all[..3],
            [tab(first, BOOK), tab(second, CRATES), tab(third, STD)]
        );
        assert!(all[3..].iter().all(|(id, _)| *id == 0));
        let mut rest: Vec<&str> = all[3..].iter().map(|(_, url)| url.as_str()).collect();
        rest.sort_unstable();
        assert_eq!(rest, [RUST, FORUM]);
        let _ = fourth;

        // The shell's own number is the whole list's: tabs take their part of it.
        let two = suggest(&mut shell, window, "rust", 2).await;
        assert_eq!(
            two.iter().map(|one| one.tab_id).collect::<Vec<_>>(),
            [first, second]
        );
        let four = suggest(&mut shell, window, "rust", 4).await;
        assert_eq!(
            four.iter().map(|one| one.tab_id != 0).collect::<Vec<_>>(),
            [true, true, true, false]
        );
        // And with a tab in front that is one of the first three, the fourth takes its place.
        shell.activate_tab(second).await;
        let now = offered(&mut shell, window, "rust").await;
        assert_eq!(
            now[..3],
            [tab(first, BOOK), tab(third, STD), tab(fourth, FORUM)]
        );
    });
}

#[test]
fn only_the_row_the_window_shows_and_no_other_windows() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, BOOK).await;
        shell.open_tab(window, HOME).await;
        let home = shell.window(subscription).space_id;
        // Another window, with Rust's site open in a tab that is not in front.
        let (other, theirs, _) = shell.open_window().await;
        shell.submit(theirs, RUST).await;
        shell.open_tab(other, HOME).await;

        // Each window is offered its own tab, and not the other's (whose page is, by now,
        // a place of the history they share).
        assert_eq!(
            offered(&mut shell, window, "rust").await,
            [tab(first, BOOK), place(RUST)]
        );
        assert_eq!(
            offered(&mut shell, other, "rust").await,
            [tab(theirs, RUST), place(BOOK)]
        );

        // Another Space in the first window: its row has one tab, and its history is its own.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        let response::Kind::SpaceCreated(created) = made else {
            panic!("expected a Space, got {made:?}");
        };
        shell
            .request(request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: created.space_id,
            }))
            .await;
        assert_eq!(
            offered(&mut shell, window, "rust").await,
            [],
            "the tabs of the row that is not shown are not offered"
        );
        // In the Space's own row: a tab there is offered, and the first Space's are not.
        let here = front(&shell, subscription);
        shell.submit(here, STD).await;
        shell.open_tab(window, HOME).await;
        assert_eq!(offered(&mut shell, window, "rust").await, [tab(here, STD)]);
        shell
            .request(request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: home,
            }))
            .await;
        assert_eq!(
            offered(&mut shell, window, "rust").await,
            [tab(first, BOOK), place(RUST)]
        );
    });
}

#[test]
fn a_private_window_is_offered_its_own_tabs_and_nothing_of_history() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, BOOK).await;
        shell.open_tab(window, HOME).await;
        let (private, theirs, _) = shell.open_private_window().await;
        shell.submit(theirs, RUST).await;
        shell.open_tab(private, CRATES).await;

        // Its own open tab; no place, though it has been to two that match, and the other
        // window to one.
        assert_eq!(
            offered(&mut shell, private, "rust").await,
            [tab(theirs, RUST)]
        );
        // And the ordinary window is offered nothing of the private one's.
        assert_eq!(
            offered(&mut shell, window, "rust").await,
            [tab(first, BOOK)]
        );
    });
}

#[test]
fn choosing_it_brings_the_tab_in_front_and_loads_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, BOOK).await;
        let typing = shell.open_tab(window, HOME).await;
        let before = loads(&shell);

        let rust = offered(&mut shell, window, "rust").await;
        assert_eq!(rust, [tab(first, BOOK)]);
        let answer = shell.activate_tab(rust[0].0).await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");

        assert_eq!(front(&shell, subscription), first);
        assert_eq!(
            loads(&shell),
            before,
            "the page was open: nothing is loaded"
        );
        // The tab that was typed in is still there, as it was; and is now the one offered.
        let row: Vec<u64> = shell
            .window(subscription)
            .tabs
            .into_iter()
            .map(|one| one.tab_id)
            .collect();
        assert_eq!(row, [first, typing]);
        assert_eq!(
            offered(&mut shell, window, "example").await[0],
            tab(typing, HOME)
        );
    });
}

#[test]
fn a_window_there_is_not_is_offered_nothing_but_a_code() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, _) = shell.open_window().await;
        shell.submit(first, BOOK).await;
        let answer = shell
            .request(request::Kind::SuggestAddresses(SuggestAddresses {
                window_id: window + 100,
                text: "rust".to_owned(),
                limit: 0,
            }))
            .await;
        match answer {
            response::Kind::Error(error) => assert_eq!(error.code, "window.unknown"),
            other => panic!("expected an error, got {other:?}"),
        }
    });
}
