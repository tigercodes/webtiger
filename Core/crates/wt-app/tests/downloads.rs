//! Downloads (013), through the contract, against the scripted engine:
//! which downloads start and under what name, what the list and the
//! snapshot say of them, and what is asked of the shell.
//!
//! A site's first download asks the person (015): every page here is of a
//! site that was allowed to download first, as a person would allow it. What
//! asking does is in `first_download.rs`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;
use std::time::Duration;

use wt_app::{DOWNLOAD_INTERVAL, PROMPT_DELAY};
use wt_proto::v1::{
    AnswerPrompt, CancelDownload, CreateSpace, DownloadEntry, DownloadState, ForgetDownloads,
    OpenDownload, QueryDownloads, SwitchSpace, TabView, forget_downloads, host_request, request,
    response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const REPORT: &str = "https://example.com/files/report.pdf";

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// How many downloads the window's snapshot says are coming.
fn coming(shell: &FakeShell, subscription: u64) -> u32 {
    shell.window(subscription).downloads_active
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

async fn page_of(
    shell: &mut FakeShell,
    window: u64,
    before: u64,
    limit: u32,
) -> (Vec<DownloadEntry>, bool) {
    let answer = shell
        .request(request::Kind::QueryDownloads(QueryDownloads {
            window_id: window,
            before_id: before,
            limit,
        }))
        .await;
    match answer {
        response::Kind::DownloadsPage(page) => (page.downloads, page.more),
        other => panic!("QueryDownloads answers DownloadsPage, not {other:?}"),
    }
}

/// The window's downloads, newest first.
async fn list(shell: &mut FakeShell, window: u64) -> Vec<DownloadEntry> {
    page_of(shell, window, 0, 0).await.0
}

/// Each download's name and how it stands: "coming", "finished", or the
/// code it failed with.
async fn standing(shell: &mut FakeShell, window: u64) -> Vec<(String, String)> {
    list(shell, window)
        .await
        .into_iter()
        .map(|download| {
            let state = match download.state() {
                DownloadState::Coming => "coming".to_owned(),
                DownloadState::Finished => "finished".to_owned(),
                DownloadState::Failed => download.error_code,
                DownloadState::Unspecified => panic!("a download has a state"),
            };
            (download.file_name, state)
        })
        .collect()
}

async fn cancel(shell: &mut FakeShell, download: u64) -> response::Kind {
    shell
        .request(request::Kind::CancelDownload(CancelDownload {
            download_id: download,
        }))
        .await
}

async fn forget(
    shell: &mut FakeShell,
    window: u64,
    what: Option<forget_downloads::What>,
) -> response::Kind {
    shell
        .request(request::Kind::ForgetDownloads(ForgetDownloads {
            window_id: window,
            what,
        }))
        .await
}

/// Has the person allow the site of the page in the tab in front to
/// download (015): its first download asks, they say yes, and that download
/// is stopped and forgotten. A test then begins with nothing listed, nothing
/// coming, nothing asked of the shell, and no download in the last second.
async fn allow_downloads(shell: &mut FakeShell, window: u64, tab: u64, subscription: u64) {
    let web_view = front(shell, subscription).web_view_id;
    shell.report(FakeEngine::download_requested(
        web_view,
        9_000,
        REPORT,
        "the first of all.pdf",
        "application/pdf",
        1,
    ));
    shell.settle().await;
    let asked = front(shell, subscription)
        .prompt
        .expect("a site's first download asks");
    tokio::time::advance(PROMPT_DELAY).await;
    let allowed = shell
        .request(request::Kind::AnswerPrompt(AnswerPrompt {
            tab_id: tab,
            prompt_id: asked.prompt_id,
            allow: true,
            ..Default::default()
        }))
        .await;
    assert!(matches!(allowed, response::Kind::Ack(_)), "{allowed:?}");
    let everything = forget_downloads::What::Everything(true);
    forget(shell, window, Some(everything)).await;
    tokio::time::advance(DOWNLOAD_INTERVAL).await;
    shell.engine.download_starts.clear();
    shell.engine.download_cancels.clear();
}

/// A window whose one tab shows example.com, which may download. Its
/// window, tab, subscription and the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    shell.engine.page("https://example.com/", "Example");
    shell.engine.page("https://example.org/", "Another");
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    allow_downloads(shell, window, tab, subscription).await;
    (window, tab, subscription, web_view)
}

/// The page asks for a download the shell names `download`, of the report.
async fn ask(shell: &mut FakeShell, web_view: u64, download: u64, suggested: &str) {
    shell.report(FakeEngine::download_requested(
        web_view,
        download,
        REPORT,
        suggested,
        "application/pdf",
        1_000,
    ));
    shell.settle().await;
}

#[test]
fn a_download_starts_under_a_name_the_core_makes_and_is_listed() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;
        assert_eq!(coming(&shell, subscription), 0);
        assert_eq!(list(&mut shell, window).await, []);

        // The site suggests a name with folders in it.
        ask(&mut shell, web_view, 7, "../../Report 2026.pdf").await;

        assert_eq!(
            shell.engine.download_starts,
            [(7, "Report 2026.pdf".to_owned())],
            "the shell is asked to start it, under the safe name"
        );
        assert_eq!(shell.engine.download_cancels, []);
        let listed = list(&mut shell, window).await;
        assert_eq!(listed.len(), 1);
        let download = &listed[0];
        assert_eq!(download.file_name, "Report 2026.pdf");
        assert_eq!(download.url, REPORT);
        assert_eq!(download.host, "example.com");
        assert_eq!(download.mime_type, "application/pdf");
        assert_eq!(download.state(), DownloadState::Coming);
        assert_eq!((download.received_bytes, download.total_bytes), (0, 1_000));
        assert_eq!(
            coming(&shell, subscription),
            1,
            "the snapshot says one is coming"
        );
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn with_nothing_suggested_the_address_names_the_file() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "").await;
        assert_eq!(shell.engine.download_starts, [(7, "report.pdf".to_owned())]);
        assert_eq!(list(&mut shell, window).await[0].file_name, "report.pdf");
    });
}

