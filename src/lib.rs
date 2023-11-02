// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use std::{env, fs, io, path};

#[cfg(feature = "cfg_checks")]
mod cfg;
#[cfg(feature = "compiler_checks")]
mod compiler;
mod manifest;
mod rustc;

pub fn evaluate() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let root = env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest environment variable unset");
    let p: path::PathBuf = [root, "Cargo.toml".to_string()].iter().collect();
    let contents = fs::read_to_string(p).expect("Could not read Cargo.toml");
    let checks = manifest::parse(&contents);

    #[cfg(feature = "compiler_checks")]
    {
        compiler::check(&mut io::stdout(), &checks);
    }
    #[cfg(feature = "cfg_checks")]
    {
        cfg::check(&mut io::stdout(), &checks);
    }
}
