//! The catalogue as written, `Localization/strings.json`: its strings by key,
//! each checked as every platform needs it.

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, Visitor};

/// The language every string has, and every other is checked against.
pub(crate) const SOURCE: &str = "en";

/// A platform that may have a wording of its own for a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Platform {
    Apple,
    Android,
    Windows,
}

impl Platform {
    const ALL: [Platform; 3] = [Platform::Apple, Platform::Android, Platform::Windows];

    /// The field of an entry that holds this platform's own wording.
    pub(crate) fn field(self) -> &'static str {
        match self {
            Platform::Apple => "apple",
            Platform::Android => "android",
            Platform::Windows => "windows",
        }
    }

    fn of(field: &str) -> Option<Platform> {
        Platform::ALL
            .into_iter()
            .find(|platform| platform.field() == field)
    }
}

/// One string of the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    /// Its key: dotted lower-case words, `tabs.new`.
    pub(crate) key: String,
    /// What it is for, for whoever translates it.
    pub(crate) note: String,
    /// Its text, by language, with Apple's placeholders (`%@`, `%1$d`).
    texts: BTreeMap<String, String>,
    /// A platform's own wording, by language, where it differs.
    own: BTreeMap<Platform, BTreeMap<String, String>>,
}

impl Entry {
    /// The text a platform shows in a language: its own wording where it has
    /// one, otherwise the common text. `None` when the string is not in that
    /// language.
    pub(crate) fn text(&self, platform: Platform, language: &str) -> Option<&str> {
        self.own
            .get(&platform)
            .and_then(|own| own.get(language))
            .or_else(|| self.texts.get(language))
            .map(String::as_str)
    }

    /// The English, as the catalogue has it for every platform.
    pub(crate) fn english(&self) -> &str {
        self.texts.get(SOURCE).map_or("", String::as_str)
    }

    /// The languages a platform has this string in.
    fn languages(&self, platform: Platform) -> impl Iterator<Item = &String> {
        let own = self.own.get(&platform);
        self.texts
            .keys()
            .chain(own.into_iter().flat_map(BTreeMap::keys))
    }

    /// Its Android and Windows name: `_` for `.`, since neither takes a dot.
    pub(crate) fn resource_name(&self) -> String {
        self.key.replace('.', "_")
    }
}

/// The catalogue: its strings, in the order of their keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Catalogue {
    pub(crate) entries: Vec<Entry>,
}

impl Catalogue {
    /// Every language a platform has a string in, in order.
    pub(crate) fn languages(&self, platform: Platform) -> Vec<String> {
        let mut languages: Vec<String> = self
            .entries
            .iter()
            .flat_map(|entry| entry.languages(platform))
            .cloned()
            .collect();
        languages.sort();
        languages.dedup();
        languages
    }
}

/// A JSON object whose keys are each there once. `serde_json`'s own map keeps
/// the last of two, silently: a string written twice would lose one wording.
struct Unique<V>(BTreeMap<String, V>);

impl<'de, V: Deserialize<'de>> Deserialize<'de> for Unique<V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Once<V>(PhantomData<V>);
        impl<'de, V: Deserialize<'de>> Visitor<'de> for Once<V> {
            type Value = Unique<V>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Unique<V>, A::Error> {
                let mut fields = BTreeMap::new();
                while let Some((name, value)) = map.next_entry::<String, V>()? {
                    if fields.contains_key(&name) {
                        return Err(de::Error::custom(format!("'{name}' is there twice")));
                    }
                    fields.insert(name, value);
                }
                Ok(Unique(fields))
            }
        }
        deserializer.deserialize_map(Once(PhantomData))
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Field {
    /// A language's text, or the note.
    Text(String),
    /// A platform's own wording, by language.
    Own(Unique<String>),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    #[serde(rename = "comment")]
    _comment: String,
    strings: Unique<Unique<Field>>,
}

