//! The core through its C ABI, as a shell drives it.
#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "the tests call the C ABI, and their helpers fail the test the way assertions do"
)]

mod support;

use std::time::Duration;

use prost::Message as _;
use support::{TestShell, fake_config};
use webtiger_core::{
    MAX_MESSAGE_LEN, Status, WT_NAV_ALLOW, WT_NAV_CANCEL, WT_NAV_EXTERNAL, WT_NAV_MAIN_FRAME,
    WT_STORAGE_APART, WT_STORAGE_OWN, WT_VERDICT_ALLOW, WT_VERDICT_BLOCK, WtFrame, WtNavVerdict,
    WtNavigation, WtRequest, WtStorageVerdict, WtVerdict, wt_core_send, wt_core_start,
    wt_core_stop, wt_guard_check, wt_nav_check, wt_storage_check,
};
use wt_proto::v1::{
    EngineEvent, LoadState, NavigationCommitted, NavigationFinished, OpenWindow, Ping,
    SubmitAddress, Subscribe, TitleChanged, WindowTopic, core_status, engine_event, event,
    host_request, request, response, snapshot, subscribe,
};

fn ping(payload: &[u8], via_host: bool) -> request::Kind {
    request::Kind::Ping(Ping {
        payload: payload.to_vec(),
        via_host,
        panic: false,
    })
}

fn fatal_code(shell: &mut TestShell) -> String {
    match shell.next_event() {
        event::Kind::Status(status) => {
            assert_eq!(status.state(), core_status::State::Fatal);
            status.code
        }
        other => panic!("expected a status, got {other:?}"),
    }
}

fn nav(handle: u64, url: &[u8], flags: u32) -> (i32, u32) {
    let navigation = WtNavigation {
        url: url.as_ptr(),
        url_len: url.len(),
        web_view_id: 1,
        flags,
    };
    let mut verdict = WtNavVerdict { action: 99 };
    // SAFETY: both structures are live for the call, and `url` covers its length.
    let status = unsafe { wt_nav_check(handle, &raw const navigation, &raw mut verdict) };
    (status, verdict.action)
}

#[test]
fn a_core_starts_says_ready_answers_a_ping_and_stops() {
    let mut shell = TestShell::start(&fake_config());
    assert_ne!(shell.handle, 0);
    let event::Kind::Status(status) = shell.next_event() else {
        panic!("the first envelope is a status");
    };
    assert_eq!(status.state(), core_status::State::Ready);

    let response::Kind::Pong(pong) = shell.request(ping(b"hello", false)) else {
        panic!("Ping answers Pong");
    };
    assert_eq!(pong.payload, b"hello");

    assert_eq!(shell.stop(), Status::Ok as i32);
    assert_eq!(
        wt_core_stop(shell.handle),
        Status::UnknownCore as i32,
        "stopped once"
    );
    assert_eq!(shell.send_bytes(&[]), Status::UnknownCore as i32);
    assert!(
        shell.try_next(Duration::from_millis(50)).is_none(),
        "the sink is silent after stop"
    );
}

#[test]
fn a_ping_goes_round_through_the_shell() {
    let mut shell = TestShell::fake();
    let response::Kind::Pong(pong) = shell.request(ping(b"round", true)) else {
        panic!("Ping answers Pong");
    };
    assert!(pong.via_host);
    assert_eq!(pong.payload, b"round");
    assert!(matches!(
        shell.host_requests.as_slice(),
        [host_request::Kind::Ping(_)]
    ));
}

#[test]
fn a_page_loads_through_the_contract() {
    let mut shell = TestShell::fake();
    let response::Kind::WindowOpened(opened) =
        shell.request(request::Kind::OpenWindow(OpenWindow::default()))
    else {
        panic!("OpenWindow answers WindowOpened");
    };
    let response::Kind::Subscribed(subscribed) =
        shell.request(request::Kind::Subscribe(Subscribe {
            topic: Some(subscribe::Topic::Window(WindowTopic {
                window_id: opened.window_id,
            })),
        }))
    else {
        panic!("Subscribe answers Subscribed");
    };
    let answer = shell.request(request::Kind::SubmitAddress(SubmitAddress {
        tab_id: opened.tab_id,
        text: "example.com".to_owned(),
    }));
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
    assert_eq!(load.url, "https://example.com/");
    let web_view = create.web_view_id;

    // The shell's engine reports the page arriving.
    for kind in [
        engine_event::Kind::NavigationCommitted(NavigationCommitted {
            url: "https://example.com/".to_owned(),
        }),
        engine_event::Kind::TitleChanged(TitleChanged {
            title: "Example Domain".to_owned(),
        }),
        engine_event::Kind::NavigationFinished(NavigationFinished {}),
    ] {
        shell.report(EngineEvent {
            web_view_id: web_view,
            kind: Some(kind),
        });
    }
    // The last snapshot shows the finished page.
    let finished = loop {
        let event::Kind::Snapshot(snapshot) = shell.next_event() else {
            continue;
        };
        assert_eq!(snapshot.subscription_id, subscribed.subscription_id);
        let Some(snapshot::Model::Window(window)) = snapshot.model else {
            panic!("a window subscription publishes windows");
        };
        let tab = window.tab.unwrap();
        if tab.load_state() == LoadState::Idle && !tab.title.is_empty() {
            break tab;
        }
    };
    assert_eq!(finished.web_view_id, web_view);
    assert_eq!(finished.title, "Example Domain");
    assert_eq!(finished.address.unwrap().display, "example.com");
}

#[test]
fn a_refused_start_returns_no_handle_and_says_why() {
    let mut shell = TestShell::start_bytes(&[0xFF, 0xFF, 0xFF]);
    assert_eq!(shell.handle, 0);
    assert_eq!(fatal_code(&mut shell), "start.bad_config");

    let mut config = fake_config();
    config.contract_major += 1;
    let mut shell = TestShell::start(&config);
    assert_eq!(shell.handle, 0);
    assert_eq!(fatal_code(&mut shell), "start.contract_major");

    let mut config = fake_config();
    config.engines.clear();
    let mut shell = TestShell::start(&config);
    assert_eq!(shell.handle, 0);
    assert_eq!(fatal_code(&mut shell), "start.no_engine");
}

