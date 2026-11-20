//! The person asks for a link in a new tab (018), through the contract,
//! against the scripted engine: when a tab opens, where, and what it is
//! allowed.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{LoadState, TabSummary, TabView, host_request, prompt_view};
use wt_testkit::{FakeEngine, FakeShell, run};

/// Where the link in these tests leads.
const LINK: &str = "https://example.org/";
/// A link that turns out to be a file.
const FILE: &str = "https://files.example.net/2026/report.pdf";

/// The window's tabs, in the order they stand.
fn tabs(shell: &FakeShell, subscription: u64) -> Vec<TabSummary> {
    shell.window(subscription).tabs
}

fn ids(shell: &FakeShell, subscription: u64) -> Vec<u64> {
    tabs(shell, subscription)
        .iter()
        .map(|tab| tab.tab_id)
        .collect()
}

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// The web views the core has asked for, in order.
fn created(shell: &FakeShell) -> Vec<u64> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewCreate(create) => Some(create.web_view_id),
            _ => None,
        })
        .collect()
}

/// What the core has asked web views to load, in order.
fn loads(shell: &FakeShell) -> Vec<(u64, String)> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewLoad(load) => Some((load.web_view_id, load.url.clone())),
            _ => None,
        })
        .collect()
}

/// What was handed to the system, in order.
fn opened_outside(shell: &FakeShell) -> usize {
    shell
        .host_requests
        .iter()
        .filter(|request| matches!(request, host_request::Kind::OpenOutside(_)))
        .count()
}

/// The person asks for a link of a web view's page in a new tab, as the
/// shell would report it.
async fn open(shell: &mut FakeShell, web_view: u64, url: &str, background: bool) {
    shell.report(FakeEngine::link_open_requested(
        web_view, url, background, true,
    ));
    shell.settle().await;
}

/// A window whose one tab shows example.com. Its window, tab, subscription
/// and the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    for (url, title) in [
        ("https://example.com/", "Example"),
        ("https://example.com/more", "More of it"),
        ("https://example.com/a", "A"),
        ("https://example.com/b", "B"),
        ("https://example.com/c", "C"),
        ("https://example.org/", "Another"),
    ] {
        shell.engine.page(url, title);
    }
    shell.engine.fail("https://nowhere.example.com/", "net.dns");
    shell.engine.fail(FILE, "net.cancelled");
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

#[test]
fn a_link_opens_in_a_tab_behind_the_one_that_has_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        let before = front(&shell, subscription);

        open(&mut shell, web_view, LINK, true).await;

        // A second tab, after the first, with a web view of the core's own
        // naming that was sent to the link.
        let now = tabs(&shell, subscription);
        assert_eq!(now.len(), 2);
        assert_eq!(now[0].tab_id, tab);
        let made = created(&shell);
        assert_eq!(made.len(), 2, "one for the page, one for the link");
        assert_eq!(loads(&shell).last(), Some(&(made[1], LINK.to_owned())));
        // Its page loaded there, behind.
        assert_eq!(
            (now[1].title.as_str(), now[1].host.as_str()),
            ("Another", "example.org")
        );
        // The page that has the link is as it was, and still in front.
        assert_eq!(front(&shell, subscription), before);

        // Brought forward, it is the link's page.
        shell.activate_tab(now[1].tab_id).await;
        let new = front(&shell, subscription);
        assert_eq!(new.web_view_id, made[1]);
        assert_eq!(new.title, "Another");
        assert_eq!(new.address.unwrap().url, LINK);
        assert_eq!(
            new.login_set_id, before.login_set_id,
            "in the same login set"
        );
    });
}

#[test]
fn it_opens_in_front_when_the_shell_says_so() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        open(&mut shell, web_view, LINK, false).await;

        let now = ids(&shell, subscription);
        assert_eq!(now.len(), 2);
        assert_eq!(now[0], tab);
        let new = front(&shell, subscription);
        assert_eq!(new.tab_id, now[1]);
        assert_eq!(new.title, "Another");
    });
}

#[test]
fn links_opened_one_after_another_stand_in_that_order() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        // Another tab stands after the first already.
        let other = shell.open_tab(window, "example.com/more").await;
        shell.activate_tab(tab).await;

        for link in [
            "https://example.com/a",
            "https://example.com/b",
            "https://example.com/c",
        ] {
            open(&mut shell, web_view, link, true).await;
        }

        let titles: Vec<String> = tabs(&shell, subscription)
            .into_iter()
            .map(|tab| tab.title)
            .collect();
        assert_eq!(titles, ["Example", "A", "B", "C", "More of it"]);
        assert_eq!(ids(&shell, subscription).last(), Some(&other));

        // A link of one of them opens after that one, not at the end of the
        // first tab's.
        let a = ids(&shell, subscription)[1];
        shell.activate_tab(a).await;
        let a_view = front(&shell, subscription).web_view_id;
        open(&mut shell, a_view, LINK, true).await;
        let titles: Vec<String> = tabs(&shell, subscription)
            .into_iter()
            .map(|tab| tab.title)
            .collect();
        assert_eq!(titles, ["Example", "A", "Another", "B", "C", "More of it"]);
    });
}

