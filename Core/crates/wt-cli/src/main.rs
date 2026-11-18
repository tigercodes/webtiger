//! `webtiger`: the command line.
//!
//! W0 has one command, `webtiger import arc` (002). A command line is a
//! shell like any other: the core's crates give it codes and counts, and the
//! words are here. It prints counts and the paths it was given, and never an
//! address or a title from the data it reads.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use wt_import::arc::{Options, default_folder, import};
use wt_import::{Portable, Summary};

/// The file `import arc` writes when `--out` is not given.
const DEFAULT_OUT: &str = "webtiger-arc-import.json";

#[derive(Parser)]
#[command(name = "webtiger", version, about = "WebTiger's command line")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read another browser's data into a file WebTiger can open
    Import {
        #[command(subcommand)]
        source: Source,
    },
}

#[derive(Subcommand)]
enum Source {
    /// Arc: Spaces, folders, pinned tabs, Favorites, link rules, legacy Boosts
    Arc(ArcArgs),
}

#[derive(Args)]
struct ArcArgs {
    /// Arc's data folder, or a copy of it [default: where Arc keeps it on this machine]
    #[arg(long, value_name = "FOLDER")]
    from: Option<PathBuf>,
    /// The file to write [default: webtiger-arc-import.json]
    #[arg(long, value_name = "FILE")]
    out: Option<PathBuf>,
    /// Replace the output file if it exists
    #[arg(long)]
    force: bool,
    /// Leave out the unpinned tabs under "Today"
    #[arg(long)]
    no_today: bool,
    /// Leave out archived tabs
    #[arg(long)]
    no_archive: bool,
    /// Leave out legacy Boosts
    #[arg(long)]
    no_boosts: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let outcome = match cli.command {
        Command::Import {
            source: Source::Arc(args),
        } => import_arc(&args),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            // Not eprintln!: it panics when stderr is a closed pipe.
            let _ = writeln!(
                std::io::stderr(),
                "error: {} ({})",
                failure.words,
                failure.code
            );
            ExitCode::FAILURE
        }
    }
}

/// Why a command failed: words for a person and the code behind them.
struct Failure {
    words: String,
    code: &'static str,
}

fn import_arc(args: &ArcArgs) -> Result<(), Failure> {
    let folder = match &args.from {
        Some(folder) => folder.clone(),
        None => default_folder().ok_or_else(|| Failure {
            words: "Arc's data folder was not found on this machine; name it with --from"
                .to_owned(),
            code: "arc.folder_missing",
        })?,
    };
    let out = args
        .out
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUT));
    // Checked before reading, so a long import is not made only to be refused.
    if out.exists() && !args.force {
        return Err(Failure {
            words: format!("{} exists; pass --force to replace it", out.display()),
            code: "import.output_exists",
        });
    }
    let options = Options {
        today: !args.no_today,
        archive: !args.no_archive,
        boosts: !args.no_boosts,
    };
    let file = import(&folder, options).map_err(|error| Failure {
        words: arc_error_words(error.code()).to_owned(),
        code: error.code(),
    })?;
    write_private(&out, &file, args.force).map_err(|()| Failure {
        words: format!("{} could not be written", out.display()),
        code: "import.output_unwritable",
    })?;
    report(&folder, &out, &file.summary());
    Ok(())
}

/// Writes the portable file. It holds a person's addresses and titles, so on
/// Unix it is readable by its owner alone.
fn write_private(path: &Path, file: &Portable, replace: bool) -> Result<(), ()> {
    let mut open = std::fs::OpenOptions::new();
    open.write(true);
    if replace {
        open.create(true).truncate(true);
    } else {
        open.create_new(true);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        open.mode(0o600);
    }
    let mut output = open.open(path).map_err(|_| ())?;
    output
        .write_all(file.to_json().as_bytes())
        .map_err(|_| ())?;
    output.sync_all().map_err(|_| ())
}

fn report(folder: &Path, out: &Path, summary: &Summary) {
    let mut lines = vec![
        format!("Read Arc's data in {}", folder.display()),
        format!(
            "  {}, with {}",
            counted(summary.spaces, "Space", "Spaces"),
            counted(summary.login_sets, "set of logins", "sets of logins")
        ),
        format!(
            "  {}, {}, {}, {}",
            counted(summary.groups, "folder", "folders"),
            counted(summary.sites, "pinned tab", "pinned tabs"),
            counted(summary.favourites, "Favorite", "Favorites"),
            counted(summary.tabs, "tab from Today", "tabs from Today")
        ),
        format!(
            "  {}, {}, {}",
            counted(summary.link_rules, "link rule", "link rules"),
            counted(summary.tweaks, "legacy Boost", "legacy Boosts"),
            counted(summary.archived, "archived tab", "archived tabs")
        ),
    ];
    if !summary.notes.is_empty() {
        lines.push("Left out or changed:".to_owned());
        for (code, count) in &summary.notes {
            lines.push(format!("  {count} × {}", note_words(code)));
        }
    }
    lines.push(
        "Not read: Boosts made in Arc's visual editor, and the contents of Easels and Notes \
         (docs/features/002-arc-import.md)."
            .to_owned(),
    );
    lines.push(format!(
        "Wrote {}. It holds your addresses and titles: keep it somewhere private.",
        out.display()
    ));
    let mut stdout = std::io::stdout().lock();
    for line in lines {
        // Not println!: it panics when stdout is a closed pipe.
        let _ = writeln!(stdout, "{line}");
    }
}

fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

fn arc_error_words(code: &str) -> &'static str {
    match code {
        "arc.folder_missing" => "there is no such folder",
        "arc.sidebar_missing" => {
            "the folder has no StorableSidebar.json: it is not Arc's data folder"
        }
        "arc.sidebar_too_large" => "StorableSidebar.json is larger than any sidebar file should be",
        "arc.sidebar_unreadable" => "StorableSidebar.json could not be read",
        "arc.sidebar_malformed" => {
            "StorableSidebar.json is not in a form this importer understands"
        }
        _ => "the import failed",
    }
}

fn note_words(code: &str) -> &str {
    match code {
        "arc.address_skipped" => {
            "address WebTiger cannot open (Arc's own pages, local files, scripts)"
        }
        "arc.easel_skipped" => "Easel (its content is on Arc's servers)",
        "arc.note_skipped" => "Note (its content is not in the sidebar file)",
        "arc.unknown_item_skipped" => "sidebar row of a kind this importer does not know",
        "arc.split_flattened" => "split view kept as its tabs, side by side no longer",
        "arc.favourite_folder_flattened" => "Favorites row with a folder, opened out into its tabs",
        "arc.little_arc_skipped" => "Little Arc window",
        "arc.missing_item" => "row that names an item the file does not have",
        "arc.repeated_item_skipped" => "row listed more than once",
        "arc.too_deep" => "folder nested deeper than 32 levels",
        "arc.rule_unsupported" => "link rule with a condition WebTiger has no match for",
        "arc.rule_space_missing" => "link rule for a Space that no longer exists",
        "arc.link_rules_unreadable" => "link rules file that could not be read",
        "arc.archive_unreadable" => "archive file that could not be read",
        "arc.profiles_unreadable" => "profile list that could not be read",
        "arc.boost_unreadable" => "Boost that could not be read",
        "arc.boost_empty" => "Boost with nothing in it",
        "arc.boost_file_too_large" => "Boost file over 1 MB",
        other => other,
    }
}
