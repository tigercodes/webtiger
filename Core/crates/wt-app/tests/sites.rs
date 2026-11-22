//! Sites (039): a tab that is kept. Through the contract, against the scripted
//! engine: what keeping a tab makes of it, where it sits, what closing it
//! means, how it wakes, and what comes back after a quit.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_proto::v1::{
    OpenTab, StepTab, SuggestAddresses, Undo, host_request, request, response, step_tab::To,
};
use wt_testkit::{FakeEngine, FakeShell, run};

const MAIL: &str = "https://mail.example.com/";
const MESSAGE: &str = "https://mail.example.com/inbox/7";
const BOARD: &str = "https://board.example.com/";
const CARD: &str = "https://board.example.com/card/3";
const NEWS: &str = "https://news.example.com/";
const STORY: &str = "https://news.example.com/story";

fn web(shell: &mut FakeShell) {
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(MESSAGE, "Re: lunch");
    shell.engine.page(BOARD, "Board");
    shell.engine.page(CARD, "Card 3");
    shell.engine.page(NEWS, "News");
    shell.engine.page(STORY, "A story");
}

/// A tab of the row as the strip is told of it: its name, and whether it is
/// a Site and asleep, as two letters: `S` kept, `z` asleep.
fn row(shell: &FakeShell, subscription: u64) -> Vec<String> {
    shell
        .window(subscription)
        .tabs
        .into_iter()
        .map(|tab| {
            let name = if tab.title.is_empty() {
                tab.host
            } else {
                tab.title
            };
            format!(
                "{name}{}{}",
                if tab.kept { " S" } else { "" },
                if tab.asleep { " z" } else { "" }
            )
        })
        .collect()
}

/// The ids of the row's tabs, in order.
fn ids(shell: &FakeShell, subscription: u64) -> Vec<u64> {
    shell
        .window(subscription)
        .tabs
        .into_iter()
        .map(|tab| tab.tab_id)
        .collect()
}

/// The tab in front: its title, and whether it is a Site.
fn front(shell: &FakeShell, subscription: u64) -> (String, bool) {
    let tab = shell.window(subscription).tab.expect("a tab");
    (tab.title, tab.kept)
}

fn front_id(shell: &FakeShell, subscription: u64) -> u64 {
    shell.window(subscription).tab.expect("a tab").tab_id
}

/// The addresses the shell was asked to load, in order.
fn loads(shell: &FakeShell) -> Vec<String> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewLoad(load) => Some(load.url.clone()),
            _ => None,
        })
        .collect()
}

/// How many web views the shell was asked to destroy.
fn destroyed(shell: &FakeShell) -> usize {
    shell
        .host_requests
        .iter()
        .filter(|request| matches!(request, host_request::Kind::WebViewDestroy(_)))
        .count()
}

fn code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

fn acked(answer: &response::Kind) {
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

/// A window with three tabs, Mail, Board and News, and News in front.
async fn three_tabs(shell: &mut FakeShell) -> (u64, u64, [u64; 3]) {
    web(shell);
    let (window, mail, subscription) = shell.open_window().await;
    shell.submit(mail, MAIL).await;
    let board = shell.open_tab(window, BOARD).await;
    let news = shell.open_tab(window, NEWS).await;
    (window, subscription, [mail, board, news])
}

#[test]
fn a_kept_tab_is_a_site_and_sites_are_the_first_of_the_row() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, [mail, board, news]) = three_tabs(&mut shell).await;
        assert_eq!(row(&shell, subscription), ["Mail", "Board", "News"]);
        let before = (loads(&shell), destroyed(&shell));

        // Kept, the tab in front becomes the first of the row: its only Site. It is still
        // in front, and nothing of its page changes.
        acked(&shell.keep_tab(news).await);
        assert_eq!(row(&shell, subscription), ["News S", "Mail", "Board"]);
        assert_eq!(front(&shell, subscription), ("News".to_owned(), true));

        // The next one kept goes after the Sites there are, wherever it was.
        acked(&shell.keep_tab(board).await);
        assert_eq!(row(&shell, subscription), ["News S", "Board S", "Mail"]);
        assert_eq!(ids(&shell, subscription), [news, board, mail]);
        // Kept again, a Site is as it was.
        acked(&shell.keep_tab(news).await);
        assert_eq!(row(&shell, subscription), ["News S", "Board S", "Mail"]);

        // Keeping loads nothing and destroys nothing.
        assert_eq!((loads(&shell), destroyed(&shell)), before);
        // A tab that is no Site says so when it is in front.
        shell.activate_tab(mail).await;
        assert_eq!(front(&shell, subscription), ("Mail".to_owned(), false));
    });
}

