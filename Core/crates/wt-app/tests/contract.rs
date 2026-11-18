//! The contract's own rules: starting, the health check, subscriptions, the
//! host bridge's patience, and the panic fence.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::time::Duration;

use wt_proto::v1::{
    Ack, CoreConfig, EngineInfo, EngineKind, Envelope, Error, Ping, Request, Subscribe, TestMode,
    Unsubscribe, WindowTopic, core_status, envelope, event, host_request, host_response, request,
    response, subscribe,
};
use wt_proto::{CONTRACT_MAJOR, CONTRACT_MINOR};
use wt_testkit::{FakeShell, run};

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

fn ping(payload: &[u8], via_host: bool, panic: bool) -> request::Kind {
    request::Kind::Ping(Ping {
        payload: payload.to_vec(),
        via_host,
        panic,
    })
}

fn start_error(config: &CoreConfig) -> String {
    match FakeShell::start(config) {
        Ok(_) => panic!("the core should have refused to start"),
        Err(error) => error.code().to_owned(),
    }
}

#[test]
fn a_started_core_says_it_is_ready() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.settle().await;
        let [event::Kind::Status(status)] = shell.events.as_slice() else {
            panic!("expected one status event, got {:?}", shell.events);
        };
        assert_eq!(status.state(), core_status::State::Ready);
        assert_eq!(status.code, "");
    });
}

#[test]
fn another_contract_major_is_refused() {
    let mut config = FakeShell::config(EngineKind::Webkit);
    config.contract_major = CONTRACT_MAJOR + 1;
    assert_eq!(start_error(&config), "start.contract_major");
}

#[test]
fn a_newer_minor_is_welcome() {
    let mut config = FakeShell::config(EngineKind::Webkit);
    config.contract_minor = CONTRACT_MINOR + 40;
    assert!(FakeShell::start(&config).is_ok());
}

#[test]
fn a_shell_with_no_engine_is_refused() {
    let mut config = FakeShell::config(EngineKind::Webkit);
    config.engines.clear();
    assert_eq!(start_error(&config), "start.no_engine");
    config.engines.push(EngineInfo {
        kind: EngineKind::Unspecified as i32,
        version: String::new(),
    });
    assert_eq!(start_error(&config), "start.no_engine");
}

#[test]
fn live_mode_needs_a_data_folder() {
    let mut config = FakeShell::config(EngineKind::Webkit);
    config.test_mode = TestMode::Live as i32;
    assert_eq!(start_error(&config), "start.data_dir");
}

#[test]
fn live_mode_keeps_its_login_set_across_restarts() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut config = FakeShell::config(EngineKind::Webkit);
        config.test_mode = TestMode::Live as i32;
        config.data_dir = folder.path().join("data").to_string_lossy().into_owned();

        let login_set = |config: &CoreConfig| {
            let config = config.clone();
            async move {
                let mut shell = FakeShell::start(&config).unwrap();
                let (_, _, subscription) = shell.open_window().await;
                shell.window(subscription).tab.unwrap().login_set_id
            }
        };
        let first = login_set(&config).await;
        let second = login_set(&config).await;
        assert_eq!(first.len(), 36);
        assert_ne!(
            first, "57575757-5757-4757-9757-575757575757",
            "live mode draws its own"
        );
        assert_eq!(first, second, "the same logins tomorrow");
    });
}

#[test]
fn ping_echoes_and_names_the_contract() {
    run(async {
        let mut shell = FakeShell::fake();
        let response::Kind::Pong(pong) = shell.request(ping(b"hello", false, false)).await else {
            panic!("Ping answers Pong");
        };
        assert_eq!(pong.payload, b"hello");
        assert!(!pong.via_host);
        assert_eq!(pong.core_version, wt_app::CORE_VERSION);
        assert_eq!(
            (pong.contract_major, pong.contract_minor),
            (CONTRACT_MAJOR, CONTRACT_MINOR)
        );
    });
}

#[test]
fn ping_can_go_round_through_the_shell() {
    run(async {
        let mut shell = FakeShell::fake();
        let response::Kind::Pong(pong) = shell.request(ping(b"round", true, false)).await else {
            panic!("Ping answers Pong");
        };
        assert_eq!(pong.payload, b"round");
        assert!(pong.via_host);
        assert!(matches!(
            shell.host_requests.as_slice(),
            [host_request::Kind::Ping(_)]
        ));
    });
}

#[test]
fn an_oversized_ping_is_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        let payload = vec![0u8; wt_app::MAX_PING_PAYLOAD + 1];
        assert_eq!(
            error_code(shell.request(ping(&payload, false, false)).await),
            "ping.payload_too_large"
        );
    });
}

#[test]
fn a_panic_inside_a_request_is_answered_and_the_core_carries_on() {
    run(async {
        let mut shell = FakeShell::fake();
        assert_eq!(
            error_code(shell.request(ping(b"", false, true)).await),
            "core.panic"
        );
        let response::Kind::Pong(pong) = shell.request(ping(b"still here", false, false)).await
        else {
            panic!("the core must still answer");
        };
        assert_eq!(pong.payload, b"still here");
    });
}

#[test]
fn the_panic_hook_is_fake_modes_alone() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut config = FakeShell::config(EngineKind::Webkit);
        config.test_mode = TestMode::Live as i32;
        config.data_dir = folder.path().to_string_lossy().into_owned();
        let mut shell = FakeShell::start(&config).unwrap();
        let answer = shell.request(ping(b"x", false, true)).await;
        assert!(matches!(answer, response::Kind::Pong(_)), "{answer:?}");
    });
}

