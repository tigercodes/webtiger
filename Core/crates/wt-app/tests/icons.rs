//! A site's icon (046), where an engine hands it over. Through the contract,
//! against the scripted engine: a page's icon is kept for its host and the
//! window told by number; what is not an icon, or not that page's, is
//! ignored; a Site has its home's; each login set has its own; a page is
//! heard once a minute; and icons outlive a quit.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::Path;

use wt_core::Timestamp;

use wt_proto::v1::{
    CloseWindow, CreateSpace, ProceedPastCertificate, SwitchSpace, request, response,
};
use wt_testkit::{FakeEngine, FakeShell, png, run};

const MAIL: &str = "https://mail.example.com/";
const INBOX: &str = "https://mail.example.com/inbox";
const BOARD: &str = "https://board.example.com/";
const NEWS: &str = "https://news.example.com/";

fn web(shell: &mut FakeShell) {
    shell.engine.page(MAIL, "Mail");
    shell.engine.page(INBOX, "Inbox");
    shell.engine.page(BOARD, "Board");
    shell.engine.page(NEWS, "News");
}

/// The row as a window is told of it: each tab's name and the number of its
/// icon.
fn row(shell: &FakeShell, subscription: u64) -> Vec<(String, u64)> {
    shell
        .window(subscription)
        .tabs
        .into_iter()
        .map(|tab| (tab.title, tab.icon_id))
        .collect()
}

/// The number of the icon of the tab called this.
fn icon_of(shell: &FakeShell, subscription: u64, name: &str) -> u64 {
    row(shell, subscription)
        .into_iter()
        .find(|(title, _)| title == name)
        .unwrap_or_else(|| panic!("no tab is called {name}"))
        .1
}

/// The web view of the tab in front.
fn web_view(shell: &FakeShell, subscription: u64) -> u64 {
    shell
        .window(subscription)
        .tab
        .expect("a tab in front")
        .web_view_id
}

/// The engine hands over a picture as the icon of the page the tab in front
/// shows, or of the page named.
async fn hand_over(shell: &mut FakeShell, subscription: u64, picture: &[u8], page: &str) {
    let view = web_view(shell, subscription);
    shell.report(FakeEngine::icon_received(view, picture, page));
    shell.settle().await;
}

/// The pictures the core has for these numbers, each with its number.
async fn pictures(shell: &mut FakeShell, numbers: &[u64]) -> Vec<(u64, Vec<u8>)> {
    shell
        .query_icons(numbers)
        .await
        .into_iter()
        .map(|icon| (icon.icon_id, icon.png))
        .collect()
}

fn acked(answer: &response::Kind) {
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

#[test]
fn a_pages_icon_is_kept_for_its_host_and_the_window_is_told_by_number() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        // No icon yet: the tab has none, and there is no picture to ask for.
        assert_eq!(row(&shell, subscription), [("Mail".to_owned(), 0)]);
        assert_eq!(pictures(&mut shell, &[0, 1, 2]).await, []);
        let asked = shell.host_requests.len();

        // The engine hands the page's icon over: the tab has a number, and the number is
        // that picture. The shell was asked for nothing.
        let first = png(64, 64, 1);
        hand_over(&mut shell, subscription, &first, MAIL).await;
        let one = icon_of(&shell, subscription, "Mail");
        assert_ne!(one, 0);
        assert_eq!(pictures(&mut shell, &[one]).await, [(one, first.clone())]);
        assert_eq!(shell.host_requests.len(), asked);

        // Another page of the same host has the same icon, from the moment it is there; a
        // page of another host has none.
        let inbox = shell.open_tab(window, INBOX).await;
        let board = shell.open_tab(window, BOARD).await;
        assert_eq!(
            row(&shell, subscription),
            [
                ("Mail".to_owned(), one),
                ("Inbox".to_owned(), one),
                ("Board".to_owned(), 0)
            ]
        );
        // The other host's own: another number, and each number its own picture, in the
        // order asked; a number nobody has is left out.
        let second = png(32, 32, 2);
        hand_over(&mut shell, subscription, &second, BOARD).await;
        let two = icon_of(&shell, subscription, "Board");
        assert!(two != 0 && two != one);
        assert_eq!(icon_of(&shell, subscription, "Mail"), one);
        assert_eq!(
            pictures(&mut shell, &[two, 999, one]).await,
            [(two, second.clone()), (one, first.clone())]
        );

        // The same picture again, from another page of the host: the number it has.
        acked(&shell.activate_tab(inbox).await);
        hand_over(&mut shell, subscription, &first, INBOX).await;
        assert_eq!(icon_of(&shell, subscription, "Inbox"), one);
        // Another picture, when the host's page next arrives: another number, for every
        // tab of the host; the number it had is nobody's, and a number is never used twice.
        acked(&shell.activate_tab(mail).await);
        shell.submit(mail, INBOX).await;
        let third = png(64, 64, 3);
        hand_over(&mut shell, subscription, &third, INBOX).await;
        let three = icon_of(&shell, subscription, "Inbox");
        assert!(three != 0 && three != one && three != two);
        assert_eq!(
            row(&shell, subscription),
            [
                ("Inbox".to_owned(), three),
                ("Inbox".to_owned(), three),
                ("Board".to_owned(), two)
            ]
        );
        assert_eq!(
            pictures(&mut shell, &[one, two, three]).await,
            [(two, second), (three, third)]
        );
        let _ = board;
    });
}