#[test]
fn a_start_without_a_sink_or_with_a_null_config_is_refused() {
    let config = fake_config().encode_to_vec();
    // SAFETY: the slice is live; there is no sink to call.
    let handle =
        unsafe { wt_core_start(config.as_ptr(), config.len(), None, std::ptr::null_mut()) };
    assert_eq!(handle, 0);

    // NULL with a length: refused without being read. NULL with no length is
    // an empty configuration, which has no contract version.
    extern "C" fn quiet(_: *mut std::ffi::c_void, _: *const u8, _: usize) {}
    // SAFETY: the pointer is NULL and is not read.
    let handle = unsafe { wt_core_start(std::ptr::null(), 12, Some(quiet), std::ptr::null_mut()) };
    assert_eq!(handle, 0);
    // SAFETY: as above.
    let handle = unsafe { wt_core_start(std::ptr::null(), 0, Some(quiet), std::ptr::null_mut()) };
    assert_eq!(handle, 0);
}

#[test]
fn bad_sends_are_refused_with_a_status() {
    let shell = TestShell::fake();
    assert_eq!(
        shell.send_bytes(&[0xFF, 0xFF, 0xFF]),
        Status::Malformed as i32
    );
    // SAFETY: NULL with a length is refused without being read.
    let status = unsafe { wt_core_send(shell.handle, std::ptr::null(), 8) };
    assert_eq!(status, Status::NullArgument as i32);
    let huge = vec![0u8; MAX_MESSAGE_LEN + 1];
    assert_eq!(shell.send_bytes(&huge), Status::TooLarge as i32);
    // SAFETY: an empty slice.
    let status = unsafe { wt_core_send(9_999_999, [].as_ptr(), 0) };
    assert_eq!(status, Status::UnknownCore as i32);
    // An empty envelope is well-formed, and ignored.
    assert_eq!(shell.send_bytes(&[]), Status::Ok as i32);
}

#[test]
fn a_panic_inside_the_core_is_an_answer_not_a_crash() {
    let mut shell = TestShell::fake();
    let answer = shell.request(request::Kind::Ping(Ping {
        payload: Vec::new(),
        via_host: false,
        panic: true,
    }));
    let response::Kind::Error(error) = answer else {
        panic!("expected an error, got {answer:?}");
    };
    assert_eq!(error.code, "core.panic");
    assert!(matches!(
        shell.request(ping(b"alive", false)),
        response::Kind::Pong(_)
    ));
}

#[test]
fn stopping_drops_requests_that_wait_on_the_shell() {
    let mut shell = TestShell::fake();
    shell.answer_host = false;
    shell.send(ping(b"never answered", true));
    // The host request arrives; the shell never answers it.
    let waiting = shell.try_next(support::PATIENCE).expect("the host request");
    assert_ne!(waiting.id, 0);
    assert_eq!(shell.stop(), Status::Ok as i32);
    assert!(shell.try_next(Duration::from_millis(50)).is_none());
}

