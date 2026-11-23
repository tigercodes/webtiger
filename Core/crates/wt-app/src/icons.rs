//! Sites' icons (046): the numbers windows are told a tab's icon by, and what
//! may be one. The pictures themselves are the store's: one for a host in a
//! login set.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use wt_core::{Address, LoginSetId, WebViewId};
use wt_proto::v1::{
    Icon, IconNamed, IconReceived, IconsPage, NamedIcon, QueryIcons, WebViewFetchIcon,
    host_request, response,
};
use wt_store::MAX_ICON_BYTES;

use crate::app::App;

/// The widest and the tallest a picture may be, in pixels.
const MAX_PIXELS: u32 = 256;

/// How long a tab's page is not heard again about its icon, once it has
/// been: a page that changes its icon all the time is heard once a minute.
pub(crate) const ICON_EVERY_MILLIS: i64 = 60_000;

/// The most pictures one answer carries: a shell asks for more in turn.
pub(crate) const MOST_ICONS_ASKED: usize = 64;

/// How large an icon is wanted, in pixels across: a tile is drawn from it,
/// and a tab's small icon (046 D4).
const WANTED_PIXELS: u32 = 64;

/// How large a touch icon is taken to be when its page does not say; and
/// any other icon.
const TOUCH_ICON_PIXELS: u32 = 180;
const UNSAID_ICON_PIXELS: u32 = 32;

/// The icon to fetch for a page, of those it names (046 D4). One that came
/// over `https` (or never leaves this device) and does not say it is SVG:
/// the one nearest [`WANTED_PIXELS`] across by what the page says of its
/// size, one that is at least that large before one that is smaller, and
/// of two alike the first named. With none that will do, the icon every
/// site may have, at the page's own origin; `None` when the page has no
/// origin to ask.
pub(crate) fn chosen_icon(page: &Address, named: &[NamedIcon]) -> Option<Address> {
    let mut best: Option<((bool, u32), Address)> = None;
    for icon in named {
        let called: Vec<String> = icon
            .rel
            .split_ascii_whitespace()
            .map(str::to_ascii_lowercase)
            .collect();
        let touch = called
            .iter()
            .any(|word| word == "apple-touch-icon" || word == "apple-touch-icon-precomposed");
        if !touch && !called.iter().any(|word| word == "icon") {
            continue;
        }
        let Ok(address) = Address::parse(&icon.url) else {
            continue;
        };
        // A drawing in SVG is no picture every system decodes alike.
        let svg = icon.r#type.to_ascii_lowercase().contains("svg")
            || address.path().to_ascii_lowercase().ends_with(".svg");
        if svg || !address.is_trustworthy() {
            continue;
        }
        let pixels = said_size(&icon.sizes).unwrap_or(if touch {
            TOUCH_ICON_PIXELS
        } else {
            UNSAID_ICON_PIXELS
        });
        let rank = (pixels < WANTED_PIXELS, pixels.abs_diff(WANTED_PIXELS));
        if best.as_ref().is_none_or(|(now, _)| rank < *now) {
            best = Some((rank, address));
        }
    }
    best.map(|(_, address)| address).or_else(|| {
        let origin = page.origin()?;
        Address::parse(&format!("{origin}/favicon.ico"))
            .ok()
            .filter(Address::is_trustworthy)
    })
}

/// How large a page says an icon is, by its larger side: of the sizes it
/// lists ("16x16 32x32"), the one nearest what is wanted. `None` when it
/// says nothing that is a size ("any", or no word at all).
fn said_size(sizes: &str) -> Option<u32> {
    sizes
        .split_ascii_whitespace()
        .filter_map(|size| {
            let (wide, tall) = size
                .to_ascii_lowercase()
                .split_once('x')
                .map(|(wide, tall)| (wide.parse::<u32>().ok(), tall.parse::<u32>().ok()))?;
            Some(wide?.max(tall?)).filter(|pixels| *pixels > 0)
        })
        .min_by_key(|pixels| (*pixels < WANTED_PIXELS, pixels.abs_diff(WANTED_PIXELS)))
}

/// The first bytes of every PNG.
const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Whether bytes may be kept as an icon: a PNG, by its first bytes and not
/// by anyone's word, of at most 256 pixels each way and at most 64 KB. (The
/// core reads what the picture says of itself, and decodes none of it.)
pub(crate) fn is_icon(png: &[u8]) -> bool {
    if png.len() > MAX_ICON_BYTES || !png.starts_with(&PNG) {
        return false;
    }
    // A PNG begins with its header: thirteen bytes, named IHDR, whose first
    // eight are how wide and how tall it is.
    let number = |at: usize| {
        png.get(at..at + 4)
            .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
            .map(u32::from_be_bytes)
    };
    let sized =
        |pixels: Option<u32>| pixels.is_some_and(|pixels| (1..=MAX_PIXELS).contains(&pixels));
    number(8) == Some(13)
        && png.get(12..16) == Some(b"IHDR")
        && sized(number(16))
        && sized(number(20))
}

