// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use crate::manifest::Checks;

use std::io;

pub fn check<W: io::Write>(writer: &mut W, checks: &Checks) {
    checks.cfgs.iter().for_each(|(name, allowed)| {
        match allowed {
            None => writeln!(writer, "cargo:rustc-check-cfg=cfg({})", name).unwrap(),
            Some(a) => {
                let mut formatted = Vec::<String>::new();
                for fv in a {
                    formatted.push(format!("\"{fv}\""));
                }
                let fv = formatted.join(", ");
                writeln!(writer, "cargo:rustc-check-cfg=cfg({}, values({}))", name, fv).unwrap();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::parse;
    use std::{env, fs, path};
    use temp_env;

    fn get_checks() -> Checks {
        let root =
            env::var("CARGO_MANIFEST_DIR").expect("Cargo manifest environment variable unset");
        let p: path::PathBuf = [root, "Cargo.toml".to_string()].iter().collect();
        let contents = fs::read_to_string(p).expect("Could not read Cargo.toml");
        parse(&contents)
    }

    #[test]
    fn test_basic() {
        temp_env::with_vars(
            [
                ("CARGO_MANIFEST_DIR", Some("test_cases/basic")),
                ("RUSTC", Some("rustc")),
                ("TARGET", Some("x86_64-unknown-linux-gnu")),
            ],
            || {
                let mut out = Vec::new();
                check(&mut out, &get_checks());
                assert_eq!(out, b"cargo:rustc-check-cfg=cfg(foo)\ncargo:rustc-check-cfg=cfg(bar, values(\"a\", \"b\"))\n");
            },
        )
    }
}
