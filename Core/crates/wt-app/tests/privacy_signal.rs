//! Global Privacy Control (069), in the core: every web view the core asks a
//! shell for is to be told the signal. Through the contract, against the
//! scripted engine.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{SleepOthers, host_request, request, response};
use wt_testkit::{FakeEngine, FakeShell, run};

const NEWS: &str = "https://news.example.com/story";
const SHOP: &str = "https://shop.example.org/";

/// Each web view the shell was asked to make, with whether its pages are to
/// be told the signal.
fn asked(shell: &FakeShell) -> Vec<(u64, bool)> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewCreate(create) => {
                Some((create.web_view_id, create.global_privacy_control))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn every_web_view_the_core_asks_for_is_told_the_signal() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page(NEWS, "News");
        shell.engine.page(SHOP, "Shop");
        // An ordinary window's tab; a second tab; a link opened in a tab of its own; a
        // private window's tab.
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, NEWS).await;
        shell.open_tab(window, SHOP).await;
        let web_view = shell.window(subscription).tab.unwrap().web_view_id;
        shell.report(FakeEngine::link_open_requested(web_view, NEWS, true, true));
        shell.settle().await;
        let (_, private_tab, _) = shell.open_private_window().await;
        shell.submit(private_tab, NEWS).await;
        assert_eq!(asked(&shell).len(), 4);

        // A tab that slept and wakes has a new web view: told again.
        let answer = shell
            .request(request::Kind::SleepOthers(SleepOthers {
                window_id: window,
            }))
            .await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        shell.settle().await;
        let answer = shell.activate_tab(tab).await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        shell.settle().await;
        let all = asked(&shell);
        assert!(all.len() >= 5, "{all:?}");
        assert!(all.iter().all(|(_, told)| *told), "{all:?}");
    });
}
