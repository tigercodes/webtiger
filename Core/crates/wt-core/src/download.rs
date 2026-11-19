//! The name a downloaded file is saved under (013 §4). A page suggests one;
//! the core makes it safe, and a shell never writes a name a page gave it.

use crate::Address;

/// The longest name, in bytes. Every file system WebTiger writes to takes
/// 255; the rest is left for a shell's " (2)".
const LONGEST: usize = 200;

/// What a file is called when nothing better is known.
const NAMELESS: &str = "download";

/// The name to save a download under: from what the site suggested, else
/// the last part of its address, else "download"; with no folder in it,
/// nothing a file system refuses or hides, nothing that makes it read as
/// another name, and an ending when the kind of file says one.
#[must_use]
pub fn download_file_name(suggested: &str, address: &str, mime_type: &str) -> String {
    let mut name = clean(suggested);
    if name.is_empty() {
        name = clean(&last_part(address));
    }
    if name.is_empty() {
        name = NAMELESS.to_owned();
    }
    if !name.contains('.')
        && let Some(ending) = ending_of(mime_type)
    {
        name.push('.');
        name.push_str(ending);
    }
    if is_reserved(&name) {
        let dot = name.find('.').unwrap_or(name.len());
        name.insert(dot, '_');
    }
    shorten(&name)
}

/// A suggested name with everything unsafe taken out of it. Empty when
/// nothing is left.
fn clean(suggested: &str) -> String {
    // No folders: what follows the last separator of either kind.
    let last = suggested.rsplit(['/', '\\']).next().unwrap_or_default();
    let kept: String = last.chars().filter(|c| !is_refused(*c)).collect();
    // Not hidden, and nothing a system drops from the end of a name.
    kept.trim_start_matches(['.', ' '])
        .trim_end_matches(['.', ' '])
        .to_owned()
}