#[test]
fn what_cannot_be_kept() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        shell
            .engine
            .fail("https://gone.example.com/", "net.unreachable");
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;

        // An empty tab; a tab whose page never arrived; a tab there is not.
        let empty = shell.open_tab(window, "").await;
        assert_eq!(code(shell.keep_tab(empty).await), "tab.no_page");
        let failed = shell.open_tab(window, "https://gone.example.com/").await;
        assert_eq!(code(shell.keep_tab(failed).await), "tab.no_page");
        assert_eq!(code(shell.keep_tab(failed + 100).await), "tab.unknown");
        assert_eq!(code(shell.let_go_site(failed + 100).await), "tab.unknown");
        // A page that is not the web's, whatever it is called.
        shell.engine.page("about:blank", "A blank page");
        let blank = shell.open_tab(window, "about:blank").await;
        assert_eq!(code(shell.keep_tab(blank).await), "tab.no_page");

        // A tab of a private window: nothing of one is kept.
        let (_, private, private_subscription) = shell.open_private_window().await;
        shell.submit(private, BOARD).await;
        assert_eq!(code(shell.keep_tab(private).await), "window.private");
        assert_eq!(row(&shell, private_subscription), ["Board"]);

        // None of it made a Site.
        assert!(
            row(&shell, subscription)
                .iter()
                .all(|tab| !tab.contains(" S"))
        );
        // Letting go a tab that is no Site changes nothing, and is not refused.
        acked(&shell.let_go_site(mail).await);
        assert_eq!(ids(&shell, subscription), [mail, empty, failed, blank]);
    });
}

#[test]
fn closing_a_site_puts_it_to_sleep_at_its_home_and_a_press_wakes_it() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, _, news]) = three_tabs(&mut shell).await;
        shell.activate_tab(mail).await;
        acked(&shell.keep_tab(mail).await);
        // It wanders from its home.
        shell.submit(mail, MESSAGE).await;
        assert_eq!(row(&shell, subscription), ["Re: lunch S", "Board", "News"]);
        let (loaded, gone) = (loads(&shell), destroyed(&shell));

        // Closed: its web view is destroyed; it is still in the row, by the name it was
        // kept with, asleep; the tab after it comes in front.
        acked(&shell.close_tab(mail).await);
        assert_eq!(row(&shell, subscription), ["Mail S z", "Board", "News"]);
        assert_eq!(front(&shell, subscription), ("Board".to_owned(), false));
        assert_eq!(destroyed(&shell), gone + 1);
        assert_eq!(loads(&shell), loaded, "nothing is loaded by closing it");
        // Nothing was closed that could be brought back: it is still there.
        assert_eq!(shell.window(subscription).undo, "");
        let undone = shell
            .request(request::Kind::Undo(Undo { window_id: window }))
            .await;
        assert_eq!(code(undone), "undo.nothing");
        // Closed again while asleep, it is as it was: there is no page to close.
        acked(&shell.close_tab(mail).await);
        assert_eq!(row(&shell, subscription), ["Mail S z", "Board", "News"]);
        assert_eq!(destroyed(&shell), gone + 1);

        // A press wakes it: at its home, not where it had wandered.
        acked(&shell.activate_tab(mail).await);
        assert_eq!(row(&shell, subscription), ["Mail S", "Board", "News"]);
        assert_eq!(front(&shell, subscription), ("Mail".to_owned(), true));
        let mut now = loaded.clone();
        now.push(MAIL.to_owned());
        assert_eq!(loads(&shell), now);
        let _ = news;
    });
}