#[test]
fn how_far_it_has_got_is_in_the_list_and_how_it_finished_is_kept() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "report.pdf").await;

        shell.report(FakeEngine::download_progress(7, 400, 1_000));
        shell.settle().await;
        let listed = list(&mut shell, window).await;
        assert_eq!(
            (listed[0].received_bytes, listed[0].total_bytes),
            (400, 1_000)
        );
        assert_eq!(coming(&shell, subscription), 1);

        // The name was taken: the shell says what it used.
        shell.report(FakeEngine::download_finished(7, "report (2).pdf"));
        shell.settle().await;

        assert_eq!(
            standing(&mut shell, window).await,
            [("report (2).pdf".to_owned(), "finished".to_owned())]
        );
        assert_eq!(coming(&shell, subscription), 0);
        // A word about a download that has ended changes nothing.
        shell.report(FakeEngine::download_failed(7, "download.network"));
        shell.report(FakeEngine::download_progress(7, 1, 2));
        shell.settle().await;
        assert_eq!(
            standing(&mut shell, window).await,
            [("report (2).pdf".to_owned(), "finished".to_owned())]
        );
    });
}

#[test]
fn a_size_the_site_did_not_say_is_the_one_the_shell_learns() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, web_view) = a_page(&mut shell).await;
        shell.report(FakeEngine::download_requested(
            web_view, 7, REPORT, "", "", 0,
        ));
        shell.settle().await;
        assert_eq!(list(&mut shell, window).await[0].total_bytes, 0);

        shell.report(FakeEngine::download_progress(7, 10, 500));
        shell.settle().await;

        let listed = list(&mut shell, window).await;
        assert_eq!((listed[0].received_bytes, listed[0].total_bytes), (10, 500));
        // A finish that names no file leaves the name that was asked for.
        shell.report(FakeEngine::download_finished(7, ""));
        shell.settle().await;
        assert_eq!(
            standing(&mut shell, window).await,
            [("report.pdf".to_owned(), "finished".to_owned())]
        );
    });
}

