//! Several Spaces (024), through the contract, against the scripted engine:
//! a Space is made and named, a window is switched between them, a tab is
//! in the Space it was opened in, and each Space's tabs come back.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_core::MOST_SPACES;
use wt_proto::v1::{
    CloseWindow, CreateSpace, PermissionKind, QueryHistory, QuerySpaces, RenameSpace, SwitchSpace,
    TabView, host_request, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const HOME_PAGE: &str = "https://example.com/";
const WORK_PAGE: &str = "https://work.example.com/";
const OTHER_PAGE: &str = "https://example.org/";

/// The web the tests browse.
fn web(mut shell: FakeShell) -> FakeShell {
    shell.engine.page(HOME_PAGE, "Example Domain");
    shell.engine.page(OTHER_PAGE, "Another Place");
    shell.engine.page("https://example.net/", "A Third");
    shell.engine.page(WORK_PAGE, "Work");
    shell.engine.page("https://work.example.com/wiki", "Wiki");
    shell.engine.page("https://work.example.com/popup", "Popup");
    shell
}

fn fake() -> FakeShell {
    web(FakeShell::fake())
}

/// Starts the app in `folder`: a store on disk, and a session.
fn live(folder: &Path) -> FakeShell {
    web(FakeShell::live(folder))
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

fn is_ack(answer: &response::Kind) -> bool {
    matches!(answer, response::Kind::Ack(_))
}

/// The Spaces as the core lists them: each one's id, name and the number
/// that says whose logins it has.
async fn spaces(shell: &mut FakeShell) -> Vec<(u64, String, u32)> {
    let answer = shell
        .request(request::Kind::QuerySpaces(QuerySpaces {}))
        .await;
    let response::Kind::SpacesPage(page) = answer else {
        panic!("QuerySpaces answers SpacesPage, not {answer:?}");
    };
    page.spaces
        .into_iter()
        .map(|space| (space.space_id, space.name, space.logins))
        .collect()
}

/// The names of the Spaces, in their order.
async fn names(shell: &mut FakeShell) -> Vec<String> {
    spaces(shell)
        .await
        .into_iter()
        .map(|(_, name, _)| name)
        .collect()
}

async fn ask_for_space(shell: &mut FakeShell, name: &str, share: u64) -> response::Kind {
    shell
        .request(request::Kind::CreateSpace(CreateSpace {
            name: name.to_owned(),
            share_logins_with: share,
            colour: String::new(),
        }))
        .await
}

/// Makes a Space: with logins of its own for `share` 0. Its id.
async fn make(shell: &mut FakeShell, name: &str, share: u64) -> u64 {
    let answer = ask_for_space(shell, name, share).await;
    let response::Kind::SpaceCreated(made) = answer else {
        panic!("CreateSpace answers SpaceCreated, not {answer:?}");
    };
    made.space_id
}

async fn rename(shell: &mut FakeShell, space: u64, name: &str) -> response::Kind {
    shell
        .request(request::Kind::RenameSpace(RenameSpace {
            space_id: space,
            name: name.to_owned(),
        }))
        .await
}

async fn switch(shell: &mut FakeShell, window: u64, space: u64) -> response::Kind {
    shell
        .request(request::Kind::SwitchSpace(SwitchSpace {
            window_id: window,
            space_id: space,
        }))
        .await
}

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// The Space a window shows: its id and its name.
fn shown(shell: &FakeShell, subscription: u64) -> (u64, String) {
    let window = shell.window(subscription);
    (window.space_id, window.space_name)
}

/// The strip: each tab's id.
fn ids(shell: &FakeShell, subscription: u64) -> Vec<u64> {
    shell
        .window(subscription)
        .tabs
        .iter()
        .map(|tab| tab.tab_id)
        .collect()
}

/// The strip: each tab's title.
fn titles(shell: &FakeShell, subscription: u64) -> Vec<String> {
    shell
        .window(subscription)
        .tabs
        .into_iter()
        .map(|tab| tab.title)
        .collect()
}

/// The web views the core asked for, in order, each with its login set.
fn created(shell: &FakeShell) -> Vec<(u64, String)> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewCreate(create) => Some((
                create.web_view_id,
                create.login_set.clone().unwrap_or_default().id,
            )),
            _ => None,
        })
        .collect()
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

