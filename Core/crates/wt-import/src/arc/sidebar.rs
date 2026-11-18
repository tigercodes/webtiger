//! `StorableSidebar.json` (002 §5): Spaces, their pinned trees and unpinned
//! tabs, and Favorites.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde_json::Value;

use super::json::{Notes, at, is_case, objects, pairs, text, time, web_address};
use crate::portable::{Icon, LoginSet, Node, Space, Tab};

/// How deep a tree is followed. Real files hold cycles.
const MAX_DEPTH: usize = 32;

/// What the sidebar file gave.
pub(crate) struct Read {
    pub(crate) login_sets: Vec<LoginSet>,
    pub(crate) spaces: Vec<Space>,
    /// Arc's Space ids, in the order of `spaces`: link rules and the archive
    /// name Spaces by them.
    pub(crate) space_ids: Vec<String>,
}

/// Reads the sidebar. `None` when the file has no main sidebar in it.
pub(crate) fn read(
    root: &Value,
    profile_names: &BTreeMap<String, String>,
    include_today: bool,
    notes: &mut Notes,
) -> Option<Read> {
    let main = main_container(root, notes)?;
    let mut walker = Walker::new(main, notes);

    // Favorites belong to a profile. Each is read once, then handed to every
    // Space on that profile, which is what Arc showed.
    let mut favourites: BTreeMap<String, Vec<Node>> = BTreeMap::new();
    for (profile, container) in favourite_containers(main) {
        let row = favourites.entry(profile_key(profile)).or_default();
        let found = walker.favourites(container);
        for site in found {
            let repeated = row.iter().any(|kept| address_of(kept) == address_of(&site));
            if !repeated {
                row.push(site);
            }
        }
    }

    let mut login_sets: Vec<LoginSet> = Vec::new();
    let mut spaces = Vec::new();
    let mut space_ids = Vec::new();
    for space in objects(main.get("spaces")) {
        let Some(id) = text(space, "id") else {
            continue;
        };
        let profile = space.get("profile");
        let key = profile.map_or_else(|| "default".to_owned(), profile_key);
        if !login_sets.iter().any(|set| set.key == key) {
            login_sets.push(LoginSet {
                name: profile_name(profile, profile_names),
                key: key.clone(),
            });
        }
        let (pinned, unpinned) = containers_of(space);
        let sites = pinned
            .iter()
            .flat_map(|container| walker.children(container, 0))
            .collect();
        let tabs = if include_today {
            unpinned
                .iter()
                .flat_map(|container| walker.tabs(container))
                .collect()
        } else {
            Vec::new()
        };
        let custom = space.get("customInfo");
        spaces.push(Space {
            name: text(space, "title").unwrap_or_default().to_owned(),
            icon: custom.and_then(icon),
            colour: custom.and_then(colour),
            favourites: favourites.get(&key).cloned().unwrap_or_default(),
            login_set: key,
            sites,
            tabs,
        });
        space_ids.push(id.to_owned());
    }
    Some(Read {
        login_sets,
        spaces,
        space_ids,
    })
}

/// The main sidebar: the container that follows the `{"global": {}}` key.
/// Each Little Arc window has a container of its own, sometimes first.
fn main_container<'a>(root: &'a Value, notes: &mut Notes) -> Option<&'a Value> {
    let containers = at(root, &["sidebar", "containers"])?.as_array()?;
    let is_container =
        |value: &Value| value.get("spaces").is_some() && value.get("items").is_some();
    let mut main = None;
    let mut after_little = false;
    let mut fallback = None;
    let mut after_global = false;
    for element in containers {
        if is_case(element, "littleBrowser") && !is_container(element) {
            after_little = true;
            notes.add("arc.little_arc_skipped");
            continue;
        }
        if is_case(element, "global") && !is_container(element) {
            after_global = true;
            continue;
        }
        if is_container(element) {
            if after_global && main.is_none() {
                main = Some(element);
            } else if !after_little && fallback.is_none() {
                fallback = Some(element);
            }
        }
        after_little = false;
        after_global = false;
    }
    main.or(fallback)
}

/// A Space's pinned and unpinned containers, from both of its lists: the
/// markers are strings in `containerIDs` and objects in `newContainerIDs`,
/// the pairs come in either order, and a file may have either list or both.
fn containers_of(space: &Value) -> (Vec<&str>, Vec<&str>) {
    let mut pinned: Vec<&str> = Vec::new();
    let mut unpinned: Vec<&str> = Vec::new();
    for list in ["containerIDs", "newContainerIDs"] {
        for (marker, id) in pairs(space.get(list)) {
            let Some(id) = id.as_str() else { continue };
            let into = if is_case(marker, "pinned") {
                &mut pinned
            } else if is_case(marker, "unpinned") {
                &mut unpinned
            } else {
                continue;
            };
            if !into.contains(&id) {
                into.push(id);
            }
        }
    }
    (pinned, unpinned)
}

