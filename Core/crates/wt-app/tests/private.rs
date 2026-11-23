//! Private windows (037): a login set of its own, thrown away with the
//! window; nothing of it written or offered back; nothing crossing between
//! it and an ordinary window. Through the contract, against the scripted
//! engine, and over a store on disk that is read afterwards for what it
//! holds.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_app::PROMPT_DELAY;
use wt_proto::v1::PermissionKind::Camera;
use wt_proto::v1::{
    AnswerPrompt, CloseWindow, OpenAddress, QueryDownloads, QueryHistory, QueryPermissions,
    QuerySpaces, SearchWords, SuggestAddresses, SwitchSpace, TabView, Undo, WindowSnapshot,
    ZoomChange, ZoomPage, host_request, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

/// What an ordinary window visits.
const OPEN: &str = "https://example.com/";
/// What a private window visits: a name nothing else has.
const HIDDEN: &str = "https://hidden.example.org/";
const HIDDEN_MORE: &str = "https://hidden.example.org/more";
const HIDDEN_FILE: &str = "https://hidden.example.org/files/report.pdf";
/// The name to look for in what was written.
const HIDDEN_NAME: &[u8] = b"hidden.example";

fn web(shell: &mut FakeShell) {
    shell.engine.page(OPEN, "Example");
    shell.engine.page(HIDDEN, "Hidden");
    shell.engine.page(HIDDEN_MORE, "Hidden More");
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

fn window(shell: &FakeShell, subscription: u64) -> WindowSnapshot {
    shell.window(subscription)
}

fn front(shell: &FakeShell, subscription: u64) -> TabView {
    window(shell, subscription).tab.expect("a window has a tab")
}

fn strip(shell: &FakeShell, subscription: u64) -> Vec<String> {
    window(shell, subscription)
        .tabs
        .into_iter()
        .map(|tab| tab.title)
        .collect()
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// The login sets the shell was told to make web views in, in order: each
/// one's name, and whether the engine is to keep nothing of it.
fn made_in(shell: &FakeShell) -> Vec<(String, bool)> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewCreate(create) => create
                .login_set
                .as_ref()
                .map(|login_set| (login_set.id.clone(), login_set.ephemeral)),
            _ => None,
        })
        .collect()
}

async fn history(shell: &mut FakeShell, window: u64) -> Vec<String> {
    let answer = shell
        .request(request::Kind::QueryHistory(QueryHistory {
            window_id: window,
            ..Default::default()
        }))
        .await;
    match answer {
        response::Kind::HistoryPage(page) => {
            page.entries.into_iter().map(|entry| entry.url).collect()
        }
        other => panic!("QueryHistory answers HistoryPage, not {other:?}"),
    }
}

async fn offered(shell: &mut FakeShell, window: u64, text: &str) -> Vec<String> {
    let answer = shell
        .request(request::Kind::SuggestAddresses(SuggestAddresses {
            window_id: window,
            text: text.to_owned(),
            limit: 0,
            ..SuggestAddresses::default()
        }))
        .await;
    match answer {
        response::Kind::AddressSuggestions(offered) => offered
            .suggestions
            .into_iter()
            .map(|place| place.url)
            .collect(),
        other => panic!("SuggestAddresses answers AddressSuggestions, not {other:?}"),
    }
}

/// Every file of the folder the store is in, read whole: the database, and
/// whatever else it keeps beside it.
fn written(folder: &Path) -> Vec<u8> {
    let mut all = Vec::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        all.extend(std::fs::read(entry.unwrap().path()).unwrap());
    }
    all
}

fn holds(bytes: &[u8], name: &[u8]) -> bool {
    bytes.windows(name.len()).any(|part| part == name)
}

