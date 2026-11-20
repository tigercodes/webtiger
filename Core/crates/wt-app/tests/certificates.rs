//! Going on past a certificate that is not trusted (026), through the
//! contract, against the scripted engine: what may be gone on with, what
//! going on does, what the exception covers, and how the page is marked.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    CreateSpace, EngineEvent, LoadState, NavigationFailed, NavigationStarted,
    ProceedPastCertificate, SwitchSpace, TabView, address_view::Security, engine_event,
    host_request, request, response,
};
use wt_testkit::{FakeShell, run};

const ROUTER: &str = "https://router.example.com/";
const STATUS: &str = "https://router.example.com/status";
const OTHER: &str = "https://example.org/";

/// A shell whose web has a router with a certificate nobody signed, and a
/// site the system trusts.
fn shell() -> FakeShell {
    let mut shell = FakeShell::fake();
    shell
        .engine
        .untrusted(ROUTER, "Router", "net.tls.untrusted");
    shell
        .engine
        .untrusted(STATUS, "Status", "net.tls.untrusted");
    shell.engine.page(OTHER, "Another Place");
    shell
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the mark beside the address says of the tab in front.
fn mark(shell: &FakeShell, subscription: u64) -> Security {
    front(shell, subscription).address.unwrap().security()
}

async fn go_on(shell: &mut FakeShell, tab: u64) -> response::Kind {
    shell
        .request(request::Kind::ProceedPastCertificate(
            ProceedPastCertificate { tab_id: tab },
        ))
        .await
}

/// A failure of a web view's load, as a shell would report it.
fn failed(web_view: u64, code: &str, url: &str, held: bool) -> EngineEvent {
    EngineEvent {
        web_view_id: web_view,
        kind: Some(engine_event::Kind::NavigationFailed(NavigationFailed {
            code: code.to_owned(),
            url: url.to_owned(),
            refused: false,
            certificate_held: held,
        })),
    }
}

/// What the shell was asked about certificates and loads, in order.
fn asked(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewTrustCertificate(trust) => {
                Some(format!("trust {}", trust.web_view_id))
            }
            host_request::Kind::WebViewLoad(load) => {
                Some(format!("load {} {}", load.web_view_id, load.url))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_load_that_fails_for_a_certificate_the_adapter_holds_may_be_gone_on_with() {
    run(async {
        for code in ["net.tls.untrusted", "net.tls.date", "net.tls.name"] {
            let mut shell = FakeShell::fake();
            shell.engine.untrusted(ROUTER, "Router", code);
            let (_, tab, subscription) = shell.open_window().await;

            shell.submit(tab, "https://router.example.com").await;

            let view = front(&shell, subscription);
            assert_eq!(view.load_state(), LoadState::Failed, "{code}");
            assert_eq!(view.error_code, code);
            assert!(view.certificate_passable, "{code}");
            assert_eq!(view.certificate_host, "router.example.com");
            // Nothing was let through by saying so.
            assert!(shell.engine.trust_asked.is_empty());
        }
    });
}

/// The address field never lies: a load that failed left no page, and the
/// mark beside the address it failed at says nothing of one. Beside a
/// certificate that was just refused, "secure" would be the lie.
#[test]
fn the_mark_says_nothing_of_a_load_that_failed() {
    run(async {
        let mut shell = shell();
        let (_, tab, subscription) = shell.open_window().await;

        // A certificate that is not trusted: the address that failed is
        // shown, as before, and nothing is said of how safe it is.
        shell.submit(tab, ROUTER).await;
        let view = front(&shell, subscription);
        assert_eq!(view.load_state(), LoadState::Failed);
        assert_eq!(view.address.as_ref().unwrap().url, ROUTER);
        assert_eq!(mark(&shell, subscription), Security::Unspecified);

        // Any failure: a site that is not there, over https and over http.
        shell.submit(tab, "https://nowhere.example.com").await;
        assert_eq!(front(&shell, subscription).error_code, "net.dns");
        assert_eq!(mark(&shell, subscription), Security::Unspecified);
        shell.submit(tab, "http://nowhere.example.com").await;
        assert_eq!(front(&shell, subscription).error_code, "net.dns");
        assert_eq!(mark(&shell, subscription), Security::Unspecified);

        // A page that loads is marked as it always was.
        shell.submit(tab, OTHER).await;
        assert_eq!(mark(&shell, subscription), Security::Secure);

        // Where a person went on past a certificate, a later failure there
        // is still marked as not secure: that much is known of it.
        shell.submit(tab, ROUTER).await;
        go_on(&mut shell, tab).await;
        assert_eq!(mark(&shell, subscription), Security::Insecure);
        let web_view = front(&shell, subscription).web_view_id;
        shell.report(failed(web_view, "net.connect", STATUS, false));
        shell.settle().await;
        assert_eq!(front(&shell, subscription).load_state(), LoadState::Failed);
        assert_eq!(mark(&shell, subscription), Security::Insecure);
    });
}

#[test]
fn any_other_failure_is_as_it_was_and_going_on_is_refused() {
    run(async {
        let mut shell = shell();
        let (_, tab, subscription) = shell.open_window().await;
        // A tab with no page, and no tab at all.
        assert_eq!(
            error_code(go_on(&mut shell, tab).await),
            "certificate.not_passable"
        );
        assert_eq!(
            error_code(go_on(&mut shell, tab + 999).await),
            "tab.unknown"
        );

        // A site that is not there.
        shell.submit(tab, "https://nowhere.example.com").await;
        let view = front(&shell, subscription);
        assert_eq!(view.error_code, "net.dns");
        assert!(!view.certificate_passable);
        assert_eq!(view.certificate_host, "");
        assert_eq!(
            error_code(go_on(&mut shell, tab).await),
            "certificate.not_passable"
        );

        // A page that loaded.
        shell.submit(tab, "https://example.org").await;
        assert!(!front(&shell, subscription).certificate_passable);
        let web_view = front(&shell, subscription).web_view_id;
        let not_passable = [
            // No certificate was the trouble: there is nothing to accept.
            ("net.tls", ROUTER, true),
            // The adapter holds nothing it could let through.
            ("net.tls.untrusted", ROUTER, false),
            // Not a page over https.
            ("net.tls.untrusted", "http://router.example.com/", true),
            ("net.tls.untrusted", "wss://router.example.com/", true),
            ("net.tls.untrusted", "not an address", true),
            // Another failure that says a certificate is held.
            ("net.connect", ROUTER, true),
        ];
        for (code, url, held) in not_passable {
            shell.report(failed(web_view, code, url, held));
            shell.settle().await;
            let view = front(&shell, subscription);
            assert_eq!(view.load_state(), LoadState::Failed, "{code} {url}");
            assert!(!view.certificate_passable, "{code} {url} {held}");
            assert_eq!(
                error_code(go_on(&mut shell, tab).await),
                "certificate.not_passable",
                "{code} {url} {held}"
            );
        }
        // Nothing was asked of the shell about any certificate.
        assert!(shell.engine.trust_asked.is_empty());
    });
}

#[test]
fn going_on_lets_the_certificate_through_then_loads_and_the_page_is_never_shown_as_secure() {
    run(async {
        let mut shell = shell();
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "https://router.example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        let before = asked(&shell).len();

        let answer = go_on(&mut shell, tab).await;

        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        // The certificate is let through before the load is asked for.
        assert_eq!(
            asked(&shell)[before..],
            [
                format!("trust {web_view}"),
                format!("load {web_view} {ROUTER}")
            ]
        );
        let view = front(&shell, subscription);
        assert_eq!(view.load_state(), LoadState::Idle);
        assert_eq!(view.title, "Router");
        assert_eq!(view.address.as_ref().unwrap().url, ROUTER);
        assert!(
            !view.certificate_passable,
            "there is nothing more to go on past"
        );
        assert_eq!(mark(&shell, subscription), Security::Insecure);

        // Every page of that host is marked so, however it is reached; and a
        // site the system trusts is as it was.
        shell.submit(tab, STATUS).await;
        assert_eq!(front(&shell, subscription).title, "Status");
        assert_eq!(mark(&shell, subscription), Security::Insecure);
        shell.submit(tab, OTHER).await;
        assert_eq!(mark(&shell, subscription), Security::Secure);

        // And for as long as the exception lasts: also once the certificate
        // has been put right.
        shell.engine.page(ROUTER, "Router");
        shell.submit(tab, ROUTER).await;
        assert_eq!(front(&shell, subscription).title, "Router");
        assert_eq!(mark(&shell, subscription), Security::Insecure);
        // It was let through once, and never asked for again.
        assert_eq!(shell.engine.trust_asked, [web_view]);
    });
}

#[test]
fn the_exception_is_one_login_sets_and_spaces_that_share_logins_share_it() {
    run(async {
        let mut shell = shell();
        let (window, tab, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        shell.submit(tab, ROUTER).await;
        go_on(&mut shell, tab).await;
        assert_eq!(mark(&shell, subscription), Security::Insecure);

        // Another tab of the same Space: let through already, and marked.
        shell.open_tab(window, ROUTER).await;
        assert_eq!(front(&shell, subscription).title, "Router");
        assert_eq!(mark(&shell, subscription), Security::Insecure);

        let make = |name: &str, share: u64| {
            request::Kind::CreateSpace(CreateSpace {
                name: name.to_owned(),
                share_logins_with: share,
                colour: String::new(),
            })
        };
        let response::Kind::SpaceCreated(work) = shell.request(make("Work", 0)).await else {
            panic!("a Space is made");
        };
        let response::Kind::SpaceCreated(also) = shell.request(make("Also home", home)).await
        else {
            panic!("a Space is made");
        };
        let switch = |space: u64| {
            request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: space,
            })
        };

        // A Space with logins of its own is asked for itself: nothing was let
        // through for it.
        shell.request(switch(work.space_id)).await;
        let work_tab = front(&shell, subscription).tab_id;
        shell.submit(work_tab, ROUTER).await;
        let view = front(&shell, subscription);
        assert_eq!(view.load_state(), LoadState::Failed);
        assert!(view.certificate_passable);

        // And the first Space's exception is not its: once the certificate
        // is one the system trusts, the page is secure here, and still
        // marked where somebody went on past it.
        shell.engine.page(ROUTER, "Router");
        shell.submit(work_tab, ROUTER).await;
        assert_eq!(front(&shell, subscription).title, "Router");
        assert_eq!(mark(&shell, subscription), Security::Secure);
        shell.request(switch(home)).await;
        assert_eq!(mark(&shell, subscription), Security::Insecure);

        // A Space that shares the first one's logins shares its exception.
        shell.request(switch(also.space_id)).await;
        let shared_tab = front(&shell, subscription).tab_id;
        shell.submit(shared_tab, ROUTER).await;
        assert_eq!(front(&shell, subscription).title, "Router");
        assert_eq!(mark(&shell, subscription), Security::Insecure);
        assert_eq!(
            shell.engine.trust_asked.len(),
            1,
            "let through once, by one login set"
        );
    });
}

#[test]
fn another_certificate_at_the_same_host_is_refused_again() {
    run(async {
        let mut shell = shell();
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, ROUTER).await;
        go_on(&mut shell, tab).await;
        assert_eq!(front(&shell, subscription).title, "Router");

        // The server has another certificate: what was let through was the
        // one that was refused, and no other.
        shell.engine.certificate_changes(ROUTER);
        shell.submit(tab, ROUTER).await;
        let view = front(&shell, subscription);
        assert_eq!(view.load_state(), LoadState::Failed);
        assert_eq!(view.error_code, "net.tls.untrusted");
        assert!(view.certificate_passable, "it may be gone on with, again");

        assert!(matches!(
            go_on(&mut shell, tab).await,
            response::Kind::Ack(_)
        ));
        assert_eq!(front(&shell, subscription).title, "Router");
        assert_eq!(shell.engine.trust_asked.len(), 2);
    });
}

#[test]
fn an_adapter_that_no_longer_holds_the_certificate_makes_no_exception() {
    run(async {
        let mut shell = shell();
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, ROUTER).await;
        assert!(front(&shell, subscription).certificate_passable);
        shell.engine.forgets_what_it_refused();

        let answer = go_on(&mut shell, tab).await;

        // The shell's own word for why, and the load failed as it did.
        assert_eq!(error_code(answer), "certificate.unknown");
        let view = front(&shell, subscription);
        assert_eq!(view.load_state(), LoadState::Failed);
        assert_eq!(view.error_code, "net.tls.untrusted");
        // There is no exception: once the certificate is one the system
        // trusts, the page is as secure as any.
        shell.engine.page(ROUTER, "Router");
        shell.submit(tab, ROUTER).await;
        assert_eq!(front(&shell, subscription).title, "Router");
        assert_eq!(mark(&shell, subscription), Security::Secure);
    });
}

