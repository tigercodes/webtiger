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
}
