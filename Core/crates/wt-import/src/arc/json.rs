//! Reading Arc's JSON (002 §5). Arc's files are Swift `Codable` output:
//! a dictionary is a flat array alternating key and value, an enum is an
//! object with one key, and a time is seconds since 2001. Everything here
//! returns `None` for a shape it does not know, so an unknown key is
//! skipped and never fails a file.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::Value;

use crate::ImportError;

/// The largest file read: a sidebar file is a few megabytes at most.
pub(crate) const MAX_FILE: u64 = 64 * 1024 * 1024;

/// Seconds between 1970-01-01 and 2001-01-01, the epoch Arc's times use.
const APPLE_EPOCH: f64 = 978_307_200.0;

/// What was left out or changed, counted by code.
#[derive(Debug, Default)]
pub(crate) struct Notes(BTreeMap<&'static str, u64>);

impl Notes {
    pub(crate) fn add(&mut self, code: &'static str) {
        *self.0.entry(code).or_insert(0) += 1;
    }

    pub(crate) fn into_notes(self) -> Vec<crate::Note> {
        self.0
            .into_iter()
            .map(|(code, count)| crate::Note {
                code: code.to_owned(),
                count,
            })
            .collect()
    }
}

/// How a file failed to be read.
pub(crate) enum ReadError {
    Missing,
    TooLarge,
    Unreadable,
    Malformed,
}

/// Reads a JSON file, refusing one over [`MAX_FILE`] without reading it.
pub(crate) fn read(path: &Path) -> Result<Value, ReadError> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ReadError::Missing
        } else {
            ReadError::Unreadable
        }
    })?;
    if metadata.len() > MAX_FILE {
        return Err(ReadError::TooLarge);
    }
    let bytes = std::fs::read(path).map_err(|_| ReadError::Unreadable)?;
    serde_json::from_slice(&bytes).map_err(|_| ReadError::Malformed)
}

/// Reads an optional JSON file: `None` when it is not there or cannot be
/// used, with a note when it was there and could not be.
pub(crate) fn read_optional(
    path: &Path,
    notes: &mut Notes,
    unreadable: &'static str,
) -> Option<Value> {
    match read(path) {
        Ok(value) => Some(value),
        Err(ReadError::Missing) => None,
        Err(_) => {
            notes.add(unreadable);
            None
        }
    }
}

/// The sidebar file's failures as import errors.
pub(crate) fn sidebar_error(error: &ReadError) -> ImportError {
    ImportError::new(match error {
        ReadError::Missing => "arc.sidebar_missing",
        ReadError::TooLarge => "arc.sidebar_too_large",
        ReadError::Unreadable => "arc.sidebar_unreadable",
        ReadError::Malformed => "arc.sidebar_malformed",
    })
}

/// Follows keys into nested objects.
pub(crate) fn at<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().try_fold(value, |value, key| value.get(key))
}

/// A string under a key, when it is there and not empty.
pub(crate) fn text<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key)?.as_str().filter(|text| !text.is_empty())
}

/// Whether an enum value is the case `name`: an object with that key,
/// whatever its payload, or the bare string.
pub(crate) fn is_case(value: &Value, name: &str) -> bool {
    value.get(name).is_some() || value.as_str() == Some(name)
}

/// The objects of an array, whatever sits between them. An alternating
/// `[id, object, id, object]` list is read this way: every object in Arc's
/// lists carries its own `id`, so the keys add nothing.
pub(crate) fn objects(value: Option<&Value>) -> impl Iterator<Item = &Value> {
    value
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|element| element.is_object())
}

/// The pairs of an alternating `[key, value, key, value]` array.
pub(crate) fn pairs(value: Option<&Value>) -> impl Iterator<Item = (&Value, &Value)> {
    value
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[key, value]| (key, value))
}

/// One of Arc's times as milliseconds since 1970-01-01 UTC.
pub(crate) fn time(value: Option<&Value>) -> Option<i64> {
    let seconds = value?.as_f64()?;
    let millis = ((seconds + APPLE_EPOCH) * 1000.0).round();
    // Between 1970 and the year 3000: anything else is not a time.
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the range is checked on the line above the cast"
    )]
    (millis.is_finite() && (0.0..32_503_680_000_000.0).contains(&millis)).then_some(millis as i64)
}

/// The address of a tab when it is one WebTiger can open: `http` or
/// `https`. Arc's own pages, local files and script are not.
pub(crate) fn web_address(address: Option<&str>) -> Option<String> {
    let parsed = wt_core::Address::parse(address?.trim()).ok()?;
    matches!(parsed.scheme(), "http" | "https")
        .then(|| parsed.as_str().to_owned())
        .filter(|_| parsed.host().is_some())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn times_count_from_2001() {
        // 2024-08-27T10:05:47.5Z.
        assert_eq!(time(Some(&json!(746_426_747.5))), Some(1_724_733_947_500));
        assert_eq!(time(Some(&json!(0))), Some(978_307_200_000));
        assert_eq!(time(Some(&json!(-63_114_076_800.0))), None, "Arc's 'never'");
        assert_eq!(time(Some(&json!("yesterday"))), None);
        assert_eq!(time(Some(&json!(1e300))), None);
        assert_eq!(time(None), None);
    }

    #[test]
    fn only_web_addresses_are_kept() {
        assert_eq!(
            web_address(Some(" https://example.com/a ")).as_deref(),
            Some("https://example.com/a")
        );
        assert_eq!(
            web_address(Some("http://localhost:3000")).as_deref(),
            Some("http://localhost:3000/")
        );
        for other in [
            "arc://settings",
            "chrome://extensions",
            "javascript:void(0)",
            "file:///Users/ada/notes.html",
            "about:blank",
            "data:text/html,x",
            "not an address",
            "",
        ] {
            assert_eq!(web_address(Some(other)), None, "{other}");
        }
        assert_eq!(web_address(None), None);
    }

    #[test]
    fn enum_cases_are_found_by_their_key() {
        assert!(is_case(&json!({"default": {}}), "default"));
        assert!(is_case(&json!({"default": true}), "default"));
        assert!(is_case(&json!("pinned"), "pinned"));
        assert!(!is_case(&json!({"custom": {}}), "default"));
        assert!(!is_case(&json!(null), "default"));
    }

    #[test]
    fn alternating_lists_are_walked_without_trusting_their_shape() {
        let list = json!(["a", {"id": "a"}, "b", {"id": "b"}, "dangling"]);
        assert_eq!(objects(Some(&list)).count(), 2);
        assert_eq!(pairs(Some(&list)).count(), 2);
        assert_eq!(objects(Some(&json!("not a list"))).count(), 0);
        assert_eq!(pairs(None).count(), 0);
    }
}