#[test]
fn what_is_not_an_icon_or_is_not_the_pages_is_ignored() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        shell
            .engine
            .page("http://plain.example.com/", "A plain page");
        shell
            .engine
            .page("http://127.0.0.1:8080/", "On this device");
        shell
            .engine
            .fail("https://gone.example.com/", "net.unreachable");
        shell
            .engine
            .untrusted("https://router.example.com/", "Router", "net.tls.untrusted");
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let good = png(64, 64, 1);
        let none = |shell: &FakeShell| {
            row(shell, subscription)
                .iter()
                .all(|(_, number)| *number == 0)
        };

        // Not a picture the core keeps: no bytes; another kind of picture; one too wide.
        for picture in [
            Vec::new(),
            b"GIF89a and the rest of a small picture".to_vec(),
            png(257, 64, 1),
            png(64, 0, 1),
        ] {
            hand_over(&mut shell, subscription, &picture, MAIL).await;
            assert!(
                none(&shell),
                "{} bytes were taken for an icon",
                picture.len()
            );
        }
        // Not this page's: of another host, of no page, of an address that is none.
        for page in [BOARD, "", "not an address", "about:blank"] {
            hand_over(&mut shell, subscription, &good, page).await;
            assert!(none(&shell), "the icon of <{page}> was taken for the tab's");
        }
        // Of a web view that is no tab's.
        let view = web_view(&shell, subscription);
        shell.report(FakeEngine::icon_received(view + 100, &good, MAIL));
        shell.settle().await;
        assert!(none(&shell));
        // None of those used up the page's turn: its own icon is taken now.
        hand_over(&mut shell, subscription, &good, MAIL).await;
        assert_ne!(icon_of(&shell, subscription, "Mail"), 0);

        // A page that did not come over https has none; one that never left this device
        // has.
        shell.open_tab(window, "http://plain.example.com/").await;
        hand_over(&mut shell, subscription, &good, "http://plain.example.com/").await;
        assert_eq!(icon_of(&shell, subscription, "A plain page"), 0);
        shell.open_tab(window, "http://127.0.0.1:8080/").await;
        hand_over(&mut shell, subscription, &good, "http://127.0.0.1:8080/").await;
        assert_ne!(icon_of(&shell, subscription, "On this device"), 0);

        // A page that failed has none, whatever the engine hands over for its address.
        let gone = shell.open_tab(window, "https://gone.example.com/").await;
        hand_over(&mut shell, subscription, &good, "https://gone.example.com/").await;
        assert_eq!(shell.window(subscription).tab.unwrap().tab_id, gone);
        assert_eq!(
            row(&shell, subscription).last().map(|(_, number)| *number),
            Some(0)
        );

        // A page whose certificate was gone on past (026) has none.
        let router = shell.open_tab(window, "https://router.example.com/").await;
        let answer = shell
            .request(request::Kind::ProceedPastCertificate(
                ProceedPastCertificate { tab_id: router },
            ))
            .await;
        acked(&answer);
        assert_eq!(shell.window(subscription).tab.unwrap().title, "Router");
        hand_over(
            &mut shell,
            subscription,
            &good,
            "https://router.example.com/",
        )
        .await;
        assert_eq!(icon_of(&shell, subscription, "Router"), 0);
    });
}

