//! The files beside the sidebar (002 §4): link rules, the archive, legacy
//! Boosts, and the names of Arc's profiles.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use super::json::{Notes, at, is_case, objects, read_optional, text, time, web_address};
use crate::portable::{Archived, LinkRule, Match, RunAt, Target, Tweak};

/// The largest Boost stylesheet or script read.
const MAX_BOOST_FILE: u64 = 1024 * 1024;
/// How many Boost folders are looked at.
const MAX_BOOSTS: usize = 1000;

/// The index of an Arc Space among the imported ones.
fn space_index(space_ids: &[String], id: Option<&str>) -> Option<usize> {
    let id = id?;
    space_ids.iter().position(|known| known == id)
}

/// `StorableLinkRouting.json`: Arc's Air Traffic Control.
pub(crate) fn link_rules(folder: &Path, space_ids: &[String], notes: &mut Notes) -> Vec<LinkRule> {
    let Some(file) = read_optional(
        &folder.join("StorableLinkRouting.json"),
        notes,
        "arc.link_rules_unreadable",
    ) else {
        return Vec::new();
    };
    let mut rules = Vec::new();
    for rule in objects(file.get("rules")) {
        // A rule with one address condition is understood. What several
        // conditions mean together (all, or any) is not known, and a
        // condition that is not an address match has no equivalent.
        let conditions: Vec<&Value> = objects(rule.get("sourceComponents")).collect();
        let condition = match conditions.as_slice() {
            [only] => at(only, &["componentType", "urlMatch"]),
            _ => None,
        };
        let matched = condition.and_then(|condition| {
            let kind = condition.get("_0")?;
            let matching = if is_case(kind, "contains") {
                Match::Contains
            } else if is_case(kind, "matches") {
                Match::Equals
            } else {
                return None;
            };
            Some((matching, text(condition, "_1")?))
        });
        let Some((matching, pattern)) = matched else {
            notes.add("arc.rule_unsupported");
            continue;
        };
        let destination = rule.get("destination");
        let (target, space) = if destination.is_some_and(|d| is_case(d, "littleArc")) {
            (Target::QuickWindow, None)
        } else if let Some(space) = destination.and_then(|d| at(d, &["space", "_0"])) {
            if is_case(space, "mostRecent") {
                (Target::MostRecentSpace, None)
            } else {
                let id = at(space, &["space", "_0"]).and_then(Value::as_str);
                match space_index(space_ids, id) {
                    Some(index) => (Target::Space, Some(index)),
                    None => {
                        notes.add("arc.rule_space_missing");
                        continue;
                    }
                }
            }
        } else {
            notes.add("arc.rule_unsupported");
            continue;
        };
        rules.push(LinkRule {
            matching,
            pattern: pattern.to_owned(),
            target,
            space,
        });
    }
    rules
}

/// `StorableArchiveItems.json`: archived tabs.
pub(crate) fn archive(folder: &Path, space_ids: &[String], notes: &mut Notes) -> Vec<Archived> {
    let Some(file) = read_optional(
        &folder.join("StorableArchiveItems.json"),
        notes,
        "arc.archive_unreadable",
    ) else {
        return Vec::new();
    };
    let mut archived = Vec::new();
    for entry in objects(file.get("items")) {
        let Some(item) = entry.get("sidebarItem") else {
            continue;
        };
        let Some(tab) = at(item, &["data", "tab"]) else {
            notes.add("arc.unknown_item_skipped");
            continue;
        };
        let Some(address) = web_address(text(tab, "savedURL")) else {
            notes.add("arc.address_skipped");
            continue;
        };
        archived.push(Archived {
            title: text(item, "title")
                .or_else(|| text(tab, "savedTitle"))
                .unwrap_or_default()
                .to_owned(),
            address,
            archived_at: time(entry.get("archivedAt")),
            space: space_index(
                space_ids,
                at(entry, &["source", "space", "_0"]).and_then(Value::as_str),
            ),
        });
    }
    archived
}

/// `User Data/Local State`: what Arc calls each profile folder.
pub(crate) fn profile_names(folder: &Path, notes: &mut Notes) -> BTreeMap<String, String> {
    let mut names = BTreeMap::new();
    let Some(file) = read_optional(
        &folder.join("User Data").join("Local State"),
        notes,
        "arc.profiles_unreadable",
    ) else {
        return names;
    };
    if let Some(cache) = at(&file, &["profile", "info_cache"]).and_then(Value::as_object) {
        for (directory, info) in cache {
            if let Some(name) = text(info, "name")
                && name != "__ARC_SYSTEM_PROFILE"
            {
                names.insert(directory.clone(), name.to_owned());
            }
        }
    }
    names
}