#[test]
fn another_load_ends_what_may_be_gone_on_with() {
    run(async {
        let mut shell = shell();
        let (_, tab, subscription) = shell.open_window().await;

        // The person goes somewhere else.
        shell.submit(tab, ROUTER).await;
        assert!(front(&shell, subscription).certificate_passable);
        shell.submit(tab, OTHER).await;
        assert!(!front(&shell, subscription).certificate_passable);
        assert_eq!(
            error_code(go_on(&mut shell, tab).await),
            "certificate.not_passable"
        );

        // The page that is still there starts a load of its own.
        shell.submit(tab, ROUTER).await;
        assert!(front(&shell, subscription).certificate_passable);
        let web_view = front(&shell, subscription).web_view_id;
        shell.report(EngineEvent {
            web_view_id: web_view,
            kind: Some(engine_event::Kind::NavigationStarted(NavigationStarted {
                url: OTHER.to_owned(),
            })),
        });
        shell.settle().await;
        assert!(!front(&shell, subscription).certificate_passable);

        // A load that is stopped.
        shell.submit(tab, ROUTER).await;
        assert!(front(&shell, subscription).certificate_passable);
        shell.report(failed(web_view, "net.cancelled", ROUTER, true));
        shell.settle().await;
        assert!(!front(&shell, subscription).certificate_passable);
        assert!(shell.engine.trust_asked.is_empty());
    });
}

#[test]
fn the_exception_is_gone_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let web = |mut shell: FakeShell| {
            shell
                .engine
                .untrusted(ROUTER, "Router", "net.tls.untrusted");
            shell
        };
        let mut shell = web(FakeShell::live(folder.path()));
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, ROUTER).await;
        go_on(&mut shell, tab).await;
        assert_eq!(front(&shell, subscription).title, "Router");
        drop(shell);

        // The tab comes back and is loaded; nothing is let through for it.
        let mut shell = web(FakeShell::live(folder.path()));
        let (_, _, subscription) = shell.open_window().await;
        let view = front(&shell, subscription);
        assert_eq!(view.load_state(), LoadState::Failed);
        assert_eq!(view.error_code, "net.tls.untrusted");
        assert!(view.certificate_passable, "to be gone on with afresh");
        assert!(shell.engine.trust_asked.is_empty());
    });
}
