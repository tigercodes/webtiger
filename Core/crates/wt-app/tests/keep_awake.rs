//! What keeps a tab awake (052), through the contract, against the scripted
//! engine: the reasons a tab says, from what its engine's adapter reports of
//! the page and from what the core knows itself; when each begins and ends;
//! and who is told.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_app::{DOWNLOAD_INTERVAL, PROMPT_DELAY};
use wt_proto::v1::{
    AnswerPrompt, CancelDownload, KeepAwake, PermissionKind, SubmitAddress, TabView, request,
    response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const HOME: &str = "https://example.com/";
const NEWS: &str = "https://news.example.com/";
const MAIL: &str = "https://mail.example.com/";
const REPORT: &str = "https://example.com/files/report.pdf";
/// Another page of the first site, which may download as the first may.
const ELSEWHERE: &str = "https://example.com/elsewhere";

use KeepAwake::{Capture, Download, Loading, Question, Sound, Typed};

fn web(shell: &mut FakeShell) {
    shell.engine.page(HOME, "Home");
    shell.engine.page(NEWS, "News");
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(ELSEWHERE, "Elsewhere");
}

fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

fn reasons(numbers: &[i32]) -> Vec<KeepAwake> {
    numbers
        .iter()
        .map(|number| KeepAwake::try_from(*number).expect("a kind the contract has"))
        .collect()
}

/// Why the tab in front is kept awake, as its view says.
fn awake(shell: &FakeShell, subscription: u64) -> Vec<KeepAwake> {
    reasons(&front(shell, subscription).keeps_awake)
}

/// Why each tab of the row is kept awake, as the row says, in the row's order.
fn row(shell: &FakeShell, subscription: u64) -> Vec<Vec<KeepAwake>> {
    shell
        .window(subscription)
        .tabs
        .iter()
        .map(|tab| reasons(&tab.keeps_awake))
        .collect()
}

/// The adapter says what the page of a web view is doing, all of it.
async fn says(shell: &mut FakeShell, web_view: u64, sound: bool, capture: bool, typed: bool) {
    shell.report(FakeEngine::page_activity(web_view, sound, capture, typed));
    shell.settle().await;
}

async fn answer(shell: &mut FakeShell, tab: u64, prompt: u64, allow: bool) {
    let answered = shell
        .request(request::Kind::AnswerPrompt(AnswerPrompt {
            tab_id: tab,
            prompt_id: prompt,
            allow,
            ..Default::default()
        }))
        .await;
    assert!(matches!(answered, response::Kind::Ack(_)), "{answered:?}");
}

#[test]
fn a_tab_says_what_its_page_is_doing_and_nothing_when_nothing_would_keep_it() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        // An empty tab, and a page that is doing nothing: no reason.
        assert_eq!(awake(&shell, subscription), []);
        shell.submit(tab, HOME).await;
        let view = front(&shell, subscription).web_view_id;
        assert_eq!(awake(&shell, subscription), []);
        assert_eq!(row(&shell, subscription), [vec![]]);

        // The person types into it: the tab says so, in its view and in the row.
        says(&mut shell, view, false, false, true).await;
        assert_eq!(awake(&shell, subscription), [Typed]);
        assert_eq!(row(&shell, subscription), [vec![Typed]]);
        // It plays a sound too: the reasons are in the order of the kinds, whatever
        // order they began in.
        says(&mut shell, view, true, false, true).await;
        assert_eq!(awake(&shell, subscription), [Sound, Typed]);
        // And holds the camera. What the adapter says is all of it, as it now is.
        says(&mut shell, view, true, true, true).await;
        assert_eq!(awake(&shell, subscription), [Sound, Capture, Typed]);
        // The sound stops, and the camera is let go: what was typed is typed still.
        says(&mut shell, view, false, false, true).await;
        assert_eq!(awake(&shell, subscription), [Typed]);

        // The windows are told when a reason changes, and not when the same is said
        // again.
        let told = shell.snapshots(subscription).len();
        says(&mut shell, view, false, false, true).await;
        assert_eq!(shell.snapshots(subscription).len(), told);
        says(&mut shell, view, true, false, true).await;
        assert_eq!(shell.snapshots(subscription).len(), told + 1);
    });
}