#[test]
fn a_failure_is_listed_by_its_code_and_what_is_not_a_code_is_not_passed_on() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "a.pdf").await;
        shell.report(FakeEngine::download_failed(7, "download.network"));
        shell.settle().await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        ask(&mut shell, web_view, 8, "b.pdf").await;
        shell.report(FakeEngine::download_failed(8, "The disk is full!"));
        shell.settle().await;

        assert_eq!(
            standing(&mut shell, window).await,
            [
                ("b.pdf".to_owned(), "download.failed".to_owned()),
                ("a.pdf".to_owned(), "download.network".to_owned()),
            ],
            "newest first; words that are not a code are not kept"
        );
        assert_eq!(coming(&shell, subscription), 0);
    });
}

#[test]
fn a_web_view_starts_one_download_a_second() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;
        let other_tab = shell.open_tab(window, "example.org").await;
        let other_view = front(&shell, subscription).web_view_id;
        assert_ne!(other_view, web_view);
        allow_downloads(&mut shell, window, other_tab, subscription).await;

        ask(&mut shell, web_view, 7, "first.pdf").await;
        ask(&mut shell, web_view, 8, "second.pdf").await;
        // Another tab's page is not held back by this one's.
        ask(&mut shell, other_view, 9, "theirs.pdf").await;

        assert_eq!(
            shell.engine.download_starts,
            [(7, "first.pdf".to_owned()), (9, "theirs.pdf".to_owned())]
        );
        assert_eq!(shell.engine.download_cancels, [8], "the second is dropped");
        assert_eq!(list(&mut shell, window).await.len(), 2, "and is in no list");

        // A millisecond short of a second later is still too soon; then not.
        tokio::time::advance(DOWNLOAD_INTERVAL - Duration::from_millis(1)).await;
        ask(&mut shell, web_view, 10, "too soon.pdf").await;
        assert_eq!(shell.engine.download_cancels, [8, 10]);
        tokio::time::advance(Duration::from_millis(1)).await;
        ask(&mut shell, web_view, 11, "third.pdf").await;
        assert_eq!(
            shell.engine.download_starts.last(),
            Some(&(11, "third.pdf".to_owned()))
        );
        assert_eq!(coming(&shell, subscription), 3);
    });
}

#[test]
fn a_web_view_the_core_does_not_have_gets_no_download() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;

        ask(&mut shell, web_view + 1_000, 7, "nobody's.pdf").await;

        assert_eq!(shell.engine.download_starts, []);
        assert_eq!(shell.engine.download_cancels, [7]);
        assert_eq!(list(&mut shell, window).await, []);
        assert_eq!(coming(&shell, subscription), 0);
    });
}

#[test]
fn a_report_said_twice_starts_one_download() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "once.pdf").await;
        ask(&mut shell, web_view, 7, "once.pdf").await;

        assert_eq!(shell.engine.download_starts.len(), 1);
        assert_eq!(
            shell.engine.download_cancels,
            [],
            "and the one that is coming is not dropped"
        );
        assert_eq!(list(&mut shell, window).await.len(), 1);
    });
}

#[test]
fn a_shell_that_will_not_start_it_leaves_a_download_that_failed() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;
        shell.engine.downloads_not_started.push(7);

        ask(&mut shell, web_view, 7, "gone.pdf").await;

        assert_eq!(
            standing(&mut shell, window).await,
            [("gone.pdf".to_owned(), "download.failed".to_owned())]
        );
        assert_eq!(coming(&shell, subscription), 0);
    });
}

