//! A tab's life through the contract, against the scripted engine: what the
//! shell is asked for, and what the address field is told to show.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    AddressView, CloseWindow, LoadState, NavigationCommand, TabCommand, TabView, address_view,
    host_request, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const FAKE_LOGIN_SET: &str = "57575757-5757-4757-9757-575757575757";

fn tab(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

fn address(shell: &FakeShell, subscription: u64) -> AddressView {
    tab(shell, subscription)
        .address
        .expect("a tab has an address view")
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn a_new_window_has_one_empty_tab_and_no_web_view() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab_id, subscription) = shell.open_window().await;
        let snapshot = shell.window(subscription);
        assert_eq!(snapshot.window_id, window);
        let tab = snapshot.tab.unwrap();
        assert_eq!(tab.tab_id, tab_id);
        assert_eq!(tab.web_view_id, 0, "no web view until there is a page");
        assert_eq!(tab.load_state(), LoadState::Idle);
        assert_eq!(tab.login_set_id, FAKE_LOGIN_SET);
        let address = tab.address.unwrap();
        assert_eq!(address.url, "");
        assert_eq!(address.security(), address_view::Security::Unspecified);
        assert!(
            shell.host_requests.is_empty(),
            "an empty tab costs the shell nothing"
        );
    });
}

#[test]
fn typing_an_address_creates_a_web_view_in_the_spaces_login_set_and_loads_it() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example Domain");
        let (_, tab_id, subscription) = shell.open_window().await;

        let answer = shell.submit(tab_id, "example.com").await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");

        let [
            host_request::Kind::WebViewCreate(create),
            host_request::Kind::WebViewLoad(load),
        ] = shell.host_requests.as_slice()
        else {
            panic!(
                "expected a create then a load, got {:?}",
                shell.host_requests
            );
        };
        assert_ne!(create.web_view_id, 0);
        assert_eq!(create.login_set.as_ref().unwrap().id, FAKE_LOGIN_SET);
        assert!(!create.login_set.as_ref().unwrap().ephemeral);
        assert_eq!(load.web_view_id, create.web_view_id);
        assert_eq!(load.url, "https://example.com/");

        let tab = tab(&shell, subscription);
        assert_eq!(tab.web_view_id, create.web_view_id);
        assert_eq!(tab.title, "Example Domain");
        assert_eq!(tab.load_state(), LoadState::Idle);
        assert!((tab.progress - 1.0).abs() < f64::EPSILON);
        let address = tab.address.unwrap();
        assert_eq!(address.url, "https://example.com/");
        assert_eq!(address.display, "example.com");
        assert_eq!(address.host, "example.com");
        assert_eq!(address.registrable_domain, "example.com");
        assert_eq!(address.security(), address_view::Security::Secure);
    });
}

#[test]
fn the_web_view_is_asked_for_before_any_snapshot_names_it() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, _) = shell.open_window().await;
        let before = shell.order.len();
        shell.submit(tab_id, "example.com").await;
        let after = &shell.order[before..];
        assert_eq!(
            &after[..2],
            ["host_request", "host_request"],
            "create and load leave before the snapshot that names the web view: {after:?}"
        );
    });
}

#[test]
fn a_second_load_reuses_the_web_view() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.org/", "Org");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        shell.submit(tab_id, "example.org").await;
        let creates = shell
            .host_requests
            .iter()
            .filter(|request| matches!(request, host_request::Kind::WebViewCreate(_)))
            .count();
        assert_eq!(creates, 1);
        assert_eq!(address(&shell, subscription).url, "https://example.org/");
        assert_eq!(tab(&shell, subscription).title, "Org");
    });
}

#[test]
fn two_loads_typed_in_a_row_reach_the_shell_in_that_order() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://first.example.com/", "First");
        shell.engine.page("https://second.example.com/", "Second");
        let (_, tab_id, subscription) = shell.open_window().await;
        // Both are sent before the shell has answered anything.
        let submit = |text: &str| {
            request::Kind::SubmitAddress(wt_proto::v1::SubmitAddress {
                tab_id,
                text: text.to_owned(),
            })
        };
        shell.send(submit("first.example.com"));
        shell.send(submit("second.example.com"));
        shell.settle().await;
        let loads: Vec<&str> = shell
            .host_requests
            .iter()
            .filter_map(|request| match request {
                host_request::Kind::WebViewLoad(load) => Some(load.url.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(
            loads,
            ["https://first.example.com/", "https://second.example.com/"]
        );
        assert_eq!(
            address(&shell, subscription).url,
            "https://second.example.com/"
        );
    });
}

