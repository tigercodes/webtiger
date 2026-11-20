//! A page asks to use the camera, the microphone or the location (014),
//! through the contract, against the scripted engine: what the core answers
//! by itself, what it asks the person, what it remembers, and what it tells
//! the shell.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::PermissionKind::{Camera, Location, Microphone};
use wt_proto::v1::{
    AnswerPrompt, EngineEvent, ForgetPermissions, PermissionKind, PermissionRequested,
    QueryPermissions, TabView, engine_event, forget_permissions, prompt_view, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const MEET: &str = "https://meet.example.com";

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the tab in front asks the person: the prompt's id, who it says
/// asks, and for what.
fn prompt(shell: &FakeShell, subscription: u64) -> Option<(u64, String, Vec<PermissionKind>)> {
    let prompt = front(shell, subscription).prompt?;
    let Some(prompt_view::Kind::Permission(permission)) = prompt.kind else {
        panic!("a prompt of some other kind: {prompt:?}");
    };
    let kinds = permission
        .kinds
        .iter()
        .map(|kind| PermissionKind::try_from(*kind).expect("a kind the contract has"))
        .collect();
    Some((prompt.prompt_id, permission.site, kinds))
}

/// The id of the prompt the tab in front shows.
fn shown(shell: &FakeShell, subscription: u64) -> u64 {
    prompt(shell, subscription)
        .expect("the tab asks something")
        .0
}

/// A web view's page asks, as the shell would report it.
async fn ask(shell: &mut FakeShell, web_view: u64, request: u64, kinds: &[PermissionKind]) {
    ask_as(shell, web_view, request, MEET, kinds).await;
}

async fn ask_as(
    shell: &mut FakeShell,
    web_view: u64,
    request: u64,
    origin: &str,
    kinds: &[PermissionKind],
) {
    shell.report(FakeEngine::permission_requested(
        web_view, request, origin, kinds,
    ));
    shell.settle().await;
}

async fn answer(shell: &mut FakeShell, tab: u64, prompt: u64, allow: bool) -> response::Kind {
    shell
        .request(request::Kind::AnswerPrompt(AnswerPrompt {
            tab_id: tab,
            prompt_id: prompt,
            allow,
        }))
        .await
}

/// Answers the prompt the tab in front shows, and expects to be told it was
/// done.
async fn answered(shell: &mut FakeShell, tab: u64, subscription: u64, allow: bool) {
    let prompt = shown(shell, subscription);
    let done = answer(shell, tab, prompt, allow).await;
    assert!(
        matches!(done, response::Kind::Ack(_)),
        "expected Ack, got {done:?}"
    );
}

/// How the shell was told to answer the pages, in order.
fn told(shell: &FakeShell) -> Vec<(u64, bool)> {
    shell.engine.permission_answers.clone()
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// What the sites of a window's login set were allowed: each as it is
/// shown, its origin, and what it may use.
async fn allowed(shell: &mut FakeShell, window: u64) -> Vec<(String, String, Vec<PermissionKind>)> {
    let answer = shell
        .request(request::Kind::QueryPermissions(QueryPermissions {
            window_id: window,
        }))
        .await;
    let response::Kind::PermissionsPage(page) = answer else {
        panic!("expected the permissions, got {answer:?}");
    };
    page.sites
        .into_iter()
        .map(|site| {
            let kinds = site
                .kinds
                .iter()
                .map(|kind| PermissionKind::try_from(*kind).expect("a kind the contract has"))
                .collect();
            (site.site, site.origin, kinds)
        })
        .collect()
}

async fn forget(
    shell: &mut FakeShell,
    window: u64,
    what: Option<forget_permissions::What>,
) -> response::Kind {
    shell
        .request(request::Kind::ForgetPermissions(ForgetPermissions {
            window_id: window,
            what,
        }))
        .await
}

/// The web a shell browses in these tests.
fn pages(shell: &mut FakeShell) {
    for (url, title) in [
        ("https://meet.example.com/", "Meet"),
        ("https://meet.example.com/room", "A room"),
        ("https://example.org/", "Another"),
        ("http://shop.example.com/", "A shop, not over https"),
        ("http://127.0.0.1:8080/", "This device"),
    ] {
        shell.engine.page(url, title);
    }
}

/// A window whose one tab shows meet.example.com. Its window, tab,
/// subscription and the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    pages(shell);
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "meet.example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

#[test]
fn a_page_that_asks_is_asked_about_in_its_tab_by_the_sites_name() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        assert_eq!(prompt(&shell, subscription), None);

        ask(&mut shell, web_view, 1, &[Camera]).await;

        let (id, site, kinds) = prompt(&shell, subscription).expect("the tab asks");
        assert_ne!(id, 0);
        assert_eq!(site, "meet.example.com", "as the address field names it");
        assert_eq!(kinds, [Camera]);
        assert_eq!(told(&shell), [], "the page waits for the person");
    });
}