#[test]
fn a_private_window_has_a_login_set_of_its_own_that_the_engine_is_to_keep_nothing_of() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, ordinary_tab, ordinary) = shell.open_window().await;
        shell.submit(ordinary_tab, OPEN).await;
        let (first_window, first_tab, first) = shell.open_private_window().await;
        shell.submit(first_tab, HIDDEN).await;
        let (second_window, second_tab, second) = shell.open_private_window().await;
        shell.submit(second_tab, HIDDEN).await;

        // The windows are told which they are, and a private one names no Space.
        assert!(!window(&shell, ordinary).private_window);
        assert_ne!(window(&shell, ordinary).space_id, 0);
        for private in [first, second] {
            let told = window(&shell, private);
            assert!(told.private_window);
            assert_eq!(
                (
                    told.space_id,
                    told.space_name.as_str(),
                    told.space_colour.as_str()
                ),
                (0, "", "")
            );
            assert_eq!(told.tabs.len(), 1, "one tab, as any new window has");
        }

        // The engine: an ordinary login set it keeps; each private window's is its own,
        // and one it is to keep nothing of.
        let made = made_in(&shell);
        assert_eq!(made.len(), 3);
        assert!(!made[0].1, "an ordinary window's login set is kept");
        assert!(
            made[1].1 && made[2].1,
            "a private window's is thrown away: {made:?}"
        );
        assert_ne!(made[1].0, made[0].0);
        assert_ne!(made[2].0, made[0].0);
        assert_ne!(made[1].0, made[2].0, "two private windows share nothing");
        // And the tab says the same login set as its web view was made in.
        assert_eq!(front(&shell, first).login_set_id, made[1].0);
        assert_eq!(front(&shell, second).login_set_id, made[2].0);

        // A tab opened later in a private window is in that window's login set, and in
        // no other's: not the other private window's.
        for (private_window, seen, own) in [
            (first_window, first, made[1].0.clone()),
            (second_window, second, made[2].0.clone()),
        ] {
            shell.open_tab(private_window, HIDDEN_MORE).await;
            assert_eq!(front(&shell, seen).title, "Hidden More");
            assert_eq!(front(&shell, seen).login_set_id, own);
        }
        let later = made_in(&shell);
        assert_eq!(
            (later[3].clone(), later[4].clone()),
            (made[1].clone(), made[2].clone())
        );
    });
}

#[test]
fn nothing_a_private_window_visits_is_written_or_offered_back() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (ordinary, ordinary_tab, _) = shell.open_window().await;
        shell.submit(ordinary_tab, OPEN).await;
        let (private, private_tab, private_seen) = shell.open_private_window().await;
        shell.submit(private_tab, HIDDEN).await;
        shell.submit(private_tab, HIDDEN_MORE).await;
        assert_eq!(front(&shell, private_seen).title, "Hidden More");

        // The private window has no history, and is offered nothing from any: not what
        // it visited, and not what the ordinary window did.
        assert_eq!(history(&mut shell, private).await, Vec::<String>::new());
        assert_eq!(
            offered(&mut shell, private, "hidden").await,
            Vec::<String>::new()
        );
        assert_eq!(
            offered(&mut shell, private, "example").await,
            Vec::<String>::new()
        );
        // The ordinary window has its own visit, and nothing of the private one's.
        assert_eq!(history(&mut shell, ordinary).await, [OPEN]);
        assert_eq!(
            offered(&mut shell, ordinary, "hidden").await,
            Vec::<String>::new()
        );
        assert_eq!(offered(&mut shell, ordinary, "example").await, [OPEN]);

        // Left open as a quit or a crash leaves it. What was written holds the ordinary
        // window's site, and nothing of the private one's name.
        drop(shell);
        let kept = written(folder.path());
        assert!(
            holds(&kept, b"example.com"),
            "the search finds what is written"
        );
        assert!(
            !holds(&kept, HIDDEN_NAME),
            "nothing of the private window's site is in the store"
        );
    });
}

#[test]
fn a_private_window_is_not_the_session_and_never_comes_back() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        // A private window opened first is not the window whose tabs are kept.
        let (_, private_tab, _) = shell.open_private_window().await;
        shell.submit(private_tab, HIDDEN).await;
        let (_, ordinary_tab, _) = shell.open_window().await;
        shell.submit(ordinary_tab, OPEN).await;
        drop(shell);
        assert!(!holds(&written(folder.path()), HIDDEN_NAME));

        // The next run: a private window opened first is given nothing of the session;
        // the first ordinary window is, and it is the ordinary window's tab.
        let mut shell = start(folder.path());
        let (_, _, private) = shell.open_private_window().await;
        assert_eq!(strip(&shell, private), [""], "one empty tab");
        assert_eq!(
            made_in(&shell),
            [],
            "nothing is loaded for a private window that opens"
        );
        let (_, _, ordinary) = shell.open_window().await;
        assert_eq!(strip(&shell, ordinary), ["Example"]);
        assert_eq!(strip(&shell, private), [""]);
    });
}

