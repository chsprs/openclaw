//! The gateway client must build from a `crates/`-only tree.
//!
//! Real failure this guards: `build.rs` read
//! `../../packages/gateway-protocol/src/version.ts` unconditionally. A checkout
//! containing only `crates/` (the `crates-only` branch) therefore failed with
//! `canonical Gateway protocol versions: No such file or directory` before any
//! crate compiled. The recorded snapshot keeps that path building while a full
//! checkout still reads the canonical file.

const BUILD_RS: &str = include_str!("../build.rs");
const SNAPSHOT: &str = include_str!("../protocol-version.tsv");

/// Parse the recorded snapshot the same way build.rs does.
fn recorded(name: &str) -> u32 {
    SNAPSHOT
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .filter_map(|line| line.split_once('\t'))
        .find(|(key, _)| key.trim() == name)
        .map(|(_, value)| value.trim().parse().expect("recorded value is a u32"))
        .unwrap_or_else(|| panic!("snapshot missing {name}"))
}

#[test]
fn snapshot_records_every_protocol_constant() {
    for name in [
        "PROTOCOL_VERSION",
        "MIN_CLIENT_PROTOCOL_VERSION",
        "MIN_NODE_PROTOCOL_VERSION",
        "MIN_PROBE_PROTOCOL_VERSION",
    ] {
        let value = recorded(name);
        assert!(value >= 1, "{name} must be a positive protocol version");
    }
}

#[test]
fn snapshot_matches_the_compiled_constants() {
    // The constants the rest of the workspace uses come from build.rs output; if
    // the snapshot drifted, this test fails instead of shipping a mismatch.
    assert_eq!(recorded("PROTOCOL_VERSION"), openclaw_gateway_client::protocol::PROTOCOL_VERSION);
    assert_eq!(
        recorded("MIN_CLIENT_PROTOCOL_VERSION"),
        openclaw_gateway_client::protocol::MIN_CLIENT_PROTOCOL_VERSION
    );
    assert_eq!(
        recorded("MIN_NODE_PROTOCOL_VERSION"),
        openclaw_gateway_client::protocol::MIN_NODE_PROTOCOL_VERSION
    );
    assert_eq!(
        recorded("MIN_PROBE_PROTOCOL_VERSION"),
        openclaw_gateway_client::protocol::MIN_PROBE_PROTOCOL_VERSION
    );
}

#[test]
fn build_script_falls_back_instead_of_requiring_the_full_checkout() {
    assert!(
        BUILD_RS.contains("protocol-version.tsv"),
        "build.rs must have a recorded fallback for crates-only trees"
    );
    assert!(
        BUILD_RS.contains("NotFound"),
        "build.rs must distinguish a missing canonical file from other IO errors"
    );
    assert!(
        !BUILD_RS.contains(".expect(\"canonical Gateway protocol versions\")"),
        "the unconditional read that broke crates-only builds must be gone"
    );
}

#[test]
fn workspace_protocol_constants_agree() {
    // The gateway protocol module hardcodes the same contract; read it as text so
    // this crate does not need a dependency on openclaw-protocol just for a test.
    const PROTOCOL_LIB: &str = include_str!("../../openclaw-protocol/src/lib.rs");
    for name in [
        "PROTOCOL_VERSION",
        "MIN_CLIENT_PROTOCOL_VERSION",
        "MIN_NODE_PROTOCOL_VERSION",
    ] {
        let prefix = format!("pub const {name}: u32 = ");
        let value: u32 = PROTOCOL_LIB
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .map(|rest| rest.trim_end_matches(';').trim())
            .unwrap_or_else(|| panic!("openclaw-protocol does not define {name}"))
            .parse()
            .unwrap_or_else(|_| panic!("openclaw-protocol {name} is not a u32"));
        assert_eq!(
            value,
            recorded(name),
            "{name} drifted between openclaw-protocol and the recorded snapshot"
        );
    }
}