#[test]
fn allow_grants_and_is_remembered_for_the_origin() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;

        answered(&mut shell, tab, subscription, true).await;

        assert_eq!(told(&shell), [(1, true)]);
        assert_eq!(prompt(&shell, subscription), None, "the prompt is gone");
        assert_eq!(
            allowed(&mut shell, window).await,
            [("meet.example.com".to_owned(), MEET.to_owned(), vec![Camera])]
        );

        // Asked for again, it is granted with nobody asked.
        ask(&mut shell, web_view, 2, &[Camera]).await;
        assert_eq!(told(&shell), [(1, true), (2, true)]);
        assert_eq!(prompt(&shell, subscription), None);

        // Another page of the same origin is the same origin.
        shell.submit(tab, "meet.example.com/room").await;
        ask(&mut shell, web_view, 3, &[Camera]).await;
        assert_eq!(told(&shell).last(), Some(&(3, true)));

        // Something else of the device's was not allowed by that.
        ask(&mut shell, web_view, 4, &[Microphone]).await;
        assert_eq!(told(&shell).len(), 3, "the page waits");
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Microphone])
        );
    });
}

#[test]
fn another_origin_was_never_asked() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;
        answered(&mut shell, tab, subscription, true).await;

        shell.submit(tab, "example.org").await;
        ask_as(&mut shell, web_view, 2, "https://example.org", &[Camera]).await;

        assert_eq!(told(&shell), [(1, true)], "nothing is granted to it");
        let (_, site, _) = prompt(&shell, subscription).expect("it is asked about");
        assert_eq!(site, "example.org");
    });
}

/// Starts the app in `folder`, with the web it browses.
fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    pages(&mut shell);
    shell
}

#[test]
fn what_was_allowed_is_still_allowed_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "meet.example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        ask(&mut shell, web_view, 1, &[Camera, Microphone]).await;
        answered(&mut shell, tab, subscription, true).await;
        // And something that was not allowed.
        ask(&mut shell, web_view, 2, &[Location]).await;
        answered(&mut shell, tab, subscription, false).await;
        drop(shell);

        let mut shell = start(folder.path());
        let (window, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "meet.example.com").await;
        let web_view = front(&shell, subscription).web_view_id;

        ask(&mut shell, web_view, 1, &[Microphone]).await;
        assert_eq!(told(&shell), [(1, true)], "granted, with nobody asked");
        // A no is for the page it was said to: it was not written down.
        ask(&mut shell, web_view, 2, &[Location]).await;
        assert_eq!(told(&shell), [(1, true)]);
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Location])
        );
        assert_eq!(
            allowed(&mut shell, window).await,
            [(
                "meet.example.com".to_owned(),
                MEET.to_owned(),
                vec![Camera, Microphone]
            )]
        );
    });
}

#[test]
fn dont_allow_refuses_and_the_page_does_not_get_to_ask_again() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;

        answered(&mut shell, tab, subscription, false).await;

        assert_eq!(told(&shell), [(1, false)]);
        assert_eq!(prompt(&shell, subscription), None);
        assert_eq!(allowed(&mut shell, window).await, [], "nothing is written");

        // The same again, and the same with something else: refused, with
        // nobody asked.
        ask(&mut shell, web_view, 2, &[Camera]).await;
        ask(&mut shell, web_view, 3, &[Microphone, Camera]).await;
        assert_eq!(told(&shell), [(1, false), (2, false), (3, false)]);
        assert_eq!(prompt(&shell, subscription), None);

        // What it was not refused, it may still ask for.
        ask(&mut shell, web_view, 4, &[Location]).await;
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Location])
        );
        answered(&mut shell, tab, subscription, false).await;

        // A page that only changes its own address is the same page.
        shell.report(FakeEngine::address_changed(
            web_view,
            "https://meet.example.com/#again",
        ));
        shell.settle().await;
        ask(&mut shell, web_view, 5, &[Camera]).await;
        assert_eq!(told(&shell).last(), Some(&(5, false)));

        // Another page has loaded: it may ask, and the person is asked.
        shell.submit(tab, "meet.example.com/room").await;
        ask(&mut shell, web_view, 6, &[Camera]).await;
        assert_eq!(told(&shell).len(), 5, "the page waits");
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Camera])
        );
    });
}

