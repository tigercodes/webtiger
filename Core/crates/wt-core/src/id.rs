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

    /// The next id. Saturates rather than wrapping: 2^64 ids outlive the app.
    pub fn mint(&mut self) -> u64 {
        let id = self.next;
        self.next = self.next.saturating_add(1);
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
}
