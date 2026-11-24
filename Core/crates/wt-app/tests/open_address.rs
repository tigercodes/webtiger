//! An address the system handed a shell from outside WebTiger (031): a link
//! pressed in another app. Through the contract, against the scripted
//! engine: where it opens, what may be opened, and that it is never mended,
//! searched for, or handed on to the system.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    CreateSpace, LoadState, OpenAddress, SwitchSpace, TabView, address_view::Security,
    host_request, request, response,
};
use wt_testkit::{FakeShell, run};

const NEWS: &str = "https://news.example.net/story?id=7#top";
const PLAIN: &str = "http://plain.example.net/";

/// The ids of the window's tabs, in the order they stand.
fn ids(shell: &FakeShell, subscription: u64) -> Vec<u64> {
    shell
        .window(subscription)
        .tabs
        .iter()
        .map(|tab| tab.tab_id)
        .collect()
}

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the core has asked web views to load, in order.
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

/// How many times the core has asked the shell to hand something to the
/// system: an outside link (007), or a file (030).
fn handed_to_the_system(shell: &FakeShell) -> usize {
    shell
        .host_requests
        .iter()
        .filter(|request| {
            matches!(
                request,
                host_request::Kind::OpenOutside(_) | host_request::Kind::OpenFile(_)
            )
        })
        .count()
}

fn handed(window: u64, url: &str) -> request::Kind {
    request::Kind::OpenAddress(OpenAddress {
        window_id: window,
        url: url.to_owned(),
    })
}

/// The shell hands the core an address for a window, as the system handed
/// it to the shell.
async fn open(shell: &mut FakeShell, window: u64, url: &str) -> response::Kind {
    shell.request(handed(window, url)).await
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// A window with two tabs: example.com in front, and a tab after it.
/// The window, its subscription, and the two tabs.
async fn a_window(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    shell.engine.page("https://example.com/", "Example");
    shell.engine.page("https://example.org/", "Another Place");
    shell.engine.page(NEWS, "A Story");
    shell.engine.page(PLAIN, "A Plain Page");
    let (window, first, subscription) = shell.open_window().await;
    shell.submit(first, "example.com").await;
    let second = shell.open_tab(window, "example.org").await;
    shell.activate_tab(first).await;
    (window, subscription, first, second)
}

#[test]
fn an_address_from_outside_opens_in_a_new_tab_in_front_after_the_tab_in_front() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, second) = a_window(&mut shell).await;
        let before = front(&shell, subscription);
        assert_eq!(before.tab_id, first);

        let answer = open(&mut shell, window, NEWS).await;

        // A third tab, after the one that was in front, in front, sent to
        // the address as it was handed over.
        let response::Kind::TabOpened(opened) = answer else {
            panic!("expected the tab that opened, got {answer:?}");
        };
        assert_eq!(ids(&shell, subscription), [first, opened.tab_id, second]);
        assert_eq!(loads(&shell).last().map(String::as_str), Some(NEWS));
        let new = front(&shell, subscription);
        assert_eq!(new.tab_id, opened.tab_id);
        assert_eq!(new.title, "A Story");
        let shown = new.address.clone().unwrap();
        assert_eq!(shown.url, NEWS);
        assert_eq!(shown.security(), Security::Secure);
        assert_eq!(new.login_set_id, before.login_set_id);

        // An address that is not secure is one of the web's too, and is
        // opened as it is: not made https, and marked as what it is.
        let answer = open(&mut shell, window, PLAIN).await;
        assert!(matches!(answer, response::Kind::TabOpened(_)));
        assert_eq!(loads(&shell).last().map(String::as_str), Some(PLAIN));
        let plain = front(&shell, subscription).address.unwrap();
        assert_eq!(plain.url, PLAIN);
        assert_eq!(plain.security(), Security::Insecure);
        assert_eq!(ids(&shell, subscription).len(), 4);

        // The tab that was in front is as it was, behind.
        shell.activate_tab(first).await;
        assert_eq!(front(&shell, subscription), before);
        assert_eq!(handed_to_the_system(&shell), 0);
    });
}

#[test]
fn the_field_says_where_the_tab_was_sent_and_the_mark_waits_for_the_page() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, _) = a_window(&mut shell).await;

        // The shell has been asked for the page and has not answered yet.
        shell.hold_host = true;
        shell.send(handed(window, NEWS));
        shell.settle().await;
        let waiting = front(&shell, subscription);
        shell.release_host();
        shell.settle().await;

        // In front from the start, with the address it was sent to in its
        // field, as an address typed there is; and nothing said of how
        // secure it is until its page has arrived (029).
        assert_ne!(waiting.tab_id, first);
        assert_eq!(waiting.load_state(), LoadState::Loading);
        let shown = waiting.address.clone().unwrap();
        assert_eq!(shown.url, NEWS);
        assert_eq!(shown.security(), Security::Unspecified);

        let arrived = front(&shell, subscription);
        assert_eq!(arrived.tab_id, waiting.tab_id);
        assert_eq!(arrived.load_state(), LoadState::Idle);
        assert_eq!(arrived.address.unwrap().security(), Security::Secure);
    });
}

