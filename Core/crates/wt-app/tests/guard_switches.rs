//! Guard's switches (067), through the contract, against the scripted
//! engine: everywhere's, a Space's and a site's; the nearest that says
//! anything wins; what a window is told of the page in front; which tab is
//! reloaded; what is refused; Spaces that share logins; a private window;
//! what is kept across a start; and whoever answers the fast path told of a
//! web view before the shell is asked for it.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    CreateSpace, GuardLevel, GuardSwitch, NavigationCommand, SetGuard, SwitchSpace, host_request,
    request, response, set_guard,
};
use wt_testkit::{FakeShell, run};

const NEWS: &str = "https://news.example.com/story";
const SHOP: &str = "https://shop.example.com/basket";
const OTHER: &str = "https://www.example.org/";

fn web(shell: &mut FakeShell) {
    for (url, title) in [(NEWS, "News"), (SHOP, "Shop"), (OTHER, "Other")] {
        shell.engine.page(url, title);
    }
}

use GuardLevel::{Everywhere, Site, Space};
use GuardSwitch::{Off, On, Unspecified};

async fn set(shell: &mut FakeShell, of: set_guard::Of, switch: GuardSwitch) -> response::Kind {
    shell
        .request(request::Kind::SetGuard(SetGuard {
            of: Some(of),
            switch: switch.into(),
        }))
        .await
}

async fn set_ok(shell: &mut FakeShell, of: set_guard::Of, switch: GuardSwitch) {
    let answer = set(shell, of, switch).await;
    assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
}

fn everywhere() -> set_guard::Of {
    set_guard::Of::Everywhere(true)
}

fn space(id: u64) -> set_guard::Of {
    set_guard::Of::SpaceId(id)
}

fn site_of(tab: u64) -> set_guard::Of {
    set_guard::Of::TabId(tab)
}

/// What the window is told of Guard for the page in front.
fn guard(shell: &FakeShell, subscription: u64) -> (bool, GuardLevel) {
    let view = shell
        .window(subscription)
        .tab
        .expect("a tab")
        .guard
        .expect("a tab's view says what Guard is");
    (view.on, view.said_by())
}

fn refusal(answer: response::Kind) -> (String, String) {
    let response::Kind::Error(error) = answer else {
        panic!("a refusal, not {answer:?}");
    };
    (error.code, error.field)
}

/// The web views the shell was asked to reload, in order.
fn reloads(shell: &FakeShell) -> Vec<u64> {
    shell
        .host_requests
        .iter()
        .filter_map(|request| match request {
            host_request::Kind::WebViewCommand(command)
                if command.command() == NavigationCommand::Reload =>
            {
                Some(command.web_view_id)
            }
            _ => None,
        })
        .collect()
}

/// A window whose tab shows the news site's page: the window, the tab, the
/// subscription, and the Space the window shows.
async fn a_page(shell: &mut FakeShell) -> (u64, u64, u64, u64) {
    web(shell);
    let (window, tab, subscription) = shell.open_window().await;
    shell.submit(tab, NEWS).await;
    let home = shell.window(subscription).space_id;
    (window, tab, subscription, home)
}

#[test]
fn guard_is_on_until_a_switch_says_otherwise_and_the_nearest_that_says_wins() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, home) = a_page(&mut shell).await;
        assert_eq!(guard(&shell, subscription), (true, Everywhere));

        // Each switch in turn, and what the window is told at once.
        set_ok(&mut shell, everywhere(), Off).await;
        assert_eq!(guard(&shell, subscription), (false, Everywhere));
        set_ok(&mut shell, space(home), On).await;
        assert_eq!(guard(&shell, subscription), (true, Space));
        set_ok(&mut shell, site_of(tab), Off).await;
        assert_eq!(guard(&shell, subscription), (false, Site));
        // The Space and everywhere change beneath a site that says: it still says.
        set_ok(&mut shell, space(home), Off).await;
        set_ok(&mut shell, everywhere(), On).await;
        assert_eq!(guard(&shell, subscription), (false, Site));
        set_ok(&mut shell, site_of(tab), On).await;
        assert_eq!(guard(&shell, subscription), (true, Site));
        // A site that says nothing is as its Space has it; a Space that says nothing, as
        // everywhere has it.
        set_ok(&mut shell, site_of(tab), Unspecified).await;
        assert_eq!(guard(&shell, subscription), (false, Space));
        set_ok(&mut shell, space(home), Unspecified).await;
        assert_eq!(guard(&shell, subscription), (true, Everywhere));

        // Another site in the same tab is not under the first site's switch; any host of
        // the first site is.
        set_ok(&mut shell, site_of(tab), Off).await;
        shell.submit(tab, OTHER).await;
        assert_eq!(guard(&shell, subscription), (true, Everywhere));
        shell.submit(tab, SHOP).await;
        assert_eq!(guard(&shell, subscription), (false, Site));
    });
}

