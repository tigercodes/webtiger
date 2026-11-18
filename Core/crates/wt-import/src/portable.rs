//! The portable file (002 §6): Spaces, their Groups and Sites, Favourites,
//! temporary tabs, link rules, Tweaks and an archive, as one JSON document.
//!
//! It is WebTiger's own format, not a copy of any source's. Every importer
//! writes it, and WebTiger's own export will.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The `format` every portable file carries.
pub const FORMAT: &str = "webtiger.portable";
/// The version this build writes, and the newest it reads.
pub const VERSION: u32 = 1;

/// A portable file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Portable {
    pub format: String,
    pub version: u32,
    pub source: Source,
    pub login_sets: Vec<LoginSet>,
    pub spaces: Vec<Space>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_rules: Vec<LinkRule>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tweaks: Vec<Tweak>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub archive: Vec<Archived>,
    /// What was left out or changed on the way, as codes and counts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
}

/// Where the file's contents came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// "arc", "webtiger", …
    pub app: String,
}

/// An identity. The key names it inside this file only; no cookie or
/// credential is ever in a portable file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginSet {
    pub key: String,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Space {
    /// Empty when the source gave it none.
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<Icon>,
    /// `#RRGGBB`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
    /// A [`LoginSet::key`]. Spaces that name the same one share logins.
    pub login_set: String,
    /// The row of tiles at the top of the Space: Sites only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub favourites: Vec<Node>,
    /// The saved tree.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sites: Vec<Node>,
    /// Temporary tabs, in sidebar order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tabs: Vec<Tab>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Icon {
    Emoji(String),
    /// A name from the source app's own icon set.
    Name(String),
    /// An SF Symbol name.
    Symbol(String),
}

/// A row of the saved tree.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Node {
    /// A folder. Groups nest.
    Group { name: String, children: Vec<Node> },
    Site {
        title: String,
        address: String,
        /// Milliseconds since 1970-01-01 UTC.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        added_at: Option<i64>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tab {
    pub title: String,
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<i64>,
}

/// "An address like this opens there."
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkRule {
    #[serde(rename = "match")]
    pub matching: Match,
    pub pattern: String,
    pub target: Target,
    /// With [`Target::Space`]: an index into [`Portable::spaces`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Match {
    Contains,
    Equals,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Space,
    MostRecentSpace,
    QuickWindow,
}

/// A page fix: CSS and script for matching pages.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tweak {
    pub name: String,
    /// Match patterns, such as `*://*.example.com/*`.
    pub matches: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    pub run_at: RunAt,
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunAt {
    DocumentStart,
    DocumentIdle,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Archived {
    pub title: String,
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<i64>,
    /// An index into [`Portable::spaces`], when the Space still exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space: Option<usize>,
}

/// Something left out or changed, counted. Never an address or a title.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub code: String,
    pub count: u64,
}

/// Why a file is not a portable file this build can read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortableError {
    /// Not JSON, or not this shape.
    Malformed,
    /// Its `format` is not [`FORMAT`].
    WrongFormat,
    /// Written by a newer WebTiger than this one.
    Newer,
}

impl PortableError {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            PortableError::Malformed => "portable.malformed",
            PortableError::WrongFormat => "portable.wrong_format",
            PortableError::Newer => "portable.newer",
        }
    }
}