#[test]
fn stop_from_inside_the_sink_is_refused() {
    use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
    static HANDLE: AtomicU64 = AtomicU64::new(0);
    static SEEN: AtomicI32 = AtomicI32::new(-1);
    extern "C" fn stopping_sink(_: *mut std::ffi::c_void, _: *const u8, _: usize) {
        let handle = HANDLE.load(Ordering::SeqCst);
        if handle != 0 && SEEN.load(Ordering::SeqCst) == -1 {
            SEEN.store(wt_core_stop(handle), Ordering::SeqCst);
        }
    }
    let config = fake_config().encode_to_vec();
    // SAFETY: the slice is live; the sink is a static function with no context.
    let handle = unsafe {
        wt_core_start(
            config.as_ptr(),
            config.len(),
            Some(stopping_sink),
            std::ptr::null_mut(),
        )
    };
    assert_ne!(handle, 0);
    HANDLE.store(handle, Ordering::SeqCst);
    let request = wt_proto::v1::Envelope {
        id: 1,
        body: Some(wt_proto::v1::envelope::Body::Request(
            wt_proto::v1::Request {
                kind: Some(ping(b"x", false)),
            },
        )),
    }
    .encode_to_vec();
    // SAFETY: the slice is live.
    assert_eq!(
        unsafe { wt_core_send(handle, request.as_ptr(), request.len()) },
        0
    );
    for _ in 0..200 {
        if SEEN.load(Ordering::SeqCst) != -1 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(SEEN.load(Ordering::SeqCst), Status::WrongThread as i32);
    assert_eq!(
        wt_core_stop(handle),
        Status::Ok as i32,
        "and it still stops from outside"
    );
}

#[test]
fn many_requests_from_many_threads_are_all_answered() {
    let shell = TestShell::fake();
    let handle = shell.handle;
    let senders: Vec<_> = (0..8u64)
        .map(|thread| {
            std::thread::spawn(move || {
                for index in 0..250u64 {
                    let envelope = wt_proto::v1::Envelope {
                        id: 1_000 + thread * 1_000 + index,
                        body: Some(wt_proto::v1::envelope::Body::Request(
                            wt_proto::v1::Request {
                                kind: Some(ping(&index.to_le_bytes(), false)),
                            },
                        )),
                    }
                    .encode_to_vec();
                    // SAFETY: the slice is live.
                    let status = unsafe { wt_core_send(handle, envelope.as_ptr(), envelope.len()) };
                    assert_eq!(status, 0);
                }
            })
        })
        .collect();
    for sender in senders {
        sender.join().unwrap();
    }
    let mut answered = std::collections::BTreeSet::new();
    while answered.len() < 2_000 {
        let envelope = shell
            .try_next(support::PATIENCE)
            .expect("every ping is answered");
        if let Some(wt_proto::v1::envelope::Body::Response(_)) = envelope.body {
            assert!(
                answered.insert(envelope.id),
                "answered twice: {}",
                envelope.id
            );
        }
    }
}

#[test]
fn the_navigation_policy_answers_on_the_callers_thread() {
    let shell = TestShell::fake();
    let main = WT_NAV_MAIN_FRAME;
    assert_eq!(
        nav(shell.handle, b"https://example.com/", main),
        (0, WT_NAV_ALLOW)
    );
    assert_eq!(
        nav(shell.handle, b"http://example.com/", 0),
        (0, WT_NAV_ALLOW)
    );
    assert_eq!(nav(shell.handle, b"about:blank", main), (0, WT_NAV_ALLOW));
    assert_eq!(
        nav(shell.handle, b"javascript:alert(1)", main),
        (0, WT_NAV_CANCEL)
    );
    assert_eq!(
        nav(shell.handle, b"file:///etc/passwd", main),
        (0, WT_NAV_CANCEL)
    );
    assert_eq!(
        nav(shell.handle, b"data:text/html,x", main),
        (0, WT_NAV_CANCEL)
    );
    assert_eq!(nav(shell.handle, b"data:text/html,x", 0), (0, WT_NAV_ALLOW));
    assert_eq!(
        nav(shell.handle, b"mailto:ada@example.com", main),
        (0, WT_NAV_EXTERNAL)
    );
    assert_eq!(nav(shell.handle, b"", main), (0, WT_NAV_CANCEL));
}

#[test]
fn the_navigation_policy_fails_closed() {
    let shell = TestShell::fake();
    // Not UTF-8.
    assert_eq!(
        nav(shell.handle, &[b'h', b't', 0xFF, 0xFE], WT_NAV_MAIN_FRAME),
        (Status::BadArgument as i32, WT_NAV_CANCEL)
    );
    // No such core.
    assert_eq!(
        nav(9_999_999, b"https://example.com/", WT_NAV_MAIN_FRAME),
        (Status::UnknownCore as i32, WT_NAV_CANCEL)
    );
    assert_eq!(
        nav(0, b"https://example.com/", WT_NAV_MAIN_FRAME),
        (Status::UnknownCore as i32, WT_NAV_CANCEL)
    );
    // NULL arguments.
    let mut verdict = WtNavVerdict { action: 99 };
    // SAFETY: NULL is refused without being read; the verdict is live.
    let status = unsafe { wt_nav_check(shell.handle, std::ptr::null(), &raw mut verdict) };
    assert_eq!(
        (status, verdict.action),
        (Status::NullArgument as i32, WT_NAV_CANCEL)
    );
    let navigation = WtNavigation {
        url: std::ptr::null(),
        url_len: 5,
        web_view_id: 1,
        flags: 0,
    };
    // SAFETY: the structure is live; its NULL address is refused unread.
    let status = unsafe { wt_nav_check(shell.handle, &raw const navigation, &raw mut verdict) };
    assert_eq!(
        (status, verdict.action),
        (Status::BadArgument as i32, WT_NAV_CANCEL)
    );
    // SAFETY: a NULL verdict is refused before anything is written.
    let status = unsafe { wt_nav_check(shell.handle, &raw const navigation, std::ptr::null_mut()) };
    assert_eq!(status, Status::NullArgument as i32);
}

#[test]
fn a_stopped_core_answers_no_fast_path_call() {
    let mut shell = TestShell::fake();
    let handle = shell.handle;
    assert_eq!(nav(handle, b"https://example.com/", 0).0, 0);
    shell.stop();
    assert_eq!(
        nav(handle, b"https://example.com/", 0),
        (Status::UnknownCore as i32, WT_NAV_CANCEL)
    );
}

#[test]
fn guard_with_no_rules_allows_everything_and_fails_open() {
    let shell = TestShell::fake();
    let url = b"https://tracker.example/pixel.gif";
    let source = b"https://example.com/";
    let request = WtRequest {
        url: url.as_ptr(),
        url_len: url.len(),
        source_url: source.as_ptr(),
        source_url_len: source.len(),
        web_view_id: 1,
        resource_type: 5,
    };
    let mut verdict = WtVerdict {
        action: WT_VERDICT_BLOCK,
    };
    // SAFETY: both structures are live, and each address covers its length.
    let status = unsafe { wt_guard_check(shell.handle, &raw const request, &raw mut verdict) };
    assert_eq!((status, verdict.action), (0, WT_VERDICT_ALLOW));

    let mut verdict = WtVerdict {
        action: WT_VERDICT_BLOCK,
    };
    // SAFETY: as above; there is no such core.
    let status = unsafe { wt_guard_check(9_999_999, &raw const request, &raw mut verdict) };
    assert_eq!(
        (status, verdict.action),
        (Status::UnknownCore as i32, WT_VERDICT_ALLOW),
        "a blocker that fails must not break the page"
    );
    let mut verdict = WtVerdict {
        action: WT_VERDICT_BLOCK,
    };
    // SAFETY: NULL is refused without being read.
    let status = unsafe { wt_guard_check(shell.handle, std::ptr::null(), &raw mut verdict) };
    assert_eq!(
        (status, verdict.action),
        (Status::NullArgument as i32, WT_VERDICT_ALLOW)
    );
}

/// Guard's answer for a request, through the C function: the status, and
/// the verdict written.
fn guard(handle: u64, url: &[u8], source: &[u8], resource_type: u32) -> (i32, u32) {
    let request = WtRequest {
        url: url.as_ptr(),
        url_len: url.len(),
        source_url: source.as_ptr(),
        source_url_len: source.len(),
        web_view_id: 1,
        resource_type,
    };
    let mut verdict = WtVerdict {
        action: WT_VERDICT_BLOCK,
    };
    // SAFETY: both structures are live, and each address covers its length.
    let status = unsafe { wt_guard_check(handle, &raw const request, &raw mut verdict) };
    (status, verdict.action)
}

/// A core in fake mode whose suite gave Guard these rules (064).
fn with_rules(rules: &str) -> TestShell {
    let mut config = fake_config();
    config.fake = Some(wt_proto::v1::FakeOptions {
        guard_rules: rules.to_owned(),
        ..wt_proto::v1::FakeOptions::default()
    });
    TestShell::start(&config)
}

const PAGE: &[u8] = b"https://news.example.com/story";
const RULES: &str = "||ads.tracker.example.net^\n@@||ads.tracker.example.net/consent.js$script\n||metrics.example.org^$script,third-party\n";
const SCRIPT: u32 = 3;
const IMAGE: u32 = 5;
const DOCUMENT: u32 = 1;

#[test]
fn guard_answers_by_a_suites_rules_and_each_core_by_its_own() {
    let mut ruled = with_rules(RULES);
    let plain = TestShell::fake();
    let blocked = (0, WT_VERDICT_BLOCK);
    let allowed = (0, WT_VERDICT_ALLOW);

    // By the rules: a blocked host, whatever it is for; an exception; a rule for scripts
    // of another site alone; and what no rule names.
    let show = b"https://ads.tracker.example.net/show.js";
    assert_eq!(guard(ruled.handle, show, PAGE, SCRIPT), blocked);
    assert_eq!(
        guard(
            ruled.handle,
            b"https://ads.tracker.example.net/pixel.gif",
            PAGE,
            IMAGE
        ),
        blocked
    );
    assert_eq!(
        guard(
            ruled.handle,
            b"https://ads.tracker.example.net/consent.js",
            PAGE,
            SCRIPT
        ),
        allowed
    );
    assert_eq!(
        guard(
            ruled.handle,
            b"https://metrics.example.org/collect.js",
            PAGE,
            SCRIPT
        ),
        blocked
    );
    assert_eq!(
        guard(
            ruled.handle,
            b"https://metrics.example.org/collect.js",
            PAGE,
            IMAGE
        ),
        allowed
    );
    assert_eq!(
        guard(
            ruled.handle,
            b"https://metrics.example.org/collect.js",
            b"https://www.example.org/",
            SCRIPT
        ),
        allowed,
        "its own site's script"
    );
    assert_eq!(
        guard(
            ruled.handle,
            b"https://cdn.example.org/app.js",
            PAGE,
            SCRIPT
        ),
        allowed
    );
    // The top frame's own document is allowed whatever the rules; a kind this core has
    // not heard of is asked about as one it cannot name.
    assert_eq!(
        guard(
            ruled.handle,
            b"https://ads.tracker.example.net/landing",
            PAGE,
            DOCUMENT
        ),
        allowed
    );
    assert_eq!(guard(ruled.handle, show, PAGE, 99), blocked);
    // What is no address is allowed, and so is what is not text: a blocker that cannot
    // make sense of a request must not break the page.
    assert_eq!(
        guard(ruled.handle, b"not an address", PAGE, SCRIPT),
        allowed
    );
    assert_eq!(
        guard(ruled.handle, &[0xff, 0xfe, 0xfd], PAGE, SCRIPT),
        (Status::BadArgument as i32, WT_VERDICT_ALLOW)
    );
    // A page that is no address is nobody's site: what it asks for is the third party's.
    assert_eq!(
        guard(
            ruled.handle,
            b"https://metrics.example.org/collect.js",
            b"",
            SCRIPT
        ),
        blocked
    );

    // Another core, with no rules, allows the same: each core has its own blocker.
    assert_eq!(guard(plain.handle, show, PAGE, SCRIPT), allowed);

    // A core that has stopped answers nothing, and the request is allowed.
    let handle = ruled.handle;
    ruled.stop();
    assert_eq!(
        guard(handle, show, PAGE, SCRIPT),
        (Status::UnknownCore as i32, WT_VERDICT_ALLOW)
    );
}

#[test]
fn guard_has_a_suites_rules_in_fake_mode_alone() {
    // Outside fake mode a suite's rules are nothing: a person's own app is not given
    // rules by what started it.
    let folder = tempfile::tempdir().unwrap();
    let mut config = fake_config();
    config.test_mode = wt_proto::v1::TestMode::Live as i32;
    config.data_dir = folder.path().to_str().unwrap().to_owned();
    config.fake = Some(wt_proto::v1::FakeOptions {
        guard_rules: RULES.to_owned(),
        ..wt_proto::v1::FakeOptions::default()
    });
    let shell = TestShell::start(&config);
    assert_eq!(
        guard(
            shell.handle,
            b"https://ads.tracker.example.net/show.js",
            PAGE,
            SCRIPT
        ),
        (0, WT_VERDICT_ALLOW)
    );
}

#[test]
fn guard_is_asked_from_many_threads_at_once() {
    let shell = with_rules(RULES);
    let handle = shell.handle;
    let threads: Vec<_> = (0..8)
        .map(|thread| {
            std::thread::spawn(move || {
                let mut wrong = 0;
                for round in 0..2_000 {
                    let (url, expected): (String, u32) = if (round + thread) % 2 == 0 {
                        (
                            format!("https://ads.tracker.example.net/show.js?n={round}"),
                            WT_VERDICT_BLOCK,
                        )
                    } else {
                        (
                            format!("https://cdn{round}.example.org/app.js"),
                            WT_VERDICT_ALLOW,
                        )
                    };
                    if guard(handle, url.as_bytes(), PAGE, SCRIPT) != (0, expected) {
                        wrong += 1;
                    }
                }
                wrong
            })
        })
        .collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), 0);
    }
}

