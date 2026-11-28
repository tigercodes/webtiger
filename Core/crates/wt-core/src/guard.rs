//! Guard's switches (067): everywhere; a Space's; a site's, within a login
//! set. The nearest that says anything wins. And which web view is whose,
//! so that whoever is asked about a request, on whatever thread, can tell
//! which switches it is under from what an engine says of it: the web view,
//! and the address of the page that asks.

use std::collections::BTreeMap;

use crate::{Address, LoginSetId, SpaceId, WebViewId};

/// Which of the three switches said what Guard is for a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaidBy {
    Everywhere,
    Space,
    Site,
}

/// Guard's switches as they stand, and the web views there are.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GuardSwitches {
    /// Guard is off everywhere, unless a Space or a site says otherwise. On
    /// until a person turns it off.
    pub off_everywhere: bool,
    /// The Spaces that say: on (`true`) or off. One that is not here says
    /// nothing.
    pub spaces: BTreeMap<SpaceId, bool>,
    /// The sites that say, in each login set: on (`true`) or off.
    pub sites: BTreeMap<LoginSetId, BTreeMap<String, bool>>,
    /// Each web view's Space and its logins.
    pub web_views: BTreeMap<WebViewId, (SpaceId, LoginSetId)>,
}

impl GuardSwitches {
    /// Whether Guard is on for a page of `site`, shown in `space` with
    /// `login_set`'s logins, and which switch said so: the site's when it
    /// says anything; otherwise the Space's; otherwise everywhere's. A page
    /// of no site has no site's switch.
    #[must_use]
    pub fn for_page(
        &self,
        space: SpaceId,
        login_set: LoginSetId,
        site: Option<&str>,
    ) -> (bool, SaidBy) {
        let of_site = site.and_then(|site| self.sites.get(&login_set)?.get(site));
        if let Some(on) = of_site {
            return (*on, SaidBy::Site);
        }
        if let Some(on) = self.spaces.get(&space) {
            return (*on, SaidBy::Space);
        }
        (!self.off_everywhere, SaidBy::Everywhere)
    }

    /// Whether Guard is on for what the page at `source_url` asks for in
    /// `web_view`: the fast path's question. A web view that is not known
    /// (one that has gone, or that a page opened a moment ago) is under
    /// everywhere's switch.
    #[must_use]
    pub fn on_for(&self, web_view: WebViewId, source_url: &str) -> bool {
        let Some((space, login_set)) = self.web_views.get(&web_view).copied() else {
            return !self.off_everywhere;
        };
        // The page's address is read only where some site of these logins
        // says anything.
        let site = self
            .sites
            .get(&login_set)
            .filter(|sites| !sites.is_empty())
            .and_then(|_| Address::parse(source_url).ok());
        let site = site.as_ref().and_then(Address::site);
        self.for_page(space, login_set, site).0
    }