#[test]
fn a_page_that_only_makes_it_look_so_opens_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        shell.report(FakeEngine::link_open_requested(web_view, LINK, true, false));
        shell.report(FakeEngine::link_open_requested(
            web_view, LINK, false, false,
        ));
        shell.settle().await;

        assert_eq!(ids(&shell, subscription), [tab]);
        assert_eq!(created(&shell).len(), 1, "no web view was made for it");
    });
}

#[test]
fn only_an_address_a_tab_may_be_sent_to_opens_one() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        for link in [
            "javascript:alert(1)",
            "data:text/html,<h1>a page of no address</h1>",
            "file:///etc/passwd",
            "about:blank",
            "blob:https://example.com/5a1d7e8f",
            // For the system: opened by pressing the link itself (007).
            "mailto:ada@example.com",
            "ftp://example.com/file",
            "",
            "not an address",
        ] {
            open(&mut shell, web_view, link, true).await;
            assert_eq!(ids(&shell, subscription), [tab], "{link}");
        }

        assert_eq!(created(&shell).len(), 1);
        assert_eq!(opened_outside(&shell), 0, "nor handed to the system");
        // A page not over https is a page.
        shell.engine.page("http://shop.example.com/", "A shop");
        open(&mut shell, web_view, "http://shop.example.com/", true).await;
        assert_eq!(ids(&shell, subscription).len(), 2);
    });
}

#[test]
fn a_web_view_that_is_no_tabs_opens_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, _) = a_page(&mut shell).await;

        open(&mut shell, 99, LINK, true).await;

        assert_eq!(ids(&shell, subscription), [tab]);
        assert_eq!(created(&shell).len(), 1);
    });
}

#[test]
fn its_field_is_empty_until_its_page_commits() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;

        // The shell has not yet done what it was asked: the tab is there,
        // and nothing has come of its load.
        shell.hold_host = true;
        open(&mut shell, web_view, LINK, false).await;

        let new = front(&shell, subscription);
        assert_ne!(new.web_view_id, 0, "its web view is named");
        assert_eq!(new.load_state(), LoadState::Loading);
        let field = new.address.unwrap_or_default();
        // Where a link leads is the page's claim, however it was opened.
        assert_eq!((field.url.as_str(), field.display.as_str()), ("", ""));
        assert_eq!(new.title, "");

        shell.release_host();
        shell.settle().await;

        let new = front(&shell, subscription);
        assert_eq!(new.address.unwrap().url, LINK);
        assert_eq!(new.title, "Another");
    });
}

#[test]
fn a_first_download_there_asks() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;

        // The link turns out to be a file: no page comes of the load, and
        // the download is reported for the new tab's web view.
        open(&mut shell, web_view, FILE, false).await;
        let new = front(&shell, subscription).web_view_id;
        shell.report(FakeEngine::download_requested(
            new,
            7,
            FILE,
            "report.pdf",
            "application/pdf",
            1_000,
        ));
        shell.settle().await;

        // The person asked for a link, not for a file: it is asked about,
        // in the name of where the file comes from.
        assert_eq!(shell.engine.download_starts, []);
        let prompt = front(&shell, subscription).prompt.expect("it asks");
        match prompt.kind {
            Some(prompt_view::Kind::Download(download)) => {
                assert_eq!(download.site, "files.example.net");
                assert_eq!(download.file_name, "report.pdf");
            }
            other => panic!("a prompt of some other kind: {other:?}"),
        }
    });
}

#[test]
fn a_page_cannot_close_it_and_closing_it_brings_back_the_tab_it_came_from() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        let other = shell.open_tab(window, "example.com/more").await;
        shell.activate_tab(tab).await;
        open(&mut shell, web_view, LINK, false).await;
        let new = front(&shell, subscription);
        assert_eq!(ids(&shell, subscription), [tab, new.tab_id, other]);

        // No page opened it: none closes it.
        shell.report(FakeEngine::close_requested(new.web_view_id));
        shell.settle().await;
        assert_eq!(ids(&shell, subscription).len(), 3);
        assert_eq!(front(&shell, subscription).tab_id, new.tab_id);

        // The person closes it: back to the tab it was opened from, not on
        // to the one after.
        shell.close_tab(new.tab_id).await;
        assert_eq!(ids(&shell, subscription), [tab, other]);
        assert_eq!(front(&shell, subscription).tab_id, tab);
    });
}

#[test]
fn a_load_that_fails_shows_in_the_new_tab() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        open(&mut shell, web_view, "https://nowhere.example.com/", false).await;

        let new = front(&shell, subscription);
        assert_ne!(new.tab_id, tab, "the tab is there all the same");
        assert_eq!(new.load_state(), LoadState::Failed);
        assert_eq!(new.error_code, "net.dns");
    });
}