/// A character no name may have: what a file system refuses, and the marks
/// that turn text round, which make `gpj.exe` read as `exe.jpg`.
fn is_refused(c: char) -> bool {
    c.is_control()
        || matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*')
        || matches!(c, '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// The last part of an address's path, as its characters. Empty for an
/// address that is not one, or that ends at its host; and for one that is
/// not the web's, whose path is no file's name (the whole of a `data:`
/// address, the number of a `blob:`).
fn last_part(address: &str) -> String {
    let Ok(address) = Address::parse(address) else {
        return String::new();
    };
    if !matches!(address.scheme(), "http" | "https") {
        return String::new();
    }
    let path = address.path();
    let last = path.rsplit('/').next().unwrap_or_default();
    percent_decoded(last)
}

/// `%41` as `A`. What is not a whole escape, or does not make text, stays
/// as it was written.
fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let escaped = (bytes[index] == b'%')
            .then(|| text.get(index + 1..index + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        if let Some(byte) = escaped {
            decoded.push(byte);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).unwrap_or_else(|_| text.to_owned())
}

/// The ending a kind of file is known by. `None` for a kind that says
/// nothing of what is in it.
fn ending_of(mime_type: &str) -> Option<&'static str> {
    let kind = mime_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    Some(match kind.as_str() {
        "application/pdf" => "pdf",
        "application/zip" => "zip",
        "application/json" => "json",
        "text/plain" => "txt",
        "text/html" => "html",
        "text/csv" => "csv",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "audio/mpeg" => "mp3",
        "video/mp4" => "mp4",
        _ => return None,
    })
}

/// A name Windows keeps for a device, whatever its ending and its case.
fn is_reserved(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.ends_with(|c: char| c.is_ascii_digit() && c != '0'))
}

/// The name in at most [`LONGEST`] bytes: cut in the name, not in its
/// ending, and never inside a character.
fn shorten(name: &str) -> String {
    if name.len() <= LONGEST {
        return name.to_owned();
    }
    let ending = ending(name);
    // An ending longer than a name may be is no ending.
    let ending = if ending.len() < LONGEST / 2 {
        ending
    } else {
        ""
    };
    let room = LONGEST - ending.len();
    let stem = &name[..name.len() - ending.len()];
    let mut cut = room.min(stem.len());
    while !stem.is_char_boundary(cut) {
        cut -= 1;
    }
    let kept = stem[..cut].trim_end_matches(['.', ' ']);
    format!("{kept}{ending}")
}

/// A name's ending, with its dot: `.gz`, or `.tar.gz` whole. Empty when it
/// has none.
fn ending(name: &str) -> &str {
    let Some(dot) = name.rfind('.') else {
        return "";
    };
    let before = &name[..dot];
    if before.to_ascii_lowercase().ends_with(".tar") {
        return &name[before.len() - ".tar".len()..];
    }
    &name[dot..]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(suggested: &str) -> String {
        download_file_name(
            suggested,
            "https://example.com/files/from-the-address.bin",
            "",
        )
    }

    #[test]
    fn a_plain_name_stays_as_it_is() {
        for name in [
            "report.pdf",
            "Holiday photo (3).JPG",
            "archive.tar.gz",
            "résumé – final.docx",
            "файл.txt",
            "no ending",
        ] {
            assert_eq!(named(name), name);
        }
    }

    #[test]
    fn a_name_has_no_folder_in_it() {
        assert_eq!(named("../../etc/passwd"), "passwd");
        assert_eq!(named("/Users/ada/.ssh/id_rsa"), "id_rsa");
        assert_eq!(named("C:\\Windows\\System32\\drivers\\etc\\hosts"), "hosts");
        assert_eq!(named("a/b\\c/report.pdf"), "report.pdf");
        assert_eq!(named("..\\..\\evil.bat"), "evil.bat");
    }

    #[test]
    fn nothing_a_file_system_refuses_or_hides() {
        assert_eq!(named("what?<is>:\"this\"|*.txt"), "whatisthis.txt");
        assert_eq!(named("line\nbreak\tand\u{0}nul.txt"), "linebreakandnul.txt");
        assert_eq!(named(".bashrc"), "bashrc", "not hidden");
        assert_eq!(named("...  .hidden.txt"), "hidden.txt");
        assert_eq!(named("trailing dots... "), "trailing dots");
        assert_eq!(named("name.txt. . ."), "name.txt");
    }

    #[test]
    fn nothing_that_turns_the_name_round() {
        // Shown, this reads "photo_exe.jpg"; it is a program.
        assert_eq!(named("photo_\u{202E}gpj.exe"), "photo_gpj.exe");
        assert_eq!(named("a\u{2066}b\u{2069}c\u{200F}.txt"), "abc.txt");
    }

    #[test]
    fn a_name_a_system_keeps_for_a_device_is_changed() {
        assert_eq!(named("CON"), "CON_");
        assert_eq!(named("nul.txt"), "nul_.txt");
        assert_eq!(named("Com1.log"), "Com1_.log");
        assert_eq!(named("LPT9"), "LPT9_");
        // And what only looks like one is not.
        for name in [
            "CONTACT.txt",
            "COM0.txt",
            "COM10.txt",
            "nullable.rs",
            "aux.d/x",
        ] {
            assert!(!named(name).contains('_'), "{name}");
        }
    }

    #[test]
    fn with_nothing_suggested_the_address_names_it() {
        let from = |address: &str| download_file_name("", address, "");
        assert_eq!(from("https://example.com/files/report.pdf"), "report.pdf");
        assert_eq!(
            from("https://example.com/files/report.pdf?version=2#page"),
            "report.pdf"
        );
        assert_eq!(
            from("https://example.com/a%20long%20name.txt"),
            "a long name.txt"
        );
        assert_eq!(
            from("https://example.com/%D1%84%D0%B0%D0%B9%D0%BB.txt"),
            "файл.txt"
        );
        // An escape that hides a folder is a folder all the same.
        assert_eq!(from("https://example.com/..%2F..%2Fetc%2Fpasswd"), "passwd");
        assert_eq!(
            from("https://example.com/broken%zzname%2"),
            "broken%zzname%2"
        );
        // A suggestion that comes to nothing is no suggestion.
        assert_eq!(
            download_file_name("///", "https://example.com/x.zip", ""),
            "x.zip"
        );
        assert_eq!(
            download_file_name(" . ", "https://example.com/x.zip", ""),
            "x.zip"
        );
    }

    #[test]
    fn a_name_is_never_empty() {
        for address in [
            "https://example.com/",
            "https://example.com",
            "about:blank",
            // Not the web's: its path is no file's name.
            "blob:https://example.com/0f0e1d2c-3b4a-4958-8776-655443322110",
            "data:text/plain;base64,aGVsbG8=",
            "not an address",
            "",
        ] {
            assert_eq!(
                download_file_name("", address, ""),
                "download",
                "{address:?}"
            );
        }
        assert_eq!(
            download_file_name("", "https://example.com/", "application/pdf"),
            "download.pdf"
        );
        assert_eq!(
            download_file_name("?", "https://example.com/files/", "image/png"),
            "download.png"
        );
    }

    #[test]
    fn a_name_with_no_ending_takes_its_kinds() {
        let kind = |name: &str, mime: &str| download_file_name(name, "https://example.com/", mime);
        assert_eq!(kind("report", "application/pdf"), "report.pdf");
        assert_eq!(kind("photo", "IMAGE/JPEG; q=1"), "photo.jpg");
        assert_eq!(kind("page", "text/html; charset=utf-8"), "page.html");
        // A kind that says nothing adds nothing, and an ending there already stays.
        assert_eq!(kind("blob", "application/octet-stream"), "blob");
        assert_eq!(kind("report.docx", "application/pdf"), "report.docx");
        assert_eq!(kind("CON", "text/plain"), "CON_.txt");
    }

    #[test]
    fn a_long_name_is_cut_in_the_name_and_not_in_its_ending() {
        let long = "a".repeat(300);
        let cut = named(&format!("{long}.pdf"));
        assert_eq!(cut.len(), 200);
        assert!(cut.ends_with(".pdf") && cut.starts_with("aaaa"));

        let tarred = named(&format!("{long}.tar.gz"));
        assert_eq!(tarred.len(), 200);
        assert!(
            tarred.ends_with("a.tar.gz"),
            "the double ending stays whole"
        );

        // Never inside a character: each of these is two bytes.
        let cyrillic = named(&format!("{}.txt", "ж".repeat(150)));
        assert!(cyrillic.len() <= 200 && cyrillic.ends_with("ж.txt"));
        assert_eq!(cyrillic.len(), 200, "196 bytes of name and four of ending");

        // An ending as long as a name is not kept as one.
        let all_ending = named(&format!("x.{}", "e".repeat(400)));
        assert_eq!(all_ending.len(), 200);
        assert!(
            named(&"b".repeat(200)).len() == 200,
            "a name of exactly the most is whole"
        );
    }
}