/// The numbers of the icons known this run. A number is one picture of one
/// host in one login set, for as long as the core runs: a host whose
/// picture changes has another number.
#[derive(Debug, Default)]
pub(crate) struct Icons {
    /// The number of a host's icon in a login set; 0 when the store was
    /// asked and has none.
    of: BTreeMap<LoginSetId, BTreeMap<String, u64>>,
    /// Whose picture a number is.
    whose: BTreeMap<u64, (LoginSetId, String)>,
    /// The last number given.
    last: u64,
    /// The hosts a window was told have no icon because the store has not
    /// been asked yet: it is asked before the window is told again.
    asked: RefCell<BTreeSet<(LoginSetId, String)>>,
    /// The address the shell was last asked to fetch as a host's icon in a
    /// login set, this run (046 §5a): it is not asked for the same again.
    fetched: BTreeMap<LoginSetId, BTreeMap<String, String>>,
}

impl Icons {
    /// The number of a host's icon in a login set, for a window: 0 for
    /// none. A host the store has not been asked about is noted, to be
    /// asked about.
    pub(crate) fn number(&self, login_set: LoginSetId, host: &str) -> u64 {
        match self.of.get(&login_set).and_then(|hosts| hosts.get(host)) {
            Some(number) => *number,
            None => {
                self.asked.borrow_mut().insert((login_set, host.to_owned()));
                0
            }
        }
    }

    /// The hosts noted since this was last called.
    pub(crate) fn take_asked(&self) -> BTreeSet<(LoginSetId, String)> {
        std::mem::take(&mut *self.asked.borrow_mut())
    }

    /// Whether a host has an icon in a login set, as far as is known.
    pub(crate) fn has(&self, login_set: LoginSetId, host: &str) -> bool {
        self.of
            .get(&login_set)
            .and_then(|hosts| hosts.get(host))
            .is_some_and(|number| *number != 0)
    }

    /// A host has a picture in a login set, or another than it had: it is
    /// given a number of its own, and the number it had is nobody's.
    pub(crate) fn give(&mut self, login_set: LoginSetId, host: &str) {
        self.last = self.last.saturating_add(1);
        let had = self
            .of
            .entry(login_set)
            .or_default()
            .insert(host.to_owned(), self.last);
        if let Some(had) = had {
            self.whose.remove(&had);
        }
        self.whose.insert(self.last, (login_set, host.to_owned()));
    }

    /// The store has no icon for a host in a login set. One that is known
    /// to have one keeps it.
    pub(crate) fn none(&mut self, login_set: LoginSetId, host: &str) {
        self.of
            .entry(login_set)
            .or_default()
            .entry(host.to_owned())
            .or_insert(0);
    }

    /// Whether the shell is to be asked now to fetch an address as a host's
    /// icon in a login set: not when it was asked for that address already
    /// in this run. What is asked for is noted.
    pub(crate) fn fetch_is_new(&mut self, login_set: LoginSetId, host: &str, url: &str) -> bool {
        let asked = self.fetched.entry(login_set).or_default();
        if asked.get(host).is_some_and(|was| was == url) {
            return false;
        }
        asked.insert(host.to_owned(), url.to_owned());
        true
    }

    /// Whose picture a number is: the login set and the host.
    pub(crate) fn whose(&self, number: u64) -> Option<(LoginSetId, String)> {
        self.whose.get(&number).cloned()
    }

    /// A login set is gone (a private window closed, 037): its icons'
    /// numbers are nobody's.
    pub(crate) fn forget(&mut self, login_set: LoginSetId) {
        if let Some(hosts) = self.of.remove(&login_set) {
            for number in hosts.values() {
                self.whose.remove(number);
            }
        }
        self.asked
            .borrow_mut()
            .retain(|(asked, _)| *asked != login_set);
        self.fetched.remove(&login_set);
    }
}

impl App {
    /// An engine has the icon of a web view's page (046). A picture that may
    /// be an icon, of a page whose icon is taken, is kept for the page's
    /// host in the tab's login set: in the store, where a private window's
    /// is the one in memory. The picture the host has already is only said
    /// to be used now. One that comes a moment before its page has arrived
    /// waits for it.
    pub(crate) fn icon_received(&self, web_view: WebViewId, received: &IconReceived) {
        if is_icon(&received.png) {
            self.icon_offered(web_view, &received.page_url, &received.png, true);
        }
    }

