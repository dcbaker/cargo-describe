// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

mod manifest;

use semver::Version;
use std::vec::Vec;
use std::{env, fs, io, path, process};

struct VersionData {
    version: Version,
}

fn get_rustc() -> String {
    env::var("RUSTC")
        .unwrap_or(env::var("CARGO_BUILD_RUSTC").unwrap_or("rustc".to_string()))
        .to_string()
}

fn get_rustc_version() -> VersionData {
    let rustc = get_rustc();
    let out = process::Command::new(rustc)
        .arg("--version")
        .arg("--verbose")
        .output()
        .expect("Could not run rustc for version");

    let raw = String::from_utf8(out.stdout).expect("Did not get valid output from rustc");
    let lines = raw.split("\n").collect::<Vec<&str>>();

    let raw_version = lines[5].split(" ").collect::<Vec<&str>>()[1];
    let version = Version::parse(raw_version).expect("Invalid Rustc version");

    VersionData { version: version }
}

fn check<W: io::Write>(writer: &mut W) {
    let rustc = get_rustc_version();

    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let p: path::PathBuf = [root, "Cargo.toml".to_string()].iter().collect();
    let contents = fs::read_to_string(p).unwrap();
    let checks = manifest::parse(&contents);

    checks.iter().for_each(|(name, condition)| {
        if condition.check(&rustc.version) {
            writeln!(writer, "cargo:rustc-cfg={}", name).unwrap();
        }
    });
}

pub fn evaluate() {
    check(&mut io::stdout());
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
                assert_eq!(out, b"cargo:rustc-cfg=foo\n");
            },
        )
    }

    #[test]
    fn test_emits_features() {
        temp_env::with_vars(
            [
                ("CARGO_MANIFEST_DIR", Some("test_cases/features")),
                ("CARGO_BUILD_RUSTC", Some("rustc")),
                ("CARGO_FEATURE_A_FEATURE", None),
                ("TARGET", Some("x86_64-unknown-linux-gnu")),
            ],
            || {
                let mut out = Vec::new();
                check(&mut out);
                assert_eq!(out, b"cargo:rustc-cfg=foo\n");
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
                assert_eq!(out.len(), 0);
            },
        )
    }
}
