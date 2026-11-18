//! The contract's version is written in three places that must agree: the
//! constants here, the version line in envelope.proto, and the C header.
//! (wt-ffi's ABI test checks the header.)

use prost::Message as _;
use wt_proto::v1::{Envelope, Ping, Request, envelope, request};
use wt_proto::{CONTRACT_MAJOR, CONTRACT_MINOR};

#[test]
fn envelope_proto_names_the_same_version() {
    let proto = include_str!("../../../proto/webtiger/core/v1/envelope.proto");
    let line = format!("// Contract version: {CONTRACT_MAJOR}.{CONTRACT_MINOR} ");
    assert!(
        proto.contains(&line),
        "envelope.proto's version line must read {line:?}"
    );
}

#[test]
fn prost_build_matches_the_runtime() {
    let generator = include_str!("../../../tools/proto-gen/Cargo.toml");
    let workspace = include_str!("../../../Cargo.toml");
    let pinned = |text: &str, key: &str| -> String {
        text.lines()
            .find_map(|line| line.strip_prefix(key))
            .map(|rest| rest.trim().trim_matches('"').to_owned())
            .unwrap_or_default()
    };
    let build = pinned(generator, "prost-build = ");
    let runtime = pinned(workspace, "prost = ");
    assert!(build.starts_with('='), "prost-build must be pinned exactly");
    assert_eq!(build, runtime);
}

#[test]
fn an_envelope_round_trips() {
    let sent = Envelope {
        id: 7,
        body: Some(envelope::Body::Request(Request {
            kind: Some(request::Kind::Ping(Ping {
                payload: b"hello".to_vec(),
                via_host: false,
                panic: false,
            })),
        })),
    };
    let bytes = sent.encode_to_vec();
    let received = Envelope::decode(bytes.as_slice()).unwrap();
    assert_eq!(sent, received);
}