#[test]
fn closing_a_site_brings_forward_a_tab_that_is_awake_or_no_site_and_never_wakes_another() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news]) = three_tabs(&mut shell).await;
        for tab in [mail, board, news] {
            acked(&shell.keep_tab(tab).await);
        }
        assert_eq!(row(&shell, subscription), ["Mail S", "Board S", "News S"]);
        let loaded = loads(&shell);

        // Three Sites, all awake, the last in front. Closed, the one before comes forward:
        // it is awake.
        acked(&shell.close_tab(news).await);
        assert_eq!(front(&shell, subscription).0, "Board");
        // Closed, the one after is asleep and is passed over; the one before is awake.
        acked(&shell.close_tab(board).await);
        assert_eq!(front(&shell, subscription).0, "Mail");
        assert_eq!(
            row(&shell, subscription),
            ["Mail S", "Board S z", "News S z"]
        );
        // Closed, there is no tab that is awake and none that is no Site: an empty tab is
        // opened at the end, and comes in front. No Site was woken.
        acked(&shell.close_tab(mail).await);
        assert_eq!(
            row(&shell, subscription),
            ["Mail S z", "Board S z", "News S z", ""]
        );
        assert_eq!(front(&shell, subscription), (String::new(), false));
        assert_eq!(
            loads(&shell),
            loaded,
            "nothing was loaded: no Site was woken"
        );

        // With a tab that is no Site after it, that one comes forward, though it is
        // further along than a sleeping Site.
        let story = shell.open_tab(window, STORY).await;
        shell.activate_tab(board).await;
        acked(&shell.close_tab(board).await);
        assert_eq!(front_id(&shell, subscription), ids(&shell, subscription)[3]);

        // With a tab to come forward on either side, it is the one after: as when any
        // tab is closed.
        shell.activate_tab(mail).await;
        shell.activate_tab(board).await;
        assert_eq!(row(&shell, subscription)[..2], ["Mail S", "Board S"]);
        acked(&shell.close_tab(board).await);
        assert_eq!(
            front_id(&shell, subscription),
            ids(&shell, subscription)[3],
            "the tab after it that is no Site, not the Site before it, awake as that is"
        );
        let _ = story;
    });
}

#[test]
fn a_site_let_go_is_a_tab_like_any_other_again() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news]) = three_tabs(&mut shell).await;
        acked(&shell.keep_tab(mail).await);
        acked(&shell.keep_tab(board).await);
        assert_eq!(row(&shell, subscription), ["Mail S", "Board S", "News"]);

        // Let go, it is the first of the other tabs, as it is: awake, at its page.
        acked(&shell.let_go_site(mail).await);
        assert_eq!(row(&shell, subscription), ["Board S", "Mail", "News"]);
        assert_eq!(ids(&shell, subscription), [board, mail, news]);

        // Closed now, it is closed: gone from the row, and it can be brought back.
        acked(&shell.close_tab(mail).await);
        assert_eq!(row(&shell, subscription), ["Board S", "News"]);
        assert_eq!(shell.window(subscription).undo, "tab.closed");
        let undone = shell
            .request(request::Kind::Undo(Undo { window_id: window }))
            .await;
        assert!(matches!(undone, response::Kind::TabOpened(_)), "{undone:?}");
        // Brought back where it was, which is after the Sites.
        assert_eq!(row(&shell, subscription), ["Board S", "Mail", "News"]);

        // A sleeping Site let go is a sleeping tab, at its home.
        acked(&shell.close_tab(board).await);
        acked(&shell.let_go_site(board).await);
        assert_eq!(row(&shell, subscription), ["Board z", "Mail", "News"]);
    });
}

#[test]
fn no_tab_is_put_among_the_sites() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, board, news]) = three_tabs(&mut shell).await;
        acked(&shell.keep_tab(mail).await);
        acked(&shell.keep_tab(board).await);
        assert_eq!(ids(&shell, subscription), [mail, board, news]);

        // A tab asked for after the first Site: after the Sites.
        let answer = shell
            .request(request::Kind::OpenTab(OpenTab {
                window_id: window,
                text: String::new(),
                background: true,
                after_tab_id: mail,
            }))
            .await;
        let response::Kind::TabOpened(opened) = answer else {
            panic!("expected TabOpened, got {answer:?}");
        };
        assert_eq!(
            ids(&shell, subscription),
            [mail, board, opened.tab_id, news]
        );

        // A link of the first Site's page opened in a new tab: after the Sites too.
        shell.activate_tab(mail).await;
        let web_view = shell.window(subscription).tab.expect("a tab").web_view_id;
        shell.report(FakeEngine::link_open_requested(
            web_view, MESSAGE, true, true,
        ));
        shell.settle().await;
        let now = ids(&shell, subscription);
        assert_eq!(now.len(), 5);
        assert_eq!(now[..2], [mail, board], "the Sites are still the first");
        assert_eq!(now[3..], [opened.tab_id, news]);

        // A window the first Site's page asks for, with a click: its tab is after the
        // Sites too, and in front.
        let (_, report) = shell.engine.ask_for_window(web_view, STORY, true);
        shell.report(report);
        shell.settle().await;
        let now = ids(&shell, subscription);
        assert_eq!(now.len(), 6);
        assert_eq!(now[..2], [mail, board], "the Sites are still the first");
        assert_eq!(front_id(&shell, subscription), now[2]);

        // A closed tab that was the first of the row when there were no Sites comes back
        // after them.
        acked(&shell.let_go_site(mail).await);
        acked(&shell.let_go_site(board).await);
        assert_eq!(ids(&shell, subscription)[..2], [board, mail]);
        shell.close_tab(board).await;
        acked(&shell.keep_tab(mail).await);
        let undone = shell
            .request(request::Kind::Undo(Undo { window_id: window }))
            .await;
        assert!(matches!(undone, response::Kind::TabOpened(_)), "{undone:?}");
        let now = row(&shell, subscription);
        assert_eq!(now[0], "Mail S");
        assert_eq!(now[1], "Board", "brought back, in front, after the Site");
    });
}

