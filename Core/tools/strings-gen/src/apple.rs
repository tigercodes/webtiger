//! Apple's files: the String Catalog the system looks every string up in,
//! and the Swift names the code asks by.

use std::collections::BTreeMap;

use serde_json::{Map, json};

use crate::catalogue::{Catalogue, Entry, Platform, SOURCE};

/// The first line of the Swift file.
const SWIFT_HEADER: &str = "// Generated from Localization/strings.json by Core/tools/strings-gen (Tools/strings/generate.sh). Do not edit.";

/// The String Catalog: every string by its key, with its note and its text
/// in every language Apple has it in.
pub(crate) fn string_catalog(catalogue: &Catalogue) -> Result<String, String> {
    let mut strings = Map::new();
    for entry in &catalogue.entries {
        let mut localizations = Map::new();
        for language in catalogue.languages(Platform::Apple) {
            if let Some(value) = entry.text(Platform::Apple, &language) {
                localizations.insert(
                    language,
                    json!({ "stringUnit": { "state": "translated", "value": value } }),
                );
            }
        }
        strings.insert(
            entry.key.clone(),
            json!({
                "comment": entry.note,
                "extractionState": "manual",
                "localizations": localizations,
            }),
        );
    }
    let document = json!({ "sourceLanguage": SOURCE, "strings": strings, "version": "1.0" });
    let mut text = serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?;
    text.push('\n');
    Ok(text)
}

/// A key as a Swift name: its words, the first as it is and each of the
/// others with a capital: `tabs.new` is `tabsNew`, `space.new.shared_detail`
/// is `spaceNewSharedDetail`.
pub(crate) fn swift_name(key: &str) -> String {
    let mut name = String::with_capacity(key.len());
    for (index, word) in key
        .split(['.', '_'])
        .filter(|word| !word.is_empty())
        .enumerate()
    {
        let mut characters = word.chars();
        match characters.next() {
            Some(first) if index > 0 => name.extend(first.to_uppercase()),
            Some(first) => name.push(first),
            None => {}
        }
        name.push_str(characters.as_str());
    }
    name
}

/// The Swift names: one constant for each string, with its English and its
/// note as its comment, and every key there is, for a string that is asked
/// for by a key made while the app runs.
pub(crate) fn swift_names(catalogue: &Catalogue) -> Result<String, String> {
    let mut names: BTreeMap<String, &str> = BTreeMap::new();
    for entry in &catalogue.entries {
        let name = swift_name(&entry.key);
        if name.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(format!(
                "'{}' starts with a digit: it cannot be a name in Swift",
                entry.key
            ));
        }
        if let Some(other) = names.insert(name.clone(), &entry.key) {
            return Err(format!(
                "'{}' and '{other}' are the same name in Swift, {name}",
                entry.key
            ));
        }
    }
    let mut lines = vec![
        SWIFT_HEADER.to_owned(),
        String::new(),
        "/// Every string of the catalogue, by name (040): `Text(Words.tabsNew)`. A string that is"
            .to_owned(),
        "/// deleted or renamed in the catalogue stops the build at every place that asked for it."
            .to_owned(),
        "enum Words {".to_owned(),
    ];
    for (index, entry) in catalogue.entries.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.extend(constant(entry));
    }
    lines.push("}".to_owned());
    lines.push(String::new());
    lines.push("extension Word {".to_owned());
    lines.push(
        "    /// Every key the catalogue has: `Word(named:)` answers for these, and for no other."
            .to_owned(),
    );
    lines.push("    static let keys: Set<String> = [".to_owned());
    for entry in &catalogue.entries {
        lines.push(format!("        \"{}\",", entry.key));
    }
    lines.push("    ]".to_owned());
    lines.push("}".to_owned());
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// A string's constant: its English, its note, and its name.
fn constant(entry: &Entry) -> Vec<String> {
    let english = entry
        .text(Platform::Apple, SOURCE)
        .unwrap_or_else(|| entry.english());
    vec![
        format!("    /// {}", one_line(english)),
        "    ///".to_owned(),
        format!("    /// {}", one_line(&entry.note)),
        format!(
            "    static let {} = Word(key: \"{}\")",
            swift_name(&entry.key),
            entry.key
        ),
    ]
}

