// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use crate::manifest;

use semver::Version;
use std::{env, fs, io, path, process};


fn get_rustc() -> String {
    env::var("RUSTC")
        .unwrap_or(env::var("CARGO_BUILD_RUSTC").unwrap_or("rustc".to_string()))
        .to_string()
}

fn get_rustc_version() -> manifest::VersionData {
    let rustc = get_rustc();
    let out = process::Command::new(rustc)
        .arg("--version")
        .arg("--verbose")
        .output()
        .expect("Could not run rustc for version");

    let raw = String::from_utf8(out.stdout).expect("Did not get valid output from rustc");
    let raw_version = raw
        .split("\n")
        .nth(5)
        .expect("Got unexpected rustc version output")
        .split(" ")
        .nth(1)
        .expect("Not in 'release: <version>' form");
    let mut pieces = raw_version.split("-");
    let version = Version::parse(pieces.next().unwrap()).expect("Invalid Rustc version");
    let nightly: bool = pieces.next().map(|x| x == "nightly").unwrap_or(false);

    manifest::VersionData::new(version, nightly)
}

pub fn check<W: io::Write>(writer: &mut W) {
    let rustc = get_rustc_version();

    let root = env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest environment variable unset");
    let p: path::PathBuf = [root, "Cargo.toml".to_string()].iter().collect();
    let contents = fs::read_to_string(p).expect("Could not read Cargo.toml");
    let checks = manifest::parse(&contents);

    writeln!(writer, "cargo:rerun-if-changed=Cargo.toml").unwrap();

    checks.iter().for_each(|(name, condition)| {
        if condition.check(&rustc) {
            writeln!(writer, "cargo:rustc-cfg={}", name).unwrap();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[test]
    fn test_emits() {
        temp_env::with_vars(
            [
                ("CARGO_MANIFEST_DIR", Some("test_cases/basic")),
                ("CARGO_BUILD_RUSTC", Some("rustc")),
                ("TARGET", Some("x86_64-unknown-linux-gnu")),
            ],
            || {
                let mut out = Vec::new();
                check(&mut out);
                assert_eq!(
                    out,
                    b"cargo:rerun-if-changed=Cargo.toml\ncargo:rustc-cfg=foo\n"
                );
            },
        )
    }

    #[test]
    fn test_not_emits() {
        temp_env::with_vars(
            [
                ("CARGO_MANIFEST_DIR", Some("test_cases/not")),
                ("CARGO_BUILD_RUSTC", Some("rustc")),
                ("TARGET", Some("x86_64-unknown-linux-gnu")),
            ],
            || {
                let mut out = Vec::new();
                check(&mut out);
                assert_eq!(out, b"cargo:rerun-if-changed=Cargo.toml\n");
            },
        )
    }
}