#[test]
fn who_asks_must_be_the_page_itself() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        // A frame from elsewhere, another of the site's names, the same
        // name over another scheme or port, and no origin at all.
        for (request, origin) in [
            (1, "https://ads.example.net"),
            (2, "https://example.com"),
            (3, "https://www.meet.example.com"),
            (4, "http://meet.example.com"),
            (5, "https://meet.example.com:8443"),
            (6, ""),
            (7, "about:blank"),
            (8, "meet.example.com"),
        ] {
            ask_as(&mut shell, web_view, request, origin, &[Camera]).await;
            assert_eq!(
                told(&shell).last(),
                Some(&(request, false)),
                "{origin:?} is refused"
            );
            assert_eq!(prompt(&shell, subscription), None, "{origin:?} asks nobody");
        }

        // The page's own origin, however the shell spells it.
        ask_as(
            &mut shell,
            web_view,
            9,
            "HTTPS://Meet.Example.com:443/",
            &[Camera],
        )
        .await;
        assert_eq!(told(&shell).len(), 8);
        assert!(prompt(&shell, subscription).is_some());
        answered(&mut shell, tab, subscription, true).await;

        // The tab has gone elsewhere: a request in the name of the page it
        // left is not the page's, though that origin was allowed.
        shell.submit(tab, "example.org").await;
        ask(&mut shell, web_view, 10, &[Camera]).await;
        assert_eq!(told(&shell).last(), Some(&(10, false)));
        assert_eq!(prompt(&shell, subscription), None);
    });
}

#[test]
fn a_page_that_did_not_come_over_https_is_refused_unless_it_is_this_device() {
    run(async {
        let mut shell = FakeShell::fake();
        pages(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "http://shop.example.com/").await;
        let web_view = front(&shell, subscription).web_view_id;

        ask_as(
            &mut shell,
            web_view,
            1,
            "http://shop.example.com",
            &[Camera],
        )
        .await;

        assert_eq!(told(&shell), [(1, false)]);
        assert_eq!(prompt(&shell, subscription), None);

        // This device itself: nothing between the page and the person.
        shell.submit(tab, "http://127.0.0.1:8080/").await;
        ask_as(&mut shell, web_view, 2, "http://127.0.0.1:8080", &[Camera]).await;
        assert_eq!(told(&shell), [(1, false)]);
        let (_, site, _) = prompt(&shell, subscription).expect("it is asked about");
        assert_eq!(
            site, "http://127.0.0.1:8080",
            "named with its scheme and port"
        );
    });
}

#[test]
fn a_web_view_that_is_no_tabs_and_a_tab_with_no_page_are_refused() {
    run(async {
        let mut shell = FakeShell::fake();
        pages(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;

        ask(&mut shell, 999, 1, &[Camera]).await;
        assert_eq!(told(&shell), [(1, false)]);

        // A tab whose load failed has a web view, and no page.
        shell.submit(tab, "nowhere.example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        assert_ne!(web_view, 0);
        ask_as(
            &mut shell,
            web_view,
            2,
            "https://nowhere.example.com",
            &[Camera],
        )
        .await;
        assert_eq!(told(&shell), [(1, false), (2, false)]);
        assert_eq!(prompt(&shell, subscription), None);
    });
}

#[test]
fn what_is_asked_for_must_be_something_the_core_knows_how_to_ask_about() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        let raw = |request: u64, kinds: Vec<i32>| EngineEvent {
            web_view_id: web_view,
            kind: Some(engine_event::Kind::PermissionRequested(
                PermissionRequested {
                    request_id: request,
                    origin: MEET.to_owned(),
                    kinds,
                },
            )),
        };

        // Nothing; no kind; a kind of a later contract; and one of each.
        for (request, kinds) in [
            (1, vec![]),
            (2, vec![0]),
            (3, vec![99]),
            (4, vec![Camera as i32, 99]),
        ] {
            shell.report(raw(request, kinds));
            shell.settle().await;
            assert_eq!(told(&shell).last(), Some(&(request, false)));
            assert_eq!(prompt(&shell, subscription), None);
        }
    });
}

