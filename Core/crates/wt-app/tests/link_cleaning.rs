//! Link cleaning (068), through the contract, against the scripted engine:
//! what is there to follow a click from one site to another is taken out of
//! an address the core loads by itself, when it comes from another app or
//! from a page of another site; never within one site, never what a person
//! typed, and not where Guard is off.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    GuardSwitch, OpenAddress, QueryHistory, SetGuard, host_request, request, response, set_guard,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const NEWS: &str = "https://news.example.com/story";
const SHOP_CLEAN: &str = "https://shop.example.org/item?id=42&colour=sea%20green";
const SHOP_FOLLOWED: &str =
    "https://shop.example.org/item?utm_source=news&id=42&fbclid=AbC123&colour=sea%20green&gclid=9";
const NEWS_OWN_FOLLOWED: &str = "https://www.example.com/other?utm_source=front&id=7";

fn web(shell: &mut FakeShell) {
    for (url, title) in [
        (NEWS, "News"),
        (SHOP_CLEAN, "Shop"),
        (SHOP_FOLLOWED, "Shop, followed"),
        (NEWS_OWN_FOLLOWED, "More news"),
    ] {
        shell.engine.page(url, title);
    }
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

async fn from_outside(shell: &mut FakeShell, window: u64, url: &str) {
    let answer = shell
        .request(request::Kind::OpenAddress(OpenAddress {
            window_id: window,
            url: url.to_owned(),
        }))
        .await;
    assert!(matches!(answer, response::Kind::TabOpened(_)), "{answer:?}");
}

async fn set_guard(shell: &mut FakeShell, of: set_guard::Of, switch: GuardSwitch) {
    let answer = shell
        .request(request::Kind::SetGuard(SetGuard {
            of: Some(of),
            switch: switch.into(),
        }))
        .await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

/// Opens a link of the page in front in a new tab in front, as a person does.
async fn open_link(shell: &mut FakeShell, subscription: u64, url: &str) {
    let web_view = shell.window(subscription).tab.unwrap().web_view_id;
    shell.report(FakeEngine::link_open_requested(web_view, url, false, true));
    shell.settle().await;
}

#[test]
fn an_address_from_another_app_is_loaded_shown_and_kept_clean() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, NEWS).await;

        from_outside(&mut shell, window, SHOP_FOLLOWED).await;
        // What was loaded, what the address field shows, and what history has, are the
        // address without what followed the click; the rest of it is as it was written.
        assert_eq!(loads(&shell), [NEWS, SHOP_CLEAN]);
        let front = shell.window(subscription).tab.unwrap();
        assert_eq!(front.address.unwrap().url, SHOP_CLEAN);
        assert_eq!(front.title, "Shop");
        let answer = shell
            .request(request::Kind::QueryHistory(QueryHistory {
                window_id: window,
                limit: 10,
                ..QueryHistory::default()
            }))
            .await;
        let response::Kind::HistoryPage(history) = answer else {
            panic!("history, not {answer:?}");
        };
        let visited: Vec<&str> = history
            .entries
            .iter()
            .map(|entry| entry.url.as_str())
            .collect();
        assert!(visited.contains(&SHOP_CLEAN), "{visited:?}");
        assert!(
            !visited
                .iter()
                .any(|url| url.contains("utm_") || url.contains("fbclid")),
            "{visited:?}"
        );

        // An address with nothing of the kind is loaded as it was given, to the letter.
        from_outside(&mut shell, window, SHOP_CLEAN).await;
        assert_eq!(loads(&shell).last().unwrap(), SHOP_CLEAN);
    });
}

#[test]
fn a_link_opened_in_a_tab_is_cleaned_when_it_leads_to_another_site_and_not_within_one() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, NEWS).await;

        // To another site: cleaned.
        open_link(&mut shell, subscription, SHOP_FOLLOWED).await;
        assert_eq!(loads(&shell), [NEWS, SHOP_CLEAN]);
        // Within the site, by another of its hosts: as the site wrote it.
        let answer = shell.activate_tab(tab).await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        open_link(&mut shell, subscription, NEWS_OWN_FOLLOWED).await;
        assert_eq!(loads(&shell), [NEWS, SHOP_CLEAN, NEWS_OWN_FOLLOWED]);
        let _ = window;
    });
}

#[test]
fn what_a_person_typed_is_never_cleaned() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, NEWS).await;
        // Typed, or pasted, into the address field: theirs, as they gave it.
        shell.submit(tab, SHOP_FOLLOWED).await;
        assert_eq!(loads(&shell), [NEWS, SHOP_FOLLOWED]);
        assert_eq!(
            shell.window(subscription).tab.unwrap().address.unwrap().url,
            SHOP_FOLLOWED
        );
    });
}

#[test]
fn nothing_is_cleaned_where_guard_is_off() {
    use GuardSwitch::{Off, On, Unspecified};
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, NEWS).await;
        let home = shell.window(subscription).space_id;

        // Off everywhere: neither from outside nor from a page.
        set_guard(&mut shell, set_guard::Of::Everywhere(true), Off).await;
        from_outside(&mut shell, window, SHOP_FOLLOWED).await;
        assert_eq!(loads(&shell).last().unwrap(), SHOP_FOLLOWED);
        shell.activate_tab(tab).await;
        open_link(&mut shell, subscription, SHOP_FOLLOWED).await;
        assert_eq!(loads(&shell).last().unwrap(), SHOP_FOLLOWED);
        set_guard(&mut shell, set_guard::Of::Everywhere(true), On).await;

        // Off for the Space the window shows: the same.
        set_guard(&mut shell, set_guard::Of::SpaceId(home), Off).await;
        from_outside(&mut shell, window, SHOP_FOLLOWED).await;
        assert_eq!(loads(&shell).last().unwrap(), SHOP_FOLLOWED);
        set_guard(&mut shell, set_guard::Of::SpaceId(home), Unspecified).await;

        // Off for the site of the page that has the link: its links are not cleaned; an
        // address from outside is of no page, and is.
        shell.activate_tab(tab).await;
        set_guard(&mut shell, set_guard::Of::TabId(tab), Off).await;
        open_link(&mut shell, subscription, SHOP_FOLLOWED).await;
        assert_eq!(loads(&shell).last().unwrap(), SHOP_FOLLOWED);
        shell.activate_tab(tab).await;
        from_outside(&mut shell, window, SHOP_FOLLOWED).await;
        assert_eq!(loads(&shell).last().unwrap(), SHOP_CLEAN);
        // And on again for the site: its links are cleaned.
        shell.activate_tab(tab).await;
        set_guard(&mut shell, set_guard::Of::TabId(tab), Unspecified).await;
        open_link(&mut shell, subscription, SHOP_FOLLOWED).await;
        assert_eq!(loads(&shell).last().unwrap(), SHOP_CLEAN);
    });
}
