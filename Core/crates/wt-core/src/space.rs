//! Spaces (024 §3): what a Space's name may be, and how many there may be.

/// The most Spaces there may be: more than a person keeps apart, and no
/// more than a list shows.
pub const MOST_SPACES: usize = 32;

/// The most characters of a Space's name: what a chip can still show.
pub const SPACE_NAME_LENGTH: usize = 60;

/// A Space's name as it is kept, from what was typed: with no white space
/// at the ends, every run of white space inside as one space (a name is one
/// line), and no more than [`SPACE_NAME_LENGTH`] characters. `None` when
/// nothing is left: a Space a person makes or names has a name.
#[must_use]
pub fn space_name(typed: &str) -> Option<String> {
    let tidy = typed
        .split(|c: char| c.is_whitespace() || c.is_control())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let cut: String = tidy.chars().take(SPACE_NAME_LENGTH).collect();
    // What the cut left at its end is no part of a word.
    let name = cut.trim_end();
    (!name.is_empty()).then(|| name.to_owned())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "a test fails the way its assertions do")]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_what_was_typed_tidied_and_never_nothing() {
        assert_eq!(space_name("Work").as_deref(), Some("Work"));
        assert_eq!(
            space_name("  The \t office\n").as_deref(),
            Some("The office")
        );
        // A name is one line, with nothing in it that is not shown.
        assert_eq!(
            space_name("Wo\u{0}rk\r\nhome").as_deref(),
            Some("Wo rk home")
        );
        assert_eq!(space_name(""), None);
        assert_eq!(space_name(" \n\t\u{a0}\u{0} "), None);
    }

    #[test]
    fn a_name_is_cut_by_characters() {
        let long = "é".repeat(SPACE_NAME_LENGTH + 5);
        let cut = space_name(&long).unwrap();
        assert_eq!(cut.chars().count(), SPACE_NAME_LENGTH);
        assert!(cut.chars().all(|c| c == 'é'));
        let exact = "a".repeat(SPACE_NAME_LENGTH);
        assert_eq!(space_name(&exact).as_deref(), Some(exact.as_str()));
        // A space the cut leaves at the end goes.
        let spaced = format!("{} b", "a".repeat(SPACE_NAME_LENGTH - 1));
        assert_eq!(
            space_name(&spaced).as_deref(),
            Some("a".repeat(SPACE_NAME_LENGTH - 1).as_str())
        );
    }
}