fn destroyed(shell: &FakeShell) -> Vec<u64> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewDestroy(destroy) => Some(destroy.web_view_id),
            _ => None,
        })
        .collect()
}

/// The addresses in a window's history, newest first.
async fn history(shell: &mut FakeShell, window: u64) -> Vec<String> {
    let answer = shell
        .request(request::Kind::QueryHistory(QueryHistory {
            window_id: window,
            ..Default::default()
        }))
        .await;
    let response::Kind::HistoryPage(page) = answer else {
        panic!("QueryHistory answers HistoryPage, not {answer:?}");
    };
    page.entries.into_iter().map(|entry| entry.url).collect()
}

/// A window on the app's own Space, and a Space "Work" with logins of its
/// own that nothing has shown yet. The window, its first tab, its
/// subscription, the app's own Space and Work.
async fn home_and_work(shell: &mut FakeShell) -> (u64, u64, u64, u64, u64) {
    let (window, tab, subscription) = shell.open_window().await;
    let home = shown(shell, subscription).0;
    let work = make(shell, "Work", 0).await;
    (window, tab, subscription, home, work)
}

#[test]
fn a_space_is_made_after_the_others_and_listed_with_its_name_tidied() {
    run(async {
        let mut shell = fake();
        // The app's own Space is there from the start, and has no name.
        let before = spaces(&mut shell).await;
        assert_eq!(before.len(), 1);
        let home = before[0].0;
        assert_eq!(before[0], (home, String::new(), 1));

        let work = make(&mut shell, "  The \t office\n", 0).await;
        let project = make(&mut shell, "Project", work).await;
        let also_home = make(&mut shell, "Also home", home).await;
        let play = make(&mut shell, "Play", 0).await;
        let more = make(&mut shell, "More of the project", project).await;

        // In the order they were made. Spaces that share logins say the
        // same number; those with logins of their own say one nobody else
        // says. Sharing with a Space that shares already is sharing with
        // whoever it shares with.
        assert_eq!(
            spaces(&mut shell).await,
            [
                (home, String::new(), 1),
                (work, "The office".to_owned(), 2),
                (project, "Project".to_owned(), 2),
                (also_home, "Also home".to_owned(), 1),
                (play, "Play".to_owned(), 5),
                (more, "More of the project".to_owned(), 2),
            ]
        );
        // Two Spaces may have the same name, and are two Spaces.
        let twin = make(&mut shell, "Play", 0).await;
        assert_ne!(twin, play);
        assert_eq!(spaces(&mut shell).await.len(), 7);
    });
}

#[test]
fn a_name_is_cut_at_sixty_characters() {
    run(async {
        let mut shell = fake();
        make(&mut shell, &"é".repeat(75), 0).await;
        let listed = names(&mut shell).await;
        assert_eq!(listed[1], "é".repeat(60));
    });
}

#[test]
fn what_cannot_be_made_is_refused_and_nothing_is_made() {
    run(async {
        let mut shell = fake();
        let home = spaces(&mut shell).await[0].0;

        let empty = ask_for_space(&mut shell, " \n\t ", 0).await;
        assert_eq!(error_code(empty), "space.name_empty");
        // The logins of a Space that is not there.
        let nobodys = ask_for_space(&mut shell, "Work", home + 999).await;
        assert_eq!(error_code(nobodys), "space.unknown");
        assert_eq!(names(&mut shell).await, [""]);

        // As many as there may be, and not one more: with logins of its
        // own, or with somebody else's.
        for number in 1..MOST_SPACES {
            make(&mut shell, &format!("Space {number}"), 0).await;
        }
        assert_eq!(spaces(&mut shell).await.len(), MOST_SPACES);
        let own = ask_for_space(&mut shell, "One more", 0).await;
        assert_eq!(error_code(own), "space.too_many");
        let shared = ask_for_space(&mut shell, "One more", home).await;
        assert_eq!(error_code(shared), "space.too_many");
        assert_eq!(spaces(&mut shell).await.len(), MOST_SPACES);
    });
}