#[test]
fn camera_and_microphone_are_asked_about_together_and_granted_together() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;

        // In whatever order, and named twice, the prompt has each once.
        ask(&mut shell, web_view, 1, &[Microphone, Camera, Microphone]).await;
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Camera, Microphone])
        );
        answered(&mut shell, tab, subscription, true).await;

        assert_eq!(told(&shell), [(1, true)]);
        ask(&mut shell, web_view, 2, &[Camera]).await;
        ask(&mut shell, web_view, 3, &[Microphone]).await;
        ask(&mut shell, web_view, 4, &[Camera, Microphone]).await;
        assert_eq!(told(&shell), [(1, true), (2, true), (3, true), (4, true)]);

        // With only one of two allowed, the request is not granted in part:
        // the person is asked about the whole of it.
        ask(&mut shell, web_view, 5, &[Camera, Location]).await;
        assert_eq!(told(&shell).len(), 4);
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Camera, Location])
        );
        answered(&mut shell, tab, subscription, false).await;
        assert_eq!(told(&shell).last(), Some(&(5, false)));
        assert_eq!(
            allowed(&mut shell, window).await[0].2,
            [Camera, Microphone],
            "and a no takes nothing back"
        );
    });
}

#[test]
fn requests_wait_their_turn_and_a_page_that_asks_too_much_is_told_no() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;
        ask(&mut shell, web_view, 2, &[Microphone]).await;
        ask(&mut shell, web_view, 3, &[Location]).await;
        ask(&mut shell, web_view, 4, &[Camera, Location]).await;
        assert_eq!(told(&shell), [], "four wait: one shown, three behind it");

        ask(&mut shell, web_view, 5, &[Microphone, Location]).await;
        assert_eq!(told(&shell), [(5, false)], "a fifth is told no");

        // One at a time, in the order they were asked.
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Camera])
        );
        answered(&mut shell, tab, subscription, true).await;
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Microphone])
        );
        answered(&mut shell, tab, subscription, false).await;
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Location])
        );
        assert_eq!(told(&shell), [(5, false), (1, true), (2, false)]);

        // The last that waits wants the camera, which is allowed, and the
        // location: allowing that answers it too.
        answered(&mut shell, tab, subscription, true).await;
        assert_eq!(
            told(&shell),
            [(5, false), (1, true), (2, false), (3, true), (4, true)]
        );
        assert_eq!(prompt(&shell, subscription), None);
    });
}

#[test]
fn what_else_waited_for_the_same_answer_has_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        // A page that asks twice for the same, as pages do.
        ask(&mut shell, web_view, 1, &[Camera]).await;
        ask(&mut shell, web_view, 2, &[Camera]).await;
        ask(&mut shell, web_view, 3, &[Microphone]).await;
        answered(&mut shell, tab, subscription, true).await;
        assert_eq!(told(&shell), [(1, true), (2, true)]);
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Microphone])
        );

        // And a no: whatever else wants what was refused is refused.
        ask(&mut shell, web_view, 4, &[Microphone, Location]).await;
        ask(&mut shell, web_view, 5, &[Location]).await;
        answered(&mut shell, tab, subscription, false).await;
        assert_eq!(told(&shell), [(1, true), (2, true), (3, false), (4, false)]);
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Location]),
            "what wants only something else still waits"
        );
    });
}

#[test]
fn allowed_in_one_tab_it_is_granted_in_another_that_waited() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_view) = a_page(&mut shell).await;
        ask(&mut shell, first_view, 1, &[Camera]).await;
        let second = shell.open_tab(window, "meet.example.com/room").await;
        let second_view = front(&shell, subscription).web_view_id;
        assert_ne!(second_view, first_view);
        ask(&mut shell, second_view, 2, &[Camera]).await;
        ask(&mut shell, second_view, 3, &[Microphone]).await;

        answered(&mut shell, second, subscription, true).await;

        assert_eq!(
            told(&shell),
            [(2, true), (1, true)],
            "the tab behind waited for the same"
        );
        shell.activate_tab(first).await;
        assert_eq!(prompt(&shell, subscription), None);
        // A no is the page's own: the tab behind is not refused by it.
        ask(&mut shell, first_view, 4, &[Microphone]).await;
        shell.activate_tab(second).await;
        answered(&mut shell, second, subscription, false).await;
        assert_eq!(told(&shell).last(), Some(&(3, false)));
        shell.activate_tab(first).await;
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Microphone])
        );
    });
}

