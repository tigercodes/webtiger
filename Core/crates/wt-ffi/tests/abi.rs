//! `include/webtiger_core.h` is written by hand. These tests keep it in step
//! with the Rust exports: every function is declared, every constant has the
//! value Rust uses, and the fast path's structures have C's layout.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test's helpers fail the test the way its assertions do"
)]

use std::collections::BTreeMap;

use webtiger_core::{
    MAX_CORES, MAX_MESSAGE_LEN, Status, WT_NAV_ALLOW, WT_NAV_CANCEL, WT_NAV_EXTERNAL,
    WT_NAV_MAIN_FRAME, WT_NAV_USER_GESTURE, WT_VERDICT_ALLOW, WT_VERDICT_BLOCK, WtNavVerdict,
    WtNavigation, WtRequest, WtVerdict, wt_core_version,
};
use wt_proto::{CONTRACT_MAJOR, CONTRACT_MINOR};

const HEADER: &str = include_str!("../include/webtiger_core.h");

/// The header's `#define NAME value` lines, values without a `u` suffix.
fn defines() -> BTreeMap<String, u64> {
    HEADER
        .lines()
        .filter_map(|line| {
            let mut words = line.strip_prefix("#define ")?.split_whitespace();
            let name = words.next()?;
            let value = words.next()?.trim_end_matches('u').parse().ok()?;
            Some((name.to_owned(), value))
        })
        .collect()
}

#[test]
fn the_version_is_the_contracts() {
    assert_eq!(wt_core_version(), (CONTRACT_MAJOR << 16) | CONTRACT_MINOR);
    let defines = defines();
    assert_eq!(defines["WT_CONTRACT_MAJOR"], u64::from(CONTRACT_MAJOR));
    assert_eq!(defines["WT_CONTRACT_MINOR"], u64::from(CONTRACT_MINOR));
}

#[test]
fn every_export_is_declared() {
    for declaration in [
        "uint32_t wt_core_version(void);",
        "uint64_t wt_core_start(const uint8_t *config, size_t len, wt_sink sink, void *context);",
        "int32_t wt_core_send(uint64_t core, const uint8_t *envelope, size_t len);",
        "int32_t wt_core_stop(uint64_t core);",
        "int32_t wt_guard_check(uint64_t core, const wt_request *request, wt_verdict *out);",
        "int32_t wt_nav_check(uint64_t core, const wt_navigation *navigation, wt_nav_verdict *out);",
        "typedef void (*wt_sink)(void *context, const uint8_t *envelope, size_t len);",
    ] {
        assert!(
            HEADER.contains(declaration),
            "the header lacks: {declaration}"
        );
    }
}

#[test]
fn every_constant_has_rusts_value() {
    let defines = defines();
    let expected: [(&str, u64); 17] = [
        ("WT_OK", Status::Ok as u64),
        ("WT_UNKNOWN_CORE", Status::UnknownCore as u64),
        ("WT_NULL_ARGUMENT", Status::NullArgument as u64),
        ("WT_TOO_LARGE", Status::TooLarge as u64),
        ("WT_MALFORMED", Status::Malformed as u64),
        ("WT_WRONG_THREAD", Status::WrongThread as u64),
        ("WT_PANIC", Status::Panic as u64),
        ("WT_BAD_ARGUMENT", Status::BadArgument as u64),
        ("WT_MAX_MESSAGE_LEN", MAX_MESSAGE_LEN as u64),
        ("WT_MAX_CORES", MAX_CORES as u64),
        ("WT_VERDICT_ALLOW", u64::from(WT_VERDICT_ALLOW)),
        ("WT_VERDICT_BLOCK", u64::from(WT_VERDICT_BLOCK)),
        ("WT_NAV_MAIN_FRAME", u64::from(WT_NAV_MAIN_FRAME)),
        ("WT_NAV_USER_GESTURE", u64::from(WT_NAV_USER_GESTURE)),
        ("WT_NAV_ALLOW", u64::from(WT_NAV_ALLOW)),
        ("WT_NAV_CANCEL", u64::from(WT_NAV_CANCEL)),
        ("WT_NAV_EXTERNAL", u64::from(WT_NAV_EXTERNAL)),
    ];
    for (name, value) in expected {
        assert_eq!(defines.get(name), Some(&value), "{name}");
    }
}

#[test]
fn every_status_the_header_names_is_one_rust_returns() {
    // A status added to the header without a Rust variant, or the reverse,
    // shows here: the two lists are the same length.
    let statuses = defines()
        .into_iter()
        .filter(|(name, _)| {
            matches!(
                name.as_str(),
                "WT_OK"
                    | "WT_UNKNOWN_CORE"
                    | "WT_NULL_ARGUMENT"
                    | "WT_TOO_LARGE"
                    | "WT_MALFORMED"
                    | "WT_WRONG_THREAD"
                    | "WT_PANIC"
                    | "WT_BAD_ARGUMENT"
            )
        })
        .count();
    assert_eq!(statuses, 8);
    assert_eq!(Status::BadArgument as i32, 7, "the last status");
}

#[cfg(target_pointer_width = "64")]
#[test]
fn the_fast_paths_structures_have_cs_layout() {
    use std::mem::{align_of, offset_of, size_of};
    // wt_request: four 8-byte fields, a uint64_t, a uint32_t, padding to 8.
    assert_eq!(size_of::<WtRequest>(), 48);
    assert_eq!(align_of::<WtRequest>(), 8);
    assert_eq!(offset_of!(WtRequest, url_len), 8);
    assert_eq!(offset_of!(WtRequest, source_url), 16);
    assert_eq!(offset_of!(WtRequest, source_url_len), 24);
    assert_eq!(offset_of!(WtRequest, web_view_id), 32);
    assert_eq!(offset_of!(WtRequest, resource_type), 40);
    // wt_navigation: a pointer, a size_t, a uint64_t, a uint32_t, padding.
    assert_eq!(size_of::<WtNavigation>(), 32);
    assert_eq!(offset_of!(WtNavigation, web_view_id), 16);
    assert_eq!(offset_of!(WtNavigation, flags), 24);
    assert_eq!(size_of::<WtVerdict>(), 4);
    assert_eq!(size_of::<WtNavVerdict>(), 4);
}