#[test]
fn a_space_is_named_and_the_window_that_shows_it_says_so() {
    run(async {
        let mut shell = fake();
        let (_, _, subscription, home, work) = home_and_work(&mut shell).await;
        assert_eq!(shown(&shell, subscription), (home, String::new()));

        // The app's own Space is named like any other.
        assert!(is_ack(&rename(&mut shell, home, "  My  own ").await));
        assert_eq!(shown(&shell, subscription), (home, "My own".to_owned()));
        assert_eq!(names(&mut shell).await, ["My own", "Work"]);

        // A Space the window does not show is named, and the window is not
        // told anything: it shows what it showed.
        let told = shell.snapshots(subscription).len();
        assert!(is_ack(&rename(&mut shell, work, "The office").await));
        assert_eq!(names(&mut shell).await, ["My own", "The office"]);
        assert_eq!(shell.snapshots(subscription).len(), told);
        // And the same name again says nothing either.
        assert!(is_ack(&rename(&mut shell, home, "My own").await));
        assert_eq!(shell.snapshots(subscription).len(), told);

        // No name is no name, and a Space that is not there has none.
        let empty = rename(&mut shell, home, "   ").await;
        assert_eq!(error_code(empty), "space.name_empty");
        let nobody = rename(&mut shell, work + 999, "Somewhere").await;
        assert_eq!(error_code(nobody), "space.unknown");
        assert_eq!(names(&mut shell).await, ["My own", "The office"]);
    });
}

#[test]
fn a_window_is_switched_and_back_again_its_tabs_are_as_they_were() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        shell.submit(first, "example.com").await;
        let second = shell.open_tab(window, "example.org").await;
        let third = shell.open_tab(window, "example.net").await;
        shell.activate_tab(second).await;
        let home_strip = shell.window(subscription);
        assert_eq!(ids(&shell, subscription), [first, second, third]);
        let asked = shell.host_requests.len();

        // A Space with no tabs is given one empty tab, as a new window is.
        assert!(is_ack(&switch(&mut shell, window, work).await));
        assert_eq!(shown(&shell, subscription), (work, "Work".to_owned()));
        let work_first = front(&shell, subscription);
        assert_eq!(ids(&shell, subscription), [work_first.tab_id]);
        assert!(![first, second, third].contains(&work_first.tab_id));
        assert_eq!(work_first.address.unwrap().url, "");
        assert_eq!(work_first.web_view_id, 0);
        // Nothing was asked of the web views of the tabs that were left:
        // nothing was asked of the shell at all.
        assert_eq!(shell.host_requests.len(), asked);

        shell.submit(work_first.tab_id, "work.example.com").await;
        let wiki = shell.open_tab(window, "work.example.com/wiki").await;
        shell.activate_tab(work_first.tab_id).await;
        let work_strip = shell.window(subscription);
        let asked = shell.host_requests.len();

        // Back: the tabs that were left, in their order, with the one in
        // front that was in front, and every page as it was.
        assert!(is_ack(&switch(&mut shell, window, home).await));
        assert_eq!(shell.window(subscription), home_strip);
        assert_eq!(front(&shell, subscription).tab_id, second);
        // And again: Work as it was left.
        assert!(is_ack(&switch(&mut shell, window, work).await));
        assert_eq!(shell.window(subscription), work_strip);
        assert_eq!(ids(&shell, subscription), [work_first.tab_id, wiki]);
        assert_eq!(shell.host_requests.len(), asked, "no page was loaded again");
    });
}

#[test]
fn the_space_that_is_shown_changes_nothing_and_what_is_not_there_is_refused() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        shell.submit(first, "example.com").await;
        let told = shell.snapshots(subscription).len();

        assert!(is_ack(&switch(&mut shell, window, home).await));
        assert_eq!(shell.snapshots(subscription).len(), told);

        let nowhere = switch(&mut shell, window, work + 999).await;
        assert_eq!(error_code(nowhere), "space.unknown");
        let no_window = switch(&mut shell, window + 999, work).await;
        assert_eq!(error_code(no_window), "window.unknown");
        assert_eq!(shell.snapshots(subscription).len(), told);
        assert_eq!(shown(&shell, subscription).0, home);
    });
}

