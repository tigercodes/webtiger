//! A page says something, or asks (016), through the contract, against the
//! scripted engine: what the person is shown, what the page is answered,
//! and what is dismissed with nothing shown.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_app::{DIALOG_WORDS, DOWNLOAD_INTERVAL, PROMPT_DELAY};
use wt_proto::v1::DialogKind::{Alert, Confirm, Prompt};
use wt_proto::v1::{
    AnswerPrompt, DialogKind, DialogRequested, EngineEvent, PermissionKind, TabView, engine_event,
    prompt_view, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const SITE: &str = "https://example.com";

/// The tab in front, whole.
fn front(shell: &FakeShell, subscription: u64) -> TabView {
    shell.window(subscription).tab.expect("a window has a tab")
}

/// What the tab in front shows of a page's dialog.
#[derive(Debug, PartialEq)]
struct Shown {
    id: u64,
    site: String,
    kind: DialogKind,
    message: String,
    default_text: String,
    may_silence: bool,
}

/// The dialog the tab in front shows. `None` when it shows no prompt.
fn shown(shell: &FakeShell, subscription: u64) -> Option<Shown> {
    let prompt = front(shell, subscription).prompt?;
    match prompt.kind {
        Some(prompt_view::Kind::Dialog(dialog)) => Some(Shown {
            id: prompt.prompt_id,
            kind: dialog.kind(),
            site: dialog.site,
            message: dialog.message,
            default_text: dialog.default_text,
            may_silence: dialog.may_silence,
        }),
        other => panic!("a prompt of some other kind: {other:?}"),
    }
}

/// The page's words in the dialog the tab in front shows.
fn words(shell: &FakeShell, subscription: u64) -> Option<String> {
    shown(shell, subscription).map(|shown| shown.message)
}

/// A web view's page says something, or asks, as the shell would report it.
async fn say(shell: &mut FakeShell, web_view: u64, request: u64, kind: DialogKind, message: &str) {
    say_as(shell, web_view, request, SITE, kind, message, "").await;
}

async fn say_as(
    shell: &mut FakeShell,
    web_view: u64,
    request: u64,
    origin: &str,
    kind: DialogKind,
    message: &str,
    default_text: &str,
) {
    shell.report(FakeEngine::dialog_requested(
        web_view,
        request,
        origin,
        kind,
        message,
        default_text,
    ));
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

/// Answers the prompt the tab in front shows, once it has been there long
/// enough: OK or Cancel, with a prompt's text.
async fn answer(shell: &mut FakeShell, tab: u64, subscription: u64, ok: bool, text: &str) {
    tokio::time::advance(PROMPT_DELAY).await;
    let reply = AnswerPrompt {
        allow: ok,
        text: text.to_owned(),
        ..Default::default()
    };
    let done = send(shell, tab, subscription, reply).await;
    assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
}

/// The person stops the page's messages, by the prompt the tab shows.
async fn silence(shell: &mut FakeShell, tab: u64, subscription: u64) {
    tokio::time::advance(PROMPT_DELAY).await;
    let reply = AnswerPrompt {
        silence: true,
        ..Default::default()
    };
    let done = send(shell, tab, subscription, reply).await;
    assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");
}

/// How the shell was told to answer the pages, in order: the dialog,
/// whether it was accepted, and the text.
fn told(shell: &FakeShell) -> Vec<(u64, bool, String)> {
    shell.engine.dialog_answers.clone()
}

/// A dialog dismissed: the page is answered as if the person had closed it.
fn dismissed(request: u64) -> (u64, bool, String) {
    (request, false, String::new())
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

/// The web a shell browses in these tests.
fn pages(shell: &mut FakeShell) {
    for (url, title) in [
        ("https://example.com/", "Example"),
        ("https://example.com/more", "More of it"),
        ("https://example.org/", "Another"),
        ("http://shop.example.com/", "A shop, not over https"),
    ] {
        shell.engine.page(url, title);
    }
    shell.engine.fail("https://nowhere.example.com/", "net.dns");
}

/// A window whose one tab shows example.com. Its window, tab, subscription
/// and the tab's web view.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    pages(shell);
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, "example.com").await;
    let web_view = front(shell, subscription).web_view_id;
    (window, tab, subscription, web_view)
}

#[test]
fn what_a_page_says_is_shown_in_its_tab_under_the_sites_name_and_the_page_waits() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        assert_eq!(shown(&shell, subscription), None);

        say(&mut shell, web_view, 1, Alert, "Your basket is empty.").await;

        let dialog = shown(&shell, subscription).expect("the tab shows it");
        assert_ne!(dialog.id, 0);
        assert_eq!(dialog.site, "example.com", "as the address field names it");
        assert_eq!(dialog.kind, Alert);
        assert_eq!(dialog.message, "Your basket is empty.");
        assert_eq!(dialog.default_text, "");
        assert!(!dialog.may_silence, "a page's first is not to be stopped");
        assert_eq!(told(&shell), [], "the page's script waits");
    });
}