#[test]
fn a_site_has_its_homes_icon_wherever_it_has_wandered() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let home = png(64, 64, 1);
        hand_over(&mut shell, subscription, &home, MAIL).await;
        let one = icon_of(&shell, subscription, "Mail");
        acked(&shell.keep_tab(mail).await);

        // The Site wanders to another host: it has its home's icon still. The icon the
        // engine hands over there is that host's, and not the Site's.
        shell.submit(mail, BOARD).await;
        assert_eq!(row(&shell, subscription), [("Board".to_owned(), one)]);
        let away = png(64, 64, 2);
        hand_over(&mut shell, subscription, &away, BOARD).await;
        assert_eq!(row(&shell, subscription), [("Board".to_owned(), one)]);
        // A tab that is no Site, at that host, has that host's.
        shell.open_tab(window, BOARD).await;
        let two = icon_of(&shell, subscription, "Board");
        let tabs = row(&shell, subscription);
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0].1, one, "the Site, first in the row, has its home's");
        assert!(tabs[1].1 != 0 && tabs[1].1 != one);
        let _ = two;

        // Let go, the tab is at the page it shows: it has that page's icon.
        acked(&shell.let_go_site(mail).await);
        let tabs = row(&shell, subscription);
        assert_eq!(tabs[0].1, tabs[1].1);
        assert_ne!(tabs[0].1, one);
    });
}

#[test]
fn an_icon_that_comes_before_its_page_has_arrived_waits_for_it() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let view = web_view(&shell, subscription);
        let board = png(64, 64, 1);

        // The tab goes to another site by itself, and the engine has that page's icon a
        // moment before it says the page has arrived (as an engine may, for an icon the
        // page carries in itself). The tab still shows the page it had: it has no icon.
        let mut on_its_way = shell.engine.goes_to(view, BOARD).into_iter();
        shell.report(on_its_way.next().expect("the load begins"));
        shell.report(FakeEngine::icon_received(view, &board, BOARD));
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [("Mail".to_owned(), 0)]);
        // The page arrives: the icon that waited is its icon.
        for report in on_its_way {
            shell.report(report);
        }
        shell.settle().await;
        let one = icon_of(&shell, subscription, "Board");
        assert_ne!(one, 0);
        assert_eq!(pictures(&mut shell, &[one]).await, [(one, board)]);

        // An icon handed over for a page on its way, when another page arrives: it is
        // nobody's. Not the page's that arrived, and not, later, its own host's.
        let mut on_its_way = shell.engine.goes_to(view, MAIL).into_iter();
        shell.report(on_its_way.next().expect("the load begins"));
        shell.report(FakeEngine::icon_received(view, &png(64, 64, 2), NEWS));
        for report in on_its_way {
            shell.report(report);
        }
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [("Mail".to_owned(), 0)]);
        // The same tab then goes to the site the icon named: it waits no longer, and the
        // page that arrives there has none.
        for report in shell.engine.goes_to(view, NEWS) {
            shell.report(report);
        }
        shell.settle().await;
        assert_eq!(row(&shell, subscription), [("News".to_owned(), 0)]);
        shell.open_tab(window, NEWS).await;
        assert_eq!(icon_of(&shell, subscription, "News"), 0);

        // One handed over for a page that never arrives, because its load fails: nobody's
        // either, though the tab then goes to that very page.
        let news = web_view(&shell, subscription);
        shell.report(FakeEngine::navigation_started(news, BOARD));
        shell.report(FakeEngine::icon_received(news, &png(64, 64, 3), INBOX));
        shell.report(FakeEngine::navigation_failed(news, "net.connect", BOARD));
        shell.settle().await;
        for report in shell.engine.goes_to(news, INBOX) {
            shell.report(report);
        }
        shell.settle().await;
        assert_eq!(icon_of(&shell, subscription, "Inbox"), 0);
        // A page that is not on its way to anywhere is not waited for: its tab is at
        // rest, and an icon for another site is ignored (and is not there when the tab
        // does go to that site).
        shell.report(FakeEngine::icon_received(news, &png(64, 64, 4), NEWS));
        shell.settle().await;
        for report in shell.engine.goes_to(news, NEWS) {
            shell.report(report);
        }
        shell.settle().await;
        assert_eq!(icon_of(&shell, subscription, "News"), 0);
    });
}

