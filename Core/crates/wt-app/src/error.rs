//! Failures, as the stable codes the contract carries. No text a user would
//! read, and nothing of what the user typed or visited, travels in one.

use wt_proto::v1::Error;
use wt_store::StoreError;

/// Why a request failed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppError {
    code: String,
    field: &'static str,
}

impl AppError {
    #[must_use]
    pub fn new(code: &str) -> AppError {
        AppError {
            code: code.to_owned(),
            field: "",
        }
    }

    /// Names the request field at fault.
    #[must_use]
    pub fn field(mut self, field: &'static str) -> AppError {
        self.field = field;
        self
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The contract's form.
    #[must_use]
    pub fn into_proto(self) -> Error {
        Error {
            code: self.code,
            field: self.field.to_owned(),
        }
    }
}

impl From<HostError> for AppError {
    fn from(error: HostError) -> AppError {
        AppError::new(&error.code)
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> AppError {
        AppError::new(error.code())
    }
}

/// Why the shell did not do what the core asked: its own code
/// ("engine.unavailable"), or one of the bridge's ("host.timeout",
/// "host.wrong_answer", "host.failed", "host.stopped").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostError {
    pub(crate) code: String,
}

impl HostError {
    #[must_use]
    pub fn new(code: &str) -> HostError {
        HostError {
            code: code.to_owned(),
        }
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }
}

/// Whether `text` has a code's spelling: `area.reason`, each part lower-case
/// letters, digits and underscores, at most 64 bytes in all. Anything else a
/// shell sends as a code is replaced, so free text never rides in an error.
#[must_use]
pub fn is_code(text: &str) -> bool {
    let part = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    };
    text.len() <= 64
        && text
            .split_once('.')
            .is_some_and(|(area, reason)| part(area) && reason.split('.').all(part))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_have_one_spelling() {
        for code in [
            "net.dns",
            "host.wrong_answer",
            "page.renderer_gone",
            "a.b.c",
            "x1.y2",
        ] {
            assert!(is_code(code), "{code}");
        }
        for text in [
            "",
            "net",
            ".dns",
            "net.",
            "net..dns",
            "Net.Dns",
            "net dns",
            "The server at example.com could not be found.",
            "net.dns: example.com",
        ] {
            assert!(!is_code(text), "{text:?}");
        }
        assert!(!is_code(&format!("a.{}", "b".repeat(64))));
    }
}
