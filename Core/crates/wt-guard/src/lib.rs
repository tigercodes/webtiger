//! Guard (000 §5.8, 064): whether a page may fetch a thing.
//!
//! Rules go in as text, in the syntax the public lists are written in; a
//! decision comes out for one request: its address, the address of the page
//! that asks, and what it is for. Inside is Brave's `adblock-rust`; nothing
//! of it is in what this crate offers, so that the rest of the core does not
//! know what the blocker is.
//!
//! **It fails open.** A request that cannot be made sense of is allowed: a
//! blocker that fails must not break the page.
//!
//! **One blocker is shared by every thread that asks.** It is made from its
//! rules and never changed: rules that change are a new one. Its decisions
//! are made one at a time, behind a lock of the blocker's own that is held
//! for the half microsecond one takes (064 §5.1).

use adblock::lists::{ParseOptions, RuleTypes};
use adblock::request::Request;
use adblock::{Engine, FilterSet};
use wt_core::Address;

/// What a request is for, as far as an engine says: the kinds the C header
/// has (`WT_RESOURCE_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Other,
    /// The top frame's own document.
    Document,
    /// A frame's document.
    Subdocument,
    Script,
    Stylesheet,
    Image,
    Font,
    Media,
    Xhr,
    Websocket,
    Ping,
}

impl Kind {
    /// The kind by its number in the C header. A number this core has not
    /// heard of is `Other`.
    #[must_use]
    pub fn from_code(code: u32) -> Kind {
        match code {
            1 => Kind::Document,
            2 => Kind::Subdocument,
            3 => Kind::Script,
            4 => Kind::Stylesheet,
            5 => Kind::Image,
            6 => Kind::Font,
            7 => Kind::Media,
            8 => Kind::Xhr,
            9 => Kind::Websocket,
            10 => Kind::Ping,
            _ => Kind::Other,
        }
    }

    /// The kind's name in the lists' rules.
    fn in_rules(self) -> &'static str {
        match self {
            Kind::Other => "other",
            Kind::Document => "document",
            Kind::Subdocument => "subdocument",
            Kind::Script => "script",
            Kind::Stylesheet => "stylesheet",
            Kind::Image => "image",
            Kind::Font => "font",
            Kind::Media => "media",
            Kind::Xhr => "xmlhttprequest",
            Kind::Websocket => "websocket",
            Kind::Ping => "ping",
        }
    }
}

/// Guard's answer for one request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The request is made.
    Allow,
    /// The request is not made, and the page goes on without it.
    Block,
}

/// The blocker, made from rules. With no rules it allows everything.
pub struct Guard {
    /// `None` with no rules: nothing is looked up, and nothing parsed.
    blocker: Option<Engine>,
}

impl Default for Guard {
    fn default() -> Guard {
        Guard::with_rules([])
    }
}

impl Guard {
    /// A blocker of these lists' rules: each a list's whole text, a rule to
    /// a line. What is no rule is ignored, as a comment is. Only the rules
    /// for requests are taken: what hides or mends a page's own content is
    /// not Guard's yet (064 §2).
    #[must_use]
    pub fn with_rules<'a>(lists: impl IntoIterator<Item = &'a str>) -> Guard {
        let mut rules = FilterSet::new(false);
        let mut any = false;
        let requests_only = ParseOptions {
            rule_types: RuleTypes::NetworkOnly,
            ..ParseOptions::default()
        };
        for list in lists {
            any = true;
            rules.add_filter_list(list.to_owned(), requests_only);
        }
        Guard {
            blocker: any.then(|| Engine::new_with_filter_set(rules)),
        }
    }

    /// Whether a page may fetch a thing: `url`, asked for by the page at
    /// `source_url` (the top frame's address), for `kind`.
    ///
    /// Allowed, whatever the rules: the top frame's own document (there is
    /// no page yet that says a page was blocked, 064 D5); and a request
    /// whose address is none, or has no host (`data:`, `blob:`, `about:`).
    /// A page whose own address is none, or has no host, is nobody's site:
    /// what it asks for is the third party's.
    #[must_use]
    pub fn decide(&self, url: &str, source_url: &str, kind: Kind) -> Verdict {
        let Some(blocker) = &self.blocker else {
            return Verdict::Allow;
        };
        if kind == Kind::Document {
            return Verdict::Allow;
        }
        let Ok(asked) = Address::parse(url) else {
            return Verdict::Allow;
        };
        let Some(host) = asked.host() else {
            return Verdict::Allow;
        };
        let page = Address::parse(source_url).ok();
        let page_host = page.as_ref().and_then(Address::host).unwrap_or("");
        let third_party = match page.as_ref().and_then(Address::site) {
            Some(site) => asked.site() != Some(site),
            None => true,
        };
        // (The method is not known to the fast path: a rule for one method
        // alone is read as a rule for a GET.)
        let request = Request::preparsed(
            asked.as_str(),
            host,
            page_host,
            kind.in_rules(),
            third_party,
            "GET",
        );
        if blocker.check_network_request(&request).should_block() {
            Verdict::Block
        } else {
            Verdict::Allow
        }
    }
}

/// The blocker is shared between an engine's threads.
const _: fn() = || {
    fn shared<T: Send + Sync>() {}
    shared::<Guard>();
};