#[test]
fn a_sites_switch_reloads_its_tab_when_it_changes_what_guard_is_there_and_nothing_else_does() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, home) = a_page(&mut shell).await;
        let web_view = shell.window(subscription).tab.unwrap().web_view_id;
        assert_eq!(reloads(&shell), [] as [u64; 0]);

        // Off for the site: the tab is reloaded, once.
        set_ok(&mut shell, site_of(tab), Off).await;
        assert_eq!(reloads(&shell), [web_view]);
        // Said again, nothing changes and nothing is reloaded.
        set_ok(&mut shell, site_of(tab), Off).await;
        assert_eq!(reloads(&shell), [web_view]);
        // A Space's switch and everywhere's reload nothing, whatever they change.
        set_ok(&mut shell, space(home), Off).await;
        set_ok(&mut shell, everywhere(), Off).await;
        set_ok(&mut shell, everywhere(), On).await;
        set_ok(&mut shell, space(home), Unspecified).await;
        assert_eq!(reloads(&shell), [web_view]);
        // The site saying nothing again turns Guard back on there: reloaded.
        set_ok(&mut shell, site_of(tab), Unspecified).await;
        assert_eq!(reloads(&shell), [web_view, web_view]);
        // A site's switch that says what the level above says already changes nothing
        // for the page: not reloaded.
        set_ok(&mut shell, site_of(tab), On).await;
        assert_eq!(reloads(&shell), [web_view, web_view]);

        // Only that tab: another tab of the same site is not reloaded by it.
        let second = shell.open_tab(window, SHOP).await;
        let second_view = shell.window(subscription).tab.unwrap().web_view_id;
        set_ok(&mut shell, site_of(second), Off).await;
        assert_eq!(reloads(&shell), [web_view, web_view, second_view]);
    });
}

#[test]
fn what_cannot_be_set_is_refused_and_changes_nothing() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, home) = a_page(&mut shell).await;
        let told = shell.guard_switches_told();
        let before = told.borrow().len();

        // None of the three named.
        let answer = shell
            .request(request::Kind::SetGuard(SetGuard {
                of: None,
                switch: Off.into(),
            }))
            .await;
        assert_eq!(refusal(answer), ("guard.where".to_owned(), "of".to_owned()));
        // Everywhere may not say nothing; and a value this core does not know.
        assert_eq!(
            refusal(set(&mut shell, everywhere(), Unspecified).await),
            ("guard.switch".to_owned(), "switch".to_owned())
        );
        let answer = shell
            .request(request::Kind::SetGuard(SetGuard {
                of: Some(site_of(tab)),
                switch: 7,
            }))
            .await;
        assert_eq!(
            refusal(answer),
            ("guard.switch".to_owned(), "switch".to_owned())
        );
        // A Space and a tab there are not.
        assert_eq!(
            refusal(set(&mut shell, space(home + 50), Off).await),
            ("space.unknown".to_owned(), "space_id".to_owned())
        );
        assert_eq!(
            refusal(set(&mut shell, site_of(tab + 50), Off).await),
            ("tab.unknown".to_owned(), "tab_id".to_owned())
        );
        // A tab that shows no page of a site.
        let empty = shell
            .request(request::Kind::OpenTab(wt_proto::v1::OpenTab {
                window_id: window,
                text: String::new(),
                background: true,
                after_tab_id: 0,
            }))
            .await;
        let response::Kind::TabOpened(empty) = empty else {
            panic!("a tab, not {empty:?}");
        };
        assert_eq!(
            refusal(set(&mut shell, site_of(empty.tab_id), Off).await),
            ("guard.no_site".to_owned(), "tab_id".to_owned())
        );

        // Nothing changed: Guard is as it was, nothing was reloaded, and whoever answers
        // the fast path was told no new switch.
        assert_eq!(guard(&shell, subscription), (true, Everywhere));
        assert_eq!(reloads(&shell), [] as [u64; 0]);
        let switches = told.borrow().last().unwrap().0.clone();
        assert!(
            !switches.off_everywhere && switches.spaces.is_empty() && switches.sites.is_empty()
        );
        assert_eq!(
            told.borrow().len(),
            before,
            "and nothing was told for a refusal"
        );
    });
}

