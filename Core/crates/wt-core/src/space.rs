//! Spaces (024 §3): what a Space's name may be, and how many there may be.

/// The most Spaces there may be: more than a person keeps apart, and no
/// more than a list shows.
pub const MOST_SPACES: usize = 32;

/// The most characters of a Space's name: what a chip can still show.
pub const SPACE_NAME_LENGTH: usize = 60;

/// The colours a Space can have (025 §3), by the names they cross the
/// contract with, in their order. The shells have the paint and the word.
pub const SPACE_COLOURS: [&str; 9] = [
    "red", "orange", "yellow", "green", "teal", "blue", "purple", "pink", "grey",
];

/// The colour of that name, when it is one a Space can have.
#[must_use]
pub fn space_colour(name: &str) -> Option<&'static str> {
    SPACE_COLOURS.iter().copied().find(|colour| *colour == name)
}

/// The colour a new Space is given when none is asked for (025 §3): the
/// first that no Space has; and when every colour is somebody's, the one
/// the fewest Spaces have, the first of those. `taken` is the colour of
/// each Space there is; one with none, or with a colour this build does not
/// know, takes none.
#[must_use]
pub fn next_space_colour<'a>(taken: impl IntoIterator<Item = &'a str>) -> &'static str {
    let mut counts = [0usize; SPACE_COLOURS.len()];
    for colour in taken {
        if let Some(index) = SPACE_COLOURS.iter().position(|known| *known == colour) {
            counts[index] += 1;
        }
    }
    // `min_by_key` gives the first of the least: the first unused while
    // there is one.
    SPACE_COLOURS
        .iter()
        .zip(counts)
        .min_by_key(|(_, count)| *count)
        .map_or(SPACE_COLOURS[0], |(colour, _)| *colour)
}

/// The icons a Space can have (050 §3), in their order: each an emoji, which
/// is what crosses the contract and what every system draws with its own
/// font. Written by their code points, so that no editor changes one: house,
/// briefcase, books, backpack, shopping bags, money bag, aeroplane, puzzle
/// piece, guitar, popcorn, newspaper, speech balloon, hammer and spanner,
/// test tube, seedling, heart, star, lock, globe, camera, pizza, running
/// shoe, dog, light bulb. Each is one that is seen on a light ground
/// and on a dark one: a black picture (a graduation cap, a game controller)
/// is lost on a dark window.
pub const SPACE_ICONS: [&str; 24] = [
    "\u{1F3E0}",
    "\u{1F4BC}",
    "\u{1F4DA}",
    "\u{1F392}",
    "\u{1F6CD}\u{FE0F}",
    "\u{1F4B0}",
    "\u{2708}\u{FE0F}",
    "\u{1F9E9}",
    "\u{1F3B8}",
    "\u{1F37F}",
    "\u{1F4F0}",
    "\u{1F4AC}",
    "\u{1F6E0}\u{FE0F}",
    "\u{1F9EA}",
    "\u{1F331}",
    "\u{2764}\u{FE0F}",
    "\u{2B50}",
    "\u{1F512}",
    "\u{1F30D}",
    "\u{1F4F7}",
    "\u{1F355}",
    "\u{1F45F}",
    "\u{1F436}",
    "\u{1F4A1}",
];

/// That icon, when it is one a Space can have: the very same characters.
#[must_use]
pub fn space_icon(icon: &str) -> Option<&'static str> {
    SPACE_ICONS.iter().copied().find(|known| *known == icon)
}

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
    fn a_colour_is_one_of_the_list_by_its_own_name() {
        assert_eq!(SPACE_COLOURS[0], "red");
        assert_eq!(SPACE_COLOURS.len(), 9);
        for colour in SPACE_COLOURS {
            assert_eq!(space_colour(colour), Some(colour));
        }
        // Each is there once.
        let mut sorted = SPACE_COLOURS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), SPACE_COLOURS.len());
        // No colour, a colour written another way, and one nobody has heard of.
        for other in ["", "Blue", " blue", "gray", "#0000ff", "mauve"] {
            assert_eq!(space_colour(other), None, "{other}");
        }
    }

    #[test]
    fn an_icon_is_one_of_the_list_as_it_is_written_there() {
        assert_eq!(SPACE_ICONS.len(), 24);
        assert_eq!(SPACE_ICONS[0], "\u{1F3E0}", "a house is the first");
        for icon in SPACE_ICONS {
            assert_eq!(space_icon(icon), Some(icon));
            // Each is an emoji and nothing else: no letter, digit or space of ASCII, and
            // short enough to be one picture.
            assert!(icon.chars().all(|c| !c.is_ascii()), "{icon:?}");
            assert!((1..=2).contains(&icon.chars().count()), "{icon:?}");
        }
        // Each is there once.
        let mut sorted = SPACE_ICONS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), SPACE_ICONS.len());
        // None; an emoji that is not in the list; one of the list without the selector it
        // is written with, and with something after it; and what is no emoji at all.
        for other in [
            "",
            "\u{1F600}",
            "\u{2708}",
            "\u{1F3E0}\u{1F3E0}",
            "\u{1F3E0} ",
            "house",
            "W",
        ] {
            assert_eq!(space_icon(other), None, "{other:?}");
        }
    }

    #[test]
    fn a_new_space_gets_the_first_colour_nobody_has_and_then_the_least_used() {
        assert_eq!(next_space_colour([]), "red");
        // A Space with none takes none, and so does a colour nobody knows.
        assert_eq!(next_space_colour(["", "mauve"]), "red");
        assert_eq!(next_space_colour(["red"]), "orange");
        // The first that is free, whatever order they were taken in.
        assert_eq!(next_space_colour(["orange", "red", "green"]), "yellow");
        assert_eq!(next_space_colour(["blue", "red"]), "orange");
        // Every colour is somebody's: the first again.
        assert_eq!(next_space_colour(SPACE_COLOURS), "red");
        // And then the one the fewest have, the first of those.
        let mut taken = SPACE_COLOURS.to_vec();
        taken.extend(["red", "orange", "green"]);
        assert_eq!(next_space_colour(taken.iter().copied()), "yellow");
        taken.extend(["yellow", "teal", "blue", "purple", "pink"]);
        assert_eq!(next_space_colour(taken.iter().copied()), "grey");
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