#[test]
fn a_tab_is_in_the_space_it_was_opened_in_and_runs_in_that_spaces_logins() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        let project = make(&mut shell, "Project", work).await;
        shell.submit(first, "example.com").await;
        let home_logins = front(&shell, subscription).login_set_id;

        switch(&mut shell, window, work).await;
        let work_tab = shell.open_tab(window, "work.example.com").await;
        let work_logins = front(&shell, subscription).login_set_id;
        assert_ne!(work_logins, home_logins, "logins of its own");
        // Fake mode gives the same login sets every run.
        assert_eq!(work_logins, "58585858-5858-4858-9858-585858585858");

        // A Space that shares Work's logins runs in them.
        switch(&mut shell, window, project).await;
        let project_tab = shell.open_tab(window, "work.example.com/wiki").await;
        assert_eq!(front(&shell, subscription).login_set_id, work_logins);

        // Each web view was made in the login set of its tab's Space.
        let made: Vec<String> = created(&shell)
            .into_iter()
            .map(|(_, logins)| logins)
            .collect();
        assert_eq!(
            made,
            [home_logins.clone(), work_logins.clone(), work_logins]
        );

        // And each tab is in its own Space and in no other.
        assert!(ids(&shell, subscription).contains(&project_tab));
        assert!(!ids(&shell, subscription).contains(&work_tab));
        switch(&mut shell, window, home).await;
        assert_eq!(ids(&shell, subscription), [first]);
        switch(&mut shell, window, work).await;
        assert!(ids(&shell, subscription).contains(&work_tab));
        assert!(!ids(&shell, subscription).contains(&project_tab));
    });
}

#[test]
fn what_a_tab_opens_is_opened_in_that_tabs_space() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        shell.submit(first, "example.com").await;
        switch(&mut shell, window, work).await;
        let work_tab = front(&shell, subscription).tab_id;
        shell.submit(work_tab, "work.example.com").await;
        let work_view = front(&shell, subscription);
        let asking = work_view.web_view_id;

        // The window shows the other Space while Work's page asks: for a
        // link in a new tab, for a window of its own, for a search.
        switch(&mut shell, window, home).await;
        let home_strip = shell.window(subscription);
        shell.report(FakeEngine::link_open_requested(
            asking,
            "https://work.example.com/wiki",
            true,
            true,
        ));
        shell.settle().await;
        let (popup, report) =
            shell
                .engine
                .ask_for_window(asking, "https://work.example.com/popup", true);
        shell.report(report);
        shell.settle().await;
        shell.report(FakeEngine::search_requested(asking, "tigers", true));
        shell.settle().await;

        // Nothing of it is in the Space the window shows.
        assert_eq!(shell.window(subscription), home_strip);

        // It is all in Work, in Work's logins.
        switch(&mut shell, window, work).await;
        let strip = shell.window(subscription).tabs;
        assert_eq!(strip.len(), 4, "Work's tab, and the three it opened");
        assert_eq!(strip[0].tab_id, work_tab);
        let logins: Vec<String> = created(&shell)
            .into_iter()
            .skip(1)
            .map(|(_, logins)| logins)
            .collect();
        assert_eq!(logins.len(), 3, "Work's page, the link and the search");
        assert!(
            logins
                .iter()
                .all(|logins| *logins == work_view.login_set_id)
        );
        for tab in &strip {
            shell.activate_tab(tab.tab_id).await;
            let tab = front(&shell, subscription);
            assert_eq!(tab.login_set_id, work_view.login_set_id);
        }
        // The window the page asked for is one of them.
        let views: Vec<u64> = {
            let mut views = Vec::new();
            for tab in &strip {
                shell.activate_tab(tab.tab_id).await;
                views.push(front(&shell, subscription).web_view_id);
            }
            views
        };
        assert!(views.contains(&popup));
    });
}

#[test]
fn a_page_that_closes_its_tab_in_a_space_that_is_not_shown_closes_only_that() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        shell.submit(first, "example.com").await;
        switch(&mut shell, window, work).await;
        let work_tab = front(&shell, subscription).tab_id;
        shell.submit(work_tab, "work.example.com").await;
        let asking = front(&shell, subscription).web_view_id;
        let (popup, report) =
            shell
                .engine
                .ask_for_window(asking, "https://work.example.com/popup", true);
        shell.report(report);
        shell.settle().await;
        assert_eq!(ids(&shell, subscription).len(), 2);

        switch(&mut shell, window, home).await;
        let home_strip = shell.window(subscription);
        shell.report(FakeEngine::close_requested(popup));
        shell.settle().await;

        assert_eq!(destroyed(&shell), [popup]);
        assert_eq!(shell.window(subscription), home_strip);
        switch(&mut shell, window, work).await;
        // Back to the tab that opened it, as in a Space that is shown.
        assert_eq!(ids(&shell, subscription), [work_tab]);
        assert_eq!(front(&shell, subscription).tab_id, work_tab);
    });
}