#[test]
fn allowed_for_one_origin_it_is_not_granted_to_another_that_waited() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, first_view) = a_page(&mut shell).await;
        ask(&mut shell, first_view, 1, &[Camera]).await;
        let second = shell.open_tab(window, "example.org").await;
        let second_view = front(&shell, subscription).web_view_id;
        ask_as(&mut shell, second_view, 2, "https://example.org", &[Camera]).await;

        answered(&mut shell, second, subscription, true).await;

        assert_eq!(told(&shell), [(2, true)], "the other site still waits");
        shell.activate_tab(first).await;
        let (_, site, _) = prompt(&shell, subscription).expect("and is asked about");
        assert_eq!(site, "meet.example.com");
    });
}

#[test]
fn a_new_page_refuses_what_waited() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;
        ask(&mut shell, web_view, 2, &[Microphone]).await;

        // A page that only changes its own address is still asking.
        shell.report(FakeEngine::address_changed(
            web_view,
            "https://meet.example.com/#later",
        ));
        shell.settle().await;
        assert_eq!(told(&shell), []);
        assert!(prompt(&shell, subscription).is_some());

        shell.submit(tab, "example.org").await;

        assert_eq!(told(&shell), [(1, false), (2, false)]);
        assert_eq!(prompt(&shell, subscription), None);
        // And nothing was written down by it.
        shell.submit(tab, "meet.example.com").await;
        ask(&mut shell, web_view, 3, &[Camera]).await;
        assert!(
            prompt(&shell, subscription).is_some(),
            "it is asked about again"
        );
    });
}

#[test]
fn a_page_whose_renderer_is_gone_asks_no_more() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;

        shell.report(FakeEngine::renderer_gone(web_view));
        shell.settle().await;

        assert_eq!(told(&shell), [(1, false)]);
        assert_eq!(prompt(&shell, subscription), None);
    });
}

#[test]
fn a_tab_behind_keeps_its_prompt_until_it_is_in_front() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, web_view) = a_page(&mut shell).await;
        shell.open_tab(window, "example.org").await;

        // The page behind asks.
        ask(&mut shell, web_view, 1, &[Camera]).await;

        assert_eq!(prompt(&shell, subscription), None, "not in another tab");
        assert_eq!(told(&shell), [], "and its page waits");
        shell.activate_tab(first).await;
        let (_, site, kinds) = prompt(&shell, subscription).expect("it asks when it is in front");
        assert_eq!((site.as_str(), kinds), ("meet.example.com", vec![Camera]));
    });
}

#[test]
fn a_tab_that_closes_takes_what_it_asked_with_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, web_view) = a_page(&mut shell).await;
        let second = shell.open_tab(window, "example.org").await;
        ask(&mut shell, web_view, 1, &[Camera]).await;

        shell.close_tab(first).await;

        // The shell refuses what waits with a web view it destroys; the
        // core has nothing more to say of it.
        assert_eq!(told(&shell), []);
        assert_eq!(prompt(&shell, subscription), None);
        let gone = answer(&mut shell, first, 1, true).await;
        assert_eq!(error_code(gone), "tab.unknown");
        assert_eq!(front(&shell, subscription).tab_id, second);
    });
}

#[test]
fn an_answer_to_a_prompt_the_tab_does_not_show_answers_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;
        ask(&mut shell, web_view, 2, &[Microphone]).await;
        let first = shown(&shell, subscription);

        // Not a prompt; one that waits its turn, which has the next name;
        // and not a tab.
        let wrong = answer(&mut shell, tab, first + 1_000, true).await;
        assert_eq!(error_code(wrong), "prompt.unknown");
        let waiting = answer(&mut shell, tab, first + 1, true).await;
        assert_eq!(error_code(waiting), "prompt.unknown");
        let no_tab = answer(&mut shell, 999, first, true).await;
        assert_eq!(error_code(no_tab), "tab.unknown");
        assert_eq!(told(&shell), []);
        assert_eq!(shown(&shell, subscription), first);

        // Answered, it cannot be answered again.
        answered(&mut shell, tab, subscription, false).await;
        let again = answer(&mut shell, tab, first, true).await;
        assert_eq!(error_code(again), "prompt.unknown");
        assert_eq!(told(&shell), [(1, false)]);
        assert_eq!(allowed(&mut shell, window).await, []);
        assert_eq!(
            shown(&shell, subscription),
            first + 1,
            "the one that waited is shown now, by the name it had"
        );
    });
}

