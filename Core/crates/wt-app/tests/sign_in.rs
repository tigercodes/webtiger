//! A server asks who is there (017), through the contract, against the
//! scripted engine: who may ask, what the person is shown, what the engine
//! is told to answer, and when a question is over.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_app::PROMPT_DELAY;
use wt_proto::v1::{
    AnswerPrompt, DialogKind, PermissionKind, SignInRequested, TabView, prompt_view, request,
    response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

/// The page the tab shows in these tests.
const PAGE: &str = "https://example.com";
/// A server the tab's load is heading for.
const SERVER: &str = "https://staff.example.com";

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the tab in front shows of a server's question.
#[derive(Debug, PartialEq)]
struct Shown {
    site: String,
    confidential: bool,
    again: bool,
}

/// The sign-in the tab in front shows. `None` when it shows no prompt.
fn shown(shell: &FakeShell, subscription: u64) -> Option<Shown> {
    let prompt = front(shell, subscription).prompt?;
    match prompt.kind {
        Some(prompt_view::Kind::SignIn(sign_in)) => Some(Shown {
            site: sign_in.site,
            confidential: sign_in.confidential,
            again: sign_in.again,
        }),
        other => panic!("a prompt of some other kind: {other:?}"),
    }
}

/// The server a load is heading for asks, as WebKit says it: by its origin.
fn heading_for(request: u64, server: &str) -> SignInRequested {
    let host = server.split("//").nth(1).unwrap_or(server);
    SignInRequested {
        request_id: request,
        host: host.split(':').next().unwrap_or(host).to_owned(),
        origin: server.to_owned(),
        realm: "Staff".to_owned(),
        heading: format!("{server}/rota"),
        again: false,
    }
}

/// A server asks, with no load in flight, as WebKit says it.
fn of_the_page(request: u64, server: &str) -> SignInRequested {
    SignInRequested {
        heading: String::new(),
        ..heading_for(request, server)
    }
}

/// The same, as the System WebView says it: by the server's host alone.
fn by_host(asked: SignInRequested) -> SignInRequested {
    SignInRequested {
        origin: String::new(),
        ..asked
    }
}

/// A server asks who is there in a web view, as the shell would report it.
async fn ask(shell: &mut FakeShell, web_view: u64, asked: SignInRequested) {
    shell.report(FakeEngine::sign_in_requested(web_view, asked));
    shell.settle().await;
}

/// The person's answer to the prompt the tab in front shows, as it is sent.
async fn send(
    shell: &mut FakeShell,
    tab: u64,
    subscription: u64,
    answer: AnswerPrompt,
) -> response::Kind {
    let prompt = front(shell, subscription)
        .prompt
        .expect("the tab shows something")
        .prompt_id;
    shell
        .request(request::Kind::AnswerPrompt(AnswerPrompt {
            tab_id: tab,
            prompt_id: prompt,
            ..answer
        }))
        .await
}

/// What a person typed, with the button they pressed.
fn typed(sign_in: bool, name: &str, password: &str) -> AnswerPrompt {
    AnswerPrompt {
        allow: sign_in,
        text: name.to_owned(),
        secret: password.to_owned(),
        ..Default::default()
    }
}

/// Signs in at the prompt the tab in front shows, once it has been there
/// long enough.
async fn sign_in(shell: &mut FakeShell, tab: u64, subscription: u64, name: &str, password: &str) {
    tokio::time::advance(PROMPT_DELAY).await;
    let done = send(shell, tab, subscription, typed(true, name, password)).await;
    assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
}

/// Presses Cancel at the prompt the tab in front shows, with something
/// typed all the same.
async fn cancel(shell: &mut FakeShell, tab: u64, subscription: u64) {
    tokio::time::advance(PROMPT_DELAY).await;
    let done = send(shell, tab, subscription, typed(false, "ada", "lovelace")).await;
    assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
}

/// How the shell was told to answer the servers, in order: the question,
/// and the name and password, or `None` for no answer.
fn told(shell: &FakeShell) -> Vec<(u64, Option<(String, String)>)> {
    shell.engine.sign_in_answers.clone()
}

/// A server answered with a name and a password.
fn with(request: u64, name: &str, password: &str) -> (u64, Option<(String, String)>) {
    (request, Some((name.to_owned(), password.to_owned())))
}

/// A server given no answer.
fn unanswered(request: u64) -> (u64, Option<(String, String)>) {
    (request, None)
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// A window whose one tab shows example.com. Its window, tab, subscription
/// and the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    for (url, title) in [
        ("https://example.com/", "Example"),
        ("https://example.com/more", "More of it"),
        ("https://example.org/", "Another"),
        ("http://shop.example.com/", "A shop, not over https"),
    ] {
        shell.engine.page(url, title);
    }
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

#[test]
fn the_server_a_load_is_heading_for_asks_in_the_tab_under_its_own_name_and_waits() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        assert_eq!(shown(&shell, subscription), None);

        ask(&mut shell, web_view, heading_for(1, SERVER)).await;

        assert_eq!(
            shown(&shell, subscription),
            Some(Shown {
                // The server that gets what is typed, not the page that is
                // still on the screen.
                site: "staff.example.com".to_owned(),
                confidential: true,
                again: false,
            })
        );
        assert_eq!(told(&shell), [], "the load waits");
        // The page the tab shows is as it was.
        assert_eq!(front(&shell, subscription).title, "Example");
    });
}

