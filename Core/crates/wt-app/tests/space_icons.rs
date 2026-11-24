//! A Space has an icon (050), through the contract, against the scripted
//! engine: the icons there are, the one a Space is made with or given, what
//! is refused, and who is told.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_core::SPACE_ICONS;
use wt_proto::v1::{CreateSpace, IconSpace, QuerySpaces, SwitchSpace, request, response};
use wt_testkit::{FakeShell, run};

const HOUSE: &str = "\u{1F3E0}";
const CASE: &str = "\u{1F4BC}";
const PLANE: &str = "\u{2708}\u{FE0F}";
const SEEDLING: &str = "\u{1F331}";

fn error_code(answer: response::Kind) -> (String, String) {
    match answer {
        response::Kind::Error(error) => (error.code, error.field),
        other => panic!("expected an error, got {other:?}"),
    }
}

fn said(code: &str, field: &str) -> (String, String) {
    (code.to_owned(), field.to_owned())
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

/// Each Space's icon, in the order of the Spaces.
async fn icons(shell: &mut FakeShell) -> Vec<String> {
    listed(shell)
        .await
        .spaces
        .into_iter()
        .map(|space| space.icon)
        .collect()
}

async fn ask_for_space(shell: &mut FakeShell, name: &str, icon: &str) -> response::Kind {
    shell
        .request(request::Kind::CreateSpace(CreateSpace {
            name: name.to_owned(),
            icon: icon.to_owned(),
            ..CreateSpace::default()
        }))
        .await
}

/// Makes a Space with the icon asked for, or with none asked for. Its id.
async fn make(shell: &mut FakeShell, name: &str, icon: &str) -> u64 {
    let answer = ask_for_space(shell, name, icon).await;
    let response::Kind::SpaceCreated(made) = answer else {
        panic!("CreateSpace answers SpaceCreated, not {answer:?}");
    };
    made.space_id
}

async fn give(shell: &mut FakeShell, space: u64, icon: &str) -> response::Kind {
    shell
        .request(request::Kind::IconSpace(IconSpace {
            space_id: space,
            icon: icon.to_owned(),
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
fn the_list_says_the_icons_there_are_and_a_space_has_none_until_it_is_given_one() {
    run(async {
        let mut shell = FakeShell::fake();
        let page = listed(&mut shell).await;
        assert_eq!(page.icons, SPACE_ICONS);
        assert_eq!(page.icons.len(), 24);
        assert_eq!(page.spaces.len(), 1);
        assert_eq!(page.spaces[0].icon, "", "nobody gave it one");
        // And its window says none.
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(shell.window(subscription).space_icon, "");

        // A Space made with none asked for has none: an icon is not given
        // as a colour is, however many Spaces there are. It has its colour all
        // the same.
        for number in 0..3 {
            make(&mut shell, &format!("Space {number}"), "").await;
        }
        assert_eq!(icons(&mut shell).await, ["", "", "", ""]);
        let page = listed(&mut shell).await;
        assert_eq!(page.spaces[1].colour, "red");
    });
}

#[test]
fn a_space_is_made_with_the_icon_asked_for_and_two_may_have_the_same() {
    run(async {
        let mut shell = FakeShell::fake();
        make(&mut shell, "Work", CASE).await;
        make(&mut shell, "Also work", CASE).await;
        make(&mut shell, "Travel", PLANE).await;
        make(&mut shell, "Plain", "").await;
        assert_eq!(icons(&mut shell).await, ["", CASE, CASE, PLANE, ""]);
        // Every icon of the list is one a Space can be made with.
        let mut shell = FakeShell::fake();
        for (number, icon) in SPACE_ICONS.iter().enumerate() {
            make(&mut shell, &format!("Space {number}"), icon).await;
        }
        let mut expected = vec![String::new()];
        expected.extend(SPACE_ICONS.iter().map(|icon| (*icon).to_owned()));
        assert_eq!(icons(&mut shell).await, expected);
    });
}

#[test]
fn an_icon_that_is_not_in_the_list_is_refused_and_nothing_is_made_or_changed() {
    run(async {
        let mut shell = FakeShell::fake();
        let work = make(&mut shell, "Work", CASE).await;

        // An emoji the list has not; one of the list without the selector it is
        // written with; two of them; one with a space after it; and words.
        for unknown in [
            "\u{1F600}",
            "\u{2708}",
            "\u{1F3E0}\u{1F3E0}",
            "\u{1F3E0} ",
            "house",
            "W",
            "<b>",
        ] {
            let refused = ask_for_space(&mut shell, "Play", unknown).await;
            assert_eq!(
                error_code(refused),
                said("space.icon_unknown", "icon"),
                "{unknown:?}"
            );
            let refused = give(&mut shell, work, unknown).await;
            assert_eq!(
                error_code(refused),
                said("space.icon_unknown", "icon"),
                "{unknown:?}"
            );
        }
        assert_eq!(icons(&mut shell).await, ["", CASE]);

        // A Space that is not there has no icon to change, whatever is asked.
        let nobody = give(&mut shell, work + 999, HOUSE).await;
        assert_eq!(error_code(nobody), said("space.unknown", "space_id"));
        let nobody = give(&mut shell, work + 999, "house").await;
        assert_eq!(error_code(nobody), said("space.unknown", "space_id"));
        // A name that is no name is said first, and a colour nobody knows before
        // the icon: as before icons.
        let unnamed = ask_for_space(&mut shell, "  ", "house").await;
        assert_eq!(error_code(unnamed).0, "space.name_empty");
        let answer = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Play".to_owned(),
                colour: "mauve".to_owned(),
                icon: "house".to_owned(),
                ..CreateSpace::default()
            }))
            .await;
        assert_eq!(error_code(answer).0, "space.colour_unknown");
        assert_eq!(icons(&mut shell).await.len(), 2, "nothing was made");

        // A private window is no Space: it has no icon to be given, and says none.
        let (_, _, private) = shell.open_private_window().await;
        let hidden = shell.window(private).space_id;
        assert_eq!(hidden, 0);
        assert_eq!(shell.window(private).space_icon, "");
    });
}

#[test]
fn a_space_is_given_another_icon_or_none_and_the_window_that_shows_it_says_so() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        let work = make(&mut shell, "Work", "").await;

        // The app's own Space is given one like any other: its window is told.
        assert!(is_ack(&give(&mut shell, home, HOUSE).await));
        assert_eq!(shell.window(subscription).space_icon, HOUSE);
        assert_eq!(icons(&mut shell).await, [HOUSE, ""]);

        // A Space the window does not show is given one: the window is told
        // nothing, and says it when it shows that Space.
        let told = shell.snapshots(subscription).len();
        assert!(is_ack(&give(&mut shell, work, CASE).await));
        assert_eq!(shell.snapshots(subscription).len(), told);
        // The same icon again says nothing either.
        assert!(is_ack(&give(&mut shell, home, HOUSE).await));
        assert_eq!(shell.snapshots(subscription).len(), told);
        switch(&mut shell, window, work).await;
        assert_eq!(shell.window(subscription).space_icon, CASE);

        // Another; and none, the empty one.
        assert!(is_ack(&give(&mut shell, work, SEEDLING).await));
        assert_eq!(shell.window(subscription).space_icon, SEEDLING);
        assert!(is_ack(&give(&mut shell, work, "").await));
        assert_eq!(shell.window(subscription).space_icon, "");
        assert_eq!(icons(&mut shell).await, [HOUSE, ""]);
        // An icon changes nothing else of a Space: its name and its colour are
        // what they were.
        let page = listed(&mut shell).await;
        assert_eq!(
            (page.spaces[1].name.as_str(), page.spaces[1].colour.as_str()),
            ("Work", "red")
        );
        // Two windows that show one Space are both told.
        let (other, _, second) = shell.open_window().await;
        switch(&mut shell, other, work).await;
        assert!(is_ack(&give(&mut shell, work, PLANE).await));
        assert_eq!(shell.window(subscription).space_icon, PLANE);
        assert_eq!(shell.window(second).space_icon, PLANE);
    });
}

#[test]
fn the_icons_are_there_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        let work = make(&mut shell, "Work", CASE).await;
        make(&mut shell, "Plain", "").await;
        assert!(is_ack(&give(&mut shell, home, HOUSE).await));
        switch(&mut shell, window, work).await;
        drop(shell);

        let mut shell = FakeShell::live(folder.path());
        assert_eq!(icons(&mut shell).await, [HOUSE, CASE, ""]);
        // The window shows the Space it showed, with its icon.
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(shell.window(subscription).space_id, work);
        assert_eq!(shell.window(subscription).space_icon, CASE);
    });
}
