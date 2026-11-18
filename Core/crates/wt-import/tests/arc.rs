//! The Arc importer against a whole, hand-written Arc folder
//! (`Fixtures/import/arc/full`) and against folders built for one case each.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use wt_import::arc::{Options, import};
use wt_import::{Icon, Match, Node, Portable, RunAt, Target};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../Fixtures/import/arc")
}

fn full() -> Portable {
    import(&fixtures().join("full"), Options::default()).unwrap()
}

/// An Arc folder holding only a sidebar file.
fn folder_with(sidebar: &Value) -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    std::fs::write(
        folder.path().join("StorableSidebar.json"),
        serde_json::to_vec(sidebar).unwrap(),
    )
    .unwrap();
    folder
}

fn error_code(folder: &Path) -> &'static str {
    import(folder, Options::default()).unwrap_err().code()
}

fn note(file: &Portable, code: &str) -> u64 {
    file.notes
        .iter()
        .find(|note| note.code == code)
        .map_or(0, |note| note.count)
}

fn titles(nodes: &[Node]) -> Vec<String> {
    nodes
        .iter()
        .map(|node| match node {
            Node::Group { name, children } => format!("{name}[{}]", titles(children).join(",")),
            Node::Site { title, .. } => title.clone(),
        })
        .collect()
}

/// A tab item.
fn tab(id: &str, parent: &str, url: &str, title: &str) -> [Value; 2] {
    [
        json!(id),
        json!({ "id": id, "parentID": parent, "childrenIds": [], "title": null,
                "data": { "tab": { "savedURL": url, "savedTitle": title } } }),
    ]
}

/// A container item for a Space.
fn container(id: &str, space: &str, children: &[&str]) -> [Value; 2] {
    [
        json!(id),
        json!({ "id": id, "parentID": null, "childrenIds": children, "title": null,
                "data": { "itemContainer": { "containerType": { "spaceItems": { "_0": space } } } } }),
    ]
}

/// A sidebar with one Space whose pinned container is `P`.
fn one_space(items: Vec<[Value; 2]>) -> Value {
    json!({
        "version": 1,
        "sidebar": { "containers": [
            { "global": {} },
            { "spaces": ["S", { "id": "S", "title": "Only", "profile": { "default": {} },
                               "containerIDs": ["pinned", "P", "unpinned", "U"] }],
              "items": items.into_iter().flatten().collect::<Vec<Value>>(),
              "topAppsContainerIDs": [] }
        ] }
    })
}