const MAIL_ICON: &str = "https://mail.example.com/icon-64.png";
const MAIL_SMALL: &str = "https://mail.example.com/icon-16.png";
const MAIL_OTHER: &str = "https://mail.example.com/unread.png";

/// The addresses the shell was asked to fetch as icons, in order.
fn fetched(shell: &FakeShell) -> Vec<String> {
    shell
        .engine
        .icon_fetches
        .iter()
        .map(|(_, address)| address.clone())
        .collect()
}

#[test]
fn a_page_that_names_its_icons_has_one_fetched_once_in_a_run_and_kept() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        // An engine that hands an app no icon: its pages say which icons they name.
        let icons = [
            (MAIL_SMALL, "icon", "16x16", "image/png"),
            (MAIL_ICON, "icon", "64x64", "image/png"),
        ];
        shell.engine.names_icons(MAIL, &icons);
        shell.engine.names_icons(INBOX, &icons);
        let picture = png(64, 64, 1);
        shell.engine.picture(MAIL_ICON, &picture);
        shell.engine.picture(MAIL_SMALL, &png(16, 16, 9));
        let (window, mail, subscription) = shell.open_window().await;

        // The page arrives and names its icons: the shell is asked for the one the core
        // chose, and what it hands over is the tab's icon.
        shell.submit(mail, MAIL).await;
        assert_eq!(fetched(&shell), [MAIL_ICON]);
        let one = icon_of(&shell, subscription, "Mail");
        assert_ne!(one, 0);
        assert_eq!(pictures(&mut shell, &[one]).await, [(one, picture.clone())]);

        // The page again, another page of the host, and another tab at it: none is asked
        // for the same address a second time in the run.
        shell.submit(mail, MAIL).await;
        shell.submit(mail, INBOX).await;
        shell.open_tab(window, MAIL).await;
        assert_eq!(fetched(&shell), [MAIL_ICON]);
        assert_eq!(icon_of(&shell, subscription, "Inbox"), one);

        // The host's page names another address: that one is asked for, and is the host's
        // icon from then.
        let other = png(64, 64, 2);
        shell.engine.picture(MAIL_OTHER, &other);
        shell
            .engine
            .names_icons(MAIL, &[(MAIL_OTHER, "icon", "64x64", "image/png")]);
        let front = shell.window(subscription).tab.unwrap().tab_id;
        shell.submit(front, MAIL).await;
        assert_eq!(fetched(&shell), [MAIL_ICON, MAIL_OTHER]);
        let two = icon_of(&shell, subscription, "Mail");
        assert!(two != 0 && two != one);
        assert_eq!(pictures(&mut shell, &[two]).await, [(two, other)]);
        // Another page of the host names the first address still: it was asked for once,
        // and is not asked for again as the tab goes between the two pages.
        shell.submit(front, INBOX).await;
        shell.submit(front, MAIL).await;
        shell.submit(front, INBOX).await;
        assert_eq!(fetched(&shell), [MAIL_ICON, MAIL_OTHER]);

        // Another login set asks for its own: a private window, at the same page.
        let (_, tab, private_subscription) = shell.open_private_window().await;
        shell.submit(tab, MAIL).await;
        assert_eq!(fetched(&shell), [MAIL_ICON, MAIL_OTHER, MAIL_OTHER]);
        assert_ne!(icon_of(&shell, private_subscription, "Mail"), 0);
    });
}