#[test]
fn what_a_page_was_doing_goes_with_the_page_and_stays_while_it_is_the_same_page() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, HOME).await;
        let view = front(&shell, subscription).web_view_id;
        says(&mut shell, view, true, true, true).await;
        assert_eq!(awake(&shell, subscription), [Sound, Capture, Typed]);

        // The page writes its own address: it is the same page, and keeps what it had.
        shell.report(FakeEngine::address_changed(
            view,
            "https://example.com/#draft",
        ));
        shell.settle().await;
        assert_eq!(awake(&shell, subscription), [Sound, Capture, Typed]);

        // Another page commits: it is doing none of it, until the adapter says so.
        shell.submit(tab, NEWS).await;
        assert_eq!(awake(&shell, subscription), []);
        says(&mut shell, view, false, false, true).await;
        assert_eq!(awake(&shell, subscription), [Typed]);
        // A page that goes on by itself, as by a link, is another page too.
        for report in shell.engine.goes_to(view, MAIL) {
            shell.report(report);
        }
        shell.settle().await;
        assert_eq!(awake(&shell, subscription), []);

        // And a page whose renderer dies is doing nothing any more.
        says(&mut shell, view, true, false, true).await;
        shell.report(FakeEngine::renderer_gone(view));
        shell.settle().await;
        assert_eq!(awake(&shell, subscription), []);
    });
}

#[test]
fn each_tab_has_its_own_reasons_and_a_web_view_no_tab_has_keeps_nothing_awake() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let first_view = front(&shell, subscription).web_view_id;
        shell.open_tab(window, NEWS).await;
        let second_view = front(&shell, subscription).web_view_id;

        // The tab behind plays a sound; the one in front was typed into.
        says(&mut shell, first_view, true, false, false).await;
        says(&mut shell, second_view, false, false, true).await;
        assert_eq!(row(&shell, subscription), [vec![Sound], vec![Typed]]);
        assert_eq!(awake(&shell, subscription), [Typed]);

        // A web view the core has no tab for: nothing changes, and nothing is told.
        let told = shell.snapshots(subscription).len();
        says(&mut shell, second_view + 999, true, true, true).await;
        assert_eq!(shell.snapshots(subscription).len(), told);
        assert_eq!(row(&shell, subscription), [vec![Sound], vec![Typed]]);
    });
}

#[test]
fn a_tab_on_its_way_and_a_tab_with_a_question_are_kept_awake_while_that_lasts() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, HOME).await;
        let view = front(&shell, subscription).web_view_id;
        assert_eq!(awake(&shell, subscription), []);

        // Sent somewhere, with the shell slow to begin: it is on its way.
        shell.hold_host = true;
        shell.send(request::Kind::SubmitAddress(SubmitAddress {
            tab_id: tab,
            text: NEWS.to_owned(),
        }));
        shell.settle().await;
        assert_eq!(awake(&shell, subscription), [Loading]);
        shell.release_host();
        shell.settle().await;
        assert_eq!(awake(&shell, subscription), [], "it has arrived");
        // A load that failed is over: the tab is on its way nowhere.
        shell.engine.fail(MAIL, "net.offline");
        shell.submit(tab, MAIL).await;
        assert_eq!(
            front(&shell, subscription).load_state().as_str_name(),
            "LOAD_STATE_FAILED"
        );
        assert_eq!(awake(&shell, subscription), []);
        shell.submit(tab, NEWS).await;

        // Its page asks for the camera: the person is asked, and until they answer
        // the tab is kept awake by the question.
        shell.report(FakeEngine::permission_requested(
            view,
            1,
            "https://news.example.com",
            &[PermissionKind::Camera],
        ));
        shell.settle().await;
        let asked = front(&shell, subscription).prompt.expect("a prompt");
        assert_eq!(awake(&shell, subscription), [Question]);
        tokio::time::advance(PROMPT_DELAY).await;
        answer(&mut shell, tab, asked.prompt_id, false).await;
        assert_eq!(awake(&shell, subscription), []);
    });
}