/// Every Favorites container with the profile it belongs to: the ones
/// `topAppsContainerIDs` lists, then any other item that says it is one.
fn favourite_containers(main: &Value) -> Vec<(&Value, &str)> {
    let mut found: Vec<(&Value, &str)> = Vec::new();
    for (profile, id) in pairs(main.get("topAppsContainerIDs")) {
        if let Some(id) = id.as_str()
            && profile.is_object()
        {
            found.push((profile, id));
        }
    }
    for item in objects(main.get("items")) {
        let profile = at(
            item,
            &["data", "itemContainer", "containerType", "topApps", "_0"],
        );
        if let (Some(profile), Some(id)) = (profile, text(item, "id"))
            && !found.iter().any(|(_, known)| *known == id)
        {
            found.push((profile, id));
        }
    }
    found
}

/// The key of the login set a profile reference names. Two Spaces share
/// logins in Arc exactly when they name the same profile folder.
fn profile_key(profile: &Value) -> String {
    if is_case(profile, "default") {
        return "default".to_owned();
    }
    let Some(folder) =
        at(profile, &["custom", "_0"]).and_then(|custom| text(custom, "directoryBasename"))
    else {
        return "default".to_owned();
    };
    let mut key = String::new();
    for character in folder.chars() {
        if character.is_ascii_alphanumeric() {
            key.push(character.to_ascii_lowercase());
        } else if !key.ends_with('-') {
            key.push('-');
        }
    }
    let key = key.trim_matches('-');
    if key.is_empty() || key == "default" {
        "default".to_owned()
    } else {
        key.to_owned()
    }
}

/// What a profile is called: the name Arc's `Local State` gives its folder,
/// else the folder's own name.
fn profile_name(profile: Option<&Value>, names: &BTreeMap<String, String>) -> String {
    let folder = profile
        .and_then(|profile| at(profile, &["custom", "_0"]))
        .and_then(|custom| text(custom, "directoryBasename"))
        .unwrap_or("Default");
    names
        .get(folder)
        .map_or_else(|| folder.to_owned(), Clone::clone)
}

fn icon(custom: &Value) -> Option<Icon> {
    let kind = custom.get("iconType")?;
    if let Some(emoji) = text(kind, "emoji_v2") {
        return Some(Icon::Emoji(emoji.to_owned()));
    }
    if let Some(scalar) = kind.get("emoji").and_then(Value::as_u64) {
        let character = u32::try_from(scalar).ok().and_then(char::from_u32)?;
        return Some(Icon::Emoji(character.to_string()));
    }
    if let Some(name) = text(kind, "icon") {
        return Some(Icon::Name(name.to_owned()));
    }
    text(kind, "symbol").map(|symbol| Icon::Symbol(symbol.to_owned()))
}

/// One colour for a Space: the theme's mid tone, else its single background
/// colour, else the first colour of its gradient.
fn colour(custom: &Value) -> Option<String> {
    let theme = custom.get("windowTheme")?;
    let style = at(
        theme,
        &["background", "single", "_0", "style", "color", "_0"],
    );
    at(theme, &["primaryColorPalette", "midTone"])
        .and_then(hex)
        .or_else(|| {
            style
                .and_then(|style| at(style, &["blendedSingleColor", "_0", "color"]))
                .and_then(hex)
        })
        .or_else(|| {
            style
                .and_then(|style| at(style, &["blendedGradient", "_0", "baseColors"]))
                .and_then(|colours| colours.get(0))
                .and_then(hex)
        })
}

/// `{red, green, blue}` as `#RRGGBB`. Arc writes extended sRGB, whose
/// components can fall outside 0 to 1; they are clamped.
fn hex(colour: &Value) -> Option<String> {
    let component = |name: &str| -> Option<u8> {
        let value = colour.get(name)?.as_f64()?;
        if !value.is_finite() {
            return None;
        }
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to 0..=255 before the cast"
        )]
        Some((value.clamp(0.0, 1.0) * 255.0).round() as u8)
    };
    Some(format!(
        "#{:02X}{:02X}{:02X}",
        component("red")?,
        component("green")?,
        component("blue")?
    ))
}

fn address_of(node: &Node) -> Option<&str> {
    match node {
        Node::Site { address, .. } => Some(address),
        Node::Group { .. } => None,
    }
}

/// Walks the sidebar's items. One walker serves the whole file, so an item
/// reached twice (a repeated id, a cycle) is taken once.
struct Walker<'a, 'n> {
    items: HashMap<&'a str, &'a Value>,
    /// Children known only through their own `parentID`, in file order.
    by_parent: HashMap<&'a str, Vec<&'a str>>,
    visited: HashSet<&'a str>,
    notes: &'n mut Notes,
}