impl Portable {
    /// An empty file from a source.
    #[must_use]
    pub fn new(app: &str) -> Portable {
        Portable {
            format: FORMAT.to_owned(),
            version: VERSION,
            source: Source {
                app: app.to_owned(),
            },
            login_sets: Vec::new(),
            spaces: Vec::new(),
            link_rules: Vec::new(),
            tweaks: Vec::new(),
            archive: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Reads a portable file.
    ///
    /// # Errors
    /// When it is not one, or is newer than this build understands.
    pub fn from_json(text: &str) -> Result<Portable, PortableError> {
        // The format and version are looked at first, so a newer file is
        // called newer rather than malformed.
        let head: serde_json::Value =
            serde_json::from_str(text).map_err(|_| PortableError::Malformed)?;
        if head.get("format").and_then(serde_json::Value::as_str) != Some(FORMAT) {
            return Err(PortableError::WrongFormat);
        }
        match head.get("version").and_then(serde_json::Value::as_u64) {
            Some(version) if version > u64::from(VERSION) => return Err(PortableError::Newer),
            Some(_) => {}
            None => return Err(PortableError::Malformed),
        }
        serde_json::from_value(head).map_err(|_| PortableError::Malformed)
    }

    /// The file as JSON, indented, with a final newline.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).unwrap_or_default();
        text.push('\n');
        text
    }

    /// What the file holds, as counts.
    #[must_use]
    pub fn summary(&self) -> Summary {
        let mut summary = Summary {
            spaces: self.spaces.len(),
            login_sets: self.login_sets.len(),
            link_rules: self.link_rules.len(),
            tweaks: self.tweaks.len(),
            archived: self.archive.len(),
            notes: self
                .notes
                .iter()
                .map(|note| (note.code.clone(), note.count))
                .collect(),
            ..Summary::default()
        };
        for space in &self.spaces {
            summary.tabs += space.tabs.len();
            summary.favourites += space.favourites.len();
            count(&space.sites, &mut summary);
        }
        summary
    }
}

fn count(nodes: &[Node], summary: &mut Summary) {
    for node in nodes {
        match node {
            Node::Group { children, .. } => {
                summary.groups += 1;
                count(children, summary);
            }
            Node::Site { .. } => summary.sites += 1,
        }
    }
}

/// What an import found: counts and codes, safe to print.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub spaces: usize,
    pub login_sets: usize,
    pub groups: usize,
    pub sites: usize,
    /// Favourites as the Spaces show them: one shared by three Spaces counts
    /// three times.
    pub favourites: usize,
    pub tabs: usize,
    pub link_rules: usize,
    pub tweaks: usize,
    pub archived: usize,
    pub notes: BTreeMap<String, u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_round_trips() {
        let mut file = Portable::new("arc");
        file.login_sets.push(LoginSet {
            key: "default".to_owned(),
            name: "Personal".to_owned(),
        });
        file.spaces.push(Space {
            name: "Work".to_owned(),
            icon: Some(Icon::Emoji("💼".to_owned())),
            colour: Some("#6CBAD9".to_owned()),
            login_set: "default".to_owned(),
            favourites: Vec::new(),
            sites: vec![Node::Group {
                name: "Docs".to_owned(),
                children: vec![Node::Site {
                    title: "Handbook".to_owned(),
                    address: "https://docs.example.com/".to_owned(),
                    added_at: Some(1_722_470_400_000),
                }],
            }],
            tabs: Vec::new(),
        });
        let text = file.to_json();
        assert!(text.contains(r#""kind": "group""#), "{text}");
        assert!(text.contains(r#""emoji": "💼""#), "{text}");
        assert!(!text.contains("favourites"), "empty lists are left out");
        assert_eq!(Portable::from_json(&text).unwrap(), file);
        let summary = file.summary();
        assert_eq!((summary.spaces, summary.groups, summary.sites), (1, 1, 1));
    }

    #[test]
    fn other_formats_and_newer_versions_are_refused() {
        assert_eq!(
            Portable::from_json("not json"),
            Err(PortableError::Malformed)
        );
        assert_eq!(
            Portable::from_json(r#"{"format":"netscape.bookmarks","version":1}"#),
            Err(PortableError::WrongFormat)
        );
        assert_eq!(
            Portable::from_json(r#"{"format":"webtiger.portable","version":2}"#),
            Err(PortableError::Newer)
        );
        assert_eq!(
            Portable::from_json(r#"{"format":"webtiger.portable"}"#),
            Err(PortableError::Malformed)
        );
        assert_eq!(
            Portable::from_json(r#"{"format":"webtiger.portable","version":1}"#),
            Err(PortableError::Malformed),
            "a file needs its source, login sets and spaces"
        );
    }
}