    /// A page names its icons (046 §5a, WT-D33): the shell is asked to fetch
    /// the one the core chooses, once in a run for the page's host in its
    /// login set. The picture comes back as `IconReceived`, as an engine's
    /// own does. The request is on its way when this returns.
    pub(crate) fn icon_named(&self, web_view: WebViewId, named: &IconNamed) {
        let chosen = self.state.borrow_mut().icon_to_fetch(web_view, named);
        let Some(chosen) = chosen else {
            return;
        };
        let asked = self
            .host
            .request(host_request::Kind::WebViewFetchIcon(WebViewFetchIcon {
                web_view_id: web_view.get(),
                url: chosen.as_str().to_owned(),
                page_url: named.page_url.clone(),
            }));
        // Asked already; this only waits for the answer, so that a shell
        // that never gives one is given up on like any other.
        tokio::task::spawn_local(async move {
            let _ = asked.await;
        });
    }

    /// A page has arrived in a web view (046): an icon that was handed over
    /// for a page on its way, and waited, is that page's now, when it is the
    /// page that arrived; and is dropped when it is another.
    pub(crate) fn icon_waited(&self, web_view: WebViewId) {
        let waited = self.state.borrow_mut().take_waiting_icon(web_view);
        if let Some((page_url, png)) = waited {
            self.icon_offered(web_view, &page_url, &png, false);
        }
    }

    /// A picture that may be an icon, for the page named. Kept when that
    /// page is the one the tab shows; with `may_wait`, held for a page that
    /// is still on its way; otherwise nothing.
    fn icon_offered(&self, web_view: WebViewId, page_url: &str, png: &[u8], may_wait: bool) {
        let arrived = self.state.borrow().icon_page_arrived(web_view, page_url);
        match arrived {
            Some(true) => {}
            Some(false) if may_wait => {
                self.state.borrow_mut().icon_waits(web_view, page_url, png);
                return;
            }
            Some(false) | None => return,
        }
        let now = self.clock.now();
        let taker = self.state.borrow_mut().icon_taker(web_view, now);
        let Some((login_set, host)) = taker else {
            return;
        };
        let store = self.store_of(login_set);
        let had = store.borrow().icon(login_set, &host).ok().flatten();
        let changed = had.as_deref() != Some(png);
        let kept = if changed {
            store.borrow_mut().set_icon(login_set, &host, png, now)
        } else {
            store.borrow_mut().use_icons(login_set, &[&host], now)
        };
        // A store that refuses keeps what it had, and the windows are told
        // nothing new.
        if kept.is_ok() {
            self.state.borrow_mut().icon_kept(login_set, &host, changed);
        }
    }

    /// Asks the store about the hosts a window was just told have no icon
    /// because nobody had asked (046): tabs that came back with the session,
    /// or that are at a host for the first time this run. Whether any has
    /// one, so that the window is to be told again. The icons found are used
    /// now: a tab shows them.
    pub(crate) fn learn_asked_icons(&self) -> bool {
        let asked = self.state.borrow().icons.take_asked();
        if asked.is_empty() {
            return false;
        }
        let mut found: BTreeMap<LoginSetId, Vec<String>> = BTreeMap::new();
        for (login_set, host) in asked {
            let has = self
                .store_of(login_set)
                .borrow()
                .icon(login_set, &host)
                .is_ok_and(|png| png.is_some());
            let mut state = self.state.borrow_mut();
            if has {
                state.icon_kept(login_set, &host, false);
                found.entry(login_set).or_default().push(host);
            } else {
                state.icons.none(login_set, &host);
            }
        }
        if found.is_empty() {
            return false;
        }
        // (The clock is read only now that there is an icon to say it of.)
        let now = self.clock.now();
        for (login_set, hosts) in &found {
            let hosts: Vec<&str> = hosts.iter().map(String::as_str).collect();
            let _ = self
                .store_of(*login_set)
                .borrow_mut()
                .use_icons(*login_set, &hosts, now);
        }
        true
    }