#[test]
fn spaces_that_share_logins_share_a_sites_switch_and_not_a_spaces() {
    run(async {
        let mut shell = FakeShell::fake();
        let (window, tab, subscription, home) = a_page(&mut shell).await;
        // A Space that shares the first's logins, and one with logins of its own.
        let mut made = Vec::new();
        for (name, share) in [("Shares", home), ("Own", 0)] {
            let answer = shell
                .request(request::Kind::CreateSpace(CreateSpace {
                    name: name.to_owned(),
                    share_logins_with: share,
                    ..CreateSpace::default()
                }))
                .await;
            let response::Kind::SpaceCreated(space) = answer else {
                panic!("a Space, not {answer:?}");
            };
            made.push(space.space_id);
        }
        let (shares, own) = (made[0], made[1]);
        // Off for the site in the first Space, and off for the first Space itself... is
        // not what is wanted: the site's switch alone, to see who shares it.
        set_ok(&mut shell, site_of(tab), Off).await;
        assert_eq!(guard(&shell, subscription), (false, Site));

        // The same site in the Space that shares its logins: off there too.
        for (space_id, expected) in [(shares, (false, Site)), (own, (true, Everywhere))] {
            shell
                .request(request::Kind::SwitchSpace(SwitchSpace {
                    window_id: window,
                    space_id,
                }))
                .await;
            let front = shell.window(subscription).tab.unwrap().tab_id;
            shell.submit(front, SHOP).await;
            assert_eq!(
                guard(&shell, subscription),
                expected,
                "the Space {space_id}"
            );
        }
        // A Space's own switch is its own: off for the one that shares logins, and the
        // first Space, with the same logins, is as it was.
        set_ok(&mut shell, space(shares), On).await;
        set_ok(&mut shell, site_of(tab), Unspecified).await;
        set_ok(&mut shell, everywhere(), Off).await;
        for (space_id, expected) in [
            (shares, (true, Space)),
            (home, (false, Everywhere)),
            (own, (false, Everywhere)),
        ] {
            shell
                .request(request::Kind::SwitchSpace(SwitchSpace {
                    window_id: window,
                    space_id,
                }))
                .await;
            assert_eq!(
                guard(&shell, subscription),
                expected,
                "the Space {space_id}"
            );
        }
    });
}

#[test]
fn switches_are_kept_across_a_start() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        let (_, tab, subscription, home) = a_page(&mut shell).await;
        set_ok(&mut shell, everywhere(), Off).await;
        set_ok(&mut shell, space(home), On).await;
        set_ok(&mut shell, site_of(tab), Off).await;
        assert_eq!(guard(&shell, subscription), (false, Site));
        drop(shell);

        // Started again: whoever answers the fast path is told all three before anything
        // is asked, and the window is told the same of its page.
        let mut shell = FakeShell::live(folder.path());
        web(&mut shell);
        let told = shell.guard_switches_told();
        {
            let (switches, _) = &told.borrow()[0];
            assert!(switches.off_everywhere);
            assert_eq!(
                switches.spaces.values().copied().collect::<Vec<_>>(),
                [true]
            );
            let sites: Vec<_> = switches
                .sites
                .values()
                .flat_map(|sites| sites.iter())
                .collect();
            assert_eq!(sites, [(&"example.com".to_owned(), &false)]);
        }
        let (_, _, subscription) = shell.open_window().await;
        shell.settle().await;
        assert_eq!(guard(&shell, subscription), (false, Site));

        // Each said to say nothing, or on: nothing of them is left at the next start.
        let tab = shell.window(subscription).tab.unwrap().tab_id;
        set_ok(&mut shell, site_of(tab), Unspecified).await;
        set_ok(&mut shell, space(home), Unspecified).await;
        set_ok(&mut shell, everywhere(), On).await;
        drop(shell);
        let shell = FakeShell::live(folder.path());
        let told = shell.guard_switches_told();
        let (switches, _) = &told.borrow()[0];
        assert!(
            !switches.off_everywhere && switches.spaces.is_empty() && switches.sites.is_empty()
        );
    });
}

