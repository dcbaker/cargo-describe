// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use std::io;

mod rustc;
mod compiler;
mod manifest;

pub fn evaluate() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    compiler::check(&mut io::stdout());
}