#[test]
fn a_step_along_the_row_wakes_a_sleeping_site_and_it_is_offered_while_typing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, subscription, [mail, _, news]) = three_tabs(&mut shell).await;
        acked(&shell.keep_tab(mail).await);
        shell.activate_tab(mail).await;
        shell.submit(mail, MESSAGE).await;
        acked(&shell.close_tab(mail).await);
        shell.activate_tab(news).await;
        let loaded = loads(&shell);

        // Asleep, it is offered while typing in another tab: by its name and its home,
        // not by where it had wandered.
        let answer = shell
            .request(request::Kind::SuggestAddresses(SuggestAddresses {
                window_id: window,
                text: "mail".to_owned(),
                limit: 0,
            }))
            .await;
        let response::Kind::AddressSuggestions(offered) = answer else {
            panic!("expected suggestions, got {answer:?}");
        };
        let first = &offered.suggestions[0];
        assert_eq!(
            (first.tab_id, first.url.as_str(), first.title.as_str()),
            (mail, MAIL, "Mail")
        );
        assert_eq!(loads(&shell), loaded, "being offered wakes nothing");

        // A step to the first tab of the row wakes it, at its home.
        let answer = shell
            .request(request::Kind::StepTab(StepTab {
                window_id: window,
                to: To::Place.into(),
                place: 1,
            }))
            .await;
        acked(&answer);
        assert_eq!(front(&shell, subscription), ("Mail".to_owned(), true));
        assert_eq!(loads(&shell).last().map(String::as_str), Some(MAIL));
        assert_eq!(loads(&shell).len(), loaded.len() + 1);
    });
}

/// Each Site of the row: the name it was kept with, the name the strip shows for it now, and
/// whether it is away from its home.
fn sites(shell: &FakeShell, subscription: u64) -> Vec<(String, String, bool)> {
    shell
        .window(subscription)
        .tabs
        .into_iter()
        .filter(|tab| tab.kept)
        .map(|tab| (tab.site_name, tab.title, tab.away))
        .collect()
}

fn site(name: &str, shown: &str, away: bool) -> (String, String, bool) {
    (name.to_owned(), shown.to_owned(), away)
}

#[test]
fn a_site_that_has_wandered_is_away_and_is_sent_back_to_its_home() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, subscription, [mail, board, _]) = three_tabs(&mut shell).await;
        shell.activate_tab(mail).await;
        acked(&shell.keep_tab(mail).await);
        // At its home it is not away, and is known by the name it was kept with.
        assert_eq!(sites(&shell, subscription), [site("Mail", "Mail", false)]);
        let front = shell.window(subscription).tab.expect("a tab");
        assert_eq!((front.away, front.site_name.as_str()), (false, "Mail"));
        // A tab that is no Site has no such name, and is never away.
        assert!(
            shell
                .window(subscription)
                .tabs
                .iter()
                .filter(|tab| !tab.kept)
                .all(|tab| tab.site_name.is_empty() && !tab.away)
        );

        // It goes somewhere else: away, under the page's title, and still the Site it was.
        shell.submit(mail, MESSAGE).await;
        assert_eq!(
            sites(&shell, subscription),
            [site("Mail", "Re: lunch", true)]
        );
        let front = shell.window(subscription).tab.expect("a tab");
        assert_eq!((front.away, front.site_name.as_str()), (true, "Mail"));
        let loaded = loads(&shell);

        // Sent back: its home is loaded, in the same tab, and it is at home again.
        acked(&shell.send_site_home(mail).await);
        let mut now = loaded.clone();
        now.push(MAIL.to_owned());
        assert_eq!(loads(&shell), now);
        assert_eq!(sites(&shell, subscription), [site("Mail", "Mail", false)]);
        assert_eq!(front_id(&shell, subscription), mail);
        assert_eq!(
            ids(&shell, subscription).len(),
            3,
            "no tab was opened or closed"
        );

        // At its home, there is nowhere to send it: nothing is loaded, nothing refused.
        acked(&shell.send_site_home(mail).await);
        assert_eq!(loads(&shell), now);
        // A tab that is no Site, likewise; and a tab there is not is refused.
        acked(&shell.send_site_home(board).await);
        assert_eq!(loads(&shell), now);
        assert_eq!(code(shell.send_site_home(board + 100).await), "tab.unknown");

        // A page that changes its own address, as a one-page site does, is away too.
        let web_view = shell.window(subscription).tab.expect("a tab").web_view_id;
        shell.report(FakeEngine::address_changed(web_view, MESSAGE));
        shell.settle().await;
        assert!(
            sites(&shell, subscription)[0].2,
            "away by the page's own doing"
        );

        // Asleep, it is at its home: not away, and sending it home wakes nothing.
        acked(&shell.close_tab(mail).await);
        assert_eq!(sites(&shell, subscription), [site("Mail", "Mail", false)]);
        acked(&shell.send_site_home(mail).await);
        assert_eq!(
            loads(&shell),
            now,
            "a sleeping Site is not woken to be sent where it is"
        );

        // Away again, and let go: it is a tab, with no home to be away from.
        shell.activate_tab(mail).await;
        shell.submit(mail, MESSAGE).await;
        assert!(sites(&shell, subscription)[0].2);
        acked(&shell.let_go_site(mail).await);
        assert_eq!(sites(&shell, subscription), []);
        let before = loads(&shell);
        acked(&shell.send_site_home(mail).await);
        assert_eq!(loads(&shell), before);
    });
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