#[test]
fn ok_and_cancel_answer_each_of_the_three_as_the_page_expects() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        // An alert returns, whichever it is told.
        say(&mut shell, web_view, 1, Alert, "Saved.").await;
        answer(&mut shell, tab, subscription, true, "").await;
        say(&mut shell, web_view, 2, Alert, "Saved again.").await;
        answer(&mut shell, tab, subscription, false, "").await;
        // A confirm is true or false.
        say(&mut shell, web_view, 3, Confirm, "Delete it?").await;
        assert_eq!(shown(&shell, subscription).map(|s| s.kind), Some(Confirm));
        answer(&mut shell, tab, subscription, true, "").await;
        say(&mut shell, web_view, 4, Confirm, "Delete the other?").await;
        answer(&mut shell, tab, subscription, false, "").await;
        // A prompt is the text in its field, or nothing.
        say_as(&mut shell, web_view, 5, SITE, Prompt, "Your name?", "Ada").await;
        let asked = shown(&shell, subscription).expect("the prompt is shown");
        assert_eq!((asked.kind, asked.default_text.as_str()), (Prompt, "Ada"));
        answer(&mut shell, tab, subscription, true, "Ada Lovelace").await;
        say_as(&mut shell, web_view, 6, SITE, Prompt, "Your name?", "Ada").await;
        answer(
            &mut shell,
            tab,
            subscription,
            false,
            "typed, and thought better of",
        )
        .await;
        // The text of a prompt may be nothing at all, and is still an answer.
        say_as(&mut shell, web_view, 7, SITE, Prompt, "A word?", "").await;
        answer(&mut shell, tab, subscription, true, "").await;

        assert_eq!(
            told(&shell),
            [
                (1, true, String::new()),
                dismissed(2),
                (3, true, String::new()),
                dismissed(4),
                (5, true, "Ada Lovelace".to_owned()),
                dismissed(6),
                (7, true, String::new()),
            ]
        );
        assert_eq!(shown(&shell, subscription), None);
    });
}

#[test]
fn what_is_not_the_pages_own_to_say_is_dismissed_with_nothing_shown() {
    run(async {
        let mut shell = FakeShell::fake();
        pages(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;

        // A web view that is no tab's.
        say(&mut shell, 999, 1, Alert, "Hello").await;
        // A tab with a web view and no page: where it was sent failed.
        shell.submit(tab, "nowhere.example.com").await;
        let web_view = front(&shell, subscription).web_view_id;
        say_as(
            &mut shell,
            web_view,
            2,
            "https://nowhere.example.com",
            Alert,
            "Hello",
            "",
        )
        .await;
        // A frame from another site, another of the site's names, the same
        // name over another scheme, and no origin at all.
        shell.submit(tab, "example.com").await;
        for (request, origin) in [
            (3, "https://ads.example.net"),
            (4, "https://www.example.com"),
            (5, "http://example.com"),
            (6, ""),
        ] {
            say_as(
                &mut shell,
                web_view,
                request,
                origin,
                Confirm,
                "Are you sure?",
                "",
            )
            .await;
        }
        // Something that is none of the three.
        let raw = |request: u64, kind: i32| EngineEvent {
            web_view_id: web_view,
            kind: Some(engine_event::Kind::DialogRequested(DialogRequested {
                request_id: request,
                origin: SITE.to_owned(),
                kind,
                message: "Hello".to_owned(),
                default_text: String::new(),
            })),
        };
        shell.report(raw(7, 0));
        shell.report(raw(8, 99));
        shell.settle().await;

        assert_eq!(
            told(&shell),
            (1..=8).map(dismissed).collect::<Vec<_>>(),
            "each is answered as if the person had closed it"
        );
        assert_eq!(shown(&shell, subscription), None, "and nothing was shown");

        // The page's own origin, however the shell spells it, is the page.
        say_as(
            &mut shell,
            web_view,
            9,
            "HTTPS://Example.com:443/",
            Alert,
            "Hello",
            "",
        )
        .await;
        assert_eq!(words(&shell, subscription).as_deref(), Some("Hello"));
    });
}

#[test]
fn a_page_not_over_https_may_speak_and_is_named_with_its_scheme() {
    run(async {
        let mut shell = FakeShell::fake();
        pages(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        shell.submit(tab, "http://shop.example.com/").await;
        let web_view = front(&shell, subscription).web_view_id;

        say_as(
            &mut shell,
            web_view,
            1,
            "http://shop.example.com",
            Alert,
            "Sale!",
            "",
        )
        .await;

        let dialog = shown(&shell, subscription).expect("it is shown");
        assert_eq!(dialog.site, "http://shop.example.com");
    });
}

#[test]
fn what_a_page_says_at_length_is_cut_and_never_inside_a_character() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;
        // Each of these is more than one byte: a cut that counted bytes
        // would fall inside one.
        let long: String = "é".repeat(DIALOG_WORDS + 500);

        say_as(&mut shell, web_view, 1, SITE, Prompt, &long, &long).await;

        let dialog = shown(&shell, subscription).expect("it is shown");
        assert_eq!(dialog.message.chars().count(), DIALOG_WORDS);
        assert_eq!(dialog.default_text.chars().count(), DIALOG_WORDS);
        assert!(dialog.message.chars().all(|c| c == 'é'));
    });
}