/// Sets a person's own rules for Guard (066), through the contract.
fn set_rules(shell: &mut TestShell, text: &str) {
    let answer = shell.request(request::Kind::SetGuardRules(wt_proto::v1::SetGuardRules {
        text: text.to_owned(),
    }));
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

#[test]
fn a_persons_rules_are_in_force_the_moment_they_are_set_and_after_a_start() {
    let folder = tempfile::tempdir().unwrap();
    let mut config = fake_config();
    config.test_mode = wt_proto::v1::TestMode::Live as i32;
    config.data_dir = folder.path().to_str().unwrap().to_owned();
    let show = b"https://ads.tracker.example.net/show.js";
    let blocked = (0, WT_VERDICT_BLOCK);
    let allowed = (0, WT_VERDICT_ALLOW);

    let mut shell = TestShell::start(&config);
    assert_eq!(
        guard(shell.handle, show, PAGE, SCRIPT),
        allowed,
        "no rules yet"
    );
    // Set: the very next thing asked is decided by them. Nothing was started again.
    set_rules(&mut shell, RULES);
    assert_eq!(guard(shell.handle, show, PAGE, SCRIPT), blocked);
    assert_eq!(
        guard(
            shell.handle,
            b"https://ads.tracker.example.net/consent.js",
            PAGE,
            SCRIPT
        ),
        allowed,
        "the exception among them"
    );
    // Other rules in their place: what the first blocked is allowed, and the new holds.
    set_rules(&mut shell, "||other.example.net^");
    assert_eq!(guard(shell.handle, show, PAGE, SCRIPT), allowed);
    assert_eq!(
        guard(
            shell.handle,
            b"https://other.example.net/a.js",
            PAGE,
            SCRIPT
        ),
        blocked
    );
    // Taken away: everything is allowed again.
    set_rules(&mut shell, "");
    assert_eq!(
        guard(
            shell.handle,
            b"https://other.example.net/a.js",
            PAGE,
            SCRIPT
        ),
        allowed
    );

    // Set, and the core stopped and started: they are in force with nothing asked.
    set_rules(&mut shell, RULES);
    shell.stop();
    let shell = TestShell::start(&config);
    assert_eq!(guard(shell.handle, show, PAGE, SCRIPT), blocked);
}

#[test]
fn guard_is_asked_from_many_threads_while_its_rules_change() {
    let mut shell = with_rules(RULES);
    let handle = shell.handle;
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let stop = std::sync::Arc::clone(&stop);
            std::thread::spawn(move || {
                // Each answer is the old rules' or the new: never an error, and never
                // anything else.
                let (mut blocked, mut allowed, mut wrong) = (0_u32, 0_u32, 0_u32);
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    match guard(
                        handle,
                        b"https://ads.tracker.example.net/show.js",
                        PAGE,
                        SCRIPT,
                    ) {
                        (0, WT_VERDICT_BLOCK) => blocked += 1,
                        (0, WT_VERDICT_ALLOW) => allowed += 1,
                        _ => wrong += 1,
                    }
                    // What no rule ever names is allowed under both.
                    if guard(handle, b"https://cdn.example.org/app.js", PAGE, SCRIPT)
                        != (0, WT_VERDICT_ALLOW)
                    {
                        wrong += 1;
                    }
                }
                (blocked, allowed, wrong)
            })
        })
        .collect();
    for _ in 0..150 {
        set_rules(&mut shell, "");
        set_rules(&mut shell, RULES);
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let (mut blocked, mut allowed) = (0, 0);
    for thread in threads {
        let (was_blocked, was_allowed, wrong) = thread.join().unwrap();
        assert_eq!(wrong, 0);
        blocked += was_blocked;
        allowed += was_allowed;
    }
    assert!(
        blocked > 0 && allowed > 0,
        "both rules were seen: {blocked} blocked, {allowed} allowed"
    );
    // The last rules set are the ones in force.
    assert_eq!(
        guard(
            handle,
            b"https://ads.tracker.example.net/show.js",
            PAGE,
            SCRIPT
        ),
        (0, WT_VERDICT_BLOCK)
    );
}