#[test]
fn stopping_a_download_ends_it_and_has_the_shell_drop_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "stopped.pdf").await;
        let id = list(&mut shell, window).await[0].id;

        let answer = cancel(&mut shell, id).await;

        assert!(matches!(answer, response::Kind::Ack(_)));
        assert_eq!(
            shell.engine.download_cancels,
            [7],
            "by the shell's name for it"
        );
        assert_eq!(
            standing(&mut shell, window).await,
            [("stopped.pdf".to_owned(), "download.cancelled".to_owned())]
        );
        assert_eq!(coming(&shell, subscription), 0);

        // What the shell says of it afterwards changes nothing.
        shell.report(FakeEngine::download_finished(7, "stopped.pdf"));
        shell.settle().await;
        assert_eq!(
            standing(&mut shell, window).await,
            [("stopped.pdf".to_owned(), "download.cancelled".to_owned())]
        );
        // One that is not coming cannot be stopped; nor can one there is not.
        assert_eq!(error_code(cancel(&mut shell, id).await), "download.unknown");
        assert_eq!(
            error_code(cancel(&mut shell, id + 1_000).await),
            "download.unknown"
        );
        assert_eq!(shell.engine.download_cancels.len(), 1);
    });
}

#[test]
fn forgetting_takes_downloads_off_the_list_and_stops_one_that_is_coming() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;
        for (download, name) in [(7, "a.pdf"), (8, "b.pdf"), (9, "c.pdf")] {
            ask(&mut shell, web_view, download, name).await;
            tokio::time::advance(DOWNLOAD_INTERVAL).await;
        }
        shell.report(FakeEngine::download_finished(7, "a.pdf"));
        shell.settle().await;
        let listed = list(&mut shell, window).await;
        let (c, b, a) = (listed[0].id, listed[1].id, listed[2].id);

        // One that has ended: off the list, and nothing asked of the shell.
        let answer = forget(
            &mut shell,
            window,
            Some(forget_downloads::What::DownloadId(a)),
        )
        .await;
        assert!(matches!(answer, response::Kind::Ack(_)));
        assert_eq!(shell.engine.download_cancels, []);
        // One that is coming is stopped first.
        forget(
            &mut shell,
            window,
            Some(forget_downloads::What::DownloadId(b)),
        )
        .await;
        assert_eq!(shell.engine.download_cancels, [8]);
        assert_eq!(
            standing(&mut shell, window).await,
            [("c.pdf".to_owned(), "coming".to_owned())]
        );
        assert_eq!(coming(&shell, subscription), 1);
        let _ = c;

        // All of them.
        forget(
            &mut shell,
            window,
            Some(forget_downloads::What::Everything(true)),
        )
        .await;
        assert_eq!(shell.engine.download_cancels, [8, 9]);
        assert_eq!(list(&mut shell, window).await, []);
        assert_eq!(coming(&shell, subscription), 0);

        // Naming nothing forgets nothing; nor does one there is not.
        for nothing in [None, Some(forget_downloads::What::Everything(false))] {
            assert_eq!(
                error_code(forget(&mut shell, window, nothing).await),
                "download.nothing_named"
            );
        }
        let none = forget(
            &mut shell,
            window,
            Some(forget_downloads::What::DownloadId(a)),
        )
        .await;
        assert!(matches!(none, response::Kind::Ack(_)));
    });
}

#[test]
fn a_window_there_is_not_has_no_downloads() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, _) = a_page(&mut shell).await;
        let answer = shell
            .request(request::Kind::QueryDownloads(QueryDownloads {
                window_id: window + 1_000,
                before_id: 0,
                limit: 0,
            }))
            .await;
        assert_eq!(error_code(answer), "window.unknown");
        assert_eq!(
            error_code(
                forget(
                    &mut shell,
                    window + 1_000,
                    Some(forget_downloads::What::Everything(true))
                )
                .await
            ),
            "window.unknown"
        );
    });
}

