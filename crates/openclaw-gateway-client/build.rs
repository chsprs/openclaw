//! Generate the Gateway protocol constants.
//!
//! The canonical source is `packages/gateway-protocol/src/version.ts` in the full
//! upstream checkout. A `crates/`-only tree (for example a partial clone or a
//! standalone export) does not contain that file, and the build must still work
//! there: the values are part of the wire contract, not a build-time detail.
//!
//! Order of resolution:
//!   1. `packages/gateway-protocol/src/version.ts` when present — always wins, so a
//!      full checkout stays authoritative and a version bump cannot silently drift.
//!   2. `protocol-version.tsv` beside this crate — the recorded snapshot, checked
//!      into the tree so a `crates/`-only export builds reproducibly.
//!
//! If neither exists the build fails loudly rather than guessing.

use std::{env, fs, path::PathBuf};

const CONSTANTS: [&str; 4] = [
    "PROTOCOL_VERSION",
    "MIN_CLIENT_PROTOCOL_VERSION",
    "MIN_NODE_PROTOCOL_VERSION",
    "MIN_PROBE_PROTOCOL_VERSION",
];

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let canonical = manifest_dir.join("../../packages/gateway-protocol/src/version.ts");
    let snapshot = manifest_dir.join("protocol-version.tsv");
    println!("cargo:rerun-if-changed={}", canonical.display());
    println!("cargo:rerun-if-changed={}", snapshot.display());

    let mut generated = String::from(
        "// Generated from packages/gateway-protocol/src/version.ts.\n\
         // Do not edit by hand; regenerate by building this crate.\n",
    );

    match fs::read_to_string(&canonical) {
        Ok(source) => {
            for name in CONSTANTS {
                let prefix = format!("export const {name} = ");
                let value: u32 = source
                    .lines()
                    .find_map(|line| line.strip_prefix(&prefix))
                    .and_then(|value| value.strip_suffix(" as const;"))
                    .unwrap_or_else(|| panic!("missing canonical protocol constant {name}"))
                    .parse()
                    .unwrap_or_else(|_| panic!("invalid canonical protocol constant {name}"));
                generated.push_str(&format!("pub const {name}: u32 = {value};\n"));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let recorded = fs::read_to_string(&snapshot).unwrap_or_else(|snapshot_error| {
                panic!(
                    "cannot determine the Gateway protocol versions: {} is missing ({error}) \
                     and the recorded snapshot {} is unreadable ({snapshot_error}). \
                     Restore the full checkout or provide protocol-version.tsv.",
                    canonical.display(),
                    snapshot.display()
                )
            });
            for name in CONSTANTS {
                let value = recorded
                    .lines()
                    .filter(|line| !line.trim_start().starts_with('#'))
                    .filter_map(|line| line.split_once('\t'))
                    .find(|(key, _)| key.trim() == name)
                    .map(|(_, value)| value.trim())
                    .unwrap_or_else(|| {
                        panic!("protocol-version.tsv is missing the {name} row")
                    })
                    .parse::<u32>()
                    .unwrap_or_else(|_| panic!("invalid recorded value for {name}"));
                generated.push_str(&format!("pub const {name}: u32 = {value};\n"));
            }
        }
        Err(error) => panic!("cannot read {}: {error}", canonical.display()),
    }

    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    fs::write(output.join("protocol_versions.rs"), generated).expect("Rust protocol versions");
}