#[test]
fn words_become_a_search() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab_id, _) = shell.open_window().await;
        shell.submit(tab_id, "rust borrow checker").await;
        let Some(host_request::Kind::WebViewLoad(load)) = shell.host_requests.last() else {
            panic!("expected a load");
        };
        assert_eq!(load.url, "https://duckduckgo.com/?q=rust+borrow+checker");
    });
}

#[test]
fn typed_script_is_searched_for_and_never_loaded() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab_id, _) = shell.open_window().await;
        shell
            .submit(tab_id, "javascript:alert(document.cookie)")
            .await;
        let Some(host_request::Kind::WebViewLoad(load)) = shell.host_requests.last() else {
            panic!("expected a load");
        };
        assert!(
            load.url.starts_with("https://duckduckgo.com/?q=javascript"),
            "{}",
            load.url
        );
    });
}

#[test]
fn a_redirect_shows_where_the_user_asked_to_go_until_a_page_commits() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.redirect(
            "https://short.example.com/x",
            "https://long.example.com/landing",
        );
        shell
            .engine
            .page("https://long.example.com/landing", "Landing");
        let (_, tab_id, subscription) = shell.open_window().await;

        // While the shell has not yet answered, the field shows what was asked for.
        shell.hold_host = true;
        let id = shell.send(request::Kind::SubmitAddress(wt_proto::v1::SubmitAddress {
            tab_id,
            text: "short.example.com/x".to_owned(),
        }));
        shell.settle().await;
        let waiting = tab(&shell, subscription);
        assert_eq!(waiting.load_state(), LoadState::Loading);
        assert_eq!(waiting.address.unwrap().url, "https://short.example.com/x");
        assert!(shell.take_response(id).is_none());

        shell.release_host();
        shell.settle().await;
        assert!(matches!(
            shell.take_response(id),
            Some(response::Kind::Ack(_))
        ));
        let landed = tab(&shell, subscription);
        assert_eq!(landed.load_state(), LoadState::Idle);
        assert_eq!(
            landed.address.unwrap().url,
            "https://long.example.com/landing"
        );
    });
}

#[test]
fn a_load_a_page_starts_by_itself_does_not_change_the_field_until_it_commits() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;

        // The page sends itself somewhere that never answers.
        shell.report(FakeEngine::navigation_started(
            web_view,
            "https://bank.example/login",
        ));
        shell.settle().await;
        let tab = tab(&shell, subscription);
        assert_eq!(tab.load_state(), LoadState::Loading);
        assert_eq!(
            tab.address.unwrap().url,
            "https://example.com/",
            "the field must keep showing the page that is on screen"
        );
    });
}

#[test]
fn a_failed_load_shows_its_code_and_the_address_that_failed() {
    run(async {
        let mut shell = FakeShell::fake();
        shell
            .engine
            .fail("https://down.example.com/", "net.connect");
        let (_, tab_id, subscription) = shell.open_window().await;
        let answer = shell.submit(tab_id, "down.example.com").await;
        assert!(
            matches!(answer, response::Kind::Ack(_)),
            "the load was started; how it went is the tab's to show"
        );
        let tab = tab(&shell, subscription);
        assert_eq!(tab.load_state(), LoadState::Failed);
        assert_eq!(tab.error_code, "net.connect");
        assert_eq!(tab.address.unwrap().url, "https://down.example.com/");
    });
}

