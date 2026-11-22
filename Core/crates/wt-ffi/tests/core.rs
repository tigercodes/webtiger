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
    WT_VERDICT_ALLOW, WT_VERDICT_BLOCK, WtNavVerdict, WtNavigation, WtRequest, WtVerdict,
    wt_core_send, wt_core_start, wt_core_stop, wt_guard_check, wt_nav_check,
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
fn guard_allows_everything_until_it_exists_and_fails_open() {
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
