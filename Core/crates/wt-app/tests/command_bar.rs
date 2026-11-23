//! The command bar (048), through the contract, against the scripted engine:
//! a Space is offered by its name and a command by the shell's words for it,
//! to a shell that shows them; where they come among the open tabs and the
//! places; and that a shell which asks for neither is answered as before.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    AddressSuggestion, CreateSpace, OfferedCommand, SuggestAddresses, SwitchSpace, request,
    response,
};
use wt_testkit::{FakeShell, run};

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

/// The commands a shell sends: each a name and its words.
fn commands(list: &[(&str, &str)]) -> Vec<OfferedCommand> {
    list.iter()
        .map(|(name, words)| OfferedCommand {
            name: (*name).to_owned(),
            words: (*words).to_owned(),
        })
        .collect()
}

async fn ask(
    shell: &mut FakeShell,
    window: u64,
    text: &str,
    spaces: bool,
    offered: &[(&str, &str)],
    limit: u32,
) -> Vec<AddressSuggestion> {
    let answer = shell
        .request(request::Kind::SuggestAddresses(SuggestAddresses {
            window_id: window,
            text: text.to_owned(),
            limit,
            commands: commands(offered),
            spaces,
        }))
        .await;
    match answer {
        response::Kind::AddressSuggestions(offered) => offered.suggestions,
        other => panic!("SuggestAddresses answers AddressSuggestions, not {other:?}"),
    }
}

/// What each row is, in a word and its name: "tab", "space Work", "command
/// layout.side", "place".
fn kinds(rows: &[AddressSuggestion]) -> Vec<String> {
    rows.iter()
        .map(|row| {
            if row.tab_id != 0 {
                "tab".to_owned()
            } else if row.space_id != 0 {
                format!("space {}", row.title)
            } else if !row.command.is_empty() {
                format!("command {}", row.command)
            } else {
                "place".to_owned()
            }
        })
        .collect()
}

async fn make_space(shell: &mut FakeShell, name: &str) -> u64 {
    let made = shell
        .request(request::Kind::CreateSpace(CreateSpace {
            name: name.to_owned(),
            ..CreateSpace::default()
        }))
        .await;
    let response::Kind::SpaceCreated(created) = made else {
        panic!("expected a Space, got {made:?}");
    };
    created.space_id
}

async fn switch(shell: &mut FakeShell, window: u64, space: u64) {
    let answer = shell
        .request(request::Kind::SwitchSpace(SwitchSpace {
            window_id: window,
            space_id: space,
        }))
        .await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

#[test]
fn a_space_is_offered_by_its_name_to_a_shell_that_shows_them() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, _, subscription) = shell.open_window().await;
        let work = make_space(&mut shell, "Work").await;
        let workshop = make_space(&mut shell, "The Workshop").await;
        let working = make_space(&mut shell, "Working Late").await;
        let play = make_space(&mut shell, "Play").await;

        // By a word of its name, or the start of one; in the Spaces' order; two at most.
        let offered = ask(&mut shell, window, "work", true, &[], 0).await;
        assert_eq!(kinds(&offered), ["space Work", "space The Workshop"]);
        assert_eq!(offered[0].space_id, work);
        assert_eq!(offered[1].space_id, workshop);
        // A Space has a name and no address: nothing else of the row is filled.
        assert_eq!(
            offered[0],
            AddressSuggestion {
                title: "Work".to_owned(),
                space_id: work,
                ..AddressSuggestion::default()
            }
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "pl", true, &[], 0).await),
            ["space Play"]
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "LATE working", true, &[], 0).await),
            ["space Working Late"],
            "every word typed, in any order, whatever its case"
        );
        // Not by the middle of a word; not by a word it has not; and nothing typed finds none.
        for text in ["ork", "work play", "", "  "] {
            assert_eq!(
                kinds(&ask(&mut shell, window, text, true, &[], 0).await),
                [] as [&str; 0],
                "{text:?}"
            );
        }

        // A shell that does not say it shows Spaces is offered none: it is answered as it was.
        assert_eq!(
            kinds(&ask(&mut shell, window, "work", false, &[], 0).await),
            [] as [&str; 0]
        );

        // Never the Space the window shows: that is where the person is.
        switch(&mut shell, window, work).await;
        assert_eq!(shell.window(subscription).space_id, work);
        assert_eq!(
            kinds(&ask(&mut shell, window, "work", true, &[], 0).await),
            ["space The Workshop", "space Working Late"]
        );
        // Choosing one shows it, as from the list of Spaces.
        switch(&mut shell, window, working).await;
        assert_eq!(shell.window(subscription).space_id, working);
        let _ = play;

        // The Space the app made itself has no name of its own: its word is the shell's, and
        // nothing typed finds it.
        for text in ["home", "space", "w"] {
            assert!(
                ask(&mut shell, window, text, true, &[], 0)
                    .await
                    .iter()
                    .all(|row| row.space_id == 0 || !row.title.is_empty()),
                "{text:?}"
            );
        }

        // A private window has no Space, and is offered none.
        let (private_window, _, _) = shell.open_private_window().await;
        assert_eq!(
            kinds(&ask(&mut shell, private_window, "work", true, &[], 0).await),
            [] as [&str; 0]
        );
        // A window there is not is refused, as it was.
        let answer = shell
            .request(request::Kind::SuggestAddresses(SuggestAddresses {
                window_id: window + 1000,
                text: "work".to_owned(),
                spaces: true,
                ..SuggestAddresses::default()
            }))
            .await;
        assert!(
            matches!(&answer, response::Kind::Error(error) if error.code == "window.unknown"),
            "{answer:?}"
        );
    });
}