    /// The pictures of the icons asked for by their numbers (046): of those
    /// the core has, in the order asked, and of no more than an answer
    /// carries.
    pub(crate) fn query_icons(&self, query: &QueryIcons) -> response::Kind {
        let mut icons = Vec::new();
        for number in query.icon_ids.iter().take(MOST_ICONS_ASKED) {
            let Some((login_set, host)) = self.state.borrow().icons.whose(*number) else {
                continue;
            };
            let kept = self.store_of(login_set).borrow().icon(login_set, &host);
            if let Ok(Some(png)) = kept {
                icons.push(Icon {
                    icon_id: *number,
                    png,
                });
            }
        }
        response::Kind::IconsPage(IconsPage { icons })
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        reason = "a test fails the way its assertions do"
    )]

    use super::*;

    /// A PNG's first bytes: of this many pixels each way.
    fn png(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = PNG.to_vec();
        bytes.extend_from_slice(&13_u32.to_be_bytes());
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes.extend_from_slice(&[8, 6, 0, 0, 0, 1, 2, 3, 4]);
        bytes
    }

    fn named(url: &str, rel: &str, sizes: &str, kind: &str) -> NamedIcon {
        NamedIcon {
            url: url.to_owned(),
            rel: rel.to_owned(),
            sizes: sizes.to_owned(),
            r#type: kind.to_owned(),
        }
    }

    /// The address chosen for a page at https://example.com/ that names these.
    fn chosen(icons: &[NamedIcon]) -> Option<String> {
        let page = Address::parse("https://example.com/a/page").ok()?;
        chosen_icon(&page, icons).map(|address| address.as_str().to_owned())
    }

    #[test]
    fn the_icon_nearest_the_size_wanted_is_chosen_of_those_that_will_do() {
        let small = named("https://example.com/16.png", "icon", "16x16", "image/png");
        let right = named("https://example.com/64.png", "icon", "64x64", "image/png");
        let large = named(
            "https://example.com/512.png",
            "icon",
            "512x512",
            "image/png",
        );
        let some = |url: &str| Some(url.to_owned());
        // The one that is the size wanted; whatever order they are named in.
        assert_eq!(
            chosen(&[small.clone(), right.clone(), large.clone()]),
            some("https://example.com/64.png")
        );
        assert_eq!(
            chosen(&[large.clone(), small.clone(), right]),
            some("https://example.com/64.png")
        );
        // With none of that size: one that is larger before one that is smaller, and of
        // the larger the nearest.
        let nearer = named("https://example.com/96.png", "icon", "96x96", "");
        assert_eq!(
            chosen(&[small.clone(), large.clone(), nearer]),
            some("https://example.com/96.png")
        );
        // Only smaller ones: the largest of them.
        let smaller = named("https://example.com/32.png", "icon", "32x32", "");
        assert_eq!(
            chosen(&[small.clone(), smaller]),
            some("https://example.com/32.png")
        );
        // An icon that lists several sizes is as near as its nearest; one that is not square
        // is as large as its larger side.
        let many = named(
            "https://example.com/many.ico",
            "shortcut icon",
            "16x16 64x64 256x256",
            "",
        );
        let wide = named("https://example.com/wide.png", "icon", "128x48", "");
        assert_eq!(
            chosen(&[wide.clone(), many]),
            some("https://example.com/many.ico")
        );
        // A touch icon that says nothing of its size is taken to be 180 across; any other,
        // 32. So a touch icon is chosen before an icon that says nothing, and an icon that
        // says it is 64 before both. Of two alike, the first named.
        let touch = named("https://example.com/touch.png", "apple-touch-icon", "", "");
        let silent = named("https://example.com/favicon.ico", "icon", "", "");
        let any = named("https://example.com/any.png", "ICON", "any", "");
        assert_eq!(
            chosen(&[silent.clone(), touch.clone()]),
            some("https://example.com/touch.png")
        );
        assert_eq!(
            chosen(&[touch, wide.clone()]),
            some("https://example.com/wide.png")
        );
        assert_eq!(
            chosen(&[silent, any]),
            some("https://example.com/favicon.ico")
        );
    }

    #[test]
    fn what_will_not_do_is_passed_over_and_with_nothing_left_the_sites_own_is_asked_for() {
        let some = |url: &str| Some(url.to_owned());
        let good = named("https://example.com/icon.png", "icon", "16x16", "image/png");
        // Not an icon by what the page calls it; a drawing in SVG, by its kind or by its
        // name; an address that did not come over https; an address that is none.
        let passed_over = [
            named("https://example.com/style.css", "stylesheet", "64x64", ""),
            named("https://example.com/pin.svg", "mask-icon", "64x64", ""),
            named(
                "https://example.com/drawing",
                "icon",
                "64x64",
                "image/svg+xml",
            ),
            named("https://example.com/drawing.SVG?v=2", "icon", "64x64", ""),
            named("http://example.com/plain.png", "icon", "64x64", "image/png"),
            named("not an address", "icon", "64x64", ""),
            named("data:image/png;base64,AAAA", "icon", "64x64", ""),
        ];
        let mut all = passed_over.to_vec();
        all.push(good);
        assert_eq!(chosen(&all), some("https://example.com/icon.png"));
        // With nothing that will do, and for a page that names none: the icon every site
        // may have, at the page's own origin.
        assert_eq!(
            chosen(&passed_over),
            some("https://example.com/favicon.ico")
        );
        assert_eq!(chosen(&[]), some("https://example.com/favicon.ico"));
        // An icon at another site's address is an icon: many sites keep theirs elsewhere.
        let elsewhere = named("https://files.example.net/icon.png", "icon", "64x64", "");
        assert_eq!(
            chosen(&[elsewhere]),
            some("https://files.example.net/icon.png")
        );
        // A page on this device may name an icon on this device, over http.
        let page = Address::parse("http://127.0.0.1:8080/page").unwrap();
        let own = named("http://127.0.0.1:8080/icon.png", "icon", "", "");
        assert_eq!(
            chosen_icon(&page, &[own]).map(|address| address.as_str().to_owned()),
            some("http://127.0.0.1:8080/icon.png")
        );
        assert_eq!(
            chosen_icon(&page, &[]).map(|address| address.as_str().to_owned()),
            some("http://127.0.0.1:8080/favicon.ico")
        );
        // A page that did not come over https is asked for nothing: its icon is not kept.
        let plain = Address::parse("http://plain.example.com/page").unwrap();
        assert_eq!(chosen_icon(&plain, &[]), None);
    }

    #[test]
    fn an_address_is_asked_for_once_for_a_host_in_a_login_set_until_the_login_set_is_gone() {
        let mut icons = Icons::default();
        let (one, two) = (
            LoginSetId(wt_core::Uuid::from_bytes([1; 16])),
            LoginSetId(wt_core::Uuid::from_bytes([2; 16])),
        );
        assert!(icons.fetch_is_new(one, "example.com", "https://example.com/a.png"));
        assert!(!icons.fetch_is_new(one, "example.com", "https://example.com/a.png"));
        // Another host, another login set and another address are each asked for.
        assert!(icons.fetch_is_new(one, "example.net", "https://example.com/a.png"));
        assert!(icons.fetch_is_new(two, "example.com", "https://example.com/a.png"));
        assert!(icons.fetch_is_new(one, "example.com", "https://example.com/b.png"));
        // And the one before it again, since the host's page names it again.
        assert!(icons.fetch_is_new(one, "example.com", "https://example.com/a.png"));
        // A login set that is gone takes what was asked for it along.
        icons.forget(two);
        assert!(icons.fetch_is_new(two, "example.com", "https://example.com/a.png"));
        assert!(!icons.fetch_is_new(one, "example.com", "https://example.com/a.png"));
    }

    #[test]
    fn an_icon_is_a_png_by_its_own_bytes_of_a_size_that_is_kept() {
        assert!(is_icon(&png(64, 64)));
        assert!(is_icon(&png(1, 1)));
        assert!(is_icon(&png(256, 256)));
        assert!(is_icon(&png(256, 16)));
        // Too wide, too tall, and of no size.
        assert!(!is_icon(&png(257, 64)));
        assert!(!is_icon(&png(64, 257)));
        assert!(!is_icon(&png(0, 64)));
        assert!(!is_icon(&png(64, 0)));
        assert!(!is_icon(&png(u32::MAX, 64)));
        // Not a PNG: nothing; another kind of picture; a PNG's first bytes and no more;
        // a PNG whose first part is not its header, or is a header of another length.
        assert!(!is_icon(b""));
        assert!(!is_icon(
            b"GIF89a\x40\x00\x40\x00 and the rest of a small picture"
        ));
        assert!(!is_icon(&PNG));
        assert!(!is_icon(&png(64, 64)[..20]));
        let mut named = png(64, 64);
        named[12..16].copy_from_slice(b"IDAT");
        assert!(!is_icon(&named));
        let mut longer = png(64, 64);
        longer[8..12].copy_from_slice(&14_u32.to_be_bytes());
        assert!(!is_icon(&longer));
        let mut marked = png(64, 64);
        marked[0] = 0x88;
        assert!(!is_icon(&marked));
        // Too many bytes; and the most there may be.
        let mut large = png(64, 64);
        large.resize(MAX_ICON_BYTES + 1, 0);
        assert!(!is_icon(&large));
        large.truncate(MAX_ICON_BYTES);
        assert!(is_icon(&large));
    }
}