/// A text on one line of a comment.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::parse;

    fn catalogue(strings: &str) -> Result<Catalogue, String> {
        parse(&format!(
            r#"{{ "comment": "c", "strings": {{ {strings} }} }}"#
        ))
    }

    #[test]
    fn the_catalog_has_every_string_by_its_key_sorted_with_its_note_and_languages() {
        let text = catalogue(
            r#""tabs.new": { "en": "New Tab", "fr": "Nouvel onglet", "note": "A command." },
               "app.name": { "en": "Tom & “Jerry”\n\"x\"", "note": "The name." }"#,
        )
        .and_then(|catalogue| string_catalog(&catalogue))
        .unwrap_or_else(|problem| problem);
        assert_eq!(
            text,
            r#"{
  "sourceLanguage": "en",
  "strings": {
    "app.name": {
      "comment": "The name.",
      "extractionState": "manual",
      "localizations": {
        "en": {
          "stringUnit": {
            "state": "translated",
            "value": "Tom & “Jerry”\n\"x\""
          }
        }
      }
    },
    "tabs.new": {
      "comment": "A command.",
      "extractionState": "manual",
      "localizations": {
        "en": {
          "stringUnit": {
            "state": "translated",
            "value": "New Tab"
          }
        },
        "fr": {
          "stringUnit": {
            "state": "translated",
            "value": "Nouvel onglet"
          }
        }
      }
    }
  },
  "version": "1.0"
}
"#
        );
    }

    #[test]
    fn apples_own_wording_is_what_the_catalog_has() {
        let text = catalogue(
            r#""tabs.close": { "en": "Close tab", "apple": { "en": "Close Tab" }, "android": { "en": "Shut" }, "note": "n" }"#,
        )
        .and_then(|catalogue| string_catalog(&catalogue))
        .unwrap_or_else(|problem| problem);
        assert!(text.contains(r#""value": "Close Tab""#), "{text}");
        assert!(!text.contains("Shut"), "{text}");
    }

    #[test]
    fn a_key_is_a_name_in_swift() {
        assert_eq!(swift_name("tabs.new"), "tabsNew");
        assert_eq!(swift_name("toolbar.open_location"), "toolbarOpenLocation");
        assert_eq!(
            swift_name("space.new.shared.detail"),
            "spaceNewSharedDetail"
        );
        assert_eq!(swift_name("error.http.404"), "errorHttp404");
        assert_eq!(swift_name("error.net.dns"), "errorNetDns");
    }

    #[test]
    fn the_names_are_one_constant_for_each_string_and_every_key() {
        let text = catalogue(
            r#""tabs.new": { "en": "New Tab", "note": "Command:\n open a tab." },
               "find.count": { "en": "%1$d of %2$d", "apple": { "en": "%1$d of  %2$d" }, "note": "The count." }"#,
        )
        .and_then(|catalogue| swift_names(&catalogue))
        .unwrap_or_else(|problem| problem);
        assert_eq!(
            text,
            [
                SWIFT_HEADER,
                "",
                "/// Every string of the catalogue, by name (040): `Text(Words.tabsNew)`. A string that is",
                "/// deleted or renamed in the catalogue stops the build at every place that asked for it.",
                "enum Words {",
                "    /// %1$d of %2$d",
                "    ///",
                "    /// The count.",
                "    static let findCount = Word(key: \"find.count\")",
                "",
                "    /// New Tab",
                "    ///",
                "    /// Command: open a tab.",
                "    static let tabsNew = Word(key: \"tabs.new\")",
                "}",
                "",
                "extension Word {",
                "    /// Every key the catalogue has: `Word(named:)` answers for these, and for no other.",
                "    static let keys: Set<String> = [",
                "        \"find.count\",",
                "        \"tabs.new\",",
                "    ]",
                "}",
                "",
            ]
            .join("\n")
        );
    }

    #[test]
    fn two_keys_that_are_one_name_in_swift_are_refused() {
        let problem = catalogue(
            r#""tabs.new_tab": { "en": "a", "note": "n" }, "tabs.new__tab": { "en": "b", "note": "n" }"#,
        )
        .and_then(|catalogue| swift_names(&catalogue))
        .err()
        .unwrap_or_default();
        assert!(
            problem.contains("are the same name in Swift, tabsNewTab"),
            "{problem}"
        );
    }
}