#[test]
fn the_name_and_the_password_are_the_servers_answer_and_cancel_is_none() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        ask(&mut shell, web_view, heading_for(1, SERVER)).await;
        sign_in(&mut shell, tab, subscription, "ada", "love lace").await;
        assert_eq!(told(&shell), [with(1, "ada", "love lace")]);
        assert_eq!(shown(&shell, subscription), None, "answered, it is gone");

        // Cancel gives the server no answer, whatever was typed.
        ask(&mut shell, web_view, heading_for(2, SERVER)).await;
        cancel(&mut shell, tab, subscription).await;
        assert_eq!(told(&shell)[1..], [unanswered(2)]);
        assert_eq!(shown(&shell, subscription), None);

        // A server may want a password and no name.
        ask(&mut shell, web_view, heading_for(3, SERVER)).await;
        sign_in(&mut shell, tab, subscription, "", "only this").await;
        assert_eq!(told(&shell)[2..], [with(3, "", "only this")]);
    });
}

#[test]
fn the_pages_own_server_may_ask() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        ask(&mut shell, web_view, of_the_page(1, PAGE)).await;

        assert_eq!(
            shown(&shell, subscription).map(|shown| shown.site),
            Some("example.com".to_owned())
        );
        sign_in(&mut shell, tab, subscription, "ada", "lovelace").await;
        assert_eq!(told(&shell), [with(1, "ada", "lovelace")]);
    });
}

#[test]
fn any_other_server_is_given_no_answer_with_nobody_asked() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;

        // A picture from elsewhere, while the page sits there.
        ask(&mut shell, web_view, of_the_page(1, "https://example.org")).await;
        // The same, while a load is heading somewhere else again.
        let mut elsewhere = heading_for(2, SERVER);
        elsewhere.origin = "https://example.org".to_owned();
        elsewhere.host = "example.org".to_owned();
        ask(&mut shell, web_view, elsewhere).await;
        // The page's host at another port, and not over https, are other
        // servers.
        ask(
            &mut shell,
            web_view,
            of_the_page(3, "https://example.com:8443"),
        )
        .await;
        ask(&mut shell, web_view, of_the_page(4, "http://example.com")).await;
        // An origin that is none: not taken for the host beside it.
        let mut nonsense = of_the_page(5, PAGE);
        nonsense.origin = "nonsense".to_owned();
        ask(&mut shell, web_view, nonsense).await;
        // A server nobody names.
        let mut nameless = by_host(of_the_page(6, PAGE));
        nameless.host = String::new();
        ask(&mut shell, web_view, nameless).await;
        // And a web view that is no tab's.
        ask(&mut shell, 99, heading_for(7, SERVER)).await;

        assert_eq!(shown(&shell, subscription), None);
        assert_eq!(
            told(&shell),
            (1..=7).map(unanswered).collect::<Vec<_>>(),
            "each is told no answer, at once"
        );
    });
}