#[test]
fn what_a_tab_asks_waits_with_it_while_its_space_is_not_shown() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        shell.submit(first, "example.com").await;
        switch(&mut shell, window, work).await;
        let work_tab = front(&shell, subscription).tab_id;
        shell.submit(work_tab, "work.example.com").await;
        let asking = front(&shell, subscription).web_view_id;

        switch(&mut shell, window, home).await;
        shell.report(FakeEngine::permission_requested(
            asking,
            7,
            "https://work.example.com",
            &[PermissionKind::Camera],
        ));
        shell.settle().await;

        // The tab in front of the Space that is shown asks nothing.
        assert_eq!(front(&shell, subscription).prompt, None);
        // Work's tab still does, when it is shown.
        switch(&mut shell, window, work).await;
        let prompt = front(&shell, subscription).prompt;
        assert!(prompt.is_some(), "the question waited with its tab");
    });
}

#[test]
fn the_library_is_that_of_the_logins_of_the_space_shown() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        let project = make(&mut shell, "Project", work).await;
        shell.submit(first, "example.com").await;
        assert_eq!(history(&mut shell, window).await, [HOME_PAGE]);

        // Logins of its own: a history of its own.
        switch(&mut shell, window, work).await;
        assert_eq!(history(&mut shell, window).await, [""; 0]);
        let tab = front(&shell, subscription).tab_id;
        shell.submit(tab, "work.example.com").await;
        assert_eq!(history(&mut shell, window).await, [WORK_PAGE]);

        // Two Spaces that share logins share what is kept with them.
        switch(&mut shell, window, project).await;
        assert_eq!(history(&mut shell, window).await, [WORK_PAGE]);

        switch(&mut shell, window, home).await;
        assert_eq!(history(&mut shell, window).await, [HOME_PAGE]);
    });
}

#[test]
fn a_download_that_is_coming_shows_where_its_logins_are_shown() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        let project = make(&mut shell, "Project", work).await;
        shell.submit(first, "example.com").await;
        switch(&mut shell, window, work).await;
        let tab = front(&shell, subscription).tab_id;
        shell.submit(tab, "work.example.com").await;
        let asking = front(&shell, subscription).web_view_id;

        switch(&mut shell, window, home).await;
        let told = shell.snapshots(subscription).len();
        shell.report(FakeEngine::download_chosen(
            asking,
            1,
            "https://work.example.com/report.pdf",
            "report.pdf",
        ));
        shell.settle().await;

        // Not here: the window shows other logins, and is told nothing.
        assert_eq!(shell.window(subscription).downloads_active, 0);
        assert_eq!(shell.snapshots(subscription).len(), told);
        switch(&mut shell, window, work).await;
        assert_eq!(shell.window(subscription).downloads_active, 1);
        switch(&mut shell, window, project).await;
        assert_eq!(shell.window(subscription).downloads_active, 1);
        switch(&mut shell, window, home).await;
        assert_eq!(shell.window(subscription).downloads_active, 0);
    });
}

#[test]
fn closing_a_window_closes_the_tabs_of_every_space_it_showed() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        shell.submit(first, "example.com").await;
        let home_view = front(&shell, subscription).web_view_id;
        switch(&mut shell, window, work).await;
        let tab = front(&shell, subscription).tab_id;
        shell.submit(tab, "work.example.com").await;
        let work_view = front(&shell, subscription).web_view_id;
        switch(&mut shell, window, home).await;

        shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: window,
            }))
            .await;

        let mut gone = destroyed(&shell);
        gone.sort_unstable();
        assert_eq!(gone, [home_view, work_view]);
    });
}

