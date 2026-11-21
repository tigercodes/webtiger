//! The mark beside the address waits for the page (029), through the
//! contract, against the scripted engine: what the window was told of how
//! secure an address is, at each moment of a load.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    LoadState, NavigationCommand, ProceedPastCertificate, SubmitAddress, TabCommand,
    address_view::Security, request,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const SAFE: &str = "https://example.com/";
const PLAIN: &str = "http://example.org/";
const ROUTER: &str = "https://router.example.com/";
const STATUS: &str = "https://router.example.com/status";

/// How the tab in front stands: its load, the address in the field, and the
/// mark beside it.
fn now(shell: &FakeShell, subscription: u64) -> (LoadState, String, Security) {
    let tab = shell.window(subscription).tab.expect("a window has a tab");
    let address = tab.address.clone().expect("a tab has an address view");
    (tab.load_state(), address.url.clone(), address.security())
}

/// The person sends a tab somewhere, and the shell has not answered yet: how
/// the tab stands while the load is on its way. Then the shell answers, and
/// the load goes as the scripted engine says.
async fn on_the_way(
    shell: &mut FakeShell,
    subscription: u64,
    request: request::Kind,
) -> (LoadState, String, Security) {
    shell.hold_host = true;
    shell.send(request);
    shell.settle().await;
    let waiting = now(shell, subscription);
    shell.release_host();
    shell.settle().await;
    waiting
}

fn typed(tab: u64, text: &str) -> request::Kind {
    request::Kind::SubmitAddress(SubmitAddress {
        tab_id: tab,
        text: text.to_owned(),
    })
}

fn a_web() -> FakeShell {
    let mut shell = FakeShell::fake();
    shell.engine.page(SAFE, "Example");
    shell.engine.page(PLAIN, "Another Place");
    shell
        .engine
        .untrusted(ROUTER, "Router", "net.tls.untrusted");
    shell
        .engine
        .untrusted(STATUS, "Status", "net.tls.untrusted");
    shell
}

#[test]
fn an_address_a_person_typed_has_no_mark_until_its_page_arrives() {
    run(async {
        let mut shell = a_web();
        let (_, tab, subscription) = shell.open_window().await;

        let waiting = on_the_way(&mut shell, subscription, typed(tab, "example.com")).await;

        // The field said where the tab was sent from the start; the mark
        // said nothing until the page was the tab's, and then what it was.
        assert_eq!(
            waiting,
            (LoadState::Loading, SAFE.to_owned(), Security::Unspecified)
        );
        assert_eq!(
            now(&shell, subscription),
            (LoadState::Idle, SAFE.to_owned(), Security::Secure)
        );

        // The same for a page that is not secure: nothing is said early of
        // that either, though a page that is secure is still on the screen.
        let waiting = on_the_way(&mut shell, subscription, typed(tab, PLAIN)).await;
        assert_eq!(
            waiting,
            (LoadState::Loading, PLAIN.to_owned(), Security::Unspecified)
        );
        assert_eq!(
            now(&shell, subscription),
            (LoadState::Idle, PLAIN.to_owned(), Security::Insecure)
        );
    });
}

#[test]
fn a_page_keeps_its_mark_while_it_reloads_or_follows_a_link() {
    run(async {
        let mut shell = a_web();
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, SAFE).await;
        let web_view = shell.window(subscription).tab.unwrap().web_view_id;

        // Reloaded: the page that is there is still there, and still its own.
        let reload = request::Kind::TabCommand(TabCommand {
            tab_id: tab,
            command: NavigationCommand::Reload as i32,
        });
        let waiting = on_the_way(&mut shell, subscription, reload).await;
        assert_eq!(waiting.1, SAFE);
        assert_eq!(waiting.2, Security::Secure);
        assert_eq!(
            now(&shell, subscription),
            (LoadState::Idle, SAFE.to_owned(), Security::Secure)
        );

        // A link of the page's leads somewhere else: until that page
        // arrives the field and the mark are the page's that is there.
        shell.report(FakeEngine::navigation_started(web_view, PLAIN));
        shell.settle().await;
        assert_eq!(
            now(&shell, subscription),
            (LoadState::Loading, SAFE.to_owned(), Security::Secure)
        );
        shell.report(FakeEngine::navigation_committed(web_view, PLAIN));
        shell.settle().await;
        assert_eq!(
            now(&shell, subscription),
            (LoadState::Loading, PLAIN.to_owned(), Security::Insecure)
        );
    });
}

#[test]
fn a_load_that_is_stopped_gives_the_mark_back_to_the_page_that_is_there() {
    run(async {
        let mut shell = a_web();
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, SAFE).await;
        // An address whose load is stopped before anything arrives.
        let gone = "http://stopped.example.net/";
        shell.engine.fail(gone, "net.cancelled");

        let waiting = on_the_way(&mut shell, subscription, typed(tab, gone)).await;

        // On its way: where the person sent the tab, and no mark.
        assert_eq!(
            waiting,
            (LoadState::Loading, gone.to_owned(), Security::Unspecified)
        );
        // Stopped: the page that is still there, with its own mark.
        assert_eq!(
            now(&shell, subscription),
            (LoadState::Idle, SAFE.to_owned(), Security::Secure)
        );
    });
}

#[test]
fn where_a_person_went_on_past_a_certificate_the_mark_is_not_secure_from_the_start() {
    run(async {
        let mut shell = a_web();
        let (_, tab, subscription) = shell.open_window().await;
        // Refused: no mark, on the way and when it failed.
        let waiting = on_the_way(&mut shell, subscription, typed(tab, ROUTER)).await;
        assert_eq!(
            waiting,
            (LoadState::Loading, ROUTER.to_owned(), Security::Unspecified)
        );
        assert_eq!(
            now(&shell, subscription),
            (LoadState::Failed, ROUTER.to_owned(), Security::Unspecified)
        );

        // Gone on past: not secure from the moment the tab is sent there,
        // which waits on nothing: it is what the person was warned of.
        let going_on =
            request::Kind::ProceedPastCertificate(ProceedPastCertificate { tab_id: tab });
        let waiting = on_the_way(&mut shell, subscription, going_on).await;
        assert_eq!(
            waiting,
            (LoadState::Loading, ROUTER.to_owned(), Security::Insecure)
        );
        assert_eq!(
            now(&shell, subscription),
            (LoadState::Idle, ROUTER.to_owned(), Security::Insecure)
        );

        // And another page of that site, typed afterwards, the same.
        let waiting = on_the_way(&mut shell, subscription, typed(tab, STATUS)).await;
        assert_eq!(
            waiting,
            (LoadState::Loading, STATUS.to_owned(), Security::Insecure)
        );
    });
}