#[test]
fn nothing_is_asked_for_a_page_whose_icon_would_not_be_kept() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        shell
            .engine
            .page("http://plain.example.com/", "A plain page");
        shell.engine.names_icons(
            "http://plain.example.com/",
            &[("https://plain.example.com/icon.png", "icon", "64x64", "")],
        );
        shell
            .engine
            .untrusted("https://router.example.com/", "Router", "net.tls.untrusted");
        shell.engine.names_icons(
            "https://router.example.com/",
            &[("https://router.example.com/icon.png", "icon", "64x64", "")],
        );
        // A page that names no icon at all: the one every site may have is asked for.
        shell.engine.names_icons(BOARD, &[]);
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, BOARD).await;
        assert_eq!(fetched(&shell), ["https://board.example.com/favicon.ico"]);
        // There is nothing there: the tab has no icon, and the site is not asked again.
        assert_eq!(icon_of(&shell, subscription, "Board"), 0);
        shell.submit(mail, BOARD).await;
        assert_eq!(fetched(&shell).len(), 1);

        // Icons named for a page the tab does not show: of another host, of no address;
        // and for a web view that is nobody's.
        let view = web_view(&shell, subscription);
        let named = [("https://news.example.com/icon.png", "icon", "64x64", "")];
        shell.report(FakeEngine::icon_named(view, NEWS, &named));
        shell.report(FakeEngine::icon_named(view, "", &named));
        shell.report(FakeEngine::icon_named(view + 100, BOARD, &named));
        shell.settle().await;
        assert_eq!(fetched(&shell).len(), 1);
        // Named for the page it shows, they are asked for.
        shell.report(FakeEngine::icon_named(view, BOARD, &named));
        shell.settle().await;
        assert_eq!(
            fetched(&shell),
            [
                "https://board.example.com/favicon.ico",
                "https://news.example.com/icon.png"
            ]
        );

        // A page that did not come over https; a page whose certificate was gone on past.
        shell.open_tab(window, "http://plain.example.com/").await;
        let router = shell.open_tab(window, "https://router.example.com/").await;
        let answer = shell
            .request(request::Kind::ProceedPastCertificate(
                ProceedPastCertificate { tab_id: router },
            ))
            .await;
        acked(&answer);
        assert_eq!(shell.window(subscription).tab.unwrap().title, "Router");
        assert_eq!(fetched(&shell).len(), 2);

        // An engine that says nothing of icons is asked for nothing.
        shell.open_tab(window, NEWS).await;
        assert_eq!(fetched(&shell).len(), 2);
    });
}

async fn make_space(shell: &mut FakeShell, name: &str) -> u64 {
    let answer = shell
        .request(request::Kind::CreateSpace(CreateSpace {
            name: name.to_owned(),
            share_logins_with: 0,
            colour: String::new(),
        }))
        .await;
    match answer {
        response::Kind::SpaceCreated(made) => made.space_id,
        other => panic!("CreateSpace answers SpaceCreated, not {other:?}"),
    }
}