#[test]
fn only_a_prompt_offers_text() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, _, subscription, web_view) = a_page(&mut shell).await;

        say_as(
            &mut shell,
            web_view,
            1,
            SITE,
            Confirm,
            "Sure?",
            "not a prompt's",
        )
        .await;

        assert_eq!(
            shown(&shell, subscription).map(|s| s.default_text),
            Some(String::new())
        );
    });
}

#[test]
fn dialogs_wait_their_turn_and_a_page_that_says_too_much_is_dismissed() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        for request in 1..=4 {
            say(
                &mut shell,
                web_view,
                request,
                Alert,
                &format!("Number {request}"),
            )
            .await;
        }
        assert_eq!(told(&shell), [], "four wait: one shown, three behind it");

        say(&mut shell, web_view, 5, Alert, "Number 5").await;
        assert_eq!(told(&shell), [dismissed(5)]);

        // One at a time, in the order they were said.
        for request in 1..=4_u64 {
            assert_eq!(
                words(&shell, subscription),
                Some(format!("Number {request}"))
            );
            answer(&mut shell, tab, subscription, true, "").await;
        }
        assert_eq!(shown(&shell, subscription), None);
        assert_eq!(told(&shell).len(), 5);
    });
}

#[test]
fn a_page_that_says_more_can_be_stopped_until_another_page() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;

        // The first is not to be stopped: asking to is asking nothing, and
        // it is answered by what else was said.
        say(&mut shell, web_view, 1, Confirm, "One?").await;
        assert_eq!(
            shown(&shell, subscription).map(|s| s.may_silence),
            Some(false)
        );
        tokio::time::advance(PROMPT_DELAY).await;
        let reply = AnswerPrompt {
            allow: true,
            silence: true,
            ..Default::default()
        };
        send(&mut shell, tab, subscription, reply).await;
        assert_eq!(told(&shell), [(1, true, String::new())]);

        // From the second on, it can be.
        say(&mut shell, web_view, 2, Alert, "Two").await;
        say(&mut shell, web_view, 3, Confirm, "Three?").await;
        say(&mut shell, web_view, 4, Alert, "Four").await;
        assert_eq!(
            shown(&shell, subscription).map(|s| s.may_silence),
            Some(true)
        );
        silence(&mut shell, tab, subscription).await;

        // That one, and those that waited, are dismissed; nothing is shown.
        assert_eq!(
            told(&shell)[1..],
            [dismissed(2), dismissed(3), dismissed(4)]
        );
        assert_eq!(shown(&shell, subscription), None);
        // And so is whatever the page says after.
        say(&mut shell, web_view, 5, Confirm, "Five?").await;
        assert_eq!(told(&shell).last(), Some(&dismissed(5)));
        assert_eq!(shown(&shell, subscription), None);
        // A page that only changes its own address is the same page.
        shell.report(FakeEngine::address_changed(
            web_view,
            "https://example.com/#again",
        ));
        shell.settle().await;
        say(&mut shell, web_view, 6, Alert, "Six").await;
        assert_eq!(told(&shell).last(), Some(&dismissed(6)));

        // Another page has loaded: it may speak, and its first is its first.
        shell.submit(tab, "example.com/more").await;
        say(&mut shell, web_view, 7, Alert, "Seven").await;
        let dialog = shown(&shell, subscription).expect("it is shown");
        assert_eq!(
            (dialog.message.as_str(), dialog.may_silence),
            ("Seven", false)
        );
    });
}