/// Has the person allow the site of the tab's page to download (015): its
/// first download asks, they say yes, and that download is stopped.
async fn allow_downloads(shell: &mut FakeShell, tab: u64, subscription: u64) {
    let view = front(shell, subscription).web_view_id;
    shell.report(FakeEngine::download_requested(
        view,
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
    answer(shell, tab, asked.prompt_id, true).await;
    stop(shell, subscription).await;
    tokio::time::advance(DOWNLOAD_INTERVAL).await;
}

/// Stops every download that is coming, as the person does from the list.
async fn stop(shell: &mut FakeShell, subscription: u64) {
    let window = shell.window(subscription).window_id;
    let listed = shell
        .request(request::Kind::QueryDownloads(
            wt_proto::v1::QueryDownloads {
                window_id: window,
                ..Default::default()
            },
        ))
        .await;
    let response::Kind::DownloadsPage(page) = listed else {
        panic!("QueryDownloads answers DownloadsPage, not {listed:?}");
    };
    for entry in page.downloads {
        shell
            .request(request::Kind::CancelDownload(CancelDownload {
                download_id: entry.id,
            }))
            .await;
    }
    shell.settle().await;
}

fn ask_for_the_report(view: u64, download: u64) -> wt_proto::v1::EngineEvent {
    FakeEngine::download_requested(
        view,
        download,
        REPORT,
        "report.pdf",
        "application/pdf",
        1_000,
    )
}

#[test]
fn a_download_keeps_awake_the_tab_whose_page_started_it_until_it_ends() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let first_view = front(&shell, subscription).web_view_id;
        allow_downloads(&mut shell, first, subscription).await;
        assert_eq!(awake(&shell, subscription), [], "nothing is coming");
        shell.open_tab(window, NEWS).await;
        assert_eq!(row(&shell, subscription), [vec![], vec![]]);

        // The first tab's page starts a download: that tab is kept awake by it, and
        // the other is not.
        shell.report(ask_for_the_report(first_view, 7));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![Download], vec![]]);
        // How far it has got changes nothing.
        shell.report(FakeEngine::download_progress(7, 500, 1_000));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![Download], vec![]]);
        // The tab goes on to another page: the download is its web view's, and is
        // coming still.
        for report in shell.engine.goes_to(first_view, ELSEWHERE) {
            shell.report(report);
        }
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![Download], vec![]]);
        // It finishes: the reason is gone.
        shell.report(FakeEngine::download_finished(7, "report.pdf"));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![], vec![]]);

        // One that fails ends as well; and one the person stops.
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        shell.report(ask_for_the_report(first_view, 8));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![Download], vec![]]);
        shell.report(FakeEngine::download_failed(8, "download.network"));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![], vec![]]);
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        shell.report(ask_for_the_report(first_view, 9));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![Download], vec![]]);
        stop(&mut shell, subscription).await;
        assert_eq!(row(&shell, subscription), [vec![], vec![]]);

        // Two at once: the reason lasts as long as either does.
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        shell.report(ask_for_the_report(first_view, 10));
        shell.settle().await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        shell.report(ask_for_the_report(first_view, 11));
        shell.settle().await;
        shell.report(FakeEngine::download_finished(10, "report.pdf"));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![Download], vec![]]);
        shell.report(FakeEngine::download_finished(11, "report (2).pdf"));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [vec![], vec![]]);
    });
}

#[test]
fn a_tab_that_is_asleep_has_no_reason() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (window, first, subscription) = shell.open_window().await;
        shell.submit(first, HOME).await;
        let first_view = front(&shell, subscription).web_view_id;
        says(&mut shell, first_view, true, false, true).await;
        shell.open_tab(window, NEWS).await;
        assert_eq!(row(&shell, subscription), [vec![Sound, Typed], vec![]]);
        drop(shell);

        // The app starts again: the first tab is asleep, with no page to be doing
        // anything, and says nothing of what its page did before the quit.
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let (_, _, subscription) = shell.open_window().await;
        assert!(shell.window(subscription).tabs[0].asleep);
        assert_eq!(row(&shell, subscription), [vec![], vec![]]);
        // Woken, it is loading until it arrives, and then has no reason.
        let asleep = shell.window(subscription).tabs[0].tab_id;
        shell.activate_tab(asleep).await;
        assert_eq!(awake(&shell, subscription), []);
    });
}