/// `boosts/<id>/`: legacy Boosts, each a small extension folder.
pub(crate) fn boosts(folder: &Path, notes: &mut Notes) -> Vec<Tweak> {
    let Ok(entries) = std::fs::read_dir(folder.join("boosts")) else {
        return Vec::new();
    };
    // In name order, so the same folder always imports the same way.
    let mut directories: Vec<_> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    directories.sort();
    directories.truncate(MAX_BOOSTS);
    directories
        .iter()
        .filter_map(|directory| boost(directory, notes))
        .collect()
}

fn boost(directory: &Path, notes: &mut Notes) -> Option<Tweak> {
    let config = read_optional(
        &directory.join("boost.config.json"),
        notes,
        "arc.boost_unreadable",
    )?;
    let manifest = read_optional(
        &directory.join("manifest.json"),
        notes,
        "arc.boost_unreadable",
    );
    let script_entry = manifest
        .as_ref()
        .and_then(|manifest| objects(manifest.get("content_scripts")).next());

    let component = |name: &str| -> Option<&str> {
        let component = at(&config, &["components", name])?;
        (component.get("enabled").and_then(Value::as_bool) != Some(false))
            .then(|| text(component, "path"))
            .flatten()
    };
    let css = component("contentStylesheet").and_then(|name| boost_file(directory, name, notes));
    let script = component("contentScript").and_then(|name| boost_file(directory, name, notes));
    if css.is_none() && script.is_none() {
        notes.add("arc.boost_empty");
        return None;
    }

    let mut matches: Vec<String> = script_entry
        .and_then(|entry| entry.get("matches"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect();
    if matches.is_empty() {
        matches.extend(text(&config, "scope").map(pattern_for));
    }
    if matches.is_empty() {
        notes.add("arc.boost_unreadable");
        return None;
    }

    let at_start = script_entry
        .and_then(|entry| text(entry, "run_at"))
        .map(|run_at| run_at == "document_start")
        .or_else(|| {
            at(&config, &["components", "contentScript", "settings"])
                .and_then(|settings| text(settings, "run_at"))
                .map(|run_at| run_at.eq_ignore_ascii_case("Before page loads"))
        })
        .unwrap_or(false);

    Some(Tweak {
        name: text(&config, "name").unwrap_or_default().to_owned(),
        matches,
        css,
        script,
        run_at: if at_start {
            RunAt::DocumentStart
        } else {
            RunAt::DocumentIdle
        },
        enabled: config.get("extensionEnabled").and_then(Value::as_bool) != Some(false),
    })
}

/// A file of a Boost, by the name its configuration gives. Only a plain
/// file name is accepted: a configuration is data from another program, and
/// must not be able to name a file outside its own folder.
fn boost_file(directory: &Path, name: &str, notes: &mut Notes) -> Option<String> {
    let plain = Path::new(name)
        .file_name()
        .is_some_and(|file| file == std::ffi::OsStr::new(name));
    if !plain {
        notes.add("arc.boost_unreadable");
        return None;
    }
    let path = directory.join(name);
    // A link could point anywhere; the Boost's own files are plain files.
    let metadata = std::fs::symlink_metadata(&path).ok()?;
    if !metadata.is_file() {
        notes.add("arc.boost_unreadable");
        return None;
    }
    if metadata.len() > MAX_BOOST_FILE {
        notes.add("arc.boost_file_too_large");
        return None;
    }
    match std::fs::read_to_string(&path) {
        Ok(text) if !text.trim().is_empty() => Some(text),
        Ok(_) => None,
        Err(_) => {
            notes.add("arc.boost_unreadable");
            None
        }
    }
}

/// A match pattern for a Boost's scope, which is a host with an optional
/// path: `example.com/wiki` covers `*://*.example.com/wiki/*`.
fn pattern_for(scope: &str) -> String {
    let scope = scope.trim().trim_end_matches('/');
    match scope.split_once('/') {
        Some((host, path)) => format!("*://*.{host}/{path}/*"),
        None => format!("*://*.{scope}/*"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scope_becomes_a_match_pattern() {
        assert_eq!(pattern_for("news.example.com"), "*://*.news.example.com/*");
        assert_eq!(pattern_for("example.com/wiki"), "*://*.example.com/wiki/*");
        assert_eq!(pattern_for("example.com/wiki/"), "*://*.example.com/wiki/*");
    }
}
