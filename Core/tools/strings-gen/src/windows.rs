//! Windows' `Resources.resw`: one for each language. The format is the one
//! .NET's `ResXResourceWriter` writes, which is what a `.resw` file is.

use crate::catalogue::{Catalogue, Piece, Platform, SOURCE, pieces};

/// The second line of every file this writes: how one that is left over, for
/// a language the catalogue no longer has, is known to be this tool's.
pub(crate) const HEADER: &str = "<!-- Generated from Localization/strings.json by Core/tools/strings-gen (Tools/strings/generate.sh). Do not edit. -->";

const RESX: [&str; 4] = [
    r#"  <resheader name="resmimetype"><value>text/microsoft-resx</value></resheader>"#,
    r#"  <resheader name="version"><value>2.0</value></resheader>"#,
    r#"  <resheader name="reader"><value>System.Resources.ResXResourceReader, System.Windows.Forms, Version=4.0.0.0, Culture=neutral, PublicKeyToken=b77a5c561934e089</value></resheader>"#,
    r#"  <resheader name="writer"><value>System.Resources.ResXResourceWriter, System.Windows.Forms, Version=4.0.0.0, Culture=neutral, PublicKeyToken=b77a5c561934e089</value></resheader>"#,
];

/// A language's `Resources.resw`: every string Windows has in it, by its
/// resource name, in the order of the keys; in the source language each with
/// its note as the resource's comment.
pub(crate) fn resw(catalogue: &Catalogue, language: &str) -> Result<String, String> {
    let mut lines = vec![
        r#"<?xml version="1.0" encoding="utf-8"?>"#.to_owned(),
        HEADER.to_owned(),
        "<root>".to_owned(),
    ];
    lines.extend(RESX.iter().map(|line| (*line).to_owned()));
    for entry in &catalogue.entries {
        let Some(value) = entry.text(Platform::Windows, language) else {
            continue;
        };
        let comment = if language == SOURCE {
            format!("<comment>{}</comment>", escape(entry.note.trim()))
        } else {
            String::new()
        };
        lines.push(format!(
            r#"  <data name="{}" xml:space="preserve"><value>{}</value>{comment}</data>"#,
            entry.resource_name(),
            escape(&text(&entry.key, value)?)
        ));
    }
    lines.push("</root>".to_owned());
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// A string as Windows formats it: `{0}` for its first argument, `{1}` for
/// its second, whatever their order in the text; `%` for `%%`. In a string
/// that takes arguments a brace of the text's own is doubled, as the
/// formatting of such a string wants; a string that takes none is never
/// formatted, and is left as it is.
fn text(key: &str, value: &str) -> Result<String, String> {
    let pieces = pieces(key, value)?;
    let formatted = pieces
        .iter()
        .any(|piece| matches!(piece, Piece::Argument { .. }));
    let mut text = String::new();
    let mut count = 0;
    for piece in pieces {
        match piece {
            Piece::Text(plain) if formatted => {
                text.push_str(&plain.replace('{', "{{").replace('}', "}}"));
            }
            Piece::Text(plain) => text.push_str(&plain),
            Piece::PerCent => text.push('%'),
            Piece::Argument { position, .. } => {
                count += 1;
                text.push_str(&format!("{{{}}}", position.unwrap_or(count) - 1));
            }
        }
    }
    Ok(text)
}

/// XML's five escapes.
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            '&' => escaped.push_str("&amp;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::parse;

    fn written(strings: &str, language: &str) -> String {
        let catalogue = parse(&format!(
            r#"{{ "comment": "c", "strings": {{ {strings} }} }}"#
        ));
        catalogue
            .and_then(|catalogue| resw(&catalogue, language))
            .unwrap_or_else(|problem| problem)
    }

    #[test]
    fn each_string_is_a_resource_by_its_name_with_its_note_in_the_source_language() {
        let text = written(
            r#""tabs.new": { "en": "New Tab", "fr": "Nouvel onglet", "note": "A command." },
               "app.name": { "en": "Tom & \"Jerry\"", "note": "It's <the> name." }"#,
            "en",
        );
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], r#"<?xml version="1.0" encoding="utf-8"?>"#);
        assert_eq!(lines[1], HEADER);
        assert_eq!(lines[2], "<root>");
        assert_eq!(lines[3..7], RESX);
        assert_eq!(
            lines[7..],
            [
                r#"  <data name="app_name" xml:space="preserve"><value>Tom &amp; &quot;Jerry&quot;</value><comment>It&apos;s &lt;the&gt; name.</comment></data>"#,
                r#"  <data name="tabs_new" xml:space="preserve"><value>New Tab</value><comment>A command.</comment></data>"#,
                "</root>",
            ]
        );
        assert!(text.ends_with("</root>\n"));
        // Another language: the strings it has, and no notes.
        let french = written(
            r#""tabs.new": { "en": "New Tab", "fr": "Nouvel onglet", "note": "A command." },
               "app.name": { "en": "WebTiger", "note": "n" }"#,
            "fr",
        );
        assert_eq!(
            french.lines().skip(7).collect::<Vec<_>>(),
            [
                r#"  <data name="tabs_new" xml:space="preserve"><value>Nouvel onglet</value></data>"#,
                "</root>",
            ]
        );
    }

    #[test]
    fn arguments_are_numbered_from_nought_and_braces_doubled_only_where_a_string_is_formatted() {
        let of = |value: &str| text("k.k", value).unwrap_or_else(|problem| problem);
        assert_eq!(of("%@ left"), "{0} left");
        assert_eq!(of("%ld of %d, %u%%"), "{0} of {1}, {2}%");
        assert_eq!(of("%2$@ by %1$d"), "{1} by {0}");
        assert_eq!(of("{braces} and %@"), "{{braces}} and {0}");
        assert_eq!(of("{braces} alone"), "{braces} alone");
        assert_eq!(of("100%% sure"), "100% sure");
    }

    #[test]
    fn windows_own_wording_is_what_windows_has() {
        let text = written(
            r#""tabs.close": { "en": "Close Tab", "windows": { "en": "Close tab" }, "note": "n" }"#,
            "en",
        );
        assert!(text.contains("<value>Close tab</value>"), "{text}");
    }
}