#[test]
fn the_list_is_a_page_at_a_time_newest_first() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, web_view) = a_page(&mut shell).await;
        for (download, name) in [(1, "a.pdf"), (2, "b.pdf"), (3, "c.pdf")] {
            ask(&mut shell, web_view, download, name).await;
            tokio::time::advance(DOWNLOAD_INTERVAL).await;
        }

        let (first, more) = page_of(&mut shell, window, 0, 2).await;
        assert_eq!(
            first
                .iter()
                .map(|d| d.file_name.as_str())
                .collect::<Vec<_>>(),
            ["c.pdf", "b.pdf"]
        );
        assert!(more);
        let (rest, more) = page_of(&mut shell, window, first[1].id, 2).await;
        assert_eq!(
            rest.iter()
                .map(|d| d.file_name.as_str())
                .collect::<Vec<_>>(),
            ["a.pdf"]
        );
        assert!(!more);
    });
}

#[test]
fn a_download_outlives_the_tab_that_asked_for_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        shell.open_tab(window, "example.org").await;
        ask(&mut shell, web_view, 7, "kept.pdf").await;

        shell.close_tab(tab).await;
        shell.report(FakeEngine::download_progress(7, 900, 1_000));
        shell.settle().await;
        assert_eq!(list(&mut shell, window).await[0].received_bytes, 900);
        assert_eq!(coming(&shell, subscription), 1);
        shell.report(FakeEngine::download_finished(7, "kept.pdf"));
        shell.settle().await;

        assert_eq!(
            standing(&mut shell, window).await,
            [("kept.pdf".to_owned(), "finished".to_owned())]
        );
        assert_eq!(
            shell.engine.download_cancels,
            [],
            "closing its tab stops nothing"
        );
    });
}

/// The person presses a download in the list to open its file (030).
async fn open(shell: &mut FakeShell, window: u64, download: u64) -> response::Kind {
    shell
        .request(request::Kind::OpenDownload(OpenDownload {
            window_id: window,
            download_id: download,
        }))
        .await
}

/// The files the shell's System service was asked to open, as the contract
/// carried them: the name and the kind, and nothing else.
fn asked_to_open(shell: &FakeShell) -> Vec<(String, String)> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::OpenFile(open) => {
                Some((open.file_name.clone(), open.mime_type.clone()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_download_that_has_finished_is_opened_by_the_name_it_was_saved_under() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "report.pdf").await;
        // The name was taken: the shell says what it used.
        shell.report(FakeEngine::download_finished(7, "report (2).pdf"));
        shell.settle().await;
        let download = list(&mut shell, window).await[0].id;

        let answer = open(&mut shell, window, download).await;

        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        // The file as the shell saved it, and the kind the site said: not
        // the name the site suggested, and nothing of where it came from.
        let asked = [("report (2).pdf".to_owned(), "application/pdf".to_owned())];
        assert_eq!(asked_to_open(&shell), asked);
        assert_eq!(shell.engine.opened_files, asked);

        // Opened again when it is pressed again: once for each press.
        open(&mut shell, window, download).await;
        assert_eq!(shell.engine.opened_files.len(), 2);
    });
}

#[test]
fn a_download_with_no_whole_file_and_one_there_is_not_are_not_opened() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "report.pdf").await;
        let coming = list(&mut shell, window).await[0].id;

        // Still coming: there is no whole file.
        assert_eq!(
            error_code(open(&mut shell, window, coming).await),
            "download.not_finished"
        );
        // Failed: there is none either.
        shell.report(FakeEngine::download_failed(7, "download.network"));
        shell.settle().await;
        assert_eq!(
            error_code(open(&mut shell, window, coming).await),
            "download.not_finished"
        );
        // One the core does not have, and an id no download could have.
        for nobody in [coming + 999, 0, u64::MAX] {
            assert_eq!(
                error_code(open(&mut shell, window, nobody).await),
                "download.unknown",
                "{nobody}"
            );
        }
        // A window there is not.
        assert_eq!(
            error_code(open(&mut shell, window + 999, coming).await),
            "window.unknown"
        );

        // The shell was asked to open nothing.
        assert_eq!(asked_to_open(&shell), []);
    });
}