/// Guard's answer for a request a page makes in a web view, through the C
/// function: the status, and the verdict written.
fn guard_in(handle: u64, web_view: u64, url: &[u8], source: &[u8]) -> (i32, u32) {
    let request = WtRequest {
        url: url.as_ptr(),
        url_len: url.len(),
        source_url: source.as_ptr(),
        source_url_len: source.len(),
        web_view_id: web_view,
        resource_type: SCRIPT,
    };
    let mut verdict = WtVerdict {
        action: WT_VERDICT_ALLOW,
    };
    // SAFETY: both structures are live, and each address covers its length.
    let status = unsafe { wt_guard_check(handle, &raw const request, &raw mut verdict) };
    (status, verdict.action)
}

/// Sets one of Guard's switches (067), through the contract.
fn set_guard(
    shell: &mut TestShell,
    of: wt_proto::v1::set_guard::Of,
    switch: wt_proto::v1::GuardSwitch,
) {
    let answer = shell.request(request::Kind::SetGuard(wt_proto::v1::SetGuard {
        of: Some(of),
        switch: switch.into(),
    }));
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

/// A core, a window, and a tab whose page of the news site has arrived in a
/// web view: the tab, its web view, and the Space the window shows.
fn a_page_in_a_web_view(shell: &mut TestShell) -> (u64, u64, u64) {
    let response::Kind::WindowOpened(opened) =
        shell.request(request::Kind::OpenWindow(OpenWindow::default()))
    else {
        panic!("OpenWindow answers WindowOpened");
    };
    let response::Kind::Subscribed(_) = shell.request(request::Kind::Subscribe(Subscribe {
        topic: Some(subscribe::Topic::Window(WindowTopic {
            window_id: opened.window_id,
        })),
    })) else {
        panic!("Subscribe answers Subscribed");
    };
    let answer = shell.request(request::Kind::SubmitAddress(SubmitAddress {
        tab_id: opened.tab_id,
        text: "https://news.example.com/story".to_owned(),
    }));
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
    let web_view = shell
        .host_requests
        .iter()
        .find_map(|request| match request {
            host_request::Kind::WebViewCreate(create) => Some(create.web_view_id),
            _ => None,
        })
        .expect("a web view was asked for");
    for kind in [
        engine_event::Kind::NavigationCommitted(NavigationCommitted {
            url: "https://news.example.com/story".to_owned(),
        }),
        engine_event::Kind::NavigationFinished(NavigationFinished {}),
    ] {
        shell.report(EngineEvent {
            web_view_id: web_view,
            kind: Some(kind),
        });
    }
    // The window is told its page has arrived, and which Space it shows.
    let space = loop {
        let event::Kind::Snapshot(snapshot) = shell.next_event() else {
            continue;
        };
        let Some(snapshot::Model::Window(window)) = snapshot.model else {
            continue;
        };
        if window
            .tab
            .is_some_and(|tab| tab.load_state() == LoadState::Idle && tab.address.is_some())
        {
            break window.space_id;
        }
    };
    (opened.tab_id, web_view, space)
}

#[test]
fn guards_switches_are_heeded_from_the_next_request_and_the_nearest_wins() {
    use wt_proto::v1::GuardSwitch::{Off, On, Unspecified};
    use wt_proto::v1::set_guard::Of;
    let mut shell = with_rules(RULES);
    let (tab, web_view, space) = a_page_in_a_web_view(&mut shell);
    let handle = shell.handle;
    let show = b"https://ads.tracker.example.net/show.js";
    let other_page = b"https://www.example.org/";
    let blocked = (0, WT_VERDICT_BLOCK);
    let allowed = (0, WT_VERDICT_ALLOW);
    let reloads = |shell: &TestShell| {
        shell
            .host_requests
            .iter()
            .filter(|request| matches!(request, host_request::Kind::WebViewCommand(_)))
            .count()
    };

    // Guard is on: by the rules.
    assert_eq!(guard_in(handle, web_view, show, PAGE), blocked);
    // Off everywhere: the very next request is allowed, in this web view and in one
    // that is not known. On again: refused again.
    set_guard(&mut shell, Of::Everywhere(true), Off);
    assert_eq!(guard_in(handle, web_view, show, PAGE), allowed);
    assert_eq!(guard_in(handle, web_view + 500, show, PAGE), allowed);
    set_guard(&mut shell, Of::Everywhere(true), On);
    assert_eq!(guard_in(handle, web_view, show, PAGE), blocked);
    // Off for the Space: this web view's requests are allowed, whatever page asks; a
    // web view that is not known is under everywhere's switch, which is on.
    set_guard(&mut shell, Of::SpaceId(space), Off);
    assert_eq!(guard_in(handle, web_view, show, PAGE), allowed);
    assert_eq!(guard_in(handle, web_view, show, other_page), allowed);
    assert_eq!(guard_in(handle, web_view + 500, show, PAGE), blocked);
    assert_eq!(reloads(&shell), 0, "a Space's switch reloads nothing");
    // On for the site, in a Space that is off: the site's pages are under Guard, and
    // another site's in the same web view are not. The tab was reloaded.
    set_guard(&mut shell, Of::TabId(tab), On);
    assert_eq!(guard_in(handle, web_view, show, PAGE), blocked);
    assert_eq!(
        guard_in(handle, web_view, show, b"https://shop.example.com/basket"),
        blocked
    );
    assert_eq!(guard_in(handle, web_view, show, other_page), allowed);
    assert_eq!(reloads(&shell), 1);
    // The Space says nothing again: everywhere's, which is on, for the other site too.
    set_guard(&mut shell, Of::SpaceId(space), Unspecified);
    assert_eq!(guard_in(handle, web_view, show, other_page), blocked);
    // Off for the site alone.
    set_guard(&mut shell, Of::TabId(tab), Off);
    assert_eq!(guard_in(handle, web_view, show, PAGE), allowed);
    assert_eq!(guard_in(handle, web_view, show, other_page), blocked);
    assert_eq!(reloads(&shell), 2);
    // What no rule names is allowed under any switch.
    assert_eq!(
        guard_in(
            handle,
            web_view,
            b"https://cdn.example.org/app.js",
            other_page
        ),
        allowed
    );
}

#[test]
fn guards_switches_are_in_force_after_a_start_with_nothing_asked() {
    use wt_proto::v1::GuardSwitch::Off;
    use wt_proto::v1::set_guard::Of;
    let folder = tempfile::tempdir().unwrap();
    let mut config = fake_config();
    config.test_mode = wt_proto::v1::TestMode::Live as i32;
    config.data_dir = folder.path().to_str().unwrap().to_owned();
    let show = b"https://ads.tracker.example.net/show.js";

    let mut shell = TestShell::start(&config);
    set_rules(&mut shell, RULES);
    assert_eq!(guard_in(shell.handle, 1, show, PAGE), (0, WT_VERDICT_BLOCK));
    set_guard(&mut shell, Of::Everywhere(true), Off);
    assert_eq!(guard_in(shell.handle, 1, show, PAGE), (0, WT_VERDICT_ALLOW));
    shell.stop();
    // Started again: off, before anything is asked of the core.
    let shell = TestShell::start(&config);
    assert_eq!(guard_in(shell.handle, 1, show, PAGE), (0, WT_VERDICT_ALLOW));
}

#[test]
fn guard_is_asked_from_many_threads_while_a_switch_is_turned() {
    use wt_proto::v1::GuardSwitch::{Off, On};
    use wt_proto::v1::set_guard::Of;
    let mut shell = with_rules(RULES);
    let (_, web_view, _) = a_page_in_a_web_view(&mut shell);
    let handle = shell.handle;
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let stop = std::sync::Arc::clone(&stop);
            std::thread::spawn(move || {
                let (mut blocked, mut allowed, mut wrong) = (0_u32, 0_u32, 0_u32);
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    match guard_in(
                        handle,
                        web_view,
                        b"https://ads.tracker.example.net/show.js",
                        PAGE,
                    ) {
                        (0, WT_VERDICT_BLOCK) => blocked += 1,
                        (0, WT_VERDICT_ALLOW) => allowed += 1,
                        _ => wrong += 1,
                    }
                }
                (blocked, allowed, wrong)
            })
        })
        .collect();
    for _ in 0..150 {
        set_guard(&mut shell, Of::Everywhere(true), Off);
        set_guard(&mut shell, Of::Everywhere(true), On);
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let (mut blocked, mut allowed) = (0, 0);
    for thread in threads {
        let (was_blocked, was_allowed, wrong) = thread.join().unwrap();
        assert_eq!(wrong, 0);
        blocked += was_blocked;
        allowed += was_allowed;
    }
    assert!(
        blocked > 0 && allowed > 0,
        "{blocked} blocked, {allowed} allowed"
    );
    assert_eq!(
        guard_in(
            handle,
            web_view,
            b"https://ads.tracker.example.net/show.js",
            PAGE
        ),
        (0, WT_VERDICT_BLOCK),
        "the last switch set is the one in force"
    );
}