#[test]
fn a_command_is_offered_by_the_shells_words_for_it() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, _, _) = shell.open_window().await;
        let menu = [
            ("tab.new", "New Tab"),
            ("window.private", "New Private Window"),
            ("layout.side", "Tabs at the Side"),
            ("history.show", "History"),
            ("tab.reopen", "Reopen Closed Tab"),
            ("window.new", "New Window"),
            ("space.new", "New Space…"),
        ];

        // By its words: every word typed is a word of them, or the start of one. In the
        // shell's order, three at most.
        let offered = ask(&mut shell, window, "side", false, &menu, 0).await;
        assert_eq!(kinds(&offered), ["command layout.side"]);
        // A command is its name and its words as the shell sent them, and has no address.
        assert_eq!(
            offered[0],
            AddressSuggestion {
                title: "Tabs at the Side".to_owned(),
                command: "layout.side".to_owned(),
                ..AddressSuggestion::default()
            }
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "new", false, &menu, 0).await),
            [
                "command tab.new",
                "command window.private",
                "command window.new"
            ],
            "the first three of the four that match"
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "TAB re", false, &menu, 0).await),
            ["command tab.reopen"]
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "tab", false, &menu, 0).await),
            [
                "command tab.new",
                "command layout.side",
                "command tab.reopen"
            ]
        );
        // Not by its name, which is no word of its; not by the middle of a word; and nothing
        // typed finds none.
        for text in ["layout", "ide", "new old", ""] {
            assert_eq!(
                kinds(&ask(&mut shell, window, text, false, &menu, 0).await),
                [] as [&str; 0],
                "{text:?}"
            );
        }
        // A shell that sends none is offered none.
        assert_eq!(
            kinds(&ask(&mut shell, window, "new", false, &[], 0).await),
            [] as [&str; 0]
        );

        // What is no name of the kind the core's codes are is passed over, and what has no
        // words: nothing a shell sends by mistake is handed back as a command.
        let odd = [
            ("New Tab", "New Tab"),
            ("newtab", "New Tab"),
            ("tab.New", "New Tab"),
            ("", "New Tab"),
            ("tab.empty", ""),
            ("tab.new", "New Tab"),
        ];
        assert_eq!(
            kinds(&ask(&mut shell, window, "new", false, &odd, 0).await),
            ["command tab.new"]
        );
        // Sixty-four are read, and no more: the sixty-fifth is not offered.
        let names: Vec<String> = (1..=65).map(|n| format!("x.c{n}")).collect();
        let many: Vec<(&str, &str)> = names
            .iter()
            .enumerate()
            .map(|(at, name)| (name.as_str(), if at >= 62 { "Late" } else { "Early" }))
            .collect();
        assert_eq!(
            kinds(&ask(&mut shell, window, "late", false, &many, 0).await),
            ["command x.c63", "command x.c64"]
        );
    });
}