#[test]
fn a_closed_private_tab_is_not_journalled_and_not_brought_back() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (ordinary, _, ordinary_seen) = shell.open_window().await;
        let (private, first, private_seen) = shell.open_private_window().await;
        shell.submit(first, HIDDEN).await;
        let second = shell.open_tab(private, HIDDEN_MORE).await;
        assert_eq!(strip(&shell, private_seen), ["Hidden", "Hidden More"]);

        shell.close_tab(second).await;

        assert_eq!(strip(&shell, private_seen), ["Hidden"]);
        // There is nothing to undo, in the private window or in any other.
        for (asked, seen) in [(private, private_seen), (ordinary, ordinary_seen)] {
            assert_eq!(window(&shell, seen).undo, "");
            let answer = shell
                .request(request::Kind::Undo(Undo { window_id: asked }))
                .await;
            assert_eq!(error_code(answer), "undo.nothing");
        }
        assert_eq!(strip(&shell, private_seen), ["Hidden"]);
        // The last tab closed leaves an empty one, as in any window, and still nothing.
        shell.close_tab(first).await;
        assert_eq!(strip(&shell, private_seen), [""]);
        assert_eq!(window(&shell, private_seen).undo, "");
        drop(shell);
        assert!(!holds(&written(folder.path()), HIDDEN_NAME));
    });
}

async fn zoom_in(shell: &mut FakeShell, tab: u64) {
    let answer = shell
        .request(request::Kind::ZoomPage(ZoomPage {
            tab_id: tab,
            change: ZoomChange::In as i32,
        }))
        .await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

fn zoom_of(shell: &FakeShell, subscription: u64) -> u32 {
    front(shell, subscription)
        .zoom
        .expect("a page that can be zoomed")
        .percent
}

#[test]
fn a_sites_zoom_is_kept_while_the_private_window_is_open_and_no_longer() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (ordinary, ordinary_tab, ordinary_seen) = shell.open_window().await;
        let (private, private_tab, private_seen) = shell.open_private_window().await;
        shell.submit(private_tab, HIDDEN).await;
        assert_eq!(zoom_of(&shell, private_seen), 100);

        zoom_in(&mut shell, private_tab).await;

        let bigger = zoom_of(&shell, private_seen);
        assert!(bigger > 100);
        // Kept for the site while the window is open: another page of it, in another tab
        // of the private window, arrives at the same level.
        shell.open_tab(private, HIDDEN_MORE).await;
        assert_eq!(front(&shell, private_seen).title, "Hidden More");
        assert_eq!(zoom_of(&shell, private_seen), bigger);
        // Not the ordinary window's: the same site there is at its own size.
        shell.submit(ordinary_tab, HIDDEN).await;
        assert_eq!(zoom_of(&shell, ordinary_seen), 100);
        let _ = ordinary;

        // Gone with the window: another private window finds the site at its own size.
        let closed = shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: private,
            }))
            .await;
        assert!(matches!(closed, response::Kind::Ack(_)), "{closed:?}");
        let (_, again_tab, again_seen) = shell.open_private_window().await;
        shell.submit(again_tab, HIDDEN).await;
        assert_eq!(zoom_of(&shell, again_seen), 100);
    });
}

/// What the sites of a window's login set were allowed, by origin.
async fn allowed(shell: &mut FakeShell, window: u64) -> Vec<String> {
    let answer = shell
        .request(request::Kind::QueryPermissions(QueryPermissions {
            window_id: window,
        }))
        .await;
    let response::Kind::PermissionsPage(page) = answer else {
        panic!("expected the permissions, got {answer:?}");
    };
    page.sites.into_iter().map(|site| site.origin).collect()
}

