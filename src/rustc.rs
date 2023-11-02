// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use crate::manifest;
use lazy_static::lazy_static;
use semver::Version;
use std::{env, process};

fn get_rustc() -> String {
    env::var("RUSTC")
        .unwrap_or(env::var("CARGO_BUILD_RUSTC").unwrap_or("rustc".to_string()))
        .to_string()
}

lazy_static! {
    pub static ref RUSTC: manifest::VersionData = {
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
    };
}