#[test]
fn a_new_page_and_a_lost_renderer_dismiss_what_waited() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, web_view) = a_page(&mut shell).await;
        say(&mut shell, web_view, 1, Confirm, "Leave?").await;
        say(&mut shell, web_view, 2, Alert, "And another thing").await;

        shell.submit(tab, "example.org").await;

        assert_eq!(told(&shell), [dismissed(1), dismissed(2)]);
        assert_eq!(shown(&shell, subscription), None);

        say_as(
            &mut shell,
            web_view,
            3,
            "https://example.org",
            Alert,
            "Hello",
            "",
        )
        .await;
        shell.report(FakeEngine::renderer_gone(web_view));
        shell.settle().await;
        assert_eq!(told(&shell).last(), Some(&dismissed(3)));
        assert_eq!(shown(&shell, subscription), None);

        // A tab that closes takes what it showed with it: the shell
        // dismisses what waits with a web view it destroys.
        shell.submit(tab, "example.com").await;
        say(&mut shell, web_view, 4, Alert, "Goodbye").await;
        shell.open_tab(window, "example.org").await;
        shell.close_tab(tab).await;
        assert_eq!(
            told(&shell).len(),
            3,
            "the core has nothing more to say of it"
        );
    });
}

#[test]
fn a_tab_behind_keeps_its_dialog_and_it_is_not_answered_from_there() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription, web_view) = a_page(&mut shell).await;
        shell.open_tab(window, "example.org").await;

        // The page behind speaks.
        say(&mut shell, web_view, 1, Confirm, "Still there?").await;

        assert_eq!(shown(&shell, subscription), None, "not in another tab");
        assert_eq!(told(&shell), [], "its script waits");

        shell.activate_tab(first).await;
        let dialog = shown(&shell, subscription).expect("it is shown when its tab is in front");
        assert_eq!(dialog.message, "Still there?");
        // Not in the moment it appears: a key on its way to the page must
        // not say "OK" to a question nobody has read.
        let reply = AnswerPrompt {
            allow: true,
            ..Default::default()
        };
        let at_once = send(&mut shell, first, subscription, reply).await;
        assert_eq!(error_code(at_once), "prompt.too_soon");
        assert_eq!(told(&shell), []);
        answer(&mut shell, first, subscription, true, "").await;
        assert_eq!(told(&shell), [(1, true, String::new())]);
    });
}

#[test]
fn said_twice_it_is_shown_once() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        say(&mut shell, web_view, 1, Alert, "Hello").await;
        say(&mut shell, web_view, 1, Alert, "Hello").await;

        answer(&mut shell, tab, subscription, true, "").await;

        assert_eq!(told(&shell), [(1, true, String::new())]);
        assert_eq!(shown(&shell, subscription), None);
    });
}

#[test]
fn a_dialog_a_download_and_a_request_of_the_same_number_are_each_their_own() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, web_view) = a_page(&mut shell).await;
        // A shell counts each kind from one.
        say(&mut shell, web_view, 7, Confirm, "Proceed?").await;
        shell.report(FakeEngine::permission_requested(
            web_view,
            7,
            SITE,
            &[PermissionKind::Camera],
        ));
        shell.report(FakeEngine::download_requested(
            web_view,
            7,
            "https://example.com/report.pdf",
            "report.pdf",
            "application/pdf",
            1_000,
        ));
        shell.settle().await;

        // The page's question is answered: nothing is allowed by a yes to it.
        answer(&mut shell, tab, subscription, true, "").await;
        assert_eq!(told(&shell), [(7, true, String::new())]);
        assert_eq!(shell.engine.permission_answers, []);
        assert_eq!(shell.engine.download_starts, []);

        // The camera is allowed: what a page said, and a download, are not.
        say(&mut shell, web_view, 8, Alert, "Thank you").await;
        answer(&mut shell, tab, subscription, true, "").await;
        assert_eq!(shell.engine.permission_answers, [(7, true)]);
        assert_eq!(shell.engine.download_starts, []);
        assert_eq!(
            told(&shell).len(),
            1,
            "what the page said still waits its turn"
        );

        // The download is refused: what the page said is not dismissed by that.
        answer(&mut shell, tab, subscription, false, "").await;
        assert_eq!(shell.engine.download_cancels, [7]);
        assert_eq!(words(&shell, subscription).as_deref(), Some("Thank you"));
        answer(&mut shell, tab, subscription, true, "").await;
        assert_eq!(
            told(&shell),
            [(7, true, String::new()), (8, true, String::new())]
        );

        // And stopping a page's messages stops nothing else that waits.
        say(&mut shell, web_view, 9, Alert, "One more").await;
        tokio::time::advance(DOWNLOAD_INTERVAL).await;
        shell.report(FakeEngine::permission_requested(
            web_view,
            9,
            SITE,
            &[PermissionKind::Microphone],
        ));
        shell.settle().await;
        silence(&mut shell, tab, subscription).await;
        assert_eq!(told(&shell).last(), Some(&dismissed(9)));
        assert_eq!(
            shell.engine.permission_answers,
            [(7, true)],
            "the microphone is still asked about"
        );
        assert!(front(&shell, subscription).prompt.is_some());
    });
}