/// Whether a frame has its site's storage, through the C function: the
/// status, and the verdict written. The verdict starts as OWN, so that what
/// a failure leaves there is seen to have been written.
fn storage(
    handle: u64,
    web_view: u64,
    origin: &[u8],
    top_origin: &[u8],
    page: &[u8],
) -> (i32, u32) {
    let frame = WtFrame {
        origin: origin.as_ptr(),
        origin_len: origin.len(),
        top_origin: top_origin.as_ptr(),
        top_origin_len: top_origin.len(),
        page_url: page.as_ptr(),
        page_url_len: page.len(),
        web_view_id: web_view,
    };
    let mut verdict = WtStorageVerdict {
        action: WT_STORAGE_OWN,
    };
    // SAFETY: both structures are live, and each text covers its length.
    let status = unsafe { wt_storage_check(handle, &raw const frame, &raw mut verdict) };
    (status, verdict.action)
}

const TOP: &[u8] = b"https://news.example.com";
const TRACKER: &[u8] = b"https://ads.tracker.example.net";

#[test]
fn a_frame_of_another_site_is_kept_from_its_storage() {
    let mut shell = TestShell::fake();
    let (_, web_view, _) = a_page_in_a_web_view(&mut shell);
    let handle = shell.handle;
    let apart = (0, WT_STORAGE_APART);
    let own = (0, WT_STORAGE_OWN);
    // Of another site than the page's: apart. Of the page's own, by any host: not.
    assert_eq!(storage(handle, web_view, TRACKER, TOP, PAGE), apart);
    assert_eq!(storage(handle, web_view, b"null", TOP, PAGE), apart);
    assert_eq!(storage(handle, web_view, TOP, TOP, PAGE), own);
    assert_eq!(
        storage(handle, web_view, b"https://comments.example.com", TOP, PAGE),
        own
    );
    // A frame whose top frame is not of the page the shell knows: apart, though the
    // frame is of that page's site.
    assert_eq!(
        storage(handle, web_view, TOP, b"https://www.example.org", PAGE),
        apart
    );
    assert_eq!(storage(handle, web_view, TOP, TOP, b"about:blank"), apart);
    // A web view that is not known is under everywhere's switch, which is on.
    assert_eq!(storage(handle, web_view + 500, TRACKER, TOP, PAGE), apart);
    assert_eq!(storage(handle, web_view + 500, TOP, TOP, PAGE), own);
}