/// The catalogue's strings, once each is known to be usable on every platform.
pub(crate) fn parse(text: &str) -> Result<Catalogue, String> {
    let file: File = serde_json::from_str(text).map_err(|error| error.to_string())?;
    let mut entries = Vec::with_capacity(file.strings.0.len());
    for (key, fields) in file.strings.0 {
        entries.push(entry(key, fields.0)?);
    }
    names_are_each_one_string(&entries)?;
    Ok(Catalogue { entries })
}

fn entry(key: String, fields: BTreeMap<String, Field>) -> Result<Entry, String> {
    if !is_key(&key) {
        return Err(format!("'{key}' is not a dotted lower-case key"));
    }
    let mut note = None;
    let mut texts = BTreeMap::new();
    let mut own = BTreeMap::new();
    for (name, field) in fields {
        match (name.as_str(), Platform::of(&name), field) {
            ("note", _, Field::Text(text)) => note = Some(text),
            (_, Some(platform), Field::Own(wording)) => {
                if let Some(bad) = wording.0.keys().find(|language| !is_language(language)) {
                    return Err(format!(
                        "'{key}': '{bad}' in '{name}' is not a language tag"
                    ));
                }
                own.insert(platform, wording.0);
            }
            (_, Some(_), Field::Text(_)) => {
                return Err(format!(
                    "'{key}': '{name}' is a platform's own wording, by language: {{ \"en\": \"…\" }}"
                ));
            }
            (_, None, Field::Text(text)) if is_language(&name) => {
                texts.insert(name, text);
            }
            _ => {
                return Err(format!(
                    "'{key}': '{name}' is no language, no platform and no note"
                ));
            }
        }
    }
    let (Some(note), true) = (note, texts.contains_key(SOURCE)) else {
        return Err(format!("'{key}' needs both 'en' and 'note'"));
    };
    let entry = Entry {
        key,
        note,
        texts,
        own,
    };
    same_arguments_everywhere(&entry)?;
    Ok(entry)
}

/// `^[a-z0-9_]+(\.[a-z0-9_]+)+$`.
fn is_key(key: &str) -> bool {
    let mut words = 0;
    for word in key.split('.') {
        if word.is_empty()
            || !word
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return false;
        }
        words += 1;
    }
    words >= 2
}

/// A BCP 47 tag, as far as its shape: letters and digits in parts joined by
/// hyphens, the first part letters.
fn is_language(tag: &str) -> bool {
    let mut parts = tag.split('-');
    let first = parts.next().unwrap_or_default();
    (2..=3).contains(&first.len())
        && first.chars().all(|c| c.is_ascii_alphabetic())
        && parts.all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric()))
}

/// No two keys make one Android and Windows name, and none starts with a digit.
fn names_are_each_one_string(entries: &[Entry]) -> Result<(), String> {
    let mut names: BTreeMap<String, &str> = BTreeMap::new();
    for entry in entries {
        let name = entry.resource_name();
        if name.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(format!(
                "'{}' starts with a digit: it cannot name an Android or a Windows resource",
                entry.key
            ));
        }
        if let Some(other) = names.insert(name.clone(), &entry.key) {
            return Err(format!(
                "'{}' and '{other}' are the same Android and Windows resource, {name}",
                entry.key
            ));
        }
    }
    Ok(())
}

/// What a placeholder stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// `%@`, `%s`.
    Text,
    /// `%d`, `%ld`, `%u` and the like.
    Whole,
    /// `%f`.
    Fraction,
}

/// A piece of a string: its text, or an argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Piece {
    Text(String),
    /// `%%`.
    PerCent,
    /// A placeholder: the position it names (`%2$@` is 2), when it names one.
    Argument {
        position: Option<usize>,
        kind: Kind,
    },
}