#[test]
fn sites_come_back_after_a_quit_asleep_at_home_but_for_the_one_in_front() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let board = shell.open_tab(window, BOARD).await;
        let news = shell.open_tab(window, NEWS).await;
        acked(&shell.keep_tab(mail).await);
        acked(&shell.keep_tab(board).await);
        // Both wander; the second is in front at the quit.
        shell.submit(mail, MESSAGE).await;
        shell.activate_tab(board).await;
        shell.submit(board, CARD).await;
        assert_eq!(
            row(&shell, subscription),
            ["Re: lunch S", "Card 3 S", "News"]
        );
        let _ = news;
        drop(shell);

        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        // The Sites are Sites, in their order. The one that was not in front is asleep
        // at its home, by its name; the one in front is loaded where it was.
        assert_eq!(
            row(&shell, subscription),
            ["Mail S z", "Card 3 S", "News z"]
        );
        assert_eq!(loads(&shell), [CARD]);
        assert_eq!(front(&shell, subscription), ("Card 3".to_owned(), true));
        // The one in front is away from its home, as it was, and has the name it was kept
        // with; the other is at its home.
        assert_eq!(
            sites(&shell, subscription),
            [site("Mail", "Mail", false), site("Board", "Card 3", true)]
        );

        // The one in front still knows its home and its name: closed, it sleeps there.
        let board = front_id(&shell, subscription);
        acked(&shell.close_tab(board).await);
        assert_eq!(row(&shell, subscription)[..2], ["Mail S z", "Board S z"]);
        // And a Site woken after the quit wakes at its home.
        let mail = ids(&shell, subscription)[0];
        acked(&shell.activate_tab(mail).await);
        assert_eq!(loads(&shell).last().map(String::as_str), Some(MAIL));
        drop(shell);

        // A second quit, with a Site let go in between: it comes back as a tab.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(row(&shell, subscription)[..2], ["Mail S", "Board S z"]);
        let board = ids(&shell, subscription)[1];
        acked(&shell.let_go_site(board).await);
        drop(shell);
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        assert_eq!(row(&shell, subscription)[..2], ["Mail S", "Board z"]);
        let _ = window;
    });
}

#[test]
fn a_tab_that_came_back_asleep_is_offered_while_typing() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, _) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        shell.open_tab(window, NEWS).await;
        drop(shell);

        // After a quit the tab that was not in front is asleep (009): it is an open tab
        // all the same, and is offered as one (038).
        let mut shell = start(folder.path());
        let (window, _, subscription) = shell.open_window().await;
        assert_eq!(row(&shell, subscription), ["Mail z", "News"]);
        let answer = shell
            .request(request::Kind::SuggestAddresses(SuggestAddresses {
                window_id: window,
                text: "mail".to_owned(),
                limit: 0,
            }))
            .await;
        let response::Kind::AddressSuggestions(offered) = answer else {
            panic!("expected suggestions, got {answer:?}");
        };
        let first = &offered.suggestions[0];
        assert_eq!(first.tab_id, ids(&shell, subscription)[0]);
        assert_eq!(first.url, MAIL);
        // Chosen, it wakes.
        acked(&shell.choose_tab(first.tab_id).await);
        assert_eq!(loads(&shell), [NEWS, MAIL]);
    });
}