#[test]
fn the_storage_question_fails_apart() {
    let mut shell = TestShell::fake();
    let handle = shell.handle;
    // Of the page's own site: only the failure can have written APART.
    let frame = |origin: *const u8, origin_len: usize| WtFrame {
        origin,
        origin_len,
        top_origin: TOP.as_ptr(),
        top_origin_len: TOP.len(),
        page_url: PAGE.as_ptr(),
        page_url_len: PAGE.len(),
        web_view_id: 1,
    };
    let ask = |handle: u64, frame: *const WtFrame| {
        let mut verdict = WtStorageVerdict {
            action: WT_STORAGE_OWN,
        };
        // SAFETY: the verdict is live; the frame is NULL or live, and its texts cover
        // their lengths or are NULL, which is refused before anything is read.
        let status = unsafe { wt_storage_check(handle, frame, &raw mut verdict) };
        (status, verdict.action)
    };
    assert_eq!(storage(handle, 1, TOP, TOP, PAGE), (0, WT_STORAGE_OWN));
    // No frame.
    assert_eq!(
        ask(handle, std::ptr::null()),
        (Status::NullArgument as i32, WT_STORAGE_APART)
    );
    // An origin that is not text, and one that is nowhere but has a length.
    let not_text = [0xff_u8, 0xfe, 0xfd];
    let of_no_text = frame(not_text.as_ptr(), not_text.len());
    assert_eq!(
        ask(handle, &raw const of_no_text),
        (Status::BadArgument as i32, WT_STORAGE_APART)
    );
    let of_nowhere = frame(std::ptr::null(), 4);
    assert_eq!(
        ask(handle, &raw const of_nowhere),
        (Status::BadArgument as i32, WT_STORAGE_APART)
    );
    // No core of that handle, and none at all.
    assert_eq!(
        storage(handle + 1000, 1, TOP, TOP, PAGE),
        (Status::UnknownCore as i32, WT_STORAGE_APART)
    );
    assert_eq!(
        storage(0, 1, TOP, TOP, PAGE),
        (Status::UnknownCore as i32, WT_STORAGE_APART)
    );
    // Nowhere to write the verdict.
    let whole = frame(TOP.as_ptr(), TOP.len());
    // SAFETY: the frame is live and its texts cover their lengths; a NULL verdict is
    // refused before anything is written.
    let status = unsafe { wt_storage_check(handle, &raw const whole, std::ptr::null_mut()) };
    assert_eq!(status, Status::NullArgument as i32);
    // A core that has stopped.
    shell.stop();
    assert_eq!(
        storage(handle, 1, TOP, TOP, PAGE),
        (Status::UnknownCore as i32, WT_STORAGE_APART)
    );
}