/// A string as its pieces. A `%` that begins no placeholder is refused: on
/// Android it ends the app when the string is formatted.
pub(crate) fn pieces(key: &str, text: &str) -> Result<Vec<Piece>, String> {
    let stray =
        || format!("'{key}' has a '%' that is not a placeholder; write a literal one as '%%'");
    let mut pieces = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('%') {
        plain.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        // An argument's number: digits and a dollar sign.
        let digits = after.chars().take_while(char::is_ascii_digit).count();
        let (position, spec) = if digits > 0 && after[digits..].starts_with('$') {
            let position = after[..digits].parse::<usize>().map_err(|_| stray())?;
            (Some(position), &after[digits + 1..])
        } else {
            (None, after)
        };
        let longs = spec.chars().take_while(|c| *c == 'l').count().min(2);
        let (piece, used) = match spec[longs..].chars().next() {
            Some('%') if longs == 0 => (Piece::PerCent, 1),
            Some('@' | 's') if longs == 0 => (argument(position, Kind::Text), 1),
            Some('f') if longs == 0 => (argument(position, Kind::Fraction), 1),
            Some('d' | 'u') => (argument(position, Kind::Whole), longs + 1),
            _ => return Err(stray()),
        };
        if !plain.is_empty() {
            pieces.push(Piece::Text(std::mem::take(&mut plain)));
        }
        pieces.push(piece);
        rest = &spec[used..];
    }
    plain.push_str(rest);
    if !plain.is_empty() {
        pieces.push(Piece::Text(plain));
    }
    let arguments: Vec<bool> = pieces
        .iter()
        .filter_map(|piece| match piece {
            Piece::Argument { position, .. } => Some(position.is_some()),
            _ => None,
        })
        .collect();
    if arguments.contains(&true) && arguments.contains(&false) {
        return Err(format!(
            "'{key}' numbers some of its placeholders and not others"
        ));
    }
    Ok(pieces)
}

fn argument(position: Option<usize>, kind: Kind) -> Piece {
    Piece::Argument { position, kind }
}

/// The arguments a string takes: the kind of each, by its number. One that is
/// not numbered is numbered by where it comes.
pub(crate) fn arguments(key: &str, text: &str) -> Result<BTreeMap<usize, Kind>, String> {
    let mut taken = BTreeMap::new();
    let mut next = 0;
    for piece in pieces(key, text)? {
        if let Piece::Argument { position, kind } = piece {
            next += 1;
            let number = position.unwrap_or(next);
            if taken
                .insert(number, kind)
                .is_some_and(|other| other != kind)
            {
                return Err(format!(
                    "'{key}' uses its argument {number} as two kinds of thing"
                ));
            }
        }
    }
    Ok(taken)
}