#[test]
fn the_whole_folder_imports_as_the_expected_file() {
    let written = full().to_json();
    let expected_path = fixtures().join("full.expected.json");
    if std::env::var_os("WEBTIGER_UPDATE_FIXTURES").is_some() {
        std::fs::write(&expected_path, &written).unwrap();
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_default();
    assert!(
        written == expected,
        "the import of Fixtures/import/arc/full changed; review it and re-run with \
         WEBTIGER_UPDATE_FIXTURES=1 to accept"
    );
    // And what was written is a portable file this build reads back.
    assert_eq!(Portable::from_json(&written).unwrap(), full());
}

#[test]
fn spaces_come_in_sidebar_order_and_little_arc_is_not_one() {
    let file = full();
    let names: Vec<&str> = file
        .spaces
        .iter()
        .map(|space| space.name.as_str())
        .collect();
    assert_eq!(names, ["Personal", "Work", "", "Side project"]);
    assert_eq!(note(&file, "arc.little_arc_skipped"), 1);
    let everything = file.to_json();
    assert!(!everything.contains("little.example.com"));
}

#[test]
fn spaces_on_one_profile_share_a_login_set() {
    let file = full();
    let sets: Vec<(&str, &str)> = file
        .login_sets
        .iter()
        .map(|set| (set.key.as_str(), set.name.as_str()))
        .collect();
    // Named from Arc's Local State; the system profile is not one.
    assert_eq!(sets, [("default", "Personal"), ("profile-1", "Work")]);
    let of: Vec<&str> = file
        .spaces
        .iter()
        .map(|space| space.login_set.as_str())
        .collect();
    // `{"default": {}}` and `{"default": true}` are the same profile; so is
    // "Profile 1" whichever machine wrote it.
    assert_eq!(of, ["default", "profile-1", "default", "profile-1"]);
}

#[test]
fn the_tree_follows_children_ids_not_the_flat_array() {
    let file = full();
    assert_eq!(
        titles(&file.spaces[0].sites),
        [
            "Example Mail",
            "Docs[Handbook,Reference[API reference,Left pane,Right pane]]"
        ]
    );
    // "Handbook" is the name the user gave; the page's own title is not used.
    assert!(!file.to_json().contains("Company handbook"));
    assert_eq!(note(&file, "arc.split_flattened"), 1);
    assert_eq!(note(&file, "arc.missing_item"), 1, "the dangling child id");
    assert_eq!(note(&file, "arc.easel_skipped"), 1);
    assert_eq!(note(&file, "arc.note_skipped"), 1);
    assert_eq!(note(&file, "arc.unknown_item_skipped"), 1);
}

#[test]
fn children_known_only_by_their_parent_are_adopted() {
    let file = full();
    assert_eq!(titles(&file.spaces[1].sites), ["Board", "Repositories"]);
}

#[test]
fn a_cycle_ends() {
    let file = full();
    assert_eq!(titles(&file.spaces[3].sites), ["Loop[Dev server]"]);
    assert!(note(&file, "arc.repeated_item_skipped") >= 1);
}

#[test]
fn only_web_addresses_are_imported() {
    let file = full();
    let text = file.to_json();
    for unwanted in ["arc://", "javascript:", "chrome://"] {
        assert!(!text.contains(unwanted), "{unwanted} was imported");
    }
    // arc://settings, the bookmarklet, and chrome://extensions in the archive.
    assert_eq!(note(&file, "arc.address_skipped"), 3);
    assert_eq!(file.spaces[0].tabs.len(), 1);
    assert_eq!(
        file.spaces[0].tabs[0].address,
        "https://search.example.com/?q=rust"
    );
    assert_eq!(
        file.spaces[0].tabs[0].last_active_at,
        Some(1_724_733_900_250)
    );
}

#[test]
fn favourites_are_each_profiles_and_merged_across_machines() {
    let file = full();
    let addresses = |space: usize| -> Vec<&str> {
        file.spaces[space]
            .favourites
            .iter()
            .map(|node| match node {
                Node::Site { address, .. } => address.as_str(),
                Node::Group { .. } => panic!("favourites are sites"),
            })
            .collect()
    };
    // The default profile's row, its folder opened out.
    assert_eq!(
        addresses(0),
        ["https://mail.example.com/", "https://music.example.com/"]
    );
    assert_eq!(
        addresses(2),
        addresses(0),
        "every Space on a profile shows its row"
    );
    // "Profile 1" was used on two machines: one row, Git once.
    assert_eq!(
        addresses(1),
        [
            "https://jira.example.com/",
            "https://git.example.com/",
            "https://calendar.example.com/"
        ]
    );
    assert_eq!(addresses(3), addresses(1));
    assert_eq!(note(&file, "arc.favourite_folder_flattened"), 1);
}

#[test]
fn icons_and_colours_are_read_in_every_form() {
    let file = full();
    assert_eq!(file.spaces[0].icon, Some(Icon::Emoji("🏡".to_owned())));
    assert_eq!(file.spaces[1].icon, Some(Icon::Name("planet".to_owned())));
    // An emoji stored only as a number.
    assert_eq!(file.spaces[2].icon, Some(Icon::Emoji("🚀".to_owned())));
    assert_eq!(
        file.spaces[3].icon,
        Some(Icon::Symbol("hammer.fill".to_owned()))
    );
    // The mid tone wins over the background colour.
    assert_eq!(file.spaces[0].colour.as_deref(), Some("#6CBAD9"));
    // A gradient's first colour, clamped from outside 0 to 1.
    assert_eq!(file.spaces[1].colour.as_deref(), Some("#FF0080"));
    assert_eq!(file.spaces[2].colour, None);
}

#[test]
fn link_rules_keep_what_is_understood_and_count_the_rest() {
    let file = full();
    let rules: Vec<(Match, &str, Target, Option<usize>)> = file
        .link_rules
        .iter()
        .map(|rule| {
            (
                rule.matching,
                rule.pattern.as_str(),
                rule.target,
                rule.space,
            )
        })
        .collect();
    assert_eq!(
        rules,
        [
            (Match::Contains, "jira.example.com", Target::Space, Some(1)),
            (
                Match::Equals,
                "https://calendar.example.com/",
                Target::MostRecentSpace,
                None
            ),
            (
                Match::Contains,
                "meet.example.com",
                Target::QuickWindow,
                None
            ),
        ]
    );
    assert_eq!(note(&file, "arc.rule_space_missing"), 1);
    assert_eq!(
        note(&file, "arc.rule_unsupported"),
        1,
        "a condition that is not an address"
    );
}

#[test]
fn the_archive_keeps_when_and_where_from() {
    let file = full();
    assert_eq!(file.archive.len(), 2);
    assert_eq!(file.archive[0].title, "An old page");
    assert_eq!(file.archive[0].space, Some(1));
    assert_eq!(file.archive[0].archived_at, Some(1_722_307_200_500));
    assert_eq!(file.archive[1].title, "Renamed");
    assert_eq!(file.archive[1].space, None, "it came from Little Arc");
}

#[test]
fn legacy_boosts_become_tweaks() {
    let file = full();
    assert_eq!(file.tweaks.len(), 2);
    let quiet = &file.tweaks[0];
    assert_eq!(quiet.name, "Quiet wiki");
    assert_eq!(quiet.matches, ["*://*.wikipedia.example.com/wiki/*"]);
    assert!(quiet.css.as_deref().unwrap().contains("#sidebar"));
    assert!(quiet.script.as_deref().unwrap().contains("dataset.quiet"));
    assert_eq!(quiet.run_at, RunAt::DocumentStart);
    assert!(quiet.enabled);
    let serif = &file.tweaks[1];
    // No manifest: the pattern comes from the Boost's scope.
    assert_eq!(serif.matches, ["*://*.news.example.com/*"]);
    assert_eq!(serif.script, None);
    assert_eq!(serif.run_at, RunAt::DocumentIdle);
    assert!(!serif.enabled, "a Boost switched off stays off");
}

#[test]
fn each_part_can_be_left_out() {
    let folder = fixtures().join("full");
    let file = import(
        &folder,
        Options {
            today: false,
            archive: false,
            boosts: false,
        },
    )
    .unwrap();
    assert!(file.spaces.iter().all(|space| space.tabs.is_empty()));
    assert!(file.archive.is_empty());
    assert!(file.tweaks.is_empty());
    assert_eq!(
        file.spaces.len(),
        4,
        "the Spaces themselves are always read"
    );
}

#[test]
fn the_summary_counts_and_says_nothing_else() {
    let file = full();
    let summary = file.summary();
    assert_eq!(summary.spaces, 4);
    assert_eq!(summary.login_sets, 2);
    assert_eq!(summary.groups, 3);
    assert_eq!(summary.sites, 9);
    assert_eq!(summary.tabs, 1);
    assert_eq!(summary.favourites, 10);
    assert_eq!(summary.link_rules, 3);
    assert_eq!(summary.tweaks, 2);
    assert_eq!(summary.archived, 2);
    // Nothing a person visited or named is in it.
    let printed = format!("{summary:?}");
    for private in [
        "example.com",
        "Handbook",
        "Docs",
        "Mail",
        "Personal",
        "Work",
    ] {
        assert!(!printed.contains(private), "the summary names {private}");
    }
}

#[test]
fn a_folder_that_is_not_arcs_is_an_error_with_a_code() {
    let nowhere = tempfile::tempdir().unwrap();
    assert_eq!(
        error_code(&nowhere.path().join("missing")),
        "arc.folder_missing"
    );
    assert_eq!(error_code(nowhere.path()), "arc.sidebar_missing");

    std::fs::write(
        nowhere.path().join("StorableSidebar.json"),
        b"not json at all",
    )
    .unwrap();
    assert_eq!(error_code(nowhere.path()), "arc.sidebar_malformed");
    std::fs::write(nowhere.path().join("StorableSidebar.json"), b"").unwrap();
    assert_eq!(error_code(nowhere.path()), "arc.sidebar_malformed");

    for shapeless in [
        json!({}),
        json!([]),
        json!({ "sidebar": {} }),
        json!({ "sidebar": { "containers": [] } }),
    ] {
        assert_eq!(
            error_code(folder_with(&shapeless).path()),
            "arc.sidebar_malformed"
        );
    }
}

#[test]
fn a_main_container_without_its_marker_is_still_found() {
    // Some tools, and perhaps old builds, write the container with no
    // "global" key before it.
    let mut sidebar = one_space(vec![
        container("P", "S", &["T"]),
        tab("T", "P", "https://example.com/", "Example"),
        container("U", "S", &[]),
    ]);
    let containers = sidebar["sidebar"]["containers"].as_array_mut().unwrap();
    containers.remove(0);
    let file = import(folder_with(&sidebar).path(), Options::default()).unwrap();
    assert_eq!(titles(&file.spaces[0].sites), ["Example"]);
}

#[test]
fn a_space_with_only_the_new_container_list_is_read() {
    let mut sidebar = one_space(vec![
        container("P", "S", &["T"]),
        tab("T", "P", "https://example.com/", "Example"),
        container("U", "S", &["V"]),
        tab("V", "U", "https://example.org/", "Today"),
    ]);
    let space = &mut sidebar["sidebar"]["containers"][1]["spaces"][1];
    space.as_object_mut().unwrap().remove("containerIDs");
    space["newContainerIDs"] =
        json!([{ "unpinned": { "_0": { "shared": {} } } }, "U", { "pinned": {} }, "P"]);
    let file = import(folder_with(&sidebar).path(), Options::default()).unwrap();
    assert_eq!(titles(&file.spaces[0].sites), ["Example"]);
    assert_eq!(file.spaces[0].tabs[0].title, "Today");
}

#[test]
fn a_tree_deeper_than_the_limit_is_cut_not_followed_for_ever() {
    // Forty folders, each inside the last.
    let mut items = vec![container("P", "S", &["F0"]), container("U", "S", &[])];
    for depth in 0..40 {
        let id = format!("F{depth}");
        let child = format!("F{}", depth + 1);
        items.push([
            json!(id),
            json!({ "id": id, "parentID": null, "childrenIds": [child], "title": "Folder", "data": { "list": {} } }),
        ]);
    }
    let file = import(folder_with(&one_space(items)).path(), Options::default()).unwrap();
    assert_eq!(note(&file, "arc.too_deep"), 1);
    assert!(file.summary().groups <= 32);
}

#[test]
fn a_boost_cannot_name_a_file_outside_its_folder() {
    let folder = folder_with(&one_space(vec![
        container("P", "S", &[]),
        container("U", "S", &[]),
    ]));
    let secret = folder.path().join("secret.css");
    std::fs::write(&secret, "/* must not be read */").unwrap();
    let boost = folder.path().join("boosts").join("b1");
    std::fs::create_dir_all(&boost).unwrap();
    std::fs::write(
        boost.join("boost.config.json"),
        serde_json::to_vec(&json!({
            "name": "Sneaky", "scope": "example.com",
            "components": { "contentStylesheet": { "path": "../../secret.css", "enabled": true } }
        }))
        .unwrap(),
    )
    .unwrap();
    // And one whose stylesheet is a link to that file.
    #[cfg(unix)]
    {
        let linked = folder.path().join("boosts").join("b2");
        std::fs::create_dir_all(&linked).unwrap();
        std::os::unix::fs::symlink(&secret, linked.join("styles.css")).unwrap();
        std::fs::write(
            linked.join("boost.config.json"),
            serde_json::to_vec(&json!({
                "name": "Linked", "scope": "example.com",
                "components": { "contentStylesheet": { "path": "styles.css", "enabled": true } }
            }))
            .unwrap(),
        )
        .unwrap();
    }
    let file = import(folder.path(), Options::default()).unwrap();
    assert!(file.tweaks.is_empty());
    assert!(!file.to_json().contains("must not be read"));
    assert!(note(&file, "arc.boost_unreadable") >= 1);
}

#[test]
fn a_boost_file_over_the_limit_is_left_out() {
    let folder = folder_with(&one_space(vec![
        container("P", "S", &[]),
        container("U", "S", &[]),
    ]));
    let boost = folder.path().join("boosts").join("big");
    std::fs::create_dir_all(&boost).unwrap();
    std::fs::write(boost.join("styles.css"), vec![b' '; 1024 * 1024 + 1]).unwrap();
    std::fs::write(
        boost.join("boost.config.json"),
        serde_json::to_vec(&json!({
            "name": "Big", "scope": "example.com",
            "components": { "contentStylesheet": { "path": "styles.css", "enabled": true } }
        }))
        .unwrap(),
    )
    .unwrap();
    let file = import(folder.path(), Options::default()).unwrap();
    assert!(file.tweaks.is_empty());
    assert_eq!(note(&file, "arc.boost_file_too_large"), 1);
}

#[test]
fn an_optional_file_that_cannot_be_read_is_a_note_not_a_failure() {
    let folder = folder_with(&one_space(vec![
        container("P", "S", &["T"]),
        tab("T", "P", "https://example.com/", "Example"),
        container("U", "S", &[]),
    ]));
    std::fs::write(folder.path().join("StorableLinkRouting.json"), b"{ broken").unwrap();
    std::fs::write(folder.path().join("StorableArchiveItems.json"), b"[1, 2").unwrap();
    let file = import(folder.path(), Options::default()).unwrap();
    assert_eq!(file.spaces.len(), 1);
    assert_eq!(note(&file, "arc.link_rules_unreadable"), 1);
    assert_eq!(note(&file, "arc.archive_unreadable"), 1);
}

#[test]
fn the_import_leaves_arcs_folder_exactly_as_it_was() {
    fn listing(folder: &Path) -> Vec<(PathBuf, u64, std::time::SystemTime)> {
        let mut found = Vec::new();
        let mut pending = vec![folder.to_path_buf()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).unwrap() {
                let entry = entry.unwrap();
                let metadata = entry.metadata().unwrap();
                if metadata.is_dir() {
                    pending.push(entry.path());
                } else {
                    found.push((entry.path(), metadata.len(), metadata.modified().unwrap()));
                }
            }
        }
        found.sort();
        found
    }
    let folder = fixtures().join("full");
    let before = listing(&folder);
    let _ = full();
    assert_eq!(listing(&folder), before);
}
