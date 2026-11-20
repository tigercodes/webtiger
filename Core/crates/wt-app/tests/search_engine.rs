//! Where searches go (020), through the contract, against the scripted
//! engine: the engines the core lists, the person's choice among them,
//! where a search then goes, and what is kept of the choice.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{
    ChooseSearchEngine, EngineKind, FakeOptions, QuerySearch, SearchSettings, host_request,
    request, response,
};
use wt_store::Store;
use wt_testkit::{FakeShell, run};

/// A shell in fake mode that is to be a first run: nobody has chosen yet.
fn first_run() -> FakeShell {
    let mut config = FakeShell::config(EngineKind::Webkit);
    config.fake = Some(FakeOptions {
        first_run: true,
        ..Default::default()
    });
    FakeShell::start(&config).expect("fake mode starts")
}

/// What the core says of where searches go.
async fn settings(shell: &mut FakeShell) -> SearchSettings {
    match shell
        .request(request::Kind::QuerySearch(QuerySearch {}))
        .await
    {
        response::Kind::SearchSettings(settings) => settings,
        other => panic!("expected the search settings, got {other:?}"),
    }
}

/// The person's choice of an engine, as it is sent.
async fn choose(shell: &mut FakeShell, id: &str) -> response::Kind {
    shell
        .request(request::Kind::ChooseSearchEngine(ChooseSearchEngine {
            id: id.to_owned(),
        }))
        .await
}

/// The address the core last asked a web view to load.
fn last_load(shell: &FakeShell) -> String {
    shell
        .host_requests
        .iter()
        .rev()
        .find_map(|request| match request {
            host_request::Kind::WebViewLoad(load) => Some(load.url.clone()),
            _ => None,
        })
        .expect("something was loaded")
}

/// Types words that are no address in a window's tab, and says where they
/// were sent.
async fn search(shell: &mut FakeShell, tab: u64, words: &str) -> String {
    shell.submit(tab, words).await;
    last_load(shell)
}

fn error_code(answer: response::Kind) -> String {
    match answer {
        response::Kind::Error(error) => error.code,
        other => panic!("expected an error, got {other:?}"),
    }
}

#[test]
fn the_engines_are_listed_by_name_with_where_each_sends_a_search() {
    run(async {
        let mut shell = FakeShell::fake();

        let listed = settings(&mut shell).await.engines;

        let names: Vec<&str> = listed.iter().map(|engine| engine.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "Bing",
                "Brave Search",
                "DuckDuckGo",
                "Ecosia",
                "Google",
                "Qwant",
                "Startpage"
            ],
            "in the order of their names"
        );
        let ecosia = listed.iter().find(|engine| engine.id == "ecosia").unwrap();
        assert_eq!(ecosia.host, "www.ecosia.org");
        assert!(listed.iter().all(|engine| !engine.host.is_empty()));
    });
}

#[test]
fn nobody_has_chosen_at_a_first_run_and_fake_mode_takes_the_stand_in_as_chosen() {
    run(async {
        let mut asked = first_run();
        let before = settings(&mut asked).await;
        assert!(!before.chosen, "a shell that is told so asks");
        assert_eq!(before.chosen_id, "duckduckgo", "the engine that stands in");

        // Fake mode with nobody to ask: every run would be a first run.
        let mut shell = FakeShell::fake();
        let fake = settings(&mut shell).await;
        assert!(fake.chosen);
        assert_eq!(fake.chosen_id, "duckduckgo");
    });
}

#[test]
fn searches_go_to_the_stand_in_until_somebody_chooses_and_then_to_the_choice() {
    run(async {
        let mut shell = first_run();
        let (window, tab, _) = shell.open_window().await;

        assert_eq!(
            search(&mut shell, tab, "rust borrow checker").await,
            "https://duckduckgo.com/?q=rust+borrow+checker"
        );

        let done = choose(&mut shell, "ecosia").await;
        assert!(matches!(done, response::Kind::Ack(_)), "{done:?}");

        let now = settings(&mut shell).await;
        assert!(now.chosen, "somebody chose");
        assert_eq!(now.chosen_id, "ecosia");
        assert_eq!(
            search(&mut shell, tab, "rust borrow checker").await,
            "https://www.ecosia.org/search?q=rust+borrow+checker"
        );
        // In every tab: the choice is the whole app's.
        shell.open_tab(window, "a second search").await;
        assert_eq!(
            last_load(&shell),
            "https://www.ecosia.org/search?q=a+second+search"
        );

        // And another can be chosen after it.
        choose(&mut shell, "qwant").await;
        assert_eq!(
            search(&mut shell, tab, "once more").await,
            "https://www.qwant.com/?q=once+more"
        );
    });
}

#[test]
fn a_name_the_core_does_not_have_is_refused_and_changes_nothing() {
    run(async {
        let mut shell = first_run();
        let (_, tab, _) = shell.open_window().await;

        for name in ["altavista", "", "Ecosia", "ecosia "] {
            let refused = choose(&mut shell, name).await;
            assert_eq!(error_code(refused), "search.unknown_engine", "{name:?}");
        }

        let after = settings(&mut shell).await;
        assert!(!after.chosen, "nobody has chosen yet");
        assert_eq!(after.chosen_id, "duckduckgo");
        assert_eq!(
            search(&mut shell, tab, "still there").await,
            "https://duckduckgo.com/?q=still+there"
        );
    });
}

#[test]
fn an_address_is_loaded_as_before_whatever_was_chosen() {
    run(async {
        let mut shell = FakeShell::fake();
        let (_, tab, _) = shell.open_window().await;
        choose(&mut shell, "google").await;
        shell.engine.page("https://example.com/", "Example");

        shell.submit(tab, "example.com").await;

        assert_eq!(last_load(&shell), "https://example.com/");
    });
}

#[test]
fn the_choice_is_kept_when_the_app_starts_again() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        {
            let mut shell = FakeShell::live(folder.path());
            let first = settings(&mut shell).await;
            assert!(!first.chosen, "a store nobody has chosen in: a first run");
            assert_eq!(first.chosen_id, "duckduckgo");
            choose(&mut shell, "startpage").await;
        }

        let mut shell = FakeShell::live(folder.path());

        let kept = settings(&mut shell).await;
        assert!(kept.chosen, "nobody is asked twice");
        assert_eq!(kept.chosen_id, "startpage");
        let (_, tab, _) = shell.open_window().await;
        assert_eq!(
            search(&mut shell, tab, "kept").await,
            "https://www.startpage.com/do/search?q=kept"
        );
    });
}

#[test]
fn a_choice_the_list_no_longer_has_is_no_choice() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        // A store in which an engine was chosen that this build does not
        // know: one a later build knew, or one that was taken off the list.
        drop(FakeShell::live(folder.path()));
        let mut store = Store::open(&folder.path().join("webtiger.sqlite")).unwrap();
        store.set_setting("search.engine", "altavista").unwrap();
        drop(store);

        let mut shell = FakeShell::live(folder.path());

        let read = settings(&mut shell).await;
        assert!(!read.chosen, "the person is asked");
        assert_eq!(read.chosen_id, "duckduckgo");
    });
}
