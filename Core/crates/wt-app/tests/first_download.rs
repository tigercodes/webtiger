//! A site's first download asks (015), through the contract, against the
//! scripted engine: when the person is asked and when not, what their answer
//! starts or drops, and what is remembered.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_app::{DOWNLOAD_INTERVAL, PROMPT_DELAY};
use wt_proto::v1::{
    AnswerPrompt, ForgetPermissions, PermissionKind, QueryDownloads, QueryPermissions, TabView,
    forget_permissions, prompt_view, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const REPORT: &str = "https://files.example.net/2026/report.pdf";

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the tab in front asks about a download: the prompt's id, the site it
/// names and the file. `None` when it asks nothing.
fn asked(shell: &FakeShell, subscription: u64) -> Option<(u64, String, String)> {
    let prompt = front(shell, subscription).prompt?;
    match prompt.kind {
        Some(prompt_view::Kind::Download(download)) => {
            Some((prompt.prompt_id, download.site, download.file_name))
        }
        other => panic!("a prompt of some other kind: {other:?}"),
    }
}

/// A web view's page asks for a download the shell names `download`.
async fn download(shell: &mut FakeShell, web_view: u64, download: u64, suggested: &str) {
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

/// Answers the prompt the tab in front shows, once it has been there long
/// enough to be answered.
async fn answer(shell: &mut FakeShell, tab: u64, subscription: u64, allow: bool) {
    let prompt = front(shell, subscription)
        .prompt
        .expect("the tab asks something")
        .prompt_id;
    tokio::time::advance(PROMPT_DELAY).await;
    let done = shell
        .request(request::Kind::AnswerPrompt(AnswerPrompt {
            tab_id: tab,
            prompt_id: prompt,
            allow,
            ..Default::default()
        }))
        .await;
    assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
}

/// The names of the window's downloads, newest first.
async fn listed(shell: &mut FakeShell, window: u64) -> Vec<String> {
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
            .map(|download| download.file_name)
            .collect(),
        other => panic!("expected the downloads, got {other:?}"),
    }
}

/// The sites of the window's login set that may download, as they are shown.
async fn may_download(shell: &mut FakeShell, window: u64) -> Vec<String> {
    let answer = shell
        .request(request::Kind::QueryPermissions(QueryPermissions {
            window_id: window,
        }))
        .await;
    let response::Kind::PermissionsPage(page) = answer else {
        panic!("expected what sites were allowed, got {answer:?}");
    };
    page.sites
        .into_iter()
        .filter(|site| site.kinds.contains(&(PermissionKind::Downloads as i32)))
        .map(|site| site.site)
        .collect()
}

/// The downloads the shell was asked to start, and those it was told to
/// drop, by its names for them.
fn started_and_dropped(shell: &FakeShell) -> (Vec<u64>, Vec<u64>) {
    let started = shell
        .engine
        .download_starts
        .iter()
        .map(|(download, _)| *download)
        .collect();
    (started, shell.engine.download_cancels.clone())
}

/// The web a shell browses in these tests.
fn pages(shell: &mut FakeShell) {
    for (url, title) in [
        ("https://example.com/", "Example"),
        ("https://example.com/more", "More of it"),
        ("https://example.org/", "Another"),
        ("http://shop.example.com/", "A shop, not over https"),
    ] {
        shell.engine.page(url, title);
    }
    // An address that is a file: the load that goes there never becomes a
    // page, and ends as one that stopped.
    shell.engine.fail(REPORT, "net.cancelled");
    shell.engine.redirect("https://example.com/latest", REPORT);
    shell.engine.fail("https://nowhere.example.com/", "net.dns");
}

/// A window whose one tab shows example.com. Its window, tab, subscription
/// and the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    pages(shell);
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

#[test]
fn a_sites_first_download_asks_and_nothing_is_started_or_listed() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;

        download(&mut shell, web_view, 7, "../../Annual report.pdf").await;

        let (id, site, file) = asked(&shell, subscription).expect("the tab asks");
        assert_ne!(id, 0);
        assert_eq!(
            site, "example.com",
            "the page that asks, not where the file is"
        );
        assert_eq!(file, "Annual report.pdf", "the name the core made of it");
        assert_eq!(started_and_dropped(&shell), (vec![], vec![]), "it waits");
        assert_eq!(listed(&mut shell, window).await, Vec::<String>::new());
        assert_eq!(shell.window(subscription).downloads_active, 0);
    });
}

