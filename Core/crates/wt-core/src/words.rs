//! Whether what was typed is found in a text: every word typed, as a word or
//! the start of one, whatever its case. A word is letters and digits and
//! nothing else.
//!
//! It is the rule history's index applies to visits (008 §4), for what is in
//! no index: the tabs a window has open (038). One difference: the index
//! looks past an accent ("cafe" finds "Café"), and this does not.

/// The most words of what is typed that are looked for. More are dropped, as
/// the index drops them.
pub const MAX_WORDS: usize = 16;

/// The words of what was typed, in lower case. Empty when there is no word
/// in it (nothing, or only punctuation).
#[must_use]
pub fn typed_words(text: &str) -> Vec<String> {
    words(text).take(MAX_WORDS).collect()
}

/// Whether every word typed is found in one of the texts, as a word of it or
/// the start of one. `false` when nothing was typed: nothing matches no words.
#[must_use]
pub fn all_found(typed: &[String], texts: &[&str]) -> bool {
    if typed.is_empty() {
        return false;
    }
    let there: Vec<String> = texts.iter().flat_map(|text| words(text)).collect();
    typed
        .iter()
        .all(|word| there.iter().any(|found| found.starts_with(word.as_str())))
}

fn words(text: &str) -> impl Iterator<Item = String> + '_ {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(typed: &str, texts: &[&str]) -> bool {
        all_found(&typed_words(typed), texts)
    }

    const TITLE: &str = "The Rust Programming Language";
    const ADDRESS: &str = "https://doc.rust-lang.org/book/ch04-01.html";

    #[test]
    fn every_word_typed_is_a_word_or_the_start_of_one() {
        assert!(found("rust", &[TITLE, ADDRESS]));
        assert!(found("rus boo", &[TITLE, ADDRESS]), "one in each text");
        assert!(found("PROG lang", &[TITLE, ADDRESS]), "whatever its case");
        assert!(found("ch04", &[TITLE, ADDRESS]), "digits are of a word");
        assert!(found("doc.rust-lang.org/book", &[TITLE, ADDRESS]));
        // The middle of a word is not its start, and one word missing is a miss.
        assert!(!found("ust", &[TITLE, ADDRESS]));
        assert!(!found("rust python", &[TITLE, ADDRESS]));
        // A word typed twice is found once.
        assert!(found("rust rust", &[TITLE]));
    }

    #[test]
    fn nothing_typed_and_only_punctuation_find_nothing() {
        assert!(typed_words("").is_empty());
        assert!(typed_words(" -- / ").is_empty());
        assert!(!found("", &[TITLE]));
        assert!(!found("...", &[TITLE]));
        assert!(!found("rust", &[]));
        assert!(!found("rust", &[""]));
    }

    #[test]
    fn letters_of_any_script_are_words_and_an_accent_is_not_looked_past() {
        assert!(found("例え", &["例え.中国 のページ"]));
        assert!(found("café", &["Café Central"]));
        assert!(found("CAFÉ", &["café central"]));
        assert!(!found("cafe", &["Café Central"]), "unlike history's index");
    }

    #[test]
    fn no_more_words_are_looked_for_than_the_index_looks_for() {
        let many = (0..40)
            .map(|n| format!("w{n}"))
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(typed_words(&many).len(), MAX_WORDS);
        // The words past the limit are not asked for.
        let first: String = (0..MAX_WORDS).map(|n| format!("w{n} ")).collect();
        assert!(found(&many, &[&first]));
    }
}
