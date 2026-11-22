//! Writes every platform's strings from the shared catalogue,
//! `Localization/strings.json` (040): Apple's String Catalog and the Swift
//! names the code asks by, Android's `strings.xml`, and Windows'
//! `Resources.resw`. `--check` writes nothing and fails when a file is stale
//! or is left over. `Tools/strings/generate.sh` runs it, from `Core/`, so the
//! pinned toolchain builds it.

mod android;
mod apple;
mod catalogue;
mod windows;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use catalogue::Platform;

/// The catalogue, relative to the repository.
const CATALOGUE: &str = "Localization/strings.json";
/// Apple's String Catalog, relative to the repository.
const APPLE_CATALOG: &str = "Packages/Sources/WTUI/Resources/Localizable.xcstrings";
/// The Swift names, relative to the repository.
const APPLE_NAMES: &str = "Packages/Sources/WTUI/Words.generated.swift";
/// Android's resources, relative to the repository: a folder for each language.
const ANDROID: &str = "Android/app/src/main/res";
/// Windows' resources, relative to the repository: a folder for each language.
const WINDOWS: &str = "Windows/Strings";
const USAGE: &str = "usage: wt-strings-gen --root <repository> [--check]";
const RUN: &str = "run Tools/strings/generate.sh";

fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}

/// Every file the catalogue makes, by its path in the repository.
fn files(text: &str) -> Result<Vec<(PathBuf, String)>, String> {
    let catalogue = catalogue::parse(text)?;
    let mut files = vec![
        (
            PathBuf::from(APPLE_CATALOG),
            apple::string_catalog(&catalogue)?,
        ),
        (PathBuf::from(APPLE_NAMES), apple::swift_names(&catalogue)?),
    ];
    for language in catalogue.languages(Platform::Android) {
        let path = Path::new(ANDROID)
            .join(android::folder(&language))
            .join("strings.xml");
        files.push((path, android::strings_xml(&catalogue, &language)?));
    }
    for language in catalogue.languages(Platform::Windows) {
        let path = Path::new(WINDOWS).join(&language).join("Resources.resw");
        files.push((path, windows::resw(&catalogue, &language)?));
    }
    Ok(files)
}

/// The files of this tool's that are for a language the catalogue no longer
/// has: an Android or a Windows folder's file that carries the tool's line
/// and is not among the files wanted.
fn leftovers(root: &Path, wanted: &[(PathBuf, String)]) -> Vec<PathBuf> {
    let mut left = Vec::new();
    for (folder, name, header) in [
        (ANDROID, "strings.xml", android::HEADER),
        (WINDOWS, "Resources.resw", windows::HEADER),
    ] {
        let Ok(languages) = fs::read_dir(root.join(folder)) else {
            continue;
        };
        for language in languages.flatten() {
            let path = Path::new(folder).join(language.file_name()).join(name);
            let is_wanted = wanted.iter().any(|(wanted, _)| *wanted == path);
            let is_the_tools = fs::read_to_string(root.join(&path))
                .is_ok_and(|text| text.lines().nth(1) == Some(header));
            if !is_wanted && is_the_tools {
                left.push(path);
            }
        }
    }
    left.sort();
    left
}