#[test]
fn allow_starts_it_and_the_site_is_not_asked_about_again() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        download(&mut shell, web_view, 7, "report.pdf").await;

        answer(&mut shell, tab, subscription, true).await;

        assert_eq!(
            shell.engine.download_starts,
            [(7, "report.pdf".to_owned())],
            "under the core's name"
        );
        assert_eq!(asked(&shell, subscription), None);
        assert_eq!(listed(&mut shell, window).await, ["report.pdf"]);
        assert_eq!(shell.window(subscription).downloads_active, 1);
        assert_eq!(may_download(&mut shell, window).await, ["example.com"]);

        // The next one, and one from another page of the site: with nobody asked.
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 8, "second.pdf").await;
        shell.submit(tab, "example.com/more").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 9, "third.pdf").await;
        assert_eq!(started_and_dropped(&shell), (vec![7, 8, 9], vec![]));
        assert_eq!(asked(&shell, subscription), None);

        // Another site was not allowed by that.
        shell.submit(tab, "example.org").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 10, "theirs.pdf").await;
        assert_eq!(started_and_dropped(&shell).0, [7, 8, 9]);
        assert_eq!(
            asked(&shell, subscription).map(|asked| asked.1),
            Some("example.org".to_owned())
        );
    });
}

/// Starts the app in `folder`, with the web it browses.
fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    pages(&mut shell);
    shell
}

#[test]
fn a_site_that_was_allowed_is_still_allowed_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        download(&mut shell, web_view, 7, "report.pdf").await;
        answer(&mut shell, tab, subscription, true).await;
        shell.report(FakeEngine::download_finished(7, "report.pdf"));
        shell.settle().await;
        drop(shell);

        let mut shell = start(folder.path());
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "example.com").await;
        let web_view = front(&shell, subscription).web_view_id;

        download(&mut shell, web_view, 1, "again.pdf").await;

        assert_eq!(started_and_dropped(&shell), (vec![1], vec![]));
        assert_eq!(asked(&shell, subscription), None);
        assert_eq!(
            listed(&mut shell, window).await,
            ["again.pdf", "report.pdf"]
        );
    });
}

#[test]
fn dont_allow_drops_it_and_the_page_does_not_get_to_ask_again() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        download(&mut shell, web_view, 7, "report.pdf").await;

        answer(&mut shell, tab, subscription, false).await;

        assert_eq!(started_and_dropped(&shell), (vec![], vec![7]));
        assert_eq!(asked(&shell, subscription), None);
        assert_eq!(listed(&mut shell, window).await, Vec::<String>::new());
        assert_eq!(
            may_download(&mut shell, window).await,
            Vec::<String>::new(),
            "nothing is written down"
        );

        // The page's later downloads are dropped with nobody asked.
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 8, "again.pdf").await;
        assert_eq!(started_and_dropped(&shell), (vec![], vec![7, 8]));
        assert_eq!(asked(&shell, subscription), None);

        // Another page has loaded: it may ask, and the person is asked.
        shell.submit(tab, "example.com/more").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 9, "third.pdf").await;
        assert_eq!(started_and_dropped(&shell), (vec![], vec![7, 8]));
        assert!(asked(&shell, subscription).is_some());
    });
}

#[test]
fn an_address_the_person_sent_the_tab_to_is_downloaded_without_asking() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        // Typed, it turns out to be a file: the load ends without a page,
        // and the download is reported.
        shell.submit(tab, REPORT).await;
        download(&mut shell, web_view, 7, "report.pdf").await;

        assert_eq!(started_and_dropped(&shell), (vec![7], vec![]));
        assert_eq!(asked(&shell, subscription), None, "they asked for it");

        // That was good for one: the page that is still there starts the
        // next, and is asked about.
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 8, "the page's own.pdf").await;
        assert_eq!(started_and_dropped(&shell), (vec![7], vec![]));
        let (_, site, _) = asked(&shell, subscription).expect("the page's own is asked about");
        assert_eq!(site, "example.com");
        answer(&mut shell, tab, subscription, false).await;

        // An address that sends the browser on to a file is the same wish.
        shell.submit(tab, "example.org").await;
        shell.submit(tab, "example.com/latest").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 9, "latest.pdf").await;
        assert_eq!(started_and_dropped(&shell), (vec![7, 9], vec![8]));
        assert_eq!(asked(&shell, subscription), None);
    });
}

