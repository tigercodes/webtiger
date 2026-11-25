//! A person's own rules for Guard (066), through the contract: set and asked
//! for again as they were written; kept across a start; refused over the
//! limit; a suite's rules where a person's would be, in fake mode; and
//! whoever holds the blocker told at once, and before the request that set
//! them is answered.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_proto::v1::{EngineKind, FakeOptions, QueryGuardRules, SetGuardRules, request, response};
use wt_testkit::{FakeShell, run};

const RULES: &str = "! What I do not want fetched.\n||ads.tracker.example.net^\n\n@@||ads.tracker.example.net/consent.js$script\n! Größe: a comment that is not ASCII, and a line that is no rule.\nnot a rule ##\n";

async fn set(shell: &mut FakeShell, text: &str) -> response::Kind {
    shell
        .request(request::Kind::SetGuardRules(SetGuardRules {
            text: text.to_owned(),
        }))
        .await
}

async fn rules(shell: &mut FakeShell) -> String {
    let answer = shell
        .request(request::Kind::QueryGuardRules(QueryGuardRules {}))
        .await;
    let response::Kind::GuardRules(rules) = answer else {
        panic!("the rules, not {answer:?}");
    };
    rules.text
}

fn refusal(answer: response::Kind) -> (String, String) {
    let response::Kind::Error(error) = answer else {
        panic!("a refusal, not {answer:?}");
    };
    (error.code, error.field)
}

#[test]
fn rules_are_asked_for_as_they_were_written_and_one_text_replaces_another() {
    run(async {
        let mut shell = FakeShell::fake();
        assert_eq!(rules(&mut shell).await, "", "none at first");
        assert!(matches!(
            set(&mut shell, RULES).await,
            response::Kind::Ack(_)
        ));
        // Whole: the comments, the blank line, the line that is no rule.
        assert_eq!(rules(&mut shell).await, RULES);
        // Another text is in its place, and none is none.
        assert!(matches!(
            set(&mut shell, "||other.example.net^").await,
            response::Kind::Ack(_)
        ));
        assert_eq!(rules(&mut shell).await, "||other.example.net^");
        assert!(matches!(set(&mut shell, "").await, response::Kind::Ack(_)));
        assert_eq!(rules(&mut shell).await, "");
    });
}

#[test]
fn rules_are_kept_across_a_start() {
    run(async {
        let folder = tempfile::tempdir().unwrap();
        let mut shell = FakeShell::live(folder.path());
        assert_eq!(rules(&mut shell).await, "");
        assert!(matches!(
            set(&mut shell, RULES).await,
            response::Kind::Ack(_)
        ));
        drop(shell);

        // Started again: the rules are there, and the holder of the blocker is told them
        // before anything is asked.
        let mut shell = FakeShell::live(folder.path());
        let told = shell.guard_rules_told();
        assert_eq!(*told.borrow(), [RULES]);
        assert_eq!(rules(&mut shell).await, RULES);

        // Taken away, they are gone at the next start too.
        assert!(matches!(set(&mut shell, "").await, response::Kind::Ack(_)));
        drop(shell);
        let mut shell = FakeShell::live(folder.path());
        assert_eq!(rules(&mut shell).await, "");
        assert_eq!(*shell.guard_rules_told().borrow(), [""]);
    });
}

#[test]
fn a_text_over_the_limit_is_refused_and_what_there_was_stays() {
    run(async {
        let mut shell = FakeShell::fake();
        let told = shell.guard_rules_told();
        assert!(matches!(
            set(&mut shell, RULES).await,
            response::Kind::Ack(_)
        ));
        // A rule to a line, to just the limit: taken. One byte more: refused.
        let line = "||host-with-a-name-of-some-length.example.net^\n";
        let most = 256 * 1024;
        let mut at_the_limit = line.repeat(most / line.len());
        at_the_limit.push_str(&"!".repeat(most - at_the_limit.len()));
        assert_eq!(at_the_limit.len(), most);
        assert!(matches!(
            set(&mut shell, &at_the_limit).await,
            response::Kind::Ack(_)
        ));
        let over = format!("{at_the_limit}!");
        assert_eq!(
            refusal(set(&mut shell, &over).await),
            ("guard.rules_too_long".to_owned(), "text".to_owned())
        );
        // What there was is what there is, and the holder of the blocker was not told
        // of the text that was refused.
        assert_eq!(rules(&mut shell).await, at_the_limit);
        assert_eq!(
            told.borrow().len(),
            3,
            "the start, and the two that were taken"
        );
        assert_eq!(told.borrow().last().unwrap(), &at_the_limit);
    });
}

#[test]
fn the_holder_of_the_blocker_is_told_at_once_and_before_the_request_is_answered() {
    run(async {
        let mut shell = FakeShell::fake();
        // Told as it begins to listen: the rules the core started with, which are none.
        let told = shell.guard_rules_told();
        assert_eq!(*told.borrow(), [""]);

        // Sent, and not yet answered: the holder has the new rules already.
        let request = shell.send(request::Kind::SetGuardRules(SetGuardRules {
            text: RULES.to_owned(),
        }));
        assert_eq!(*told.borrow(), ["", RULES]);
        assert!(shell.take_response(request).is_none(), "not answered yet");
        shell.settle().await;
        assert!(matches!(
            shell.take_response(request),
            Some(response::Kind::Ack(_))
        ));
        assert_eq!(told.borrow().len(), 2, "told once for it");

        // Asking for the rules tells nobody anything.
        assert_eq!(rules(&mut shell).await, RULES);
        assert_eq!(told.borrow().len(), 2);
    });
}

#[test]
fn in_fake_mode_a_suites_rules_stand_where_a_persons_would_and_are_replaced_like_them() {
    run(async {
        let mut config = FakeShell::config(EngineKind::Webkit);
        config.fake = Some(FakeOptions {
            guard_rules: "||suite.example.net^".to_owned(),
            ..FakeOptions::default()
        });
        let mut shell = FakeShell::start(&config).unwrap();
        let told = shell.guard_rules_told();
        assert_eq!(*told.borrow(), ["||suite.example.net^"]);
        assert_eq!(rules(&mut shell).await, "||suite.example.net^");
        assert!(matches!(
            set(&mut shell, RULES).await,
            response::Kind::Ack(_)
        ));
        assert_eq!(rules(&mut shell).await, RULES);
        assert_eq!(*told.borrow(), ["||suite.example.net^", RULES]);
    });
    run(async {
        // Outside fake mode a suite's rules are nothing: the store's are the rules.
        let folder = tempfile::tempdir().unwrap();
        let mut config = FakeShell::live_config(folder.path());
        config.fake = Some(FakeOptions {
            guard_rules: "||suite.example.net^".to_owned(),
            ..FakeOptions::default()
        });
        let mut shell = FakeShell::start(&config).unwrap();
        assert_eq!(*shell.guard_rules_told().borrow(), [""]);
        assert_eq!(rules(&mut shell).await, "");
    });
}