fn run(mut arguments: impl Iterator<Item = String>) -> Result<String, String> {
    let mut root = None;
    let mut check = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => root = Some(PathBuf::from(arguments.next().ok_or(USAGE)?)),
            "--check" => check = true,
            _ => return Err(format!("unknown argument '{argument}'; {USAGE}")),
        }
    }
    let root = root.ok_or(USAGE)?;
    let text = fs::read_to_string(root.join(CATALOGUE))
        .map_err(|error| format!("{CATALOGUE}: {error}"))?;
    let files = files(&text).map_err(|problem| format!("{CATALOGUE}: {problem}"))?;
    let stale: Vec<&(PathBuf, String)> = files
        .iter()
        .filter(|(path, text)| {
            fs::read_to_string(root.join(path)).ok().as_deref() != Some(text.as_str())
        })
        .collect();
    let left = leftovers(&root, &files);
    if check {
        if stale.is_empty() && left.is_empty() {
            return Ok(format!(
                "Every platform's strings are up to date ({} files).",
                files.len()
            ));
        }
        let paths: Vec<String> = stale
            .iter()
            .map(|(path, _)| path.display().to_string())
            .chain(
                left.iter()
                    .map(|path| format!("{} (left over)", path.display())),
            )
            .collect();
        return Err(format!(
            "Stale against {CATALOGUE}; {RUN}:\n  {}",
            paths.join("\n  ")
        ));
    }
    // Only what changed is written, so an unchanged file keeps its date and
    // no build redoes it.
    for (path, text) in &stale {
        write(&root.join(path), text)?;
    }
    for path in &left {
        fs::remove_file(root.join(path)).map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(format!(
        "Wrote {} of {} files from {CATALOGUE}, and removed {}.",
        stale.len(),
        files.len(),
        left.len()
    ))
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    }
    fs::write(path, text).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TWO: &str = r#"{ "comment": "c", "strings": {
        "tabs.new": { "en": "New Tab", "fr": "Nouvel onglet", "note": "A command." },
        "find.count": { "en": "%1$d of %2$d", "note": "The count." } } }"#;
    const ENGLISH_ONLY: &str = r#"{ "comment": "c", "strings": {
        "tabs.new": { "en": "New Tab", "note": "A command." } } }"#;

    fn arguments(root: &Path, check: bool) -> impl Iterator<Item = String> {
        let mut arguments = vec!["--root".to_owned(), root.display().to_string()];
        if check {
            arguments.push("--check".to_owned());
        }
        arguments.into_iter()
    }

    /// A folder of its own under the system's, with the catalogue in it.
    fn repository(name: &str, catalogue: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("wt-strings-gen-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let _ = fs::create_dir_all(root.join("Localization"));
        let _ = fs::write(root.join(CATALOGUE), catalogue);
        root
    }

    #[test]
    fn the_checked_in_catalogue_makes_every_platforms_files() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../Localization/strings.json");
        let text = fs::read_to_string(path).unwrap_or_default();
        let made = files(&text).unwrap_or_default();
        let paths: Vec<String> = made
            .iter()
            .map(|(path, _)| path.display().to_string())
            .collect();
        assert_eq!(
            paths,
            [
                APPLE_CATALOG,
                APPLE_NAMES,
                "Android/app/src/main/res/values/strings.xml",
                "Windows/Strings/en/Resources.resw",
            ]
        );
        assert!(made.iter().all(|(_, text)| text.len() > 1000));
    }

    #[test]
    fn every_file_is_written_checked_and_only_rewritten_when_it_changed() {
        let root = repository("write", TWO);
        // Nothing is there: a check says every file is stale, and writes none.
        let problem = run(arguments(&root, true)).err().unwrap_or_default();
        assert!(
            problem.starts_with(
                "Stale against Localization/strings.json; run Tools/strings/generate.sh:"
            ),
            "{problem}"
        );
        assert_eq!(problem.lines().count(), 7, "{problem}");
        assert!(!root.join(APPLE_CATALOG).exists());

        assert_eq!(
            run(arguments(&root, false)),
            Ok("Wrote 6 of 6 files from Localization/strings.json, and removed 0.".to_owned())
        );
        for path in [
            APPLE_CATALOG,
            APPLE_NAMES,
            "Android/app/src/main/res/values/strings.xml",
            "Android/app/src/main/res/values-fr/strings.xml",
            "Windows/Strings/en/Resources.resw",
            "Windows/Strings/fr/Resources.resw",
        ] {
            assert!(root.join(path).exists(), "{path}");
        }
        assert_eq!(
            run(arguments(&root, true)),
            Ok("Every platform's strings are up to date (6 files).".to_owned())
        );
        // Again: nothing changed, nothing is written.
        assert_eq!(
            run(arguments(&root, false)),
            Ok("Wrote 0 of 6 files from Localization/strings.json, and removed 0.".to_owned())
        );

        // One file meddled with: it is the stale one, and the one written.
        let android = root.join("Android/app/src/main/res/values/strings.xml");
        let _ = fs::write(&android, "by hand");
        let problem = run(arguments(&root, true)).err().unwrap_or_default();
        assert!(
            problem.ends_with(":\n  Android/app/src/main/res/values/strings.xml"),
            "{problem}"
        );
        assert_eq!(
            run(arguments(&root, false)),
            Ok("Wrote 1 of 6 files from Localization/strings.json, and removed 0.".to_owned())
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_language_the_catalogue_no_longer_has_leaves_no_file_behind() {
        let root = repository("left", TWO);
        assert!(run(arguments(&root, false)).is_ok());
        // A file of somebody's own beside the tool's: it is not the tool's to take away.
        let own = root.join("Android/app/src/main/res/values-de/strings.xml");
        let _ = fs::create_dir_all(own.parent().unwrap_or(&root));
        let _ = fs::write(
            &own,
            "<?xml version=\"1.0\"?>\n<!-- by hand -->\n<resources/>\n",
        );

        let _ = fs::write(root.join(CATALOGUE), ENGLISH_ONLY);
        let problem = run(arguments(&root, true)).err().unwrap_or_default();
        assert!(
            problem.contains("Android/app/src/main/res/values-fr/strings.xml (left over)"),
            "{problem}"
        );
        assert!(
            problem.contains("Windows/Strings/fr/Resources.resw (left over)"),
            "{problem}"
        );
        assert!(!problem.contains("values-de"), "{problem}");

        let report = run(arguments(&root, false)).unwrap_or_default();
        assert!(report.ends_with("and removed 2."), "{report}");
        assert!(
            !root
                .join("Android/app/src/main/res/values-fr/strings.xml")
                .exists()
        );
        assert!(!root.join("Windows/Strings/fr/Resources.resw").exists());
        assert!(own.exists(), "a file that is not the tool's stays");
        assert!(run(arguments(&root, true)).is_ok());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_catalogue_that_is_wrong_writes_nothing_and_says_which_string() {
        let root = repository(
            "wrong",
            r#"{ "comment": "c", "strings": { "tabs.new": { "en": "%@ tabs", "fr": "onglets", "note": "n" } } }"#,
        );
        let problem = run(arguments(&root, false)).err().unwrap_or_default();
        assert_eq!(
            problem,
            "Localization/strings.json: 'tabs.new': 'fr' does not take the same arguments as its English"
        );
        assert!(!root.join("Android").exists());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn arguments_are_checked() {
        let problem = run(["--root".to_owned()].into_iter())
            .err()
            .unwrap_or_default();
        assert!(problem.starts_with("usage:"), "{problem}");
        let problem = run(["--rot".to_owned(), ".".to_owned()].into_iter())
            .err()
            .unwrap_or_default();
        assert!(problem.contains("unknown argument '--rot'"), "{problem}");
        let problem = run(std::iter::empty()).err().unwrap_or_default();
        assert!(problem.starts_with("usage:"), "{problem}");
    }
}
