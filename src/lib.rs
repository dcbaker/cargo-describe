// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use std::io;

#[cfg(feature = "compiler_checks")]
mod compiler;
mod manifest;
mod rustc;

pub fn evaluate() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    #[cfg(feature = "compiler_checks")]
    if cfg!(feature = "compiler_checks") {
        compiler::check(&mut io::stdout());
    }
}