#[test]
fn what_the_person_asked_for_is_not_lent_to_a_page() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        // The person went to a page, and it arrived: what the page then
        // starts is the page's.
        shell.submit(tab, "example.com/more").await;
        download(&mut shell, web_view, 7, "a page's.pdf").await;
        assert_eq!(started_and_dropped(&shell), (vec![], vec![]));
        assert!(asked(&shell, subscription).is_some());
        answer(&mut shell, tab, subscription, false).await;

        // The person went somewhere that failed: the page that is still
        // there cannot use that either.
        shell.submit(tab, "example.org").await;
        shell.submit(tab, "nowhere.example.com").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 8, "theirs.pdf").await;
        assert_eq!(started_and_dropped(&shell).0, Vec::<u64>::new());
        assert_eq!(
            asked(&shell, subscription).map(|asked| asked.1),
            Some("example.org".to_owned())
        );
    });
}

#[test]
fn a_tab_with_no_page_asks_in_the_name_of_where_the_file_comes_from() {
    run(async {
        let mut shell = FakeShell::fake();
        pages(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        // A web view, and no page: where the tab was sent failed.
        shell.submit(tab, "nowhere.example.com").await;
        let web_view = front(&shell, subscription).web_view_id;

        download(&mut shell, web_view, 7, "report.pdf").await;

        let (_, site, file) = asked(&shell, subscription).expect("it is asked about");
        assert_eq!(site, "files.example.net");
        assert_eq!(file, "report.pdf");
    });
}

#[test]
fn a_page_not_over_https_may_ask_to_download_and_is_named_with_its_scheme() {
    run(async {
        let mut shell = FakeShell::fake();
        pages(&mut shell);
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "http://shop.example.com/").await;
        let web_view = front(&shell, subscription).web_view_id;

        download(&mut shell, web_view, 7, "price list.pdf").await;

        let (_, site, _) = asked(&shell, subscription).expect("it is asked about");
        assert_eq!(site, "http://shop.example.com");
        answer(&mut shell, tab, subscription, true).await;
        assert_eq!(started_and_dropped(&shell), (vec![7], vec![]));
        assert_eq!(
            may_download(&mut shell, window).await,
            ["http://shop.example.com"]
        );
    });
}

#[test]
fn what_waited_is_dropped_with_its_page() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;

        // Another page loads in the tab.
        download(&mut shell, web_view, 7, "one.pdf").await;
        shell.submit(tab, "example.org").await;
        assert_eq!(started_and_dropped(&shell), (vec![], vec![7]));
        assert_eq!(asked(&shell, subscription), None);

        // The page's renderer is lost.
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 8, "two.pdf").await;
        assert!(asked(&shell, subscription).is_some());
        shell.report(FakeEngine::renderer_gone(web_view));
        shell.settle().await;
        assert_eq!(started_and_dropped(&shell), (vec![], vec![7, 8]));

        // The tab closes. (Another is opened first, so that the window stays.)
        shell.submit(tab, "example.com").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 9, "three.pdf").await;
        assert!(asked(&shell, subscription).is_some());
        shell.open_tab(window, "example.org").await;
        shell.close_tab(tab).await;
        assert_eq!(started_and_dropped(&shell), (vec![], vec![7, 8, 9]));

        assert_eq!(listed(&mut shell, window).await, Vec::<String>::new());
        assert_eq!(may_download(&mut shell, window).await, Vec::<String>::new());
    });
}

#[test]
fn what_else_waited_for_the_same_answer_has_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_view) = a_page(&mut shell).await;
        download(&mut shell, first_view, 1, "one.pdf").await;
        let second = shell.open_tab(window, "example.com/more").await;
        let second_view = front(&shell, subscription).web_view_id;
        download(&mut shell, second_view, 2, "two.pdf").await;

        // A yes for the site: its download that waited in the tab behind
        // starts too.
        answer(&mut shell, second, subscription, true).await;
        assert_eq!(started_and_dropped(&shell), (vec![2, 1], vec![]));
        shell.activate_tab(first).await;
        assert_eq!(asked(&shell, subscription), None);

        // A no: whatever else that page wants to download is dropped with it.
        shell.submit(first, "example.org").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, first_view, 3, "three.pdf").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, first_view, 4, "four.pdf").await;
        assert_eq!(
            started_and_dropped(&shell),
            (vec![2, 1], vec![]),
            "both wait"
        );
        answer(&mut shell, first, subscription, false).await;
        assert_eq!(started_and_dropped(&shell), (vec![2, 1], vec![3, 4]));
        assert_eq!(asked(&shell, subscription), None);
    });
}