#[test]
fn a_shell_that_never_answers_times_out() {
    run(async {
        let mut shell = FakeShell::start_with(
            &FakeShell::config(EngineKind::Webkit),
            Duration::from_secs(5),
        )
        .unwrap();
        shell.hold_host = true;
        let id = shell.send(ping(b"x", true, false));
        shell.settle().await;
        assert!(shell.take_response(id).is_none(), "still waiting");
        assert_eq!(shell.pending_host_requests(), 1);

        tokio::time::sleep(Duration::from_secs(6)).await;
        shell.settle().await;
        assert_eq!(error_code(shell.take_response(id).unwrap()), "host.timeout");
        assert_eq!(
            shell.pending_host_requests(),
            0,
            "nothing is kept for an answer that is late"
        );
    });
}

#[test]
fn a_wrong_answer_and_free_text_are_replaced_by_codes() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.answer_next_with = Some(host_response::Kind::Ack(Ack {}));
        assert_eq!(
            error_code(shell.request(ping(b"x", true, false)).await),
            "host.wrong_answer"
        );
        shell.answer_next_with = Some(host_response::Kind::Error(Error {
            code: "Could not reach https://secret.example/".to_owned(),
            field: String::new(),
        }));
        assert_eq!(
            error_code(shell.request(ping(b"x", true, false)).await),
            "host.failed"
        );
        shell.answer_next_with = Some(host_response::Kind::Error(Error {
            code: "vault.locked".to_owned(),
            field: String::new(),
        }));
        assert_eq!(
            error_code(shell.request(ping(b"x", true, false)).await),
            "vault.locked"
        );
    });
}

#[test]
fn a_host_response_nobody_waits_for_is_dropped() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.send_raw(Envelope {
            id: 4_242,
            body: Some(envelope::Body::HostResponse(wt_proto::v1::HostResponse {
                kind: Some(host_response::Kind::Ack(Ack {})),
            })),
        });
        shell.settle().await;
        let response::Kind::Pong(_) = shell.request(ping(b"x", false, false)).await else {
            panic!("the core must still answer");
        };
    });
}

#[test]
fn an_empty_request_is_refused_and_an_empty_envelope_ignored() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.send_raw(Envelope { id: 0, body: None });
        shell.send_raw(Envelope {
            id: 77,
            body: Some(envelope::Body::Request(Request { kind: None })),
        });
        shell.settle().await;
        assert_eq!(
            error_code(shell.take_response(77).unwrap()),
            "request.empty"
        );
    });
}

#[test]
fn a_notification_gets_no_answer_even_when_it_fails() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.settle().await;
        let before = shell.order.len();
        shell.send_raw(Envelope {
            id: 0,
            body: Some(envelope::Body::Request(Request {
                kind: Some(request::Kind::SubmitAddress(wt_proto::v1::SubmitAddress {
                    tab_id: 9_999,
                    text: "example.com".to_owned(),
                })),
            })),
        });
        shell.settle().await;
        assert_eq!(shell.order.len(), before, "nothing was sent back");
    });
}

#[test]
fn subscribing_answers_first_and_then_sends_the_snapshot() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, _, _) = shell.open_window().await;
        let before = shell.order.len();
        let answer = shell
            .request(request::Kind::Subscribe(Subscribe {
                topic: Some(subscribe::Topic::Window(WindowTopic { window_id: window })),
            }))
            .await;
        let response::Kind::Subscribed(subscribed) = answer else {
            panic!("Subscribe answers Subscribed");
        };
        assert_eq!(&shell.order[before..], ["response", "event"]);
        assert_eq!(shell.snapshots(subscribed.subscription_id).len(), 1);
    });
}

#[test]
fn two_subscriptions_to_one_window_both_hear_of_a_change() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (window, tab_id, first) = shell.open_window().await;
        let response::Kind::Subscribed(second) = shell
            .request(request::Kind::Subscribe(Subscribe {
                topic: Some(subscribe::Topic::Window(WindowTopic { window_id: window })),
            }))
            .await
        else {
            panic!("Subscribe answers Subscribed");
        };
        assert_ne!(first, second.subscription_id);
        shell.submit(tab_id, "example.com").await;
        assert_eq!(shell.window(first), shell.window(second.subscription_id));
        assert_eq!(shell.window(first).tab.unwrap().title, "Example");
    });
}

#[test]
fn an_ended_subscription_hears_nothing_more() {
    run(async {
        let mut shell = FakeShell::fake();
        shell.engine.page("https://example.com/", "Example");
        let (_, tab_id, subscription) = shell.open_window().await;
        let unsubscribe = request::Kind::Unsubscribe(Unsubscribe {
            subscription_id: subscription,
        });
        assert!(matches!(
            shell.request(unsubscribe.clone()).await,
            response::Kind::Ack(_)
        ));
        let before = shell.snapshots(subscription).len();
        shell.submit(tab_id, "example.com").await;
        assert_eq!(shell.snapshots(subscription).len(), before);
        // Ending it twice is not an error.
        assert!(matches!(
            shell.request(unsubscribe).await,
            response::Kind::Ack(_)
        ));
    });
}

#[test]
fn a_subscription_needs_a_topic_and_a_window_that_exists() {
    run(async {
        let mut shell = FakeShell::fake();
        assert_eq!(
            error_code(
                shell
                    .request(request::Kind::Subscribe(Subscribe { topic: None }))
                    .await
            ),
            "subscribe.no_topic"
        );
        assert_eq!(
            error_code(
                shell
                    .request(request::Kind::Subscribe(Subscribe {
                        topic: Some(subscribe::Topic::Window(WindowTopic { window_id: 5 })),
                    }))
                    .await
            ),
            "window.unknown"
        );
    });
}