/// The page of the tab in front asks for the camera; whether the person is asked.
async fn asks(shell: &mut FakeShell, subscription: u64, request: u64) -> Option<u64> {
    let web_view = front(shell, subscription).web_view_id;
    shell.report(FakeEngine::permission_requested(
        web_view,
        request,
        "https://hidden.example.org",
        &[Camera],
    ));
    shell.settle().await;
    front(shell, subscription)
        .prompt
        .map(|prompt| prompt.prompt_id)
}

async fn allow(shell: &mut FakeShell, tab: u64, prompt: u64) {
    tokio::time::advance(PROMPT_DELAY).await;
    let done = shell
        .request(request::Kind::AnswerPrompt(AnswerPrompt {
            tab_id: tab,
            prompt_id: prompt,
            allow: true,
            ..Default::default()
        }))
        .await;
    assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
}

#[test]
fn what_a_site_may_use_is_kept_for_the_private_window_only() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (ordinary, ordinary_tab, ordinary_seen) = shell.open_window().await;
        shell.submit(ordinary_tab, HIDDEN).await;
        let (private, private_tab, private_seen) = shell.open_private_window().await;
        shell.submit(private_tab, HIDDEN).await;

        // Allowed in the ordinary window, the site is asked about afresh in the private
        // one: what was allowed elsewhere is not the private window's.
        let prompt = asks(&mut shell, ordinary_seen, 1)
            .await
            .expect("the person is asked");
        allow(&mut shell, ordinary_tab, prompt).await;
        assert_eq!(
            allowed(&mut shell, ordinary).await,
            ["https://hidden.example.org"]
        );
        assert_eq!(allowed(&mut shell, private).await, Vec::<String>::new());
        let prompt = asks(&mut shell, private_seen, 2)
            .await
            .expect("asked afresh in the private window");
        allow(&mut shell, private_tab, prompt).await;

        // Kept while the window is open: the site is not asked about again there.
        assert_eq!(
            allowed(&mut shell, private).await,
            ["https://hidden.example.org"]
        );
        assert_eq!(
            asks(&mut shell, private_seen, 3).await,
            None,
            "allowed already, here"
        );

        // Gone with the window: the next private window is asked afresh, and the
        // ordinary window's answer is as it was.
        let closed = shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: private,
            }))
            .await;
        assert!(matches!(closed, response::Kind::Ack(_)), "{closed:?}");
        let (again, again_tab, again_seen) = shell.open_private_window().await;
        shell.submit(again_tab, HIDDEN).await;
        assert_eq!(allowed(&mut shell, again).await, Vec::<String>::new());
        assert!(asks(&mut shell, again_seen, 4).await.is_some());
        assert_eq!(
            allowed(&mut shell, ordinary).await,
            ["https://hidden.example.org"]
        );
    });
}

/// The window's downloads: each one's number and its address.
async fn downloads(shell: &mut FakeShell, window: u64) -> Vec<(u64, String)> {
    let answer = shell
        .request(request::Kind::QueryDownloads(QueryDownloads {
            window_id: window,
            before_id: 0,
            limit: 0,
        }))
        .await;
    match answer {
        response::Kind::DownloadsPage(page) => page
            .downloads
            .into_iter()
            .map(|download| (download.id, download.url))
            .collect(),
        other => panic!("QueryDownloads answers DownloadsPage, not {other:?}"),
    }
}