#[test]
fn a_server_named_by_its_host_alone_is_the_tabs_own_by_its_host() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        // Where the load is heading: named with that address's scheme.
        ask(&mut shell, web_view, by_host(heading_for(1, SERVER))).await;
        assert_eq!(
            shown(&shell, subscription),
            Some(Shown {
                site: "staff.example.com".to_owned(),
                confidential: true,
                again: false,
            })
        );
        sign_in(&mut shell, tab, subscription, "ada", "lovelace").await;

        // The page's own, however the engine writes the host.
        let mut shouted = by_host(of_the_page(2, PAGE));
        shouted.host = "EXAMPLE.com".to_owned();
        ask(&mut shell, web_view, shouted).await;
        assert_eq!(
            shown(&shell, subscription).map(|shown| shown.site),
            Some("example.com".to_owned())
        );
        sign_in(&mut shell, tab, subscription, "ada", "lovelace").await;

        // Another host is another server.
        ask(
            &mut shell,
            web_view,
            by_host(of_the_page(3, "https://example.org")),
        )
        .await;
        assert_eq!(shown(&shell, subscription), None);

        // A page not over https whose load is heading for the same host
        // over https: the server is taken to be where the load is going.
        shell.submit(tab, "http://shop.example.com/").await;
        let web_view = front(&shell, subscription).web_view_id;
        ask(
            &mut shell,
            web_view,
            by_host(heading_for(4, "https://shop.example.com")),
        )
        .await;
        assert_eq!(
            shown(&shell, subscription),
            Some(Shown {
                site: "shop.example.com".to_owned(),
                confidential: true,
                again: false,
            })
        );
        assert_eq!(
            told(&shell),
            [
                with(1, "ada", "lovelace"),
                with(2, "ada", "lovelace"),
                unanswered(3)
            ]
        );
    });
}

#[test]
fn where_a_redirect_took_the_load_is_who_may_ask_and_who_is_named() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;

        // The person asked for one address; the load was sent on to
        // another server, and that one asks.
        let mut sent_on = heading_for(1, "https://example.org");
        sent_on.heading = "https://example.org/private?from=example.com".to_owned();
        ask(&mut shell, web_view, sent_on).await;

        assert_eq!(
            shown(&shell, subscription).map(|shown| shown.site),
            Some("example.org".to_owned())
        );
    });
}

#[test]
fn a_server_not_over_https_is_said_to_be_read_on_the_way_and_this_device_is_not() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        ask(
            &mut shell,
            web_view,
            heading_for(1, "http://shop.example.com"),
        )
        .await;
        assert_eq!(
            shown(&shell, subscription),
            Some(Shown {
                // Named with its scheme, as the address field would.
                site: "http://shop.example.com".to_owned(),
                confidential: false,
                again: false,
            })
        );
        cancel(&mut shell, tab, subscription).await;

        // Named by its host alone, it is as private as the address that
        // matched.
        ask(
            &mut shell,
            web_view,
            by_host(heading_for(2, "http://shop.example.com")),
        )
        .await;
        assert_eq!(
            shown(&shell, subscription).map(|shown| shown.confidential),
            Some(false)
        );
        cancel(&mut shell, tab, subscription).await;

        // This device itself: nothing travels.
        ask(
            &mut shell,
            web_view,
            heading_for(3, "http://127.0.0.1:8080"),
        )
        .await;
        assert_eq!(
            shown(&shell, subscription),
            Some(Shown {
                site: "http://127.0.0.1:8080".to_owned(),
                confidential: true,
                again: false,
            })
        );
    });
}

#[test]
fn a_server_that_refused_an_answer_asks_again_and_the_prompt_says_so() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, heading_for(1, SERVER)).await;
        sign_in(&mut shell, tab, subscription, "ada", "wrong").await;

        let mut again = heading_for(2, SERVER);
        again.again = true;
        ask(&mut shell, web_view, again).await;

        assert_eq!(
            shown(&shell, subscription).map(|shown| shown.again),
            Some(true)
        );
        sign_in(&mut shell, tab, subscription, "ada", "lovelace").await;
        assert_eq!(
            told(&shell),
            [with(1, "ada", "wrong"), with(2, "ada", "lovelace")]
        );
    });
}

