//! `webtiger import arc`, run as a person runs it.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../Fixtures/import/arc/full")
}

fn webtiger(arguments: &[&str], directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_webtiger"))
        .args(arguments)
        .current_dir(directory)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn it_writes_the_portable_file_and_says_what_it_found() {
    let scratch = tempfile::tempdir().unwrap();
    let from = fixture();
    let output = webtiger(
        &["import", "arc", "--from", from.to_str().unwrap()],
        scratch.path(),
    );
    assert!(output.status.success(), "{}", text(&output.stderr));

    let written = scratch.path().join("webtiger-arc-import.json");
    let expected = std::fs::read_to_string(from.join("../full.expected.json")).unwrap();
    assert_eq!(std::fs::read_to_string(&written).unwrap(), expected);

    let said = text(&output.stdout);
    assert!(said.contains("4 Spaces, with 2 sets of logins"), "{said}");
    assert!(
        said.contains("3 folders, 9 pinned tabs, 10 Favorites, 1 tab from Today"),
        "{said}"
    );
    assert!(
        said.contains("3 link rules, 2 legacy Boosts, 2 archived tabs"),
        "{said}"
    );
    assert!(said.contains("1 × Easel"), "{said}");
    // What a person visited and named is in the file, and nowhere else.
    for private in ["example.com", "Handbook", "Example Mail", "Quiet wiki"] {
        assert!(
            !said.contains(private),
            "the report names {private}:\n{said}"
        );
    }
}

#[cfg(unix)]
#[test]
fn the_file_is_its_owners_alone() {
    use std::os::unix::fs::PermissionsExt as _;
    let scratch = tempfile::tempdir().unwrap();
    let out = scratch.path().join("mine.json");
    let output = webtiger(
        &[
            "import",
            "arc",
            "--from",
            fixture().to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ],
        scratch.path(),
    );
    assert!(output.status.success(), "{}", text(&output.stderr));
    let mode = std::fs::metadata(&out).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "the file holds a person's browsing");
}

#[test]
fn an_existing_file_is_kept_unless_forced() {
    let scratch = tempfile::tempdir().unwrap();
    let out = scratch.path().join("webtiger-arc-import.json");
    std::fs::write(&out, "something of mine").unwrap();
    let from = fixture();
    let from = from.to_str().unwrap();

    let refused = webtiger(&["import", "arc", "--from", from], scratch.path());
    assert!(!refused.status.success());
    assert!(text(&refused.stderr).contains("import.output_exists"));
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "something of mine");

    let forced = webtiger(
        &["import", "arc", "--from", from, "--force"],
        scratch.path(),
    );
    assert!(forced.status.success(), "{}", text(&forced.stderr));
    assert!(std::fs::read_to_string(&out).unwrap().starts_with('{'));
}

#[test]
fn a_folder_that_is_not_arcs_fails_with_words_and_a_code_and_writes_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let output = webtiger(
        &["import", "arc", "--from", scratch.path().to_str().unwrap()],
        scratch.path(),
    );
    assert!(!output.status.success());
    let said = text(&output.stderr);
    assert!(said.contains("not Arc's data folder"), "{said}");
    assert!(said.contains("(arc.sidebar_missing)"), "{said}");
    assert!(!scratch.path().join("webtiger-arc-import.json").exists());
}

#[test]
fn parts_can_be_left_out() {
    let scratch = tempfile::tempdir().unwrap();
    let output = webtiger(
        &[
            "import",
            "arc",
            "--from",
            fixture().to_str().unwrap(),
            "--no-today",
            "--no-archive",
            "--no-boosts",
        ],
        scratch.path(),
    );
    assert!(output.status.success(), "{}", text(&output.stderr));
    let said = text(&output.stdout);
    assert!(said.contains("0 tabs from Today"), "{said}");
    assert!(said.contains("0 legacy Boosts, 0 archived tabs"), "{said}");
}