#[test]
fn a_request_the_page_withdraws_is_asked_about_no_more() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;
        ask(&mut shell, web_view, 2, &[Microphone]).await;

        // Another web view's word about it is not the page's.
        shell.report(FakeEngine::permission_withdrawn(999, 1));
        shell.settle().await;
        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Camera])
        );

        shell.report(FakeEngine::permission_withdrawn(web_view, 1));
        shell.settle().await;

        assert_eq!(
            prompt(&shell, subscription).map(|p| p.2),
            Some(vec![Microphone])
        );
        assert_eq!(told(&shell), [], "nothing is answered");
    });
}

#[test]
fn a_request_said_twice_is_asked_about_once() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;
        ask(&mut shell, web_view, 1, &[Camera]).await;

        answered(&mut shell, tab, subscription, true).await;

        assert_eq!(told(&shell), [(1, true)]);
        assert_eq!(prompt(&shell, subscription), None);
    });
}

#[test]
fn what_was_allowed_is_listed_and_taken_back() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, 1, &[Microphone, Camera]).await;
        answered(&mut shell, tab, subscription, true).await;
        shell.submit(tab, "http://127.0.0.1:8080/").await;
        ask_as(
            &mut shell,
            web_view,
            2,
            "http://127.0.0.1:8080",
            &[Location],
        )
        .await;
        answered(&mut shell, tab, subscription, true).await;
        shell.submit(tab, "example.org").await;
        ask_as(&mut shell, web_view, 3, "https://example.org", &[Camera]).await;
        answered(&mut shell, tab, subscription, true).await;

        assert_eq!(
            allowed(&mut shell, window).await,
            [
                (
                    "example.org".to_owned(),
                    "https://example.org".to_owned(),
                    vec![Camera]
                ),
                (
                    "http://127.0.0.1:8080".to_owned(),
                    "http://127.0.0.1:8080".to_owned(),
                    vec![Location]
                ),
                (
                    "meet.example.com".to_owned(),
                    MEET.to_owned(),
                    vec![Camera, Microphone]
                ),
            ],
            "in the order of their names as shown"
        );

        // One origin's is taken back: it is asked about the next time.
        let origin = forget_permissions::What::Origin("https://example.org/".to_owned());
        let done = forget(&mut shell, window, Some(origin)).await;
        assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
        assert_eq!(allowed(&mut shell, window).await.len(), 2);
        ask_as(&mut shell, web_view, 4, "https://example.org", &[Camera]).await;
        assert_eq!(told(&shell).len(), 3, "nothing is granted");
        assert!(prompt(&shell, subscription).is_some());

        // Naming nothing takes nothing back.
        for what in [
            None,
            Some(forget_permissions::What::Everything(false)),
            Some(forget_permissions::What::Origin(String::new())),
        ] {
            let refused = forget(&mut shell, window, what).await;
            assert_eq!(error_code(refused), "permission.nothing_named");
        }
        // An origin that is none of the web's was allowed nothing.
        let nothing = forget_permissions::What::Origin("about:blank".to_owned());
        let done = forget(&mut shell, window, Some(nothing)).await;
        assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
        assert_eq!(allowed(&mut shell, window).await.len(), 2);

        let all = forget_permissions::What::Everything(true);
        forget(&mut shell, window, Some(all)).await;
        assert_eq!(allowed(&mut shell, window).await, []);
    });
}

#[test]
fn a_window_there_is_not_has_no_permissions() {
    run(async {
        let mut shell = FakeShell::fake();
        let asked = shell
            .request(request::Kind::QueryPermissions(QueryPermissions {
                window_id: 99,
            }))
            .await;
        assert_eq!(error_code(asked), "window.unknown");
        let all = forget_permissions::What::Everything(true);
        let forgotten = forget(&mut shell, 99, Some(all)).await;
        assert_eq!(error_code(forgotten), "window.unknown");
    });
}