#[test]
fn a_finish_reported_after_a_failure_does_not_clear_it() {
    run(async {
        // Android's WebView commits and finishes an error page of its own
        // after a load fails. The tab must go on saying the load failed, at
        // the address that failed, until something new is loaded.
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell.engine.page("https://example.org/", "Org");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;

        let failing = "https://down.example.com/";
        shell.report(FakeEngine::navigation_started(web_view, failing));
        shell.report(FakeEngine::navigation_failed(
            web_view,
            "net.connect",
            failing,
        ));
        shell.report(FakeEngine::navigation_committed(web_view, failing));
        shell.report(FakeEngine::progress(web_view, 1.0));
        shell.report(FakeEngine::navigation_finished(web_view));
        shell.settle().await;
        let failed = tab(&shell, subscription);
        assert_eq!(failed.load_state(), LoadState::Failed);
        assert_eq!(failed.error_code, "net.connect");
        assert_eq!(failed.address.unwrap().url, failing);

        // A new load clears it.
        shell.submit(tab_id, "example.org").await;
        let loaded = tab(&shell, subscription);
        assert_eq!(loaded.load_state(), LoadState::Idle);
        assert_eq!(loaded.error_code, "");
        assert_eq!(loaded.title, "Org");
    });
}

#[test]
fn a_cancelled_load_goes_back_to_the_page_on_screen() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        shell
            .engine
            .fail("https://slow.example.com/", "net.cancelled");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        shell.submit(tab_id, "slow.example.com").await;
        let tab = tab(&shell, subscription);
        assert_eq!(tab.load_state(), LoadState::Idle);
        assert_eq!(tab.error_code, "");
        assert_eq!(tab.address.unwrap().url, "https://example.com/");
    });
}

#[test]
fn free_text_in_a_failure_never_reaches_the_shell() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;
        shell.report(FakeEngine::navigation_failed(
            web_view,
            "The server at secret.example could not be found.",
            "https://secret.example/",
        ));
        shell.settle().await;
        assert_eq!(tab(&shell, subscription).error_code, "net.failed");
    });
}

#[test]
fn a_page_changing_its_own_address_changes_the_field() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;
        shell.report(FakeEngine::address_changed(
            web_view,
            "https://example.com/inbox#42",
        ));
        shell.settle().await;
        assert_eq!(
            address(&shell, subscription).display,
            "example.com/inbox#42"
        );
    });
}

#[test]
fn back_and_forward_follow_the_web_views_history() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://one.example.com/", "One");
        shell.engine.page("https://two.example.com/", "Two");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "one.example.com").await;
        shell.submit(tab_id, "two.example.com").await;
        assert!(tab(&shell, subscription).can_go_back);
        assert!(!tab(&shell, subscription).can_go_forward);

        let command = |command: NavigationCommand| {
            request::Kind::TabCommand(TabCommand {
                tab_id,
                command: command as i32,
            })
        };
        let answer = shell.request(command(NavigationCommand::Back)).await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        let tab_now = tab(&shell, subscription);
        assert_eq!(tab_now.title, "One");
        assert!(!tab_now.can_go_back);
        assert!(tab_now.can_go_forward);

        shell.request(command(NavigationCommand::Forward)).await;
        assert_eq!(tab(&shell, subscription).title, "Two");
    });
}

#[test]
fn a_command_for_a_tab_without_a_page_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab_id, _) = shell.open_window().await;
        let reload = |tab_id: u64, command: NavigationCommand| {
            request::Kind::TabCommand(TabCommand {
                tab_id,
                command: command as i32,
            })
        };
        assert_eq!(
            error_code(
                shell
                    .request(reload(tab_id, NavigationCommand::Reload))
                    .await
            ),
            "tab.no_page"
        );
        assert_eq!(
            error_code(
                shell
                    .request(reload(9_999, NavigationCommand::Reload))
                    .await
            ),
            "tab.unknown"
        );
        shell.engine.page("https://example.com/", "Example");
        shell.submit(tab_id, "example.com").await;
        assert_eq!(
            error_code(
                shell
                    .request(reload(tab_id, NavigationCommand::Unspecified))
                    .await
            ),
            "tab.bad_command"
        );
    });
}

#[test]
fn nothing_typed_and_no_such_tab_are_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab_id, _) = shell.open_window().await;
        assert_eq!(
            error_code(shell.submit(tab_id, "   ").await),
            "address.empty"
        );
        assert_eq!(
            error_code(shell.submit(9_999, "example.com").await),
            "tab.unknown"
        );
        assert!(shell.host_requests.is_empty());
    });
}