    /// Whether a frame is to be kept from its site's own storage (071): the
    /// fast path's question for a frame, asked as its document starts, on
    /// the engines that do not keep a third party's storage apart
    /// themselves. `frame_origin` is the frame's origin and `top_origin` the
    /// origin of the top frame above it, both as the frame's own document
    /// has them; `page_url` is the page's address as the shell knows it.
    ///
    /// Kept apart, unless the core can say otherwise:
    ///
    /// - A shell learns of a new page a moment after the engine has it. A
    ///   frame whose top frame is not of the origin of the page the shell
    ///   knows is in a page nothing is known of here: it is kept apart, and
    ///   judged neither against the page before nor by that page's switches.
    ///   So what a page says of its top frame can only keep it apart: a page
    ///   that lies learns nothing of a switch that is not its own page's.
    /// - Where Guard is off for the page, nothing is kept apart (067).
    /// - A frame of the page's own site has that site's storage.
    #[must_use]
    pub fn keeps_apart(
        &self,
        web_view: WebViewId,
        frame_origin: &str,
        top_origin: &str,
        page_url: &str,
    ) -> bool {
        let Ok(page) = Address::parse(page_url) else {
            return true;
        };
        let Some(page_origin) = page.origin() else {
            return true;
        };
        let top_origin = Address::parse(top_origin).ok().and_then(|top| top.origin());
        if top_origin.as_deref() != Some(page_origin.as_str()) {
            return true;
        }
        if !self.on_for(web_view, page_url) {
            return false;
        }
        let frame = Address::parse(frame_origin).ok();
        let frame_site = frame.as_ref().and_then(Address::site);
        frame_site.is_none() || frame_site != page.site()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Uuid;

    const WORK: SpaceId = SpaceId(2);
    const HOME: SpaceId = SpaceId(1);
    const MINE: LoginSetId = LoginSetId(Uuid::from_random_bytes([0x11; 16]));
    const OTHER: LoginSetId = LoginSetId(Uuid::from_random_bytes([0x22; 16]));

    fn switches(off_everywhere: bool, space: Option<bool>, site: Option<bool>) -> GuardSwitches {
        let mut switches = GuardSwitches {
            off_everywhere,
            ..GuardSwitches::default()
        };
        if let Some(on) = space {
            switches.spaces.insert(WORK, on);
        }
        if let Some(on) = site {
            switches
                .sites
                .entry(MINE)
                .or_default()
                .insert("example.com".to_owned(), on);
        }
        switches.web_views.insert(WebViewId(7), (WORK, MINE));
        switches
    }

    #[test]
    fn the_nearest_switch_that_says_anything_wins() {
        // Everywhere off or on; the Space saying nothing, on, off; the site the same:
        // eighteen in all. The site's when it says; else the Space's; else everywhere's.
        for off_everywhere in [false, true] {
            for space in [None, Some(true), Some(false)] {
                for site in [None, Some(true), Some(false)] {
                    let expected = match (site, space) {
                        (Some(on), _) => (on, SaidBy::Site),
                        (None, Some(on)) => (on, SaidBy::Space),
                        (None, None) => (!off_everywhere, SaidBy::Everywhere),
                    };
                    let switches = switches(off_everywhere, space, site);
                    let said = format!(
                        "everywhere off: {off_everywhere}, the Space {space:?}, the site {site:?}"
                    );
                    assert_eq!(
                        switches.for_page(WORK, MINE, Some("example.com")),
                        expected,
                        "{said}"
                    );
                    assert_eq!(
                        switches.on_for(WebViewId(7), "https://www.example.com/a/page?x=1"),
                        expected.0,
                        "{said}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_sites_switch_is_its_logins_and_a_spaces_is_its_own() {
        let switches = switches(false, Some(false), Some(true));
        // Another site in the same logins, and the same site in other logins, are under
        // the Space's switch and everywhere's.
        assert_eq!(
            switches.for_page(WORK, MINE, Some("example.org")),
            (false, SaidBy::Space)
        );
        assert_eq!(
            switches.for_page(WORK, OTHER, Some("example.com")),
            (false, SaidBy::Space)
        );
        // Another Space with the same logins shares the site's switch, and not the Space's.
        assert_eq!(
            switches.for_page(HOME, MINE, Some("example.com")),
            (true, SaidBy::Site)
        );
        assert_eq!(
            switches.for_page(HOME, MINE, Some("example.org")),
            (true, SaidBy::Everywhere)
        );
        // A page of no site has no site's switch.
        assert_eq!(switches.for_page(WORK, MINE, None), (false, SaidBy::Space));
    }

    #[test]
    fn the_fast_paths_question_is_answered_from_the_web_view_and_the_pages_address() {
        let switches = switches(false, None, Some(false));
        // The site is what somebody registered: any host of it, whatever the path.
        for page in [
            "https://example.com/",
            "http://shop.example.com:8080/basket",
            "https://EXAMPLE.com/a",
        ] {
            assert!(!switches.on_for(WebViewId(7), page), "{page}");
        }
        // Another site; a host that only ends alike; a page that is no address, or of no
        // site: under everywhere's switch, which is on.
        for page in [
            "https://example.org/",
            "https://notexample.com/",
            "about:blank",
            "",
            "not an address",
        ] {
            assert!(switches.on_for(WebViewId(7), page), "{page}");
        }
        // A web view that is not known is under everywhere's switch, whatever the page.
        assert!(switches.on_for(WebViewId(99), "https://example.com/"));
        let mut off = switches.clone();
        off.off_everywhere = true;
        assert!(!off.on_for(WebViewId(99), "https://example.org/"));
        // A site that turns Guard on where everything else is off.
        let on = self::switches(true, Some(false), Some(true));
        assert!(on.on_for(WebViewId(7), "https://example.com/"));
        assert!(!on.on_for(WebViewId(7), "https://example.org/"));
    }

    const PAGE: &str = "https://www.example.com/a/page?x=1";
    const TOP: &str = "https://www.example.com";

    #[test]
    fn a_frame_of_another_site_than_the_pages_is_kept_from_its_storage() {
        let switches = switches(false, None, None);
        let apart = |frame: &str| switches.keeps_apart(WebViewId(7), frame, TOP, PAGE);
        // Of the page's own site: its origin, another host of the site, another port,
        // another scheme, whatever the case.
        for frame in [
            "https://www.example.com",
            "https://example.com",
            "https://accounts.shop.example.com",
            "https://www.example.com:8443",
            "http://www.example.com",
            "https://WWW.Example.COM",
        ] {
            assert!(!apart(frame), "{frame}");
        }
        // Of another site: another name, one that only ends alike, one under another
        // suffix, an address of numbers.
        for frame in [
            "https://tracker.example.net",
            "https://notexample.com",
            "https://example.com.evil.example.org",
            "https://example.co.uk",
            "http://127.0.0.1:8080",
            "http://localhost:8080",
        ] {
            assert!(apart(frame), "{frame}");
        }
        // An origin that is none: a frame that is nobody's is nobody the page knows.
        for frame in ["null", "", "not an address", "about:blank", "//example.com"] {
            assert!(apart(frame), "{frame:?}");
        }
        // Hosts with nothing registered are their own sites: two names of one machine
        // are two, and each is itself whatever the port.
        let local = |frame: &str| {
            switches.keeps_apart(
                WebViewId(7),
                frame,
                "http://127.0.0.1:8080",
                "http://127.0.0.1:8080/holds",
            )
        };
        assert!(!local("http://127.0.0.1:8080"));
        assert!(!local("http://127.0.0.1:9090"));
        assert!(local("http://localhost:8080"));
        assert!(local("http://127.0.0.2:8080"));
    }

    #[test]
    fn a_frame_in_a_page_the_shell_does_not_know_is_kept_apart() {
        // Whatever the switches say: the page the shell knows is not the frame's.
        for switches in [
            switches(false, None, None),
            switches(true, Some(false), Some(false)),
        ] {
            let apart = |top: &str, page: &str| {
                switches.keeps_apart(WebViewId(7), "https://www.example.com", top, page)
            };
            // The top frame is of another site than the page the shell knows, of another
            // host of its site, of another port or scheme, or of no origin at all.
            for top in [
                "https://www.example.org",
                "https://example.com",
                "https://www.example.com:8443",
                "http://www.example.com",
                "null",
                "",
                "not an address",
            ] {
                assert!(apart(top, PAGE), "{top:?}");
            }
            // The shell knows no page of the web: none, an empty one, one made by a script.
            for page in ["", "about:blank", "data:text/html,hello", "not an address"] {
                assert!(apart(TOP, page), "{page:?}");
                assert!(apart(page, page), "{page:?}");
            }
        }
        // The top frame's origin is the page's however it is written.
        let switches = switches(false, None, None);
        for top in [
            "https://www.example.com",
            "https://WWW.example.com:443",
            "https://www.example.com/",
        ] {
            assert!(
                !switches.keeps_apart(WebViewId(7), "https://example.com", top, PAGE),
                "{top}"
            );
        }
    }

    #[test]
    fn where_guard_is_off_for_the_page_no_frame_is_kept_apart() {
        let tracker = "https://tracker.example.net";
        // All eighteen: a frame of another site is kept apart exactly where Guard is on
        // for the page, and one of the page's site never.
        for off_everywhere in [false, true] {
            for space in [None, Some(true), Some(false)] {
                for site in [None, Some(true), Some(false)] {
                    let on = site.or(space).unwrap_or(!off_everywhere);
                    let switches = switches(off_everywhere, space, site);
                    let said = format!(
                        "everywhere off: {off_everywhere}, the Space {space:?}, the site {site:?}"
                    );
                    assert_eq!(
                        switches.keeps_apart(WebViewId(7), tracker, TOP, PAGE),
                        on,
                        "{said}"
                    );
                    assert!(
                        !switches.keeps_apart(WebViewId(7), "https://example.com", TOP, PAGE),
                        "{said}"
                    );
                    // A frame that is nobody's is as the WebView has it too, where Guard is off.
                    assert_eq!(
                        switches.keeps_apart(WebViewId(7), "null", TOP, PAGE),
                        on,
                        "{said}"
                    );
                }
            }
        }
        // The switches are the page's, and not the frame's own site's: a site that is
        // switched off is kept apart in another site's page.
        let switches = self::switches(false, None, Some(false));
        assert!(switches.keeps_apart(
            WebViewId(7),
            "https://www.example.com",
            "https://news.example.org",
            "https://news.example.org/story",
        ));
        // A web view that is not known is under everywhere's switch.
        assert!(switches.keeps_apart(WebViewId(99), tracker, TOP, PAGE));
        let mut off = switches.clone();
        off.off_everywhere = true;
        assert!(!off.keeps_apart(WebViewId(99), tracker, TOP, PAGE));
    }
}
