// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use crate::manifest::Checks;
use crate::rustc::RUSTC;

use std::io;

pub fn check<W: io::Write>(writer: &mut W, checks: &Checks) {
    checks.cfgs.iter().for_each(|c| {
        match RUSTC.version.minor >= 75 {
            true => writeln!(writer, "cargo:rustc-check-cfg=cfg({})", c),
            false => writeln!(writer, "cargo:rustc-check-cfg=values({})", c),
        }.unwrap();
    });
}