#[test]
fn each_login_set_has_its_own_icons_and_a_private_windows_go_with_it() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let mine = png(64, 64, 1);
        hand_over(&mut shell, subscription, &mine, MAIL).await;
        let one = icon_of(&shell, subscription, "Mail");

        // A Space with logins of its own, at the same host: no icon there until its own
        // engine hands one over; then its own number and its own picture.
        let work = make_space(&mut shell, "Work").await;
        let answer = shell
            .request(request::Kind::SwitchSpace(SwitchSpace {
                window_id: window,
                space_id: work,
            }))
            .await;
        acked(&answer);
        let there = shell.window(subscription).tab.unwrap().tab_id;
        shell.submit(there, MAIL).await;
        assert_eq!(row(&shell, subscription), [("Mail".to_owned(), 0)]);
        let theirs = png(64, 64, 2);
        hand_over(&mut shell, subscription, &theirs, MAIL).await;
        let two = icon_of(&shell, subscription, "Mail");
        assert!(two != 0 && two != one);
        assert_eq!(
            pictures(&mut shell, &[one, two]).await,
            [(one, mine.clone()), (two, theirs)]
        );

        // A private window, at the same host: none until its own; its own while it is
        // open; and when it closes, its number is nobody's.
        let (private, tab, private_subscription) = shell.open_private_window().await;
        shell.submit(tab, MAIL).await;
        assert_eq!(row(&shell, private_subscription), [("Mail".to_owned(), 0)]);
        let secret = png(64, 64, 3);
        hand_over(&mut shell, private_subscription, &secret, MAIL).await;
        let three = icon_of(&shell, private_subscription, "Mail");
        assert!(three != 0 && three != one && three != two);
        assert_eq!(
            pictures(&mut shell, &[three]).await,
            [(three, secret.clone())]
        );
        let answer = shell
            .request(request::Kind::CloseWindow(CloseWindow {
                window_id: private,
            }))
            .await;
        acked(&answer);
        assert_eq!(pictures(&mut shell, &[three]).await, []);
        // The others are as they were; and another private window starts with none.
        assert_eq!(pictures(&mut shell, &[one]).await, [(one, mine)]);
        let (_, tab, another) = shell.open_private_window().await;
        shell.submit(tab, MAIL).await;
        assert_eq!(row(&shell, another), [("Mail".to_owned(), 0)]);
    });
}

#[test]
fn a_page_that_changes_its_icon_is_heard_once_a_minute() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        hand_over(&mut shell, subscription, &png(64, 64, 1), MAIL).await;
        let one = icon_of(&shell, subscription, "Mail");

        // Another picture a moment later is not heard: the tab has the icon it had.
        hand_over(&mut shell, subscription, &png(64, 64, 2), MAIL).await;
        assert_eq!(icon_of(&shell, subscription, "Mail"), one);
        // The page goes on changing it. (Fake mode's clock moves a second each time it
        // is read, and each picture handed over reads it.) Within the minute none is
        // heard; when the minute is up, one is.
        let mut refused = 1;
        let mut mark = 3;
        while icon_of(&shell, subscription, "Mail") == one {
            hand_over(&mut shell, subscription, &png(64, 64, mark), MAIL).await;
            refused += 1;
            mark += 1;
            assert!(
                refused <= 70,
                "a minute has gone by and the page is not heard"
            );
        }
        assert!(
            refused >= 50,
            "the page was heard again after {refused} seconds"
        );
        // And then not again for a minute.
        let now = icon_of(&shell, subscription, "Mail");
        hand_over(&mut shell, subscription, &png(64, 64, 200), MAIL).await;
        assert_eq!(icon_of(&shell, subscription, "Mail"), now);

        // A page that arrives is heard at once, however lately the one before it was.
        shell.submit(mail, INBOX).await;
        hand_over(&mut shell, subscription, &png(64, 64, 201), INBOX).await;
        assert_ne!(icon_of(&shell, subscription, "Inbox"), now);
    });
}

#[test]
fn an_answer_has_at_most_sixty_four_pictures() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, first, subscription) = shell.open_window().await;
        let mut numbers = Vec::new();
        for site in 0..66_u8 {
            let address = format!("https://site{site}.example.com/");
            let name = format!("Site {site}");
            shell.engine.page(&address, &name);
            if site == 0 {
                shell.submit(first, &address).await;
            } else {
                shell.open_tab(window, &address).await;
            }
            hand_over(&mut shell, subscription, &png(16, 16, site), &address).await;
            numbers.push(icon_of(&shell, subscription, &name));
        }
        // All sixty-six asked for: the first sixty-four, in the order asked.
        let answered = pictures(&mut shell, &numbers).await;
        assert_eq!(answered.len(), 64);
        assert_eq!(
            answered
                .iter()
                .map(|(number, _)| *number)
                .collect::<Vec<_>>(),
            numbers[..64]
        );
        assert_eq!(answered[5].1, png(16, 16, 5));
        // The rest, asked for in turn.
        let rest = pictures(&mut shell, &numbers[64..]).await;
        assert_eq!(rest.len(), 2);
        assert_eq!(rest[1].1, png(16, 16, 65));
    });
}

