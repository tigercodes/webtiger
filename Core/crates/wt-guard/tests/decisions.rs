//! Guard's decisions (064), a rule at a time: what the corpus does not say.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use wt_guard::{Guard, Kind, Verdict};

const PAGE: &str = "https://news.example.com/story";

#[test]
fn a_kind_is_the_c_headers_by_its_number_and_other_when_it_is_not_known() {
    let kinds = [
        (0, Kind::Other),
        (1, Kind::Document),
        (2, Kind::Subdocument),
        (3, Kind::Script),
        (4, Kind::Stylesheet),
        (5, Kind::Image),
        (6, Kind::Font),
        (7, Kind::Media),
        (8, Kind::Xhr),
        (9, Kind::Websocket),
        (10, Kind::Ping),
        (11, Kind::Other),
        (u32::MAX, Kind::Other),
    ];
    for (code, kind) in kinds {
        assert_eq!(Kind::from_code(code), kind, "{code}");
    }
}

#[test]
fn each_kind_is_asked_by_its_name_in_the_rules() {
    // A rule for each kind alone: only a request of that kind is blocked by it.
    let named = [
        ("script", Kind::Script),
        ("stylesheet", Kind::Stylesheet),
        ("image", Kind::Image),
        ("font", Kind::Font),
        ("media", Kind::Media),
        ("xmlhttprequest", Kind::Xhr),
        ("websocket", Kind::Websocket),
        ("ping", Kind::Ping),
        ("subdocument", Kind::Subdocument),
        ("other", Kind::Other),
    ];
    for (name, kind) in named {
        let guard = Guard::with_rules([format!("||things.example.net^${name}").as_str()]);
        for (_, asked) in named {
            let expected = if asked == kind {
                Verdict::Block
            } else {
                Verdict::Allow
            };
            assert_eq!(
                guard.decide("https://things.example.net/thing", PAGE, asked),
                expected,
                "a rule for {name}, a request for {asked:?}"
            );
        }
    }
}

#[test]
fn the_rules_of_several_lists_are_one_blocker() {
    let guard = Guard::with_rules([
        "||one.example.net^",
        "||two.example.net^\n@@||one.example.net/ok.js",
    ]);
    assert_eq!(
        guard.decide("https://one.example.net/a.js", PAGE, Kind::Script),
        Verdict::Block
    );
    assert_eq!(
        guard.decide("https://two.example.net/a.js", PAGE, Kind::Script),
        Verdict::Block
    );
    // An exception of one list undoes a rule of another.
    assert_eq!(
        guard.decide("https://one.example.net/ok.js", PAGE, Kind::Script),
        Verdict::Allow
    );
    assert_eq!(
        guard.decide("https://three.example.net/a.js", PAGE, Kind::Script),
        Verdict::Allow
    );
}

#[test]
fn the_first_party_is_the_site_and_not_the_host() {
    let guard = Guard::with_rules([
        "||cdn.example.co.uk^$third-party",
        "||own.example.co.uk^$~third-party",
    ]);
    // Two hosts of one site (what somebody registered under co.uk) are the first party.
    assert_eq!(
        guard.decide(
            "https://cdn.example.co.uk/a.js",
            "https://www.example.co.uk/",
            Kind::Script
        ),
        Verdict::Allow
    );
    // Another registration under the same public suffix is a third party.
    assert_eq!(
        guard.decide(
            "https://cdn.example.co.uk/a.js",
            "https://other.co.uk/",
            Kind::Script
        ),
        Verdict::Block
    );
    // And a rule for the first party alone.
    assert_eq!(
        guard.decide(
            "https://own.example.co.uk/a.js",
            "https://shop.example.co.uk/",
            Kind::Script
        ),
        Verdict::Block
    );
    assert_eq!(
        guard.decide(
            "https://own.example.co.uk/a.js",
            "https://other.co.uk/",
            Kind::Script
        ),
        Verdict::Allow
    );
}

#[test]
fn an_address_is_matched_as_it_is_normalised_whatever_its_case() {
    let guard = Guard::with_rules(["||ads.example.net^"]);
    assert_eq!(
        guard.decide("HTTPS://ADS.Example.NET/Show.js", PAGE, Kind::Script),
        Verdict::Block
    );
    assert_eq!(
        guard.decide("https://ads.example.net:8443/show.js", PAGE, Kind::Script),
        Verdict::Block
    );
}

#[test]
fn rules_that_hide_or_mend_a_pages_content_block_nothing() {
    // A list of nothing but such rules: no request is refused for them.
    let guard = Guard::with_rules([
        "example.com##.ad-banner\n##.sponsored\nexample.com#@#.ok\nexample.com##+js(nowebrtc)",
    ]);
    assert_eq!(
        guard.decide("https://example.com/ad-banner.js", PAGE, Kind::Script),
        Verdict::Allow
    );
}

#[test]
fn a_rule_that_asks_for_a_stand_in_blocks_and_one_that_mends_an_address_does_not() {
    // No stand-ins are loaded yet (064 §2): a rule that asks for one is a rule that blocks.
    let guard = Guard::with_rules([
        "||stand.example.net/lib.js$script,redirect=noopjs",
        "||mend.example.net^$removeparam=utm_source",
    ]);
    assert_eq!(
        guard.decide("https://stand.example.net/lib.js", PAGE, Kind::Script),
        Verdict::Block
    );
    // A rule that takes a part out of an address refuses nothing, and nothing is taken out here.
    assert_eq!(
        guard.decide(
            "https://mend.example.net/page?utm_source=x",
            PAGE,
            Kind::Xhr
        ),
        Verdict::Allow
    );
}