#[test]
fn another_window_opens_on_the_first_space_with_a_row_of_its_own() {
    run(async {
        let mut shell = fake();
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        shell.submit(first, "example.com").await;
        switch(&mut shell, window, work).await;
        let work_tab = front(&shell, subscription).tab_id;

        let (other, other_tab, other_subscription) = shell.open_window().await;

        assert_eq!(shown(&shell, other_subscription).0, home);
        assert_eq!(ids(&shell, other_subscription), [other_tab]);
        // Each window has its own tabs of a Space.
        switch(&mut shell, other, work).await;
        let there = ids(&shell, other_subscription);
        assert_eq!(there.len(), 1);
        assert_ne!(there, [work_tab]);
        assert_eq!(ids(&shell, subscription), [work_tab]);
    });
}

/// Home with two tabs, the first in front; Work with three, the second in
/// front. The window is left showing `last`: "home" or "work".
async fn two_spaces_of_tabs(shell: &mut FakeShell, last: &str) -> (u64, u64) {
    let (window, first, subscription, home, work) = home_and_work(shell).await;
    shell.submit(first, "example.com").await;
    shell.open_tab(window, "example.org").await;
    shell.activate_tab(first).await;
    switch(shell, window, work).await;
    let tab = front(shell, subscription).tab_id;
    shell.submit(tab, "example.net").await;
    let second = shell.open_tab(window, "work.example.com").await;
    shell.open_tab(window, "work.example.com/wiki").await;
    shell.activate_tab(second).await;
    if last == "home" {
        switch(shell, window, home).await;
    }
    (home, work)
}

#[test]
fn each_spaces_tabs_come_back_and_the_window_shows_the_space_it_showed() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (home, work) = two_spaces_of_tabs(&mut shell, "work").await;
        // A crash: nothing is closed, nothing is said.
        drop(shell);

        let mut shell = live(folder.path());
        assert_eq!(names(&mut shell).await, ["", "Work"]);
        let (window, _, subscription) = shell.open_window().await;

        // The Space it showed, with its tabs and the one that was in front.
        assert_eq!(shown(&shell, subscription), (work, "Work".to_owned()));
        assert_eq!(titles(&shell, subscription), ["A Third", "Work", "Wiki"]);
        assert_eq!(front(&shell, subscription).title, "Work");
        // Only the tab in front of the Space that is shown was loaded: none
        // of the other Space's.
        assert_eq!(loads(&shell), [WORK_PAGE]);
        assert_eq!(created(&shell).len(), 1);

        // The other Space's tabs come back when it is shown, and its tab
        // in front is loaded then.
        assert!(is_ack(&switch(&mut shell, window, home).await));
        assert_eq!(
            titles(&shell, subscription),
            ["Example Domain", "Another Place"]
        );
        assert_eq!(front(&shell, subscription).title, "Example Domain");
        assert_eq!(loads(&shell), [WORK_PAGE, HOME_PAGE]);
        // In its own logins.
        let logins = created(&shell);
        assert_eq!(logins.len(), 2);
        assert_ne!(logins[0].1, logins[1].1);
    });
}

#[test]
fn a_space_that_was_not_shown_in_a_run_keeps_the_tabs_it_had() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (home, work) = two_spaces_of_tabs(&mut shell, "home").await;
        drop(shell);

        // A run in which Work is never shown, and Home's tabs change.
        let mut shell = live(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(shown(&shell, subscription).0, home);
        shell.open_tab(window, "example.net").await;
        drop(shell);

        let mut shell = live(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(
            titles(&shell, subscription),
            ["Example Domain", "Another Place", "A Third"]
        );
        switch(&mut shell, window, work).await;
        assert_eq!(titles(&shell, subscription), ["A Third", "Work", "Wiki"]);
        assert_eq!(front(&shell, subscription).title, "Work");
    });
}

#[test]
fn what_changes_in_a_space_that_is_not_shown_is_kept_too() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        shell.submit(first, "example.com").await;
        switch(&mut shell, window, work).await;
        let tab = front(&shell, subscription).tab_id;
        shell.submit(tab, "work.example.com").await;
        let asking = front(&shell, subscription).web_view_id;
        // Home is shown when Work's page has a link opened in a new tab.
        switch(&mut shell, window, home).await;
        shell.report(FakeEngine::link_open_requested(
            asking,
            "https://work.example.com/wiki",
            true,
            true,
        ));
        shell.settle().await;
        drop(shell);

        let mut shell = live(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(shown(&shell, subscription).0, home);
        switch(&mut shell, window, work).await;
        assert_eq!(titles(&shell, subscription), ["Work", "Wiki"]);
    });
}