impl<'a, 'n> Walker<'a, 'n> {
    fn new(main: &'a Value, notes: &'n mut Notes) -> Walker<'a, 'n> {
        let mut items = HashMap::new();
        let mut by_parent: HashMap<&str, Vec<&str>> = HashMap::new();
        for item in objects(main.get("items")) {
            let Some(id) = text(item, "id") else { continue };
            if items.insert(id, item).is_some() {
                continue;
            }
            if let Some(parent) = text(item, "parentID") {
                by_parent.entry(parent).or_default().push(id);
            }
        }
        Walker {
            items,
            by_parent,
            visited: HashSet::new(),
            notes,
        }
    }

    /// The ids of an item's children, in sidebar order: the ones it lists,
    /// then any that name it as their parent without being listed.
    fn child_ids(&self, id: &str) -> Vec<&'a str> {
        let mut ids: Vec<&'a str> = self
            .items
            .get(id)
            .and_then(|item| item.get("childrenIds"))
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        if let Some(adopted) = self.by_parent.get(id) {
            for child in adopted {
                if !ids.contains(child) {
                    ids.push(child);
                }
            }
        }
        ids
    }

    /// The rows under a container or folder.
    fn children(&mut self, id: &str, depth: usize) -> Vec<Node> {
        let mut rows = Vec::new();
        for child in self.child_ids(id) {
            rows.extend(self.row(child, depth + 1));
        }
        rows
    }

    /// One item as rows of the tree: one row, none, or (a split view) its
    /// panes in place.
    fn row(&mut self, id: &'a str, depth: usize) -> Vec<Node> {
        let Some(item) = self.items.get(id).copied() else {
            self.notes.add("arc.missing_item");
            return Vec::new();
        };
        if depth > MAX_DEPTH {
            self.notes.add("arc.too_deep");
            return Vec::new();
        }
        if !self.visited.insert(id) {
            self.notes.add("arc.repeated_item_skipped");
            return Vec::new();
        }
        let Some(data) = item.get("data") else {
            self.notes.add("arc.unknown_item_skipped");
            return Vec::new();
        };
        if let Some(tab) = data.get("tab") {
            return self.site(item, tab).into_iter().collect();
        }
        if data.get("list").is_some() {
            return vec![Node::Group {
                name: text(item, "title").unwrap_or_default().to_owned(),
                children: self.children(id, depth),
            }];
        }
        if data.get("splitView").is_some() {
            self.notes.add("arc.split_flattened");
            return self.children(id, depth);
        }
        if data.get("itemContainer").is_some() {
            return self.children(id, depth);
        }
        self.notes.add(if data.get("easel").is_some() {
            "arc.easel_skipped"
        } else if data.get("arcDocument").is_some() {
            "arc.note_skipped"
        } else {
            "arc.unknown_item_skipped"
        });
        Vec::new()
    }

    /// A tab as a Site, when its address is one WebTiger can open.
    fn site(&mut self, item: &Value, tab: &Value) -> Option<Node> {
        let Some(address) = web_address(text(tab, "savedURL")) else {
            self.notes.add("arc.address_skipped");
            return None;
        };
        Some(Node::Site {
            title: title(item, tab),
            address,
            added_at: time(item.get("createdAt")),
        })
    }

    /// The Sites of a Favorites container. A folder among them is opened
    /// out: Favourites are a row of tiles.
    fn favourites(&mut self, container: &str) -> Vec<Node> {
        let mut sites = Vec::new();
        let mut had_folder = false;
        flatten(self.children(container, 0), &mut sites, &mut had_folder);
        if had_folder {
            self.notes.add("arc.favourite_folder_flattened");
        }
        sites
    }

    /// The tabs under an unpinned container, in order.
    fn tabs(&mut self, container: &str) -> Vec<Tab> {
        let mut tabs = Vec::new();
        let mut pending: Vec<(&'a str, usize)> = self
            .child_ids(container)
            .into_iter()
            .rev()
            .map(|id| (id, 1))
            .collect();
        while let Some((id, depth)) = pending.pop() {
            let Some(item) = self.items.get(id).copied() else {
                self.notes.add("arc.missing_item");
                continue;
            };
            if depth > MAX_DEPTH {
                self.notes.add("arc.too_deep");
                continue;
            }
            if !self.visited.insert(id) {
                self.notes.add("arc.repeated_item_skipped");
                continue;
            }
            match at(item, &["data", "tab"]) {
                Some(tab) => match web_address(text(tab, "savedURL")) {
                    Some(address) => tabs.push(Tab {
                        title: title(item, tab),
                        address,
                        last_active_at: time(tab.get("timeLastActiveAt")),
                    }),
                    None => self.notes.add("arc.address_skipped"),
                },
                // A split view or a folder among today's tabs: its tabs, in place.
                None => {
                    for child in self.child_ids(id).into_iter().rev() {
                        pending.push((child, depth + 1));
                    }
                }
            }
        }
        tabs
    }
}

