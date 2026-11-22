//! Android's `strings.xml`: one for each language, in the folder Android
//! looks for that language in.

use crate::catalogue::{Catalogue, Kind, Piece, Platform, SOURCE, pieces};

/// The second line of every file this writes: how one that is left over, for
/// a language the catalogue no longer has, is known to be this tool's.
pub(crate) const HEADER: &str = "<!-- Generated from Localization/strings.json by Core/tools/strings-gen (Tools/strings/generate.sh). Do not edit. -->";

/// The resource folder for a language: `values` for the source language,
/// `values-fr`, `values-pt-rBR`, and `values-b+sr+Latn` for anything longer.
pub(crate) fn folder(language: &str) -> String {
    if language == SOURCE {
        return "values".to_owned();
    }
    let parts: Vec<&str> = language.split('-').collect();
    match parts.as_slice() {
        [only] => format!("values-{}", only.to_lowercase()),
        [first, region] if region.len() == 2 && region.chars().all(|c| c.is_ascii_alphabetic()) => {
            format!("values-{}-r{}", first.to_lowercase(), region.to_uppercase())
        }
        _ => format!("values-b+{}", parts.join("+")),
    }
}

/// A language's `strings.xml`: every string Android has in it, by its
/// resource name, in the order of the keys; in the source language each with
/// its note before it.
pub(crate) fn strings_xml(catalogue: &Catalogue, language: &str) -> Result<String, String> {
    let mut lines = vec![
        r#"<?xml version="1.0" encoding="utf-8"?>"#.to_owned(),
        HEADER.to_owned(),
        "<resources>".to_owned(),
    ];
    for entry in &catalogue.entries {
        let Some(value) = entry.text(Platform::Android, language) else {
            continue;
        };
        if language == SOURCE {
            lines.push(format!("    <!-- {} -->", comment(&entry.note)));
        }
        lines.push(format!(
            r#"    <string name="{}">{}</string>"#,
            entry.resource_name(),
            text(&entry.key, value)?
        ));
    }
    lines.push("</resources>".to_owned());
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// A note as an XML comment's text: a comment may not hold two hyphens in a
/// row.
fn comment(note: &str) -> String {
    let mut text = String::with_capacity(note.len());
    for character in note.chars() {
        if character == '-' && text.ends_with('-') {
            continue;
        }
        text.push(character);
    }
    text.trim().to_owned()
}

/// Apple's placeholders as Android's: `%@` is `%s`, `%ld` is `%d`. Android
/// wants a string with more than one placeholder to number them, so those
/// that are not numbered are.
fn placeholders(key: &str, value: &str) -> Result<String, String> {
    let pieces = pieces(key, value)?;
    let arguments = pieces
        .iter()
        .filter(|piece| matches!(piece, Piece::Argument { .. }))
        .count();
    let mut text = String::new();
    let mut count = 0;
    for piece in pieces {
        match piece {
            Piece::Text(plain) => text.push_str(&plain),
            Piece::PerCent => text.push_str("%%"),
            Piece::Argument { position, kind } => {
                count += 1;
                let position = position.or((arguments > 1).then_some(count));
                let kind = match kind {
                    Kind::Text => 's',
                    Kind::Whole => 'd',
                    Kind::Fraction => 'f',
                };
                match position {
                    Some(position) => text.push_str(&format!("%{position}${kind}")),
                    None => text.push_str(&format!("%{kind}")),
                }
            }
        }
    }
    Ok(text)
}

/// A string as the text of a `<string>` element: aapt's escapes, then XML's.
fn text(key: &str, value: &str) -> Result<String, String> {
    let mut text = placeholders(key, value)?
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t");
    if text.starts_with(['@', '?']) {
        text.insert(0, '\\');
    }
    let text = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    // aapt trims a string and collapses its runs of spaces unless it is quoted.
    if text != text.trim_matches(' ') || text.contains("  ") {
        return Ok(format!("\"{text}\""));
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::parse;

    fn xml(strings: &str, language: &str) -> String {
        let catalogue = parse(&format!(
            r#"{{ "comment": "c", "strings": {{ {strings} }} }}"#
        ));
        catalogue
            .and_then(|catalogue| strings_xml(&catalogue, language))
            .unwrap_or_else(|problem| problem)
    }

    #[test]
    fn a_language_has_its_folder() {
        assert_eq!(folder("en"), "values");
        assert_eq!(folder("fr"), "values-fr");
        assert_eq!(folder("pt-BR"), "values-pt-rBR");
        assert_eq!(folder("sr-Latn"), "values-b+sr+Latn");
        assert_eq!(folder("zh-Hant-TW"), "values-b+zh+Hant+TW");
    }

    #[test]
    fn the_source_language_has_every_string_by_its_name_with_its_note() {
        assert_eq!(
            xml(
                r#""tabs.new": { "en": "New Tab", "fr": "Nouvel onglet", "note": "Command -- a new tab. " },
                   "app.name": { "en": "WebTiger", "note": "The name." }"#,
                "en"
            ),
            [
                r#"<?xml version="1.0" encoding="utf-8"?>"#,
                HEADER,
                "<resources>",
                "    <!-- The name. -->",
                r#"    <string name="app_name">WebTiger</string>"#,
                "    <!-- Command - a new tab. -->",
                r#"    <string name="tabs_new">New Tab</string>"#,
                "</resources>",
                "",
            ]
            .join("\n")
        );
    }

    #[test]
    fn another_language_has_the_strings_it_has_and_no_notes() {
        assert_eq!(
            xml(
                r#""tabs.new": { "en": "New Tab", "fr": "Nouvel onglet", "note": "n" },
                   "app.name": { "en": "WebTiger", "note": "The name." }"#,
                "fr"
            ),
            [
                r#"<?xml version="1.0" encoding="utf-8"?>"#,
                HEADER,
                "<resources>",
                r#"    <string name="tabs_new">Nouvel onglet</string>"#,
                "</resources>",
                "",
            ]
            .join("\n")
        );
    }

    #[test]
    fn androids_own_wording_is_what_android_has() {
        let written = xml(
            r#""tabs.close": { "en": "Close Tab", "android": { "en": "Close tab" }, "apple": { "en": "Close This Tab" }, "note": "n" }"#,
            "en",
        );
        assert!(
            written.contains(r#"<string name="tabs_close">Close tab</string>"#),
            "{written}"
        );
    }

    #[test]
    fn placeholders_are_androids_and_are_numbered_when_there_are_several() {
        let of = |value: &str| placeholders("k.k", value).unwrap_or_else(|problem| problem);
        assert_eq!(of("%@ left"), "%s left");
        assert_eq!(of("%ld of them, %d%%"), "%1$d of them, %2$d%%");
        assert_eq!(of("%@ has %lld in %f"), "%1$s has %2$d in %3$f");
        assert_eq!(of("%2$@ by %1$u"), "%2$s by %1$d");
        assert_eq!(of("100%% sure"), "100%% sure");
    }

    #[test]
    fn a_strings_text_is_escaped_for_aapt_and_then_for_xml() {
        let of = |value: &str| text("k.k", value).unwrap_or_else(|problem| problem);
        assert_eq!(
            of("It's \"here\" \\ there"),
            "It\\'s \\\"here\\\" \\\\ there"
        );
        assert_eq!(of("one\ntwo\tthree"), "one\\ntwo\\tthree");
        assert_eq!(of("@home"), "\\@home");
        assert_eq!(of("?what"), "\\?what");
        assert_eq!(of("a < b & c > d"), "a &lt; b &amp; c &gt; d");
        assert_eq!(of(" kept "), "\" kept \"");
        assert_eq!(of("two  spaces"), "\"two  spaces\"");
    }
}
