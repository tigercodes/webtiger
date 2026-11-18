//! A version 4 UUID: 122 random bits. Login sets are named by one.

use std::fmt;
use std::str::FromStr;

/// A UUID, kept as its 16 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uuid([u8; 16]);

/// The system could not supply random bytes.
#[derive(Debug, PartialEq, Eq)]
pub struct RandomError;

/// The text is not a UUID in the 8-4-4-4-12 form.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError;

impl Uuid {
    /// A new random UUID from the operating system's generator.
    ///
    /// # Errors
    /// When the system has no random bytes to give.
    pub fn random() -> Result<Uuid, RandomError> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes).map_err(|_| RandomError)?;
        Ok(Uuid::from_random_bytes(bytes))
    }

    /// A version 4 UUID from 16 bytes the caller drew: the version and
    /// variant bits are set over them. Fake mode passes fixed bytes.
    #[must_use]
    pub const fn from_random_bytes(mut bytes: [u8; 16]) -> Uuid {
        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        bytes[8] = (bytes[8] & 0x3F) | 0x80;
        Uuid(bytes)
    }

    /// A UUID from its 16 bytes, as stored.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Uuid {
        Uuid(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for Uuid {
    /// Lower case with hyphens: `3f2b8c1e-…`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, byte) in self.0.iter().enumerate() {
            if matches!(index, 4 | 6 | 8 | 10) {
                f.write_str("-")?;
            }
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for Uuid {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<Uuid, ParseError> {
        let text = text.as_bytes();
        if text.len() != 36 {
            return Err(ParseError);
        }
        let mut bytes = [0u8; 16];
        let mut at = 0;
        for byte in &mut bytes {
            if matches!(at, 8 | 13 | 18 | 23) {
                if text.get(at) != Some(&b'-') {
                    return Err(ParseError);
                }
                at += 1;
            }
            let high = hex(text.get(at).copied())?;
            let low = hex(text.get(at + 1).copied())?;
            *byte = (high << 4) | low;
            at += 2;
        }
        Ok(Uuid(bytes))
    }
}

fn hex(digit: Option<u8>) -> Result<u8, ParseError> {
    match digit {
        Some(d @ b'0'..=b'9') => Ok(d - b'0'),
        Some(d @ b'a'..=b'f') => Ok(d - b'a' + 10),
        Some(d @ b'A'..=b'F') => Ok(d - b'A' + 10),
        _ => Err(ParseError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_random_uuid_is_version_four_and_round_trips() {
        let uuid = Uuid::random().unwrap();
        let text = uuid.to_string();
        assert_eq!(text.len(), 36);
        assert_eq!(text.as_bytes()[14], b'4');
        assert!(matches!(text.as_bytes()[19], b'8' | b'9' | b'a' | b'b'));
        assert_eq!(text.parse::<Uuid>().unwrap(), uuid);
    }

    #[test]
    fn two_random_uuids_differ() {
        assert_ne!(Uuid::random().unwrap(), Uuid::random().unwrap());
    }

    #[test]
    fn fixed_bytes_give_a_fixed_uuid() {
        let uuid = Uuid::from_random_bytes([0x11; 16]);
        assert_eq!(uuid.to_string(), "11111111-1111-4111-9111-111111111111");
    }

    #[test]
    fn text_that_is_not_a_uuid_is_refused() {
        for text in [
            "",
            "11111111-1111-4111-9111-11111111111",
            "11111111-1111-4111-9111-1111111111111",
            "11111111x1111-4111-9111-111111111111",
            "1111111g-1111-4111-9111-111111111111",
            "11111111-1111-4111-9111-11111111111é",
        ] {
            assert_eq!(text.parse::<Uuid>(), Err(ParseError), "{text:?}");
        }
    }

    #[test]
    fn upper_case_parses_and_prints_lower() {
        let uuid: Uuid = "ABCDEF01-2345-4789-8ABC-DEF012345678".parse().unwrap();
        assert_eq!(uuid.to_string(), "abcdef01-2345-4789-8abc-def012345678");
    }
}
