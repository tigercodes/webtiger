//! Importers from other browsers (000 §5.4), and the portable file they
//! write (002 §6).
//!
//! An importer reads another browser's files and never changes them. What
//! it produces is a [`Portable`]: WebTiger's own interchange format, the
//! same whatever it was read from. It holds a person's Spaces, addresses
//! and titles, so nothing here logs, and a [`Summary`] and an
//! [`ImportError`] carry counts and codes only.

pub mod arc;
pub mod portable;

pub use portable::{
    Archived, Icon, LinkRule, LoginSet, Match, Node, Note, Portable, PortableError, RunAt, Source,
    Space, Summary, Tab, Target, Tweak,
};

/// Why an import could not be made, as a code ("arc.sidebar_missing").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImportError {
    code: &'static str,
}

impl ImportError {
    pub(crate) const fn new(code: &'static str) -> ImportError {
        ImportError { code }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        self.code
    }
}

impl std::fmt::Display for ImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code)
    }
}

impl std::error::Error for ImportError {}