#[test]
fn a_private_download_is_listed_in_its_window_and_written_nowhere() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (ordinary, ordinary_tab, ordinary_seen) = shell.open_window().await;
        shell.submit(ordinary_tab, OPEN).await;
        let (private, private_tab, private_seen) = shell.open_private_window().await;
        shell.submit(private_tab, HIDDEN).await;

        // The person saves a file in each window (022: chosen in a menu, nobody asked).
        let ordinary_view = front(&shell, ordinary_seen).web_view_id;
        shell.report(FakeEngine::download_chosen(
            ordinary_view,
            7,
            "https://example.com/files/open.pdf",
            "open.pdf",
        ));
        let private_view = front(&shell, private_seen).web_view_id;
        shell.report(FakeEngine::download_chosen(
            private_view,
            8,
            HIDDEN_FILE,
            "report.pdf",
        ));
        shell.settle().await;
        assert_eq!(shell.engine.download_starts.len(), 2, "both are started");

        // Each window lists its own, and neither the other's.
        let mine = downloads(&mut shell, private).await;
        let theirs = downloads(&mut shell, ordinary).await;
        assert_eq!(
            mine.iter().map(|d| d.1.as_str()).collect::<Vec<_>>(),
            [HIDDEN_FILE]
        );
        assert_eq!(
            theirs.iter().map(|d| d.1.as_str()).collect::<Vec<_>>(),
            ["https://example.com/files/open.pdf"]
        );
        // A number names one download, whichever window it is of.
        assert_ne!(mine[0].0, theirs[0].0);
        assert_eq!(window(&shell, private_seen).downloads_active, 1);

        // It finishes, and is still listed there, as finished; and is written nowhere.
        shell.report(FakeEngine::download_finished(8, "report.pdf"));
        shell.settle().await;
        assert_eq!(downloads(&mut shell, private).await.len(), 1);
        assert_eq!(window(&shell, private_seen).downloads_active, 0);
        drop(shell);
        let kept = written(folder.path());
        assert!(
            holds(&kept, b"open.pdf"),
            "the ordinary window's download is written"
        );
        assert!(!holds(&kept, HIDDEN_NAME));
        assert!(!holds(&kept, b"report.pdf"));
    });
}

#[test]
fn a_private_window_has_no_space_and_nothing_from_outside_opens_in_it() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (ordinary, _, ordinary_seen) = shell.open_window().await;
        let (private, _, private_seen) = shell.open_private_window().await;
        let home = window(&shell, ordinary_seen).space_id;

        // It is in no list of Spaces, and cannot be switched to one; nor can a window
        // be switched to a private window's row.
        let listed = shell
            .request(request::Kind::QuerySpaces(QuerySpaces {}))
            .await;
        let response::Kind::SpacesPage(page) = listed else {
            panic!("QuerySpaces answers SpacesPage, not {listed:?}");
        };
        assert_eq!(
            page.spaces.iter().map(|s| s.space_id).collect::<Vec<_>>(),
            [home]
        );
        let switch = |window_id, space_id| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id,
                space_id,
            })
        };
        assert_eq!(
            error_code(shell.request(switch(private, home)).await),
            "window.private"
        );
        assert_eq!(
            error_code(shell.request(switch(ordinary, 1 << 62)).await),
            "space.unknown"
        );
        assert!(window(&shell, private_seen).private_window);

        // What comes from outside the browser never opens in it.
        let link = shell
            .request(request::Kind::OpenAddress(OpenAddress {
                window_id: private,
                url: OPEN.to_owned(),
            }))
            .await;
        assert_eq!(error_code(link), "window.private");
        let search = shell
            .request(request::Kind::SearchWords(SearchWords {
                window_id: private,
                text: "tigers".to_owned(),
            }))
            .await;
        assert_eq!(error_code(search), "window.private");
        assert_eq!(strip(&shell, private_seen), [""], "no tab was opened there");
        assert_eq!(made_in(&shell), [], "and nothing was loaded");
    });
}

#[test]
fn a_window_a_private_page_opens_is_a_tab_of_the_same_private_window() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, _, ordinary_seen) = shell.open_window().await;
        let (_, private_tab, private_seen) = shell.open_private_window().await;
        shell.submit(private_tab, HIDDEN).await;
        let mine = front(&shell, private_seen).login_set_id;

        let opener = front(&shell, private_seen).web_view_id;
        let (_, asks) = shell.engine.ask_for_window(opener, HIDDEN_MORE, true);
        shell.report(asks);
        shell.settle().await;

        assert_eq!(window(&shell, private_seen).tabs.len(), 2);
        assert_eq!(
            front(&shell, private_seen).login_set_id,
            mine,
            "in the private window's own login set"
        );
        assert!(window(&shell, private_seen).private_window);
        assert_eq!(
            window(&shell, ordinary_seen).tabs.len(),
            1,
            "and not in the ordinary window"
        );
    });
}