fn start(folder: &Path) -> FakeShell {
    let mut shell = FakeShell::live(folder);
    web(&mut shell);
    shell
}

#[test]
fn icons_outlive_a_quit_and_a_sleeping_tab_has_its_own() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let mine = png(64, 64, 1);
        hand_over(&mut shell, subscription, &mine, MAIL).await;
        shell.open_tab(window, BOARD).await;
        let board = png(32, 32, 2);
        hand_over(&mut shell, subscription, &board, BOARD).await;
        shell.open_tab(window, NEWS).await;
        // A private window's icon is in no store on disk.
        let (_, tab, private_subscription) = shell.open_private_window().await;
        shell.submit(tab, NEWS).await;
        hand_over(&mut shell, private_subscription, &png(64, 64, 9), NEWS).await;
        assert_ne!(icon_of(&shell, private_subscription, "News"), 0);
        drop(shell);

        // After a quit the tabs come back, the one that was in front loaded and the
        // others asleep: each has the icon its host had, from the first the window is
        // told, and the picture is the one that was kept. The host that had none has none.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let tabs = row(&shell, subscription);
        assert_eq!(
            tabs.iter()
                .map(|(name, _)| name.as_str())
                .collect::<Vec<_>>(),
            ["Mail", "Board", "News"]
        );
        let (one, two) = (tabs[0].1, tabs[1].1);
        assert!(one != 0 && two != 0 && one != two);
        assert_eq!(tabs[2].1, 0, "what a private window had was never written");
        // The window was never told of those tabs without their icons: the store is
        // asked before the first word of them, not after.
        for told in shell.snapshots(subscription) {
            for tab in told.tabs {
                assert_eq!(
                    tab.icon_id == 0,
                    tab.title == "News",
                    "{} was told with icon {}",
                    tab.title,
                    tab.icon_id
                );
            }
        }
        assert_eq!(
            pictures(&mut shell, &[one, two]).await,
            [(one, mine), (two, board)]
        );
    });
}

#[test]
fn the_icons_that_tabs_show_are_kept_when_the_store_is_full() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = start(folder.path());
        let (window, mail, subscription) = shell.open_window().await;
        shell.submit(mail, MAIL).await;
        let mine = png(64, 64, 1);
        hand_over(&mut shell, subscription, &mine, MAIL).await;
        shell.open_tab(window, BOARD).await;
        drop(shell);

        // The store is filled to the thousand it keeps, and the one icon a tab shows is
        // made the one used longest ago of them all.
        let mut store = wt_store::Store::open(&folder.path().join("webtiger.sqlite")).unwrap();
        let login_set = store.spaces().unwrap()[0].login_set;
        for number in 1..i64::from(wt_store::MAX_ICONS) {
            store
                .set_icon(
                    login_set,
                    &format!("old{number}.example.com"),
                    b"an old picture",
                    Timestamp(1000 + number),
                )
                .unwrap();
        }
        store
            .use_icons(login_set, &["mail.example.com"], Timestamp(1))
            .unwrap();
        assert!(store.icon(login_set, "old1.example.com").unwrap().is_some());
        drop(store);

        // The app starts again: its tabs come back, and the sleeping tab's icon is used
        // now, because a tab shows it. One icon more, and the one that goes is the oldest
        // of those no tab shows.
        let mut shell = start(folder.path());
        let (_, _, subscription) = shell.open_window().await;
        let one = icon_of(&shell, subscription, "Mail");
        assert_ne!(one, 0);
        hand_over(&mut shell, subscription, &png(32, 32, 2), BOARD).await;
        assert_ne!(icon_of(&shell, subscription, "Board"), 0);
        assert_eq!(pictures(&mut shell, &[one]).await, [(one, mine)]);
        drop(shell);
        let store = wt_store::Store::open(&folder.path().join("webtiger.sqlite")).unwrap();
        assert_eq!(store.icon(login_set, "old1.example.com").unwrap(), None);
        assert!(store.icon(login_set, "old2.example.com").unwrap().is_some());
    });
}