#[test]
fn only_an_address_of_the_web_is_opened() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, second) = a_window(&mut shell).await;
        let loaded = loads(&shell).len();
        let before = front(&shell, subscription);

        // What typed in the address field would be mended into an address,
        // or searched for; what a page's link would hand to the system;
        // and what nothing should ever open.
        let not_the_webs = [
            "",
            " ",
            "example.net",
            "www.example.net/page",
            "//example.net/",
            "tigers in the wild",
            "https://",
            "javascript:alert(document.cookie)",
            "data:text/html,<script>alert(1)</script>",
            "file:///etc/passwd",
            "about:blank",
            "blob:https://example.com/0000",
            "content://media/external/file/1",
            "intent://example.net/#Intent;scheme=https;end",
            "mailto:somebody@example.net",
            "tel:+15550100",
            "sms:+15550100",
            "ftp://example.net/file",
            "view-source:https://example.net/",
            "webtiger://example.net/",
        ];
        for address in not_the_webs {
            assert_eq!(
                error_code(open(&mut shell, window, address).await),
                "address.not_web",
                "{address:?}"
            );
        }

        // Nothing opened, nothing was loaded: so nothing was searched for.
        // And nothing went on to the system, a letter or a number included.
        assert_eq!(ids(&shell, subscription), [first, second]);
        assert_eq!(front(&shell, subscription), before);
        assert_eq!(loads(&shell).len(), loaded, "nothing was loaded");
        assert_eq!(handed_to_the_system(&shell), 0);
    });
}

#[test]
fn a_window_there_is_not_is_refused_first() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, second) = a_window(&mut shell).await;
        let loaded = loads(&shell).len();

        assert_eq!(
            error_code(open(&mut shell, window + 999, NEWS).await),
            "window.unknown"
        );
        // Said before anything of the address.
        assert_eq!(
            error_code(open(&mut shell, window + 999, "javascript:alert(1)").await),
            "window.unknown"
        );

        assert_eq!(ids(&shell, subscription), [first, second]);
        assert_eq!(loads(&shell).len(), loaded, "nothing was loaded");
    });
}

#[test]
fn it_opens_in_the_space_the_window_shows_with_that_spaces_logins() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, second) = a_window(&mut shell).await;
        let home = front(&shell, subscription).login_set_id;
        let home_space = shell.window(subscription).space_id;
        // A Space with logins of its own, which the window now shows.
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
        shell
            .request(request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: work.space_id,
            }))
            .await;
        assert_eq!(shell.window(subscription).space_id, work.space_id);
        let empty = ids(&shell, subscription);
        assert_eq!(empty.len(), 1);

        let answer = open(&mut shell, window, NEWS).await;

        let response::Kind::TabOpened(opened) = answer else {
            panic!("expected the tab that opened, got {answer:?}");
        };
        assert_eq!(ids(&shell, subscription), [empty[0], opened.tab_id]);
        let new = front(&shell, subscription);
        assert_eq!(new.address.clone().unwrap().url, NEWS);
        assert_ne!(new.login_set_id, home, "with the Space's own logins");
        // The first Space's tabs are as they were, when the window shows it again.
        shell
            .request(request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: home_space,
            }))
            .await;
        assert_eq!(ids(&shell, subscription), [first, second]);
    });
}

#[test]
fn a_page_that_cannot_be_had_shows_in_the_tab_that_opened_for_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, first, second) = a_window(&mut shell).await;
        shell.engine.fail("https://gone.example.net/", "net.dns");

        let answer = open(&mut shell, window, "https://gone.example.net/").await;

        // The tab opened whatever became of its page, and says what did.
        let response::Kind::TabOpened(opened) = answer else {
            panic!("expected the tab that opened, got {answer:?}");
        };
        assert_eq!(ids(&shell, subscription), [first, opened.tab_id, second]);
        let failed = front(&shell, subscription);
        assert_eq!(failed.load_state(), LoadState::Failed);
        assert_eq!(failed.error_code, "net.dns");
        let shown = failed.address.unwrap();
        assert_eq!(shown.url, "https://gone.example.net/");
        assert_eq!(shown.security(), Security::Unspecified);
    });
}
