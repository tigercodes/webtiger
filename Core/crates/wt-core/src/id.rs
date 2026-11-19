//! Typed ids. A tab is never mistaken for a window, and the contract's `0`
//! ("none") is never minted.

use crate::uuid::Uuid;

macro_rules! id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u64);

        impl $name {
            /// The id as the contract carries it.
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(value: u64) -> $name {
                $name(value)
            }
        }
    };
}

id!(
    /// A window.
    WindowId
);
id!(
    /// A tab: a browsing context, with or without a web view.
    TabId
);
id!(
    /// A web view the shell holds for a tab.
    WebViewId
);
impl WebViewId {
    /// The bit that marks a web view's id as the shell's own. The core mints
    /// every other web view's id and never sets it; a shell sets it on the
    /// one web view it makes unasked, for a page that asked for a window
    /// (006).
    pub const THE_SHELLS: u64 = 1 << 63;

    /// Whether the shell chose this id.
    #[must_use]
    pub const fn is_the_shells(self) -> bool {
        self.0 & WebViewId::THE_SHELLS != 0
    }
}

id!(
    /// A Space.
    SpaceId
);
id!(
    /// A shell's subscription to a model.
    SubscriptionId
);

/// A login set: an identity, with one data store per engine. A UUID, because
/// the engines name their stores by one (`WKWebsiteDataStore(forIdentifier:)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoginSetId(pub Uuid);

impl std::fmt::Display for LoginSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Mints ids from 1 upwards and never reuses one, so a stale id names nothing
/// rather than something else.
#[derive(Debug)]
pub struct IdMint {
    next: u64,
}

impl IdMint {
    #[must_use]
    pub const fn new() -> IdMint {
        IdMint { next: 1 }
    }

    /// The next id. It stays below [`WebViewId::THE_SHELLS`], the half of
    /// the ids that is a shell's to choose from, and saturates there rather
    /// than wrapping: 2^63 ids outlive the app.
    pub fn mint(&mut self) -> u64 {
        let id = self.next;
        self.next = self.next.saturating_add(1).min(WebViewId::THE_SHELLS - 1);
        id
    }
}

impl Default for IdMint {
    fn default() -> IdMint {
        IdMint::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_start_at_one_and_never_repeat() {
        let mut mint = IdMint::new();
        assert_eq!(mint.mint(), 1);
        assert_eq!(mint.mint(), 2);
        assert_eq!(TabId::from(mint.mint()).get(), 3);
    }

    #[test]
    fn the_core_never_mints_an_id_that_is_a_shells() {
        assert!(!WebViewId(1).is_the_shells());
        assert!(WebViewId(WebViewId::THE_SHELLS | 1).is_the_shells());
        // Even at the very end of its ids the mint stops short of the shell's half.
        let mut mint = IdMint {
            next: WebViewId::THE_SHELLS - 2,
        };
        for _ in 0..4 {
            assert!(!WebViewId(mint.mint()).is_the_shells());
        }
    }
}