/// What a row is called: the name the user gave it, else the page's title.
fn title(item: &Value, tab: &Value) -> String {
    text(item, "title")
        .or_else(|| text(tab, "savedTitle"))
        .unwrap_or_default()
        .to_owned()
}

fn flatten(nodes: Vec<Node>, sites: &mut Vec<Node>, had_folder: &mut bool) {
    for node in nodes {
        match node {
            Node::Group { children, .. } => {
                *had_folder = true;
                flatten(children, sites, had_folder);
            }
            site @ Node::Site { .. } => sites.push(site),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    const FIXTURE: &str =
        include_str!("../../../../../Fixtures/import/arc/full/StorableSidebar.json");

    /// Every place in a JSON tree, as the steps that lead to it.
    fn places(value: &Value, here: &mut Vec<Step>, found: &mut Vec<Vec<Step>>) {
        found.push(here.clone());
        match value {
            Value::Array(elements) => {
                for (index, element) in elements.iter().enumerate() {
                    here.push(Step::Index(index));
                    places(element, here, found);
                    here.pop();
                }
            }
            Value::Object(fields) => {
                for (key, field) in fields {
                    here.push(Step::Key(key.clone()));
                    places(field, here, found);
                    here.pop();
                }
            }
            _ => {}
        }
    }

    #[derive(Clone)]
    enum Step {
        Index(usize),
        Key(String),
    }

    fn put(root: &mut Value, place: &[Step], replacement: Value) {
        let mut at = root;
        for step in place {
            at = match step {
                Step::Index(index) => &mut at[*index],
                Step::Key(key) => &mut at[key.as_str()],
            };
        }
        *at = replacement;
    }

    /// The reader's promise is that no file makes it panic. Every value in
    /// the fixture is replaced, one at a time, by each kind of JSON value,
    /// and the file is read again: thousands of files that are each wrong in
    /// one place.
    #[test]
    fn a_file_that_is_wrong_in_any_one_place_is_read_or_refused_never_a_panic() {
        let original: Value = serde_json::from_str(FIXTURE).unwrap();
        let mut all = Vec::new();
        places(&original, &mut Vec::new(), &mut all);
        assert!(all.len() > 500, "the fixture has {} values", all.len());
        let replacements = [
            json!(null),
            json!(-1),
            json!(1e300),
            json!("P1"),
            json!([]),
            json!(["P1", "P1"]),
            json!({}),
            json!({ "default": [] }),
        ];
        let names = BTreeMap::new();
        let mut read_ok = 0usize;
        for place in &all {
            for replacement in &replacements {
                let mut changed = original.clone();
                put(&mut changed, place, replacement.clone());
                let mut notes = Notes::default();
                if read(&changed, &names, true, &mut notes).is_some() {
                    read_ok += 1;
                }
            }
        }
        // Most single faults cost one row, not the file.
        assert!(read_ok > all.len() * replacements.len() * 9 / 10);
    }

    #[test]
    fn a_file_whose_every_item_points_at_itself_ends() {
        let knot: Vec<Value> = (0..200)
            .flat_map(|index| {
                let id = format!("I{index}");
                let everyone: Vec<String> = (0..200).map(|other| format!("I{other}")).collect();
                [
                    json!(id),
                    json!({ "id": id, "parentID": "P", "childrenIds": everyone, "title": "Folder", "data": { "list": {} } }),
                ]
            })
            .collect();
        let mut items = vec![
            json!("P"),
            json!({ "id": "P", "parentID": null, "childrenIds": ["I0"], "data": { "itemContainer": {} } }),
        ];
        items.extend(knot);
        let root = json!({ "sidebar": { "containers": [ { "global": {} }, {
            "spaces": ["S", { "id": "S", "containerIDs": ["pinned", "P"] }],
            "items": items,
            "topAppsContainerIDs": []
        } ] } });
        let mut notes = Notes::default();
        let read = read(&root, &BTreeMap::new(), true, &mut notes).unwrap();
        // Each of the 200 folders appears once, however the ids are knotted.
        let mut groups = 0;
        fn count(nodes: &[Node], groups: &mut usize) {
            for node in nodes {
                if let Node::Group { children, .. } = node {
                    *groups += 1;
                    count(children, groups);
                }
            }
        }
        count(&read.spaces[0].sites, &mut groups);
        assert!(groups <= 200, "{groups} groups from 200 items");
        assert!(groups >= 1);
    }
}