#[test]
fn an_engine_the_shell_lacks_leaves_the_tab_without_a_web_view() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.engines.clear();
        let (_, tab_id, subscription) = shell.open_window().await;
        let answer = shell.submit(tab_id, "example.com").await;
        assert_eq!(error_code(answer), "engine.unavailable");
        let tab = tab(&shell, subscription);
        assert_eq!(tab.web_view_id, 0);
        assert_eq!(tab.load_state(), LoadState::Failed);
        assert_eq!(tab.error_code, "engine.unavailable");
        assert_eq!(shell.pending_host_requests(), 0);
    });
}

#[test]
fn a_dead_renderer_shows_as_a_failure_and_reload_is_still_possible() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;
        shell.report(FakeEngine::renderer_gone(web_view));
        shell.settle().await;
        let gone = tab(&shell, subscription);
        assert_eq!(gone.load_state(), LoadState::Failed);
        assert_eq!(gone.error_code, "page.renderer_gone");

        shell
            .request(request::Kind::TabCommand(TabCommand {
                tab_id,
                command: NavigationCommand::Reload as i32,
            }))
            .await;
        let back = tab(&shell, subscription);
        assert_eq!(back.load_state(), LoadState::Idle);
        assert_eq!(back.error_code, "");
    });
}

#[test]
fn progress_is_kept_between_nought_and_one() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;
        for (reported, shown) in [(0.4, 0.4), (7.0, 1.0), (-3.0, 0.0), (f64::NAN, 0.0)] {
            shell.report(FakeEngine::progress(web_view, reported));
            shell.settle().await;
            let progress = tab(&shell, subscription).progress;
            assert!(
                (progress - shown).abs() < f64::EPSILON,
                "{reported} → {progress}"
            );
        }
    });
}

#[test]
fn reports_that_change_nothing_publish_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;
        let before = shell.snapshots(subscription).len();
        for _ in 0..50 {
            shell.report(FakeEngine::progress(web_view, 1.0));
        }
        shell.settle().await;
        assert_eq!(shell.snapshots(subscription).len(), before);
    });
}

#[test]
fn reports_that_arrive_together_publish_one_snapshot() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;
        let before = shell.snapshots(subscription).len();
        for step in 1..=20 {
            shell.report(FakeEngine::progress(web_view, f64::from(step) / 40.0));
        }
        shell.settle().await;
        assert_eq!(shell.snapshots(subscription).len(), before + 1);
    });
}

#[test]
fn closing_a_window_destroys_its_web_view_and_ends_its_subscription() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (window, tab_id, subscription) = shell.open_window().await;
        shell.submit(tab_id, "example.com").await;
        let web_view = tab(&shell, subscription).web_view_id;
        assert_eq!(shell.engine.web_views.len(), 1);

        let answer = shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: window,
            }))
            .await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        assert!(
            shell.engine.web_views.is_empty(),
            "the web view is destroyed"
        );

        // A late report about the destroyed web view changes and says nothing.
        let before = shell.events.len();
        shell.report(FakeEngine::progress(web_view, 0.5));
        shell.settle().await;
        assert_eq!(shell.events.len(), before);

        assert_eq!(
            error_code(
                shell
                    .request(request::Kind::CloseWindow(CloseWindow {
                        window_id: window
                    }))
                    .await
            ),
            "window.unknown"
        );
        assert_eq!(
            error_code(shell.submit(tab_id, "example.com").await),
            "tab.unknown"
        );
    });
}

#[test]
fn fake_mode_opens_its_start_address_in_a_new_window() {
    run(async {
        let mut config = FakeShell::config(wt_proto::v1::EngineKind::Chromium);
        config.fake.as_mut().unwrap().start_address = "https://example.com/".to_owned();
        let mut shell = FakeShell::start(&config).unwrap();
        shell.engine.page("https://example.com/", "Example");
        let (_, _, subscription) = shell.open_window().await;
        let tab = tab(&shell, subscription);
        assert_eq!(tab.title, "Example");
        assert_eq!(tab.engine(), wt_proto::v1::EngineKind::Chromium);
        assert_eq!(tab.address.unwrap().url, "https://example.com/");
    });
}
