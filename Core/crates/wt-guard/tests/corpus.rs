//! Guard's corpus (064): `Fixtures/guard/corpus.txt`, a file of requests with
//! the answers expected, against the rules of `Fixtures/guard/rules.txt`.
//! The same file is for every way of deciding there will be; here it is put
//! to the blocker in the core.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::PathBuf;

use wt_guard::{Guard, Kind, Verdict};

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../Fixtures/guard")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("no {}", path.display()))
}

/// A line of the corpus: the answer, the kind, the page, the address, why.
struct Asked {
    answer: Verdict,
    kind: Kind,
    page: String,
    url: String,
    why: String,
}

fn kind(name: &str) -> Kind {
    match name {
        "other" => Kind::Other,
        "document" => Kind::Document,
        "subdocument" => Kind::Subdocument,
        "script" => Kind::Script,
        "stylesheet" => Kind::Stylesheet,
        "image" => Kind::Image,
        "font" => Kind::Font,
        "media" => Kind::Media,
        "xhr" => Kind::Xhr,
        "websocket" => Kind::Websocket,
        "ping" => Kind::Ping,
        other => panic!("the corpus has a kind this test does not know: {other}"),
    }
}

/// The corpus's lines that are requests. The address is what is left of a
/// line after its first three words, up to the ` #` that begins the why.
fn corpus() -> Vec<Asked> {
    fixture("corpus.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (request, why) = line.split_once(" #").unwrap_or((line, ""));
            let mut words = request.split_whitespace();
            let answer = match words.next() {
                Some("allow") => Verdict::Allow,
                Some("block") => Verdict::Block,
                other => panic!("the corpus has an answer that is none: {other:?}"),
            };
            let kind = kind(words.next().expect("a kind"));
            let page = words.next().expect("a page").to_owned();
            let url = words.collect::<Vec<_>>().join(" ");
            assert!(
                !url.is_empty(),
                "a line of the corpus asks for nothing: {line}"
            );
            Asked {
                answer,
                kind,
                page,
                url,
                why: why.trim().to_owned(),
            }
        })
        .collect()
}

#[test]
fn every_request_of_the_corpus_has_the_answer_expected() {
    let guard = Guard::with_rules([fixture("rules.txt").as_str()]);
    let corpus = corpus();
    let wrong: Vec<String> = corpus
        .iter()
        .filter(|asked| guard.decide(&asked.url, &asked.page, asked.kind) != asked.answer)
        .map(|asked| {
            format!(
                "{:?} expected for {:?} {} from {} ({})",
                asked.answer, asked.kind, asked.url, asked.page, asked.why
            )
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "{} wrong:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
    // The corpus is not empty, has both answers, and every kind the header has.
    assert!(corpus.len() >= 30, "{} requests", corpus.len());
    for answer in [Verdict::Allow, Verdict::Block] {
        assert!(corpus.iter().filter(|asked| asked.answer == answer).count() >= 10);
    }
    for code in 0..=10 {
        let kind = Kind::from_code(code);
        assert!(
            corpus.iter().any(|asked| asked.kind == kind),
            "the corpus has no request for {kind:?}"
        );
    }
}

#[test]
fn with_no_rules_every_request_of_the_corpus_is_allowed() {
    for guard in [
        Guard::default(),
        Guard::with_rules([]),
        Guard::with_rules([""]),
    ] {
        for asked in corpus() {
            assert_eq!(
                guard.decide(&asked.url, &asked.page, asked.kind),
                Verdict::Allow,
                "{}",
                asked.url
            );
        }
    }
}

#[test]
fn eight_threads_asking_at_once_are_each_given_the_corpus_answers() {
    let guard = std::sync::Arc::new(Guard::with_rules([fixture("rules.txt").as_str()]));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let guard = std::sync::Arc::clone(&guard);
            std::thread::spawn(move || {
                let corpus = corpus();
                let mut wrong = 0;
                for _ in 0..200 {
                    wrong += corpus
                        .iter()
                        .filter(|asked| {
                            guard.decide(&asked.url, &asked.page, asked.kind) != asked.answer
                        })
                        .count();
                }
                wrong
            })
        })
        .collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), 0);
    }
}
