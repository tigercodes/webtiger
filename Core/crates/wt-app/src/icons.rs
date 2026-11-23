//! Sites' icons (046): the numbers windows are told a tab's icon by, and what
//! may be one. The pictures themselves are the store's: one for a host in a
//! login set.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use wt_core::{LoginSetId, WebViewId};
use wt_proto::v1::{Icon, IconReceived, IconsPage, QueryIcons, response};
use wt_store::MAX_ICON_BYTES;

use crate::app::App;

/// The widest and the tallest a picture may be, in pixels.
const MAX_PIXELS: u32 = 256;

/// How long a tab's page is not heard again about its icon, once it has
/// been: a page that changes its icon all the time is heard once a minute.
pub(crate) const ICON_EVERY_MILLIS: i64 = 60_000;

/// The most pictures one answer carries: a shell asks for more in turn.
pub(crate) const MOST_ICONS_ASKED: usize = 64;

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
    }
}

impl App {
    /// An engine has the icon of a web view's page (046). A picture that may
    /// be an icon, of a page whose icon is taken, is kept for the page's
    /// host in the tab's login set: in the store, where a private window's
    /// is the one in memory. The picture the host has already is only said
    /// to be used now.
    pub(crate) fn icon_received(&self, web_view: WebViewId, received: &IconReceived) {
        if !is_icon(&received.png) {
            return;
        }
        let now = self.clock.now();
        let taker = self
            .state
            .borrow_mut()
            .icon_taker(web_view, &received.page_url, now);
        let Some((login_set, host)) = taker else {
            return;
        };
        let store = self.store_of(login_set);
        let had = store.borrow().icon(login_set, &host).ok().flatten();
        let changed = had.as_deref() != Some(received.png.as_slice());
        let kept = if changed {
            store
                .borrow_mut()
                .set_icon(login_set, &host, &received.png, now)
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
