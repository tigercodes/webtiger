//! Regenerates the Rust side of the contract, Core/proto/webtiger/core/v1,
//! into the directory given as the first argument (normally
//! Core/crates/wt-proto/src/gen). Tools/proto/generate.sh runs it with PROTOC
//! pointing at the pinned, checksum-verified protoc. It writes
//! `webtiger.core.v1.rs`: the messages, by prost-build.

use std::error::Error;
use std::path::{Path, PathBuf};

/// The contract's directory, relative to Core/proto.
const CONTRACT_DIR: &str = "webtiger/core/v1";

fn main() -> Result<(), Box<dyn Error>> {
    let core = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let proto = core.join("proto");
    let out = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| core.join("crates/wt-proto/src/gen"));
    std::fs::create_dir_all(&out)?;

    let contract = contract_files(&proto)?;
    let contract_paths: Vec<PathBuf> = contract.iter().map(|name| proto.join(name)).collect();
    prost_build::Config::new()
        .out_dir(&out)
        .compile_protos(&contract_paths, &[&proto])?;
    Ok(())
}

/// The contract's .proto files relative to Core/proto, sorted, with `/`
/// separators so the output is the same on every machine.
fn contract_files(proto: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(proto.join(CONTRACT_DIR))? {
        let name = entry?.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.ends_with(".proto") {
            files.push(format!("{CONTRACT_DIR}/{name}"));
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!("no .proto files in {}", proto.join(CONTRACT_DIR).display()).into());
    }
    Ok(files)
}
