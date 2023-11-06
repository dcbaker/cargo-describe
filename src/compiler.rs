// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use crate::manifest::Checks;
use crate::rustc::RUSTC;

use std::io;

pub fn check<W: io::Write>(writer: &mut W, checks: &Checks) {
    for (name, condition) in checks.compiler.iter() {
        if condition.check(&RUSTC) {
            writeln!(writer, "cargo:rustc-cfg={}", name).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::parse;
    use std::{fs, path};
    use temp_env;

    fn get_checks(root: &str) -> Checks {
        let p: path::PathBuf = [root, "Cargo.toml"].iter().collect();
        let contents = fs::read_to_string(p).expect("Could not read Cargo.toml");
        parse(&contents)
    }

    #[test]
    fn test_emits() {
        temp_env::with_var("TARGET", Some("x86_64-unknown-linux-gnu"), || {
            let mut out = Vec::new();
            check(&mut out, &get_checks("test_cases/basic"));
            assert_eq!(out, b"cargo:rustc-cfg=foo\n");
        })
    }

    #[test]
    fn test_not_emits() {
        temp_env::with_var("TARGET", Some("x86_64-unknown-linux-gnu"), || {
            let mut out = Vec::new();
            check(&mut out, &get_checks("test_cases/not"));
            assert_eq!(out.len(), 0);
        })
    }
}