#[test]
fn it_is_not_answered_in_the_moment_it_appears() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        ask(&mut shell, web_view, heading_for(1, SERVER)).await;

        let early = send(
            &mut shell,
            tab,
            subscription,
            typed(true, "ada", "lovelace"),
        )
        .await;

        assert_eq!(error_code(early), "prompt.too_soon");
        assert_eq!(told(&shell), [], "nothing typed went anywhere");
        assert!(shown(&shell, subscription).is_some(), "it is still there");
    });
}

#[test]
fn every_question_of_the_same_server_and_realm_that_waits_has_the_same_answer() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first_tab, subscription, first) = a_page(&mut shell).await;
        // The first tab's server asks twice (a page and its picture), and
        // once for something else it protects.
        ask(&mut shell, first, heading_for(1, SERVER)).await;
        ask(&mut shell, first, heading_for(2, SERVER)).await;
        let mut payroll = heading_for(3, SERVER);
        payroll.realm = "Payroll".to_owned();
        ask(&mut shell, first, payroll).await;
        // A second tab, in front, is asked by the same server; and by
        // another, behind that.
        let second_tab = shell.open_tab(window, "example.com/more").await;
        let second = front(&shell, subscription).web_view_id;
        ask(&mut shell, second, heading_for(4, SERVER)).await;
        ask(&mut shell, second, heading_for(5, "https://example.org")).await;

        sign_in(&mut shell, second_tab, subscription, "ada", "lovelace").await;

        let mut answered = told(&shell);
        answered.sort();
        assert_eq!(
            answered,
            [
                with(1, "ada", "lovelace"),
                with(2, "ada", "lovelace"),
                with(4, "ada", "lovelace"),
            ],
            "the same server and realm, in whatever tab; and no other"
        );
        // The other server is next in this tab.
        assert_eq!(
            shown(&shell, subscription).map(|shown| shown.site),
            Some("example.org".to_owned())
        );
        // No answer is the same answer too.
        ask(&mut shell, second, heading_for(6, "https://example.org")).await;
        cancel(&mut shell, second_tab, subscription).await;
        assert_eq!(told(&shell)[3..], [unanswered(5), unanswered(6)]);
        // What the first tab's server protects under another name still
        // waits there.
        shell.activate_tab(first_tab).await;
        assert!(shown(&shell, subscription).is_some());
        sign_in(&mut shell, first_tab, subscription, "grace", "hopper").await;
        assert_eq!(told(&shell)[5..], [with(3, "grace", "hopper")]);
    });
}

#[test]
fn a_question_is_over_when_its_load_is() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        let endings = [
            // Another load begins.
            FakeEngine::navigation_started(web_view, "https://example.org/"),
            // The load is stopped, or fails.
            FakeEngine::navigation_failed(web_view, "net.cancelled", "https://example.org/"),
            FakeEngine::navigation_failed(web_view, "net.dns", "https://example.org/"),
            // A page arrives.
            FakeEngine::navigation_committed(web_view, "https://example.com/more"),
            // The page's process is lost.
            FakeEngine::renderer_gone(web_view),
        ];

        for (index, ending) in endings.into_iter().enumerate() {
            let request = index as u64 + 1;
            ask(&mut shell, web_view, of_the_page(request, PAGE)).await;
            assert!(shown(&shell, subscription).is_some(), "{request}");

            shell.report(ending);
            shell.settle().await;

            assert_eq!(shown(&shell, subscription), None, "{request}");
            assert_eq!(
                told(&shell).last(),
                Some(&unanswered(request)),
                "the engine is told to give no answer"
            );
        }
    });
}

#[test]
fn what_a_page_says_outlives_a_load_that_only_begins() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        shell.report(FakeEngine::dialog_requested(
            web_view,
            1,
            PAGE,
            DialogKind::Alert,
            "Saved.",
            "",
        ));
        ask(&mut shell, web_view, of_the_page(1, PAGE)).await;

        shell.report(FakeEngine::navigation_started(
            web_view,
            "https://example.org/",
        ));
        shell.settle().await;

        // The server waits no more; the page that spoke is still there,
        // and so is what it said.
        assert_eq!(told(&shell), [unanswered(1)]);
        assert_eq!(shell.engine.dialog_answers, []);
        let prompt = front(&shell, subscription).prompt.expect("still shown");
        assert!(matches!(prompt.kind, Some(prompt_view::Kind::Dialog(_))));
    });
}