#[test]
fn where_guard_is_off_for_the_page_a_frame_has_its_storage_from_the_next_question() {
    use wt_proto::v1::GuardSwitch::{Off, On, Unspecified};
    use wt_proto::v1::set_guard::Of;
    let mut shell = TestShell::fake();
    let (tab, web_view, space) = a_page_in_a_web_view(&mut shell);
    let handle = shell.handle;
    let apart = (0, WT_STORAGE_APART);
    let own = (0, WT_STORAGE_OWN);
    let other_top: &[u8] = b"https://www.example.org";
    let other_page: &[u8] = b"https://www.example.org/";
    let tracker = |web_view: u64| storage(handle, web_view, TRACKER, TOP, PAGE);
    let elsewhere = || storage(handle, web_view, TRACKER, other_top, other_page);

    assert_eq!(tracker(web_view), apart);
    // Off everywhere: the very next frame has its storage, in this web view and in
    // one that is not known. On again: apart again.
    set_guard(&mut shell, Of::Everywhere(true), Off);
    assert_eq!(tracker(web_view), own);
    assert_eq!(tracker(web_view + 500), own);
    // A frame in a page the shell does not know is apart whatever the switches.
    assert_eq!(storage(handle, web_view, TRACKER, other_top, PAGE), apart);
    set_guard(&mut shell, Of::Everywhere(true), On);
    assert_eq!(tracker(web_view), apart);
    // Off for the Space: whatever page the web view shows; a web view that is not
    // known is under everywhere's switch, which is on.
    set_guard(&mut shell, Of::SpaceId(space), Off);
    assert_eq!(tracker(web_view), own);
    assert_eq!(elsewhere(), own);
    assert_eq!(tracker(web_view + 500), apart);
    // On for the site, in a Space that is off: in that site's pages a frame of
    // another site is apart, and in another site's pages it is not.
    set_guard(&mut shell, Of::TabId(tab), On);
    assert_eq!(tracker(web_view), apart);
    assert_eq!(elsewhere(), own);
    // The Space says nothing again: everywhere's, which is on.
    set_guard(&mut shell, Of::SpaceId(space), Unspecified);
    assert_eq!(elsewhere(), apart);
    // Off for the site alone. And a frame of the page's own site has its storage
    // under any switch.
    set_guard(&mut shell, Of::TabId(tab), Off);
    assert_eq!(tracker(web_view), own);
    assert_eq!(elsewhere(), apart);
    assert_eq!(
        storage(handle, web_view, other_top, other_top, other_page),
        own
    );
}

#[test]
fn the_storage_question_is_asked_from_many_threads_while_a_switch_is_turned() {
    use wt_proto::v1::GuardSwitch::{Off, On};
    use wt_proto::v1::set_guard::Of;
    let mut shell = TestShell::fake();
    let (_, web_view, _) = a_page_in_a_web_view(&mut shell);
    let handle = shell.handle;
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let stop = std::sync::Arc::clone(&stop);
            std::thread::spawn(move || {
                let (mut apart, mut own, mut wrong) = (0_u32, 0_u32, 0_u32);
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    match storage(handle, web_view, TRACKER, TOP, PAGE) {
                        (0, WT_STORAGE_APART) => apart += 1,
                        (0, WT_STORAGE_OWN) => own += 1,
                        _ => wrong += 1,
                    }
                }
                (apart, own, wrong)
            })
        })
        .collect();
    for _ in 0..150 {
        set_guard(&mut shell, Of::Everywhere(true), Off);
        set_guard(&mut shell, Of::Everywhere(true), On);
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let (mut apart, mut own) = (0, 0);
    for thread in threads {
        let (was_apart, was_own, wrong) = thread.join().unwrap();
        assert_eq!(wrong, 0);
        apart += was_apart;
        own += was_own;
    }
    assert!(apart > 0 && own > 0, "{apart} apart, {own} own");
    assert_eq!(
        storage(handle, web_view, TRACKER, TOP, PAGE),
        (0, WT_STORAGE_APART),
        "the last switch set is the one in force"
    );
}

#[test]
fn several_cores_run_side_by_side() {
    let mut shells: Vec<TestShell> = (0..6).map(|_| TestShell::fake()).collect();
    for (index, shell) in shells.iter_mut().enumerate() {
        let payload = [u8::try_from(index).unwrap()];
        let response::Kind::Pong(pong) = shell.request(ping(&payload, false)) else {
            panic!("Ping answers Pong");
        };
        assert_eq!(pong.payload, payload);
        assert_eq!(
            nav(shell.handle, b"https://example.com/", 0),
            (0, WT_NAV_ALLOW)
        );
    }
    let handles: std::collections::BTreeSet<u64> =
        shells.iter().map(|shell| shell.handle).collect();
    assert_eq!(handles.len(), 6, "every core has its own handle");
}

#[test]
fn live_mode_writes_its_store_in_the_data_folder() {
    let folder = tempfile::tempdir().unwrap();
    let mut config = fake_config();
    config.test_mode = wt_proto::v1::TestMode::Live as i32;
    config.data_dir = folder
        .path()
        .join("WebTiger")
        .to_string_lossy()
        .into_owned();
    let mut shell = TestShell::start(&config);
    assert_ne!(shell.handle, 0);
    assert!(matches!(shell.next_event(), event::Kind::Status(_)));
    shell.stop();
    assert!(
        folder
            .path()
            .join("WebTiger")
            .join("webtiger.sqlite")
            .is_file()
    );
}