#[test]
fn the_limits_of_downloads_and_of_prompts_still_hold() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        // One a second from a web view (013), also while the first waits.
        download(&mut shell, web_view, 1, "one.pdf").await;
        download(&mut shell, web_view, 2, "too soon.pdf").await;
        assert_eq!(started_and_dropped(&shell), (vec![], vec![2]));

        // Four wait in a tab (014); a fifth is dropped.
        for another in 3..=6 {
            tokio::time::advance(DOWNLOAD_INTERVAL).await;
            download(&mut shell, web_view, another, "another.pdf").await;
        }
        assert_eq!(started_and_dropped(&shell), (vec![], vec![2, 6]));

        // Said twice, it is asked about once: one answer answers them all,
        // and each starts once.
        shell.report(FakeEngine::download_requested(
            web_view,
            1,
            REPORT,
            "one.pdf",
            "application/pdf",
            1_000,
        ));
        shell.settle().await;
        answer(&mut shell, tab, subscription, true).await;
        assert_eq!(started_and_dropped(&shell), (vec![1, 3, 4, 5], vec![2, 6]));
        assert_eq!(asked(&shell, subscription), None);
    });
}

#[test]
fn a_download_and_a_request_the_shell_gave_the_same_number_are_not_taken_for_each_other() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        // A shell counts its downloads and its pages' requests each from one.
        shell.report(FakeEngine::permission_requested(
            web_view,
            7,
            "https://example.com",
            &[PermissionKind::Camera],
        ));
        shell.settle().await;
        download(&mut shell, web_view, 7, "report.pdf").await;

        // The camera is refused: the download that waits behind it is not.
        answer(&mut shell, tab, subscription, false).await;
        assert_eq!(shell.engine.permission_answers, [(7, false)]);
        assert_eq!(started_and_dropped(&shell), (vec![], vec![]));
        assert!(
            asked(&shell, subscription).is_some(),
            "the download is asked about next"
        );

        // The download is allowed: the page is told nothing more about its camera.
        answer(&mut shell, tab, subscription, true).await;
        assert_eq!(started_and_dropped(&shell), (vec![7], vec![]));
        assert_eq!(shell.engine.permission_answers, [(7, false)]);

        // The other way round, on another site: a download waits. A word
        // that the page withdraws its request by that number withdraws
        // nothing of the download; and a request by that number is a request
        // of its own, not the download said twice.
        shell.submit(tab, "example.org").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 9, "another.pdf").await;
        shell.report(FakeEngine::permission_withdrawn(web_view, 9));
        shell.settle().await;
        assert!(
            asked(&shell, subscription).is_some(),
            "the download still waits"
        );
        shell.report(FakeEngine::permission_requested(
            web_view,
            9,
            "https://example.org",
            &[PermissionKind::Microphone],
        ));
        shell.settle().await;
        answer(&mut shell, tab, subscription, true).await;
        assert_eq!(started_and_dropped(&shell), (vec![7, 9], vec![]));
        // The request waited behind it, and is the tab's prompt now.
        answer(&mut shell, tab, subscription, false).await;
        assert_eq!(shell.engine.permission_answers, [(7, false), (9, false)]);
    });
}

#[test]
fn to_download_is_not_an_engines_to_ask_for() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, subscription, web_view) = a_page(&mut shell).await;

        shell.report(FakeEngine::permission_requested(
            web_view,
            1,
            "https://example.com",
            &[PermissionKind::Downloads],
        ));
        shell.report(FakeEngine::permission_requested(
            web_view,
            2,
            "https://example.com",
            &[PermissionKind::Camera, PermissionKind::Downloads],
        ));
        shell.settle().await;

        assert_eq!(shell.engine.permission_answers, [(1, false), (2, false)]);
        assert_eq!(front(&shell, subscription).prompt, None, "nobody is asked");
        assert_eq!(may_download(&mut shell, window).await, Vec::<String>::new());
    });
}

#[test]
fn taken_back_the_site_asks_again() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        download(&mut shell, web_view, 7, "report.pdf").await;
        answer(&mut shell, tab, subscription, true).await;
        assert_eq!(may_download(&mut shell, window).await, ["example.com"]);

        let taken = shell
            .request(request::Kind::ForgetPermissions(ForgetPermissions {
                window_id: window,
                what: Some(forget_permissions::What::Origin(
                    "https://example.com".to_owned(),
                )),
            }))
            .await;
        assert!(matches!(taken, response::Kind::Ack(_)), "{taken:?}");

        // What is coming goes on coming: taking back stops nothing.
        assert_eq!(shell.engine.download_cancels, Vec::<u64>::new());
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        download(&mut shell, web_view, 8, "again.pdf").await;
        assert_eq!(started_and_dropped(&shell), (vec![7], vec![]));
        assert!(
            asked(&shell, subscription).is_some(),
            "the person is asked again"
        );
    });
}
