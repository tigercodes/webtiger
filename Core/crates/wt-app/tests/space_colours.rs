//! A Space has a colour (025), through the contract, against the scripted
//! engine: the colours there are, the one a new Space is given, one that is
//! chosen and changed, and who is told.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_core::SPACE_COLOURS;
use wt_proto::v1::{ColourSpace, CreateSpace, QuerySpaces, SwitchSpace, request, response};
use wt_testkit::{FakeShell, run};

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

fn is_ack(answer: &response::Kind) -> bool {
    matches!(answer, response::Kind::Ack(_))
}

/// The list of Spaces as the core answers it.
async fn listed(shell: &mut FakeShell) -> wt_proto::v1::SpacesPage {
    let answer = shell
        .request(request::Kind::QuerySpaces(QuerySpaces {}))
        .await;
    let response::Kind::SpacesPage(page) = answer else {
        panic!("QuerySpaces answers SpacesPage, not {answer:?}");
    };
    page
}

/// Each Space's colour, in the order of the Spaces.
async fn colours(shell: &mut FakeShell) -> Vec<String> {
    listed(shell)
        .await
        .spaces
        .into_iter()
        .map(|space| space.colour)
        .collect()
}

async fn ask_for_space(shell: &mut FakeShell, name: &str, colour: &str) -> response::Kind {
    shell
        .request(request::Kind::CreateSpace(CreateSpace {
            name: name.to_owned(),
            share_logins_with: 0,
            colour: colour.to_owned(),
            icon: String::new(),
        }))
        .await
}

/// Makes a Space with the colour asked for, or with none asked for. Its id.
async fn make(shell: &mut FakeShell, name: &str, colour: &str) -> u64 {
    let answer = ask_for_space(shell, name, colour).await;
    let response::Kind::SpaceCreated(made) = answer else {
        panic!("CreateSpace answers SpaceCreated, not {answer:?}");
    };
    made.space_id
}

async fn colour(shell: &mut FakeShell, space: u64, colour: &str) -> response::Kind {
    shell
        .request(request::Kind::ColourSpace(ColourSpace {
            space_id: space,
            colour: colour.to_owned(),
        }))
        .await
}

async fn switch(shell: &mut FakeShell, window: u64, space: u64) {
    let answer = shell
        .request(request::Kind::SwitchSpace(SwitchSpace {
            window_id: window,
            space_id: space,
        }))
        .await;
    assert!(is_ack(&answer), "{answer:?}");
}

#[test]
fn the_list_says_the_colours_there_are_and_the_apps_own_space_has_none() {
    run(async {
        let mut shell = FakeShell::fake();
        let page = listed(&mut shell).await;
        assert_eq!(page.colours, SPACE_COLOURS);
        assert_eq!(page.spaces.len(), 1);
        assert_eq!(page.spaces[0].colour, "", "nobody gave it one");
        // And its window says none.
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(shell.window(subscription).space_colour, "");
    });
}

#[test]
fn a_new_space_is_given_the_first_colour_no_space_has() {
    run(async {
        let mut shell = FakeShell::fake();
        for number in 0..SPACE_COLOURS.len() {
            make(&mut shell, &format!("Space {number}"), "").await;
        }
        // Each a colour of its own, in the list's order; the app's own Space
        // still has none.
        let mut expected = vec![String::new()];
        expected.extend(SPACE_COLOURS.iter().map(|colour| (*colour).to_owned()));
        assert_eq!(colours(&mut shell).await, expected);

        // Every colour is somebody's: the first again, and then the next.
        make(&mut shell, "One more", "").await;
        make(&mut shell, "And another", "").await;
        let now = colours(&mut shell).await;
        assert_eq!(now[now.len() - 2..], ["red", "orange"]);
    });
}