#[test]
fn a_web_view_is_told_to_the_fast_paths_side_before_the_shell_is_asked_for_it() {
    run(async {
        let mut shell = FakeShell::fake();
        web(&mut shell);
        let (_, tab, subscription) = shell.open_window().await;
        let home = shell.window(subscription).space_id;
        let told = shell.guard_switches_told();
        assert!(told.borrow()[0].0.web_views.is_empty(), "no web view yet");

        // A page asked for, and nothing read by the shell yet: the web view that is to be
        // made is already told, with its Space and its logins; and when it was told, the
        // shell had not yet been asked to make it.
        shell.send(request::Kind::SubmitAddress(wt_proto::v1::SubmitAddress {
            tab_id: tab,
            text: NEWS.to_owned(),
        }));
        let (switches, asked_by_then) = told.borrow().last().unwrap().clone();
        assert_eq!(switches.web_views.len(), 1);
        let (web_view, (space, _)) = switches.web_views.iter().next().unwrap();
        assert_eq!(space.get(), home);
        assert!(
            !asked_by_then.contains(&web_view.get()),
            "told before the shell was asked to make it"
        );
        shell.settle().await;
        assert_eq!(
            shell.window(subscription).tab.unwrap().web_view_id,
            web_view.get()
        );

        // A tab that sleeps has no web view: whoever answers the fast path is told it
        // has gone, when the windows are told.
        let second = shell
            .open_tab(shell.window(subscription).window_id, SHOP)
            .await;
        let _ = second;
        assert_eq!(told.borrow().last().unwrap().0.web_views.len(), 2);
        let answer = shell
            .request(request::Kind::SleepOthers(wt_proto::v1::SleepOthers {
                window_id: shell.window(subscription).window_id,
            }))
            .await;
        assert!(matches!(answer, response::Kind::Ack(_)), "{answer:?}");
        shell.settle().await;
        assert_eq!(told.borrow().last().unwrap().0.web_views.len(), 1);
    });
}

#[test]
fn a_private_window_is_under_everywheres_switch_and_keeps_a_sites_for_its_life() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, home) = a_page(&mut shell).await;
        let (private_window, private_tab, private) = shell.open_private_window().await;
        shell.submit(private_tab, NEWS).await;
        assert_eq!(guard(&shell, private), (true, Everywhere));

        // The ordinary window's Space says off: the private window has no Space, and is
        // not under it. Everywhere's it is under.
        set_ok(&mut shell, space(home), Off).await;
        assert_eq!(guard(&shell, subscription), (false, Space));
        assert_eq!(guard(&shell, private), (true, Everywhere));
        set_ok(&mut shell, everywhere(), Off).await;
        assert_eq!(guard(&shell, private), (false, Everywhere));
        // A private window's own Space cannot be named.
        let its_space = shell.window(private).space_id;
        assert_eq!(its_space, 0, "a private window shows no Space");
        set_ok(&mut shell, everywhere(), On).await;

        // A site's switch set in the private window holds there, and is not the same
        // site's in the ordinary window, whose logins are others.
        set_ok(&mut shell, space(home), Unspecified).await;
        set_ok(&mut shell, site_of(private_tab), Off).await;
        assert_eq!(guard(&shell, private), (false, Site));
        assert_eq!(guard(&shell, subscription), (true, Everywhere));
        set_ok(&mut shell, site_of(tab), Off).await;
        set_ok(&mut shell, site_of(private_tab), Unspecified).await;
        assert_eq!(guard(&shell, private), (true, Everywhere));
        assert_eq!(guard(&shell, subscription), (false, Site));
        let _ = private_window;
    });
}

#[test]
fn a_sites_switch_is_told_to_the_fast_paths_side_before_its_tab_is_reloaded() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, subscription, _) = a_page(&mut shell).await;
        let web_view = shell.window(subscription).tab.unwrap().web_view_id;
        let told = shell.guard_switches_told();
        let before = told.borrow().len();

        // Sent, and nothing read by the shell yet: whoever answers the fast path has the
        // site's switch already; and when it was told, the reload had not been asked.
        shell.send(request::Kind::SetGuard(SetGuard {
            of: Some(site_of(tab)),
            switch: Off.into(),
        }));
        assert_eq!(told.borrow().len(), before + 1, "told once, at once");
        let (switches, asked_by_then) = told.borrow().last().unwrap().clone();
        let sites: Vec<_> = switches
            .sites
            .values()
            .flat_map(|sites| sites.iter())
            .collect();
        assert_eq!(sites, [(&"example.com".to_owned(), &false)]);
        assert!(
            !asked_by_then.contains(&web_view),
            "told before the tab's reload was asked of the shell"
        );
        shell.settle().await;
        assert_eq!(reloads(&shell), [web_view]);
        assert_eq!(told.borrow().len(), before + 1, "and not told again for it");
    });
}