/// Every language's text, and every platform's own, takes the arguments the
/// English takes: as many, of the same kinds, whatever their order. A wording
/// with one fewer reads past its arguments on Apple, and ends the app on
/// Android.
fn same_arguments_everywhere(entry: &Entry) -> Result<(), String> {
    let key = &entry.key;
    let english = arguments(key, entry.english())?;
    let common = entry
        .texts
        .iter()
        .map(|(language, text)| (None, language, text));
    let own = entry.own.iter().flat_map(|(platform, wording)| {
        wording
            .iter()
            .map(move |(language, text)| (Some(*platform), language, text))
    });
    for (platform, language, text) in common.chain(own) {
        if arguments(key, text)? != english {
            let whose =
                platform.map_or_else(String::new, |platform| format!("{}'s ", platform.field()));
            return Err(format!(
                "'{key}': {whose}'{language}' does not take the same arguments as its English"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogue(strings: &str) -> Result<Catalogue, String> {
        parse(&format!(
            r#"{{ "comment": "c", "strings": {{ {strings} }} }}"#
        ))
    }

    fn problem(strings: &str) -> String {
        catalogue(strings).err().unwrap_or_default()
    }

    #[test]
    fn strings_are_read_in_the_order_of_their_keys_with_every_language() {
        let read = catalogue(
            r#""tabs.new": { "en": "New Tab", "fr": "Nouvel onglet", "note": "n" },
               "app.name": { "note": "The name.", "en": "WebTiger" }"#,
        )
        .unwrap_or_else(|problem| Catalogue {
            entries: vec![Entry {
                key: problem,
                note: String::new(),
                texts: BTreeMap::new(),
                own: BTreeMap::new(),
            }],
        });
        let keys: Vec<&str> = read
            .entries
            .iter()
            .map(|entry| entry.key.as_str())
            .collect();
        assert_eq!(keys, ["app.name", "tabs.new"]);
        assert_eq!(read.entries[0].note, "The name.");
        assert_eq!(
            read.entries[1].text(Platform::Apple, "fr"),
            Some("Nouvel onglet")
        );
        assert_eq!(read.entries[0].text(Platform::Apple, "fr"), None);
        assert_eq!(read.languages(Platform::Android), ["en", "fr"]);
        assert_eq!(read.entries[1].resource_name(), "tabs_new");
    }

    #[test]
    fn a_platform_has_its_own_wording_where_it_is_given_one() {
        let read = catalogue(
            r#""tabs.close": { "en": "Close Tab", "fr": "Fermer", "android": { "en": "Close tab", "de": "Tab schließen" }, "note": "n" }"#,
        );
        let Ok(read) = read else {
            assert_eq!(read.err(), None);
            return;
        };
        let entry = &read.entries[0];
        assert_eq!(entry.text(Platform::Android, "en"), Some("Close tab"));
        assert_eq!(entry.text(Platform::Apple, "en"), Some("Close Tab"));
        assert_eq!(entry.text(Platform::Windows, "en"), Some("Close Tab"));
        // A language only the platform has, and one only the others have.
        assert_eq!(entry.text(Platform::Android, "de"), Some("Tab schließen"));
        assert_eq!(entry.text(Platform::Apple, "de"), None);
        assert_eq!(entry.text(Platform::Android, "fr"), Some("Fermer"));
        assert_eq!(read.languages(Platform::Android), ["de", "en", "fr"]);
        assert_eq!(read.languages(Platform::Apple), ["en", "fr"]);
    }

    #[test]
    fn what_is_refused_of_an_entry() {
        for (strings, says) in [
            (
                r#""Tabs.new": { "en": "x", "note": "n" }"#,
                "'Tabs.new' is not a dotted lower-case key",
            ),
            (
                r#""tabs": { "en": "x", "note": "n" }"#,
                "'tabs' is not a dotted lower-case key",
            ),
            (
                r#""tabs..new": { "en": "x", "note": "n" }"#,
                "is not a dotted lower-case key",
            ),
            (
                r#""tabs.new": { "en": "x" }"#,
                "'tabs.new' needs both 'en' and 'note'",
            ),
            (
                r#""tabs.new": { "fr": "x", "note": "n" }"#,
                "'tabs.new' needs both 'en' and 'note'",
            ),
            (
                r#""tabs.new": { "en": "x", "note": "n", "english": "y" }"#,
                "'english' is no language, no platform and no note",
            ),
            (
                r#""tabs.new": { "en": "x", "note": "n", "apple": "y" }"#,
                "'apple' is a platform's own wording, by language",
            ),
            (
                r#""tabs.new": { "en": "x", "note": "n", "apple": { "english": "y" } }"#,
                "'english' in 'apple' is not a language tag",
            ),
            (
                r#""tabs.new": { "en": "x", "note": "n", "fr": { "en": "y" } }"#,
                "'fr' is no language, no platform and no note",
            ),
            (
                r#""tabs.new": { "en": "x", "note": "n" }, "tabs.new": { "en": "y", "note": "n" }"#,
                "'tabs.new' is there twice",
            ),
            (
                r#""tabs.new": { "en": "x", "en": "y", "note": "n" }"#,
                "'en' is there twice",
            ),
            (
                r#""a.b_c": { "en": "x", "note": "n" }, "a_b.c": { "en": "y", "note": "n" }"#,
                "are the same Android and Windows resource, a_b_c",
            ),
            (
                r#""1a.b": { "en": "x", "note": "n" }"#,
                "starts with a digit",
            ),
        ] {
            let problem = problem(strings);
            assert!(problem.contains(says), "{strings}: {problem}");
        }
        // A field the file itself does not have.
        let problem = parse(r#"{ "comment": "c", "strings": {}, "extra": 1 }"#).err();
        assert!(problem.is_some_and(|problem| problem.contains("extra")));
    }

    #[test]
    fn a_string_is_its_text_and_its_placeholders() {
        let text = |text: &str| Piece::Text(text.to_owned());
        let of = |text: &str| pieces("k.k", text);
        assert_eq!(of("New Tab"), Ok(vec![text("New Tab")]));
        assert_eq!(
            of("%@ of %d, %ld and %lld; %u%% of %f in %s"),
            Ok(vec![
                argument(None, Kind::Text),
                text(" of "),
                argument(None, Kind::Whole),
                text(", "),
                argument(None, Kind::Whole),
                text(" and "),
                argument(None, Kind::Whole),
                text("; "),
                argument(None, Kind::Whole),
                Piece::PerCent,
                text(" of "),
                argument(None, Kind::Fraction),
                text(" in "),
                argument(None, Kind::Text),
            ])
        );
        assert_eq!(
            of("%2$@ by %1$d"),
            Ok(vec![
                argument(Some(2), Kind::Text),
                text(" by "),
                argument(Some(1), Kind::Whole),
            ])
        );
        for stray in ["50% off", "100%", "%x", "%1$", "%l@", "%1@"] {
            let problem = of(stray).err().unwrap_or_default();
            assert!(
                problem.contains("has a '%' that is not a placeholder"),
                "{stray}: {problem}"
            );
        }
        let problem = of("%1$@ and %@").err().unwrap_or_default();
        assert!(
            problem.contains("numbers some of its placeholders and not others"),
            "{problem}"
        );
    }

    #[test]
    fn every_wording_takes_the_arguments_its_english_takes() {
        // In another order, with numbers: the same arguments.
        assert!(
            catalogue(r#""a.b": { "en": "%@ has %d", "fr": "%2$d pour %1$@", "note": "n" }"#)
                .is_ok()
        );
        assert!(
            catalogue(
                r#""a.b": { "en": "%d%% of %@", "windows": { "en": "%2$@: %1$d%%" }, "note": "n" }"#
            )
            .is_ok()
        );
        for (strings, says) in [
            // One fewer; one more; another kind; the kinds the other way round.
            (
                r#""a.b": { "en": "%@ has %d", "fr": "%@ en a", "note": "n" }"#,
                "'a.b': 'fr' does not take the same arguments as its English",
            ),
            (
                r#""a.b": { "en": "Done", "fr": "Fini, %@", "note": "n" }"#,
                "'a.b': 'fr' does not take the same arguments",
            ),
            (
                r#""a.b": { "en": "%d left", "fr": "%@ restants", "note": "n" }"#,
                "'a.b': 'fr' does not take the same arguments",
            ),
            (
                r#""a.b": { "en": "%@ has %d", "fr": "%d pour %@", "note": "n" }"#,
                "'a.b': 'fr' does not take the same arguments",
            ),
            // A platform's own wording is checked as a language is.
            (
                r#""a.b": { "en": "%@ has %d", "android": { "en": "%@ has some" }, "note": "n" }"#,
                "'a.b': android's 'en' does not take the same arguments",
            ),
            (
                r#""a.b": { "en": "%1$@ and %1$d", "note": "n" }"#,
                "'a.b' uses its argument 1 as two kinds of thing",
            ),
            (
                r#""a.b": { "en": "50% off", "note": "n" }"#,
                "has a '%' that is not a placeholder",
            ),
            (
                r#""a.b": { "en": "Half off", "fr": "50% de moins", "note": "n" }"#,
                "has a '%' that is not a placeholder",
            ),
        ] {
            let problem = problem(strings);
            assert!(problem.contains(says), "{strings}: {problem}");
        }
    }
}
