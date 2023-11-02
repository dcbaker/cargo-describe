// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use std::io;

mod rustc;
mod compiler;
mod manifest;

pub fn evaluate() {
    compiler::check(&mut io::stdout());
}