#[test]
fn what_the_shell_says_of_opening_a_file_is_the_answer() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "report.pdf").await;
        shell.report(FakeEngine::download_finished(7, "report.pdf"));
        shell.settle().await;
        let download = list(&mut shell, window).await[0].id;

        // Nothing on the device opens that kind of file.
        shell.engine.nothing_opens_files = true;
        assert_eq!(
            error_code(open(&mut shell, window, download).await),
            "system.no_handler"
        );
        // The file is not where it was saved any more.
        shell.engine.nothing_opens_files = false;
        shell.engine.missing_files.push("report.pdf".to_owned());
        assert_eq!(
            error_code(open(&mut shell, window, download).await),
            "download.missing"
        );
        // It was asked for each time, and nothing was opened.
        assert_eq!(asked_to_open(&shell).len(), 2);
        assert!(shell.engine.opened_files.is_empty());
        // The list is as it was: a file that is gone is still a download
        // that finished.
        assert_eq!(
            standing(&mut shell, window).await,
            [("report.pdf".to_owned(), "finished".to_owned())]
        );
    });
}

#[test]
fn a_download_is_opened_only_from_the_list_it_is_on() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 7, "report.pdf").await;
        shell.report(FakeEngine::download_finished(7, "report.pdf"));
        shell.settle().await;
        let download = list(&mut shell, window).await[0].id;
        let home = shell.window(subscription).space_id;

        // A Space with logins of its own has a list of its own: the first
        // Space's download is not on it, and is not opened from it.
        let made = shell
            .request(request::Kind::CreateSpace(CreateSpace {
                name: "Work".to_owned(),
                share_logins_with: 0,
                colour: String::new(),
                icon: String::new(),
            }))
            .await;
        let response::Kind::SpaceCreated(work) = made else {
            panic!("expected a Space, got {made:?}");
        };
        let switch = |space: u64| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: space,
            })
        };
        shell.request(switch(work.space_id)).await;
        assert!(list(&mut shell, window).await.is_empty());
        assert_eq!(
            error_code(open(&mut shell, window, download).await),
            "download.unknown"
        );
        assert_eq!(asked_to_open(&shell), []);

        // Back in the Space it was downloaded in, it opens.
        shell.request(switch(home)).await;
        let answer = open(&mut shell, window, download).await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        assert_eq!(asked_to_open(&shell).len(), 1);
    });
}

/// Starts the app in `folder`, with the web it browses.
fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    shell.engine.page("https://example.com/", "Example");
    shell
}

#[test]
fn a_download_that_was_coming_when_the_app_ended_is_listed_as_interrupted() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        allow_downloads(&mut shell, window, tab, subscription).await;
        ask(&mut shell, web_view, 7, "done.pdf").await;
        shell.report(FakeEngine::download_finished(7, "done.pdf"));
        shell.settle().await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        ask(&mut shell, web_view, 8, "half.pdf").await;
        assert_eq!(
            standing(&mut shell, window).await,
            [
                ("half.pdf".to_owned(), "coming".to_owned()),
                ("done.pdf".to_owned(), "finished".to_owned()),
            ]
        );
        // Ended without a word.
        drop(shell);

        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;

        assert_eq!(
            standing(&mut shell, window).await,
            [
                ("half.pdf".to_owned(), "download.interrupted".to_owned()),
                ("done.pdf".to_owned(), "finished".to_owned()),
            ],
            "what was downloaded is still listed, and what was coming is not coming"
        );
        assert_eq!(coming(&shell, subscription), 0);
        assert_eq!(shell.engine.download_starts, [], "nothing is started again");
    });
}