#[test]
fn tabs_then_spaces_then_commands_then_places_and_history_keeps_a_row() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, _) = shell.open_window().await;
        // Three tabs that show pages about Rust; two more such places in history; and the
        // tab typed in, somewhere else.
        shell.submit(first, RUST).await;
        shell.submit(first, FORUM).await;
        shell.submit(first, BOOK).await;
        shell.open_tab(window, CRATES).await;
        shell.open_tab(window, STD).await;
        shell.open_tab(window, HOME).await;
        make_space(&mut shell, "Rust").await;
        make_space(&mut shell, "Rustaceans").await;
        make_space(&mut shell, "Rusty").await;
        let menu = [
            ("x.one", "Rust One"),
            ("x.two", "Rust Two"),
            ("x.three", "Rust Three"),
            ("x.four", "Rust Four"),
        ];

        // Six rows, as before: the tabs have their three; a Space and a command share what
        // is left but for one row, which is a place's.
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", true, &menu, 0).await),
            ["tab", "tab", "tab", "space Rust", "command x.one", "place"]
        );
        // With room for all of them: three tabs, two Spaces, three commands, and the places.
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", true, &menu, 20).await),
            [
                "tab",
                "tab",
                "tab",
                "space Rust",
                "space Rustaceans",
                "command x.one",
                "command x.two",
                "command x.three",
                "place",
                "place"
            ]
        );
        // One row short of that: a place keeps its row, and each kind has a row before
        // either has a second.
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", true, &menu, 7).await),
            [
                "tab",
                "tab",
                "tab",
                "space Rust",
                "space Rustaceans",
                "command x.one",
                "place"
            ]
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", true, &menu, 5).await),
            ["tab", "tab", "tab", "space Rust", "place"]
        );
        // The tabs have their part of the list, as they had: with a row left it is a
        // place's, and with none there is none.
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", true, &menu, 4).await),
            ["tab", "tab", "tab", "place"]
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", true, &menu, 2).await),
            ["tab", "tab"]
        );
        // Only commands, or only Spaces: they have the rows the other would have had.
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", false, &menu, 0).await),
            [
                "tab",
                "tab",
                "tab",
                "command x.one",
                "command x.two",
                "place"
            ]
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", true, &[], 0).await),
            [
                "tab",
                "tab",
                "tab",
                "space Rust",
                "space Rustaceans",
                "place"
            ]
        );

        // Where no place is to be offered, none has a row kept for it: what is typed is
        // found in no page, and the Spaces and the commands have the list.
        let quiet = [
            ("y.one", "Quiet One"),
            ("y.two", "Quiet Two"),
            ("y.three", "Quiet Three"),
            ("y.four", "Quiet Four"),
        ];
        make_space(&mut shell, "Quiet").await;
        assert_eq!(
            kinds(&ask(&mut shell, window, "quiet", true, &quiet, 0).await),
            [
                "space Quiet",
                "command y.one",
                "command y.two",
                "command y.three"
            ]
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "quiet", true, &quiet, 2).await),
            ["space Quiet", "command y.one"]
        );
        assert_eq!(
            kinds(&ask(&mut shell, window, "quiet", true, &quiet, 1).await),
            ["space Quiet"]
        );

        // A shell that asks for neither is answered as it was: tabs, then places.
        assert_eq!(
            kinds(&ask(&mut shell, window, "rust", false, &[], 0).await),
            ["tab", "tab", "tab", "place", "place"]
        );
    });
}
