//! Importing from Arc (002).
//!
//! Reads Arc's data folder and nothing else, and changes nothing in it. It
//! never opens Arc's account or sync keys, its cookies, its passwords or its
//! history.

mod extras;
mod json;
mod sidebar;

use std::path::{Path, PathBuf};

use crate::{ImportError, Portable};

/// What to take besides the Spaces and their Sites.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// The unpinned "Today" tabs.
    pub today: bool,
    /// Archived tabs.
    pub archive: bool,
    /// Legacy Boosts, as Tweaks.
    pub boosts: bool,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            today: true,
            archive: true,
            boosts: true,
        }
    }
}

/// Reads an Arc data folder into a portable file.
///
/// # Errors
/// `arc.folder_missing` when there is no such folder, and
/// `arc.sidebar_missing`, `arc.sidebar_too_large`, `arc.sidebar_unreadable`
/// or `arc.sidebar_malformed` for its sidebar file. The other files are
/// optional: one that cannot be read becomes a note, not an error.
pub fn import(folder: &Path, options: Options) -> Result<Portable, ImportError> {
    if !folder.is_dir() {
        return Err(ImportError::new("arc.folder_missing"));
    }
    let root = json::read(&folder.join("StorableSidebar.json"))
        .map_err(|error| json::sidebar_error(&error))?;

    let mut notes = json::Notes::default();
    let names = extras::profile_names(folder, &mut notes);
    let read = sidebar::read(&root, &names, options.today, &mut notes)
        .ok_or(ImportError::new("arc.sidebar_malformed"))?;

    let mut file = Portable::new("arc");
    file.login_sets = read.login_sets;
    file.spaces = read.spaces;
    file.link_rules = extras::link_rules(folder, &read.space_ids, &mut notes);
    if options.archive {
        file.archive = extras::archive(folder, &read.space_ids, &mut notes);
    }
    if options.boosts {
        file.tweaks = extras::boosts(folder, &mut notes);
    }
    file.notes = notes.into_notes();
    Ok(file)
}

/// Where Arc keeps its data on this machine, when it is there.
#[must_use]
pub fn default_folder() -> Option<PathBuf> {
    default_candidates().into_iter().find(|path| path.is_dir())
}

#[cfg(target_os = "macos")]
fn default_candidates() -> Vec<PathBuf> {
    std::env::var_os("HOME")
        .map(|home| {
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Arc")
        })
        .into_iter()
        .collect()
}

/// On Windows Arc is a packaged app: its folder sits under a package
/// directory whose name starts with `TheBrowserCompany.Arc`.
#[cfg(target_os = "windows")]
fn default_candidates() -> Vec<PathBuf> {
    let Some(local) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) else {
        return Vec::new();
    };
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(local.join("Packages"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("TheBrowserCompany.Arc")
        })
        .map(|entry| entry.path().join("LocalCache").join("Local").join("Arc"))
        .collect();
    candidates.sort();
    candidates
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn default_candidates() -> Vec<PathBuf> {
    Vec::new()
}