#[test]
fn a_colour_that_is_asked_for_is_the_one_a_space_gets_and_the_next_skips_it() {
    run(async {
        let mut shell = FakeShell::fake();
        make(&mut shell, "Work", "orange").await;
        // Taken already, and asked for all the same: two Spaces may have it.
        make(&mut shell, "Also work", "orange").await;
        // None asked for: the first that nobody has, past the one that is taken.
        make(&mut shell, "Home", "").await;
        make(&mut shell, "Play", "").await;
        assert_eq!(
            colours(&mut shell).await,
            ["", "orange", "orange", "red", "yellow"]
        );
    });
}

#[test]
fn a_colour_that_is_not_in_the_list_is_refused_and_nothing_is_made_or_changed() {
    run(async {
        let mut shell = FakeShell::fake();
        let work = make(&mut shell, "Work", "blue").await;

        for unknown in ["mauve", "Blue", "#0000ff", " blue"] {
            let refused = ask_for_space(&mut shell, "Play", unknown).await;
            assert_eq!(error_code(refused), "space.colour_unknown", "{unknown}");
            let refused = colour(&mut shell, work, unknown).await;
            assert_eq!(error_code(refused), "space.colour_unknown", "{unknown}");
        }
        assert_eq!(colours(&mut shell).await, ["", "blue"]);

        // A Space that is not there has no colour to change, whatever is asked.
        let nobody = colour(&mut shell, work + 999, "red").await;
        assert_eq!(error_code(nobody), "space.unknown");
        let nobody = colour(&mut shell, work + 999, "mauve").await;
        assert_eq!(error_code(nobody), "space.unknown");
        // A name that is no name is said first, as before colours.
        let unnamed = ask_for_space(&mut shell, "  ", "mauve").await;
        assert_eq!(error_code(unnamed), "space.name_empty");
    });
}

#[test]
fn a_space_is_given_another_colour_or_none_and_the_window_that_shows_it_says_so() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        let work = make(&mut shell, "Work", "").await;
        assert_eq!(colours(&mut shell).await, ["", "red"]);

        // The app's own Space is given one like any other.
        assert!(is_ack(&colour(&mut shell, home, "green").await));
        assert_eq!(shell.window(subscription).space_colour, "green");
        assert_eq!(colours(&mut shell).await, ["green", "red"]);

        // A Space the window does not show is given one: the window is told
        // nothing, and says it when it shows that Space.
        let told = shell.snapshots(subscription).len();
        assert!(is_ack(&colour(&mut shell, work, "purple").await));
        assert_eq!(shell.snapshots(subscription).len(), told);
        // The same colour again says nothing either.
        assert!(is_ack(&colour(&mut shell, home, "green").await));
        assert_eq!(shell.snapshots(subscription).len(), told);
        switch(&mut shell, window, work).await;
        assert_eq!(shell.window(subscription).space_colour, "purple");

        // None: the empty name.
        assert!(is_ack(&colour(&mut shell, work, "").await));
        assert_eq!(shell.window(subscription).space_colour, "");
        assert_eq!(colours(&mut shell).await, ["green", ""]);
        // And a Space with none takes none: the next new one gets the first.
        make(&mut shell, "Play", "").await;
        assert_eq!(colours(&mut shell).await, ["green", "", "red"]);
    });
}

/// Starts the app in `folder`: a store on disk.
fn live(folder: &Path) -> FakeShell {
    FakeShell::live(folder)
}

#[test]
fn the_colours_are_there_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        let work = make(&mut shell, "Work", "").await;
        make(&mut shell, "Play", "teal").await;
        colour(&mut shell, home, "pink").await;
        switch(&mut shell, window, work).await;
        drop(shell);

        let mut shell = live(folder.path());
        assert_eq!(colours(&mut shell).await, ["pink", "red", "teal"]);
        // The window shows the Space it showed, in its colour.
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(shell.window(subscription).space_id, work);
        assert_eq!(shell.window(subscription).space_colour, "red");
        // What is kept counts: the next new Space gets the first nobody has.
        make(&mut shell, "More", "").await;
        assert_eq!(colours(&mut shell).await, ["pink", "red", "teal", "orange"]);
    });
}