#[test]
fn a_window_opened_once_the_first_is_closed_shows_the_space_that_one_showed() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (home, work) = two_spaces_of_tabs(&mut shell, "home").await;
        drop(shell);

        let mut shell = live(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(shown(&shell, subscription).0, home);
        switch(&mut shell, window, work).await;
        shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: window,
            }))
            .await;

        // The session is the next window's: the Space the closed one
        // showed, with its tabs; and the other's when it is shown.
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(shown(&shell, subscription), (work, "Work".to_owned()));
        assert_eq!(titles(&shell, subscription), ["A Third", "Work", "Wiki"]);
        switch(&mut shell, window, home).await;
        assert_eq!(
            titles(&shell, subscription),
            ["Example Domain", "Another Place"]
        );
    });
}

#[test]
fn a_tab_that_comes_in_front_where_nobody_looks_sleeps_on_until_it_is_shown() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (home, work) = two_spaces_of_tabs(&mut shell, "work").await;
        drop(shell);

        // Work is shown: "Work" is in front and loaded, the rest sleep.
        let mut shell = live(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(shown(&shell, subscription).0, work);
        let in_front = front(&shell, subscription);
        // Its page opens a window, which is then all that is left after
        // the first tab: the person closes the one that opened it.
        let (popup, report) = shell.engine.ask_for_window(
            in_front.web_view_id,
            "https://work.example.com/popup",
            true,
        );
        shell.report(report);
        shell.settle().await;
        shell.close_tab(in_front.tab_id).await;
        let wiki = ids(&shell, subscription)[2];
        shell.close_tab(wiki).await;
        assert_eq!(titles(&shell, subscription).len(), 2);
        assert_eq!(front(&shell, subscription).web_view_id, popup);
        let loaded = loads(&shell);

        // With the other Space shown, the window the page opened closes
        // itself: a tab that is asleep is now in front of Work, unseen.
        switch(&mut shell, window, home).await;
        let after_home = loads(&shell);
        assert_eq!(after_home.len(), loaded.len() + 1, "Home's tab in front");
        shell.report(FakeEngine::close_requested(popup));
        shell.settle().await;
        assert_eq!(loads(&shell), after_home, "nothing is loaded for nobody");

        // Shown, it is loaded.
        switch(&mut shell, window, work).await;
        assert_eq!(titles(&shell, subscription), ["A Third"]);
        assert_eq!(loads(&shell).len(), after_home.len() + 1);
        assert_eq!(loads(&shell).last().unwrap(), "https://example.net/");
    });
}

#[test]
fn a_space_made_and_named_is_there_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = live(folder.path());
        let (window, first, subscription, home, work) = home_and_work(&mut shell).await;
        let project = make(&mut shell, "Project", work).await;
        rename(&mut shell, home, "Home").await;
        shell.submit(first, "example.com").await;
        switch(&mut shell, window, project).await;
        let tab = front(&shell, subscription).tab_id;
        shell.submit(tab, "work.example.com").await;
        let logins = front(&shell, subscription).login_set_id;
        let before = spaces(&mut shell).await;
        drop(shell);

        let mut shell = live(folder.path());
        assert_eq!(spaces(&mut shell).await, before);
        assert_eq!(
            before,
            [
                (home, "Home".to_owned(), 1),
                (work, "Work".to_owned(), 2),
                (project, "Project".to_owned(), 2),
            ]
        );
        // The window shows the Space it showed, in the logins it had: the
        // ones it shares, which were made at random and are the same now.
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(shown(&shell, subscription), (project, "Project".to_owned()));
        assert_eq!(front(&shell, subscription).login_set_id, logins);
        switch(&mut shell, window, work).await;
        let tab = front(&shell, subscription).tab_id;
        shell.submit(tab, "work.example.com/wiki").await;
        assert_eq!(front(&shell, subscription).login_set_id, logins);
    });
}