#[test]
fn said_twice_it_is_asked_once_and_a_fifth_is_given_no_answer() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        ask(&mut shell, web_view, heading_for(1, SERVER)).await;
        ask(&mut shell, web_view, heading_for(1, SERVER)).await;
        for request in 2..=5 {
            let mut other = heading_for(request, SERVER);
            other.realm = format!("Realm {request}");
            ask(&mut shell, web_view, other).await;
        }

        // Four wait; the fifth was told no answer, and nobody else was.
        assert_eq!(told(&shell), [unanswered(5)]);
        for request in 1..=4 {
            sign_in(&mut shell, tab, subscription, "ada", "lovelace").await;
            assert_eq!(told(&shell).last(), Some(&with(request, "ada", "lovelace")));
        }
        assert_eq!(shown(&shell, subscription), None, "asked once each");
    });
}

#[test]
fn a_server_and_a_page_that_the_shell_gave_one_number_are_not_taken_for_each_other() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        // The shell numbers each kind from one.
        shell.report(FakeEngine::dialog_requested(
            web_view,
            1,
            PAGE,
            DialogKind::Confirm,
            "Delete it?",
            "",
        ));
        ask(&mut shell, web_view, of_the_page(1, PAGE)).await;
        shell.report(FakeEngine::permission_requested(
            web_view,
            1,
            PAGE,
            &[PermissionKind::Camera],
        ));
        shell.settle().await;

        // The page's question is answered: the server and the camera wait.
        tokio::time::advance(PROMPT_DELAY).await;
        let done = send(&mut shell, tab, subscription, typed(true, "", "")).await;
        assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
        assert_eq!(shell.engine.dialog_answers, [(1, true, String::new())]);
        assert_eq!(told(&shell), []);
        assert_eq!(shell.engine.permission_answers, []);

        // The server is answered: the camera waits, and nothing typed is
        // taken for a yes to it.
        sign_in(&mut shell, tab, subscription, "ada", "lovelace").await;
        assert_eq!(told(&shell), [with(1, "ada", "lovelace")]);
        assert_eq!(shell.engine.permission_answers, []);
        let prompt = front(&shell, subscription).prompt.expect("the camera");
        assert!(matches!(
            prompt.kind,
            Some(prompt_view::Kind::Permission(_))
        ));
    });
}

#[test]
fn a_yes_to_the_camera_is_no_answer_to_a_server() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        shell.report(FakeEngine::permission_requested(
            web_view,
            1,
            PAGE,
            &[PermissionKind::Camera],
        ));
        ask(&mut shell, web_view, of_the_page(1, PAGE)).await;

        tokio::time::advance(PROMPT_DELAY).await;
        let done = send(&mut shell, tab, subscription, typed(true, "", "")).await;
        assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");

        assert_eq!(shell.engine.permission_answers, [(1, true)]);
        assert_eq!(told(&shell), [], "the server still waits");
        assert!(shown(&shell, subscription).is_some());
    });
}

#[test]
fn a_realm_is_told_apart_by_its_beginning() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        // Two hundred characters are kept of a server's name for what it
        // protects: these two differ after that.
        let long = "r".repeat(200);
        let mut one = heading_for(1, SERVER);
        one.realm = format!("{long} one");
        let mut two = heading_for(2, SERVER);
        two.realm = format!("{long} two");
        // And this one within it.
        let mut three = heading_for(3, SERVER);
        three.realm = format!("{}x", "r".repeat(199));
        ask(&mut shell, web_view, one).await;
        ask(&mut shell, web_view, two).await;
        ask(&mut shell, web_view, three).await;

        sign_in(&mut shell, tab, subscription, "ada", "lovelace").await;

        assert_eq!(
            told(&shell),
            [with(1, "ada", "lovelace"), with(2, "ada", "lovelace")]
        );
        assert!(shown(&shell, subscription).is_some(), "the third waits");
    });
}
