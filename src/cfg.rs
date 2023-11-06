// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use crate::manifest::Checks;

use std::io;

pub fn check<W: io::Write>(writer: &mut W, checks: &Checks) {
    for (name, allowed) in checks.cfgs.iter() {
        match allowed {
            None => writeln!(writer, "cargo:rustc-check-cfg=cfg({})", name).unwrap(),
            Some(a) => {
                let mut formatted = Vec::<String>::new();
                for fv in a {
                    formatted.push(format!("\"{fv}\""));
                }
                let fv = formatted.join(", ");
                writeln!(
                    writer,
                    "cargo:rustc-check-cfg=cfg({}, values({}))",
                    name, fv
                )
                .unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::parse;
    use std::{fs, path};

    fn get_checks(root: &str) -> Checks {
        let p: path::PathBuf = [root, "Cargo.toml"].iter().collect();
        let contents = fs::read_to_string(p).expect("Could not read Cargo.toml");
        parse(&contents)
    }

    #[test]
    fn test_basic() {
        let expected = vec![
            "cargo:rustc-check-cfg=cfg(foo)",
            "cargo:rustc-check-cfg=cfg(bar, values(\"a\", \"b\"))",
        ];

        let mut out = Vec::new();
        check(&mut out, &get_checks("test_cases/basic"));
        let val = std::str::from_utf8(&out).unwrap();
        let found: Vec<&str> = val.strip_suffix("\n").unwrap().split("\n").collect();
        assert_eq!(
            expected.len(),
            found.len(),
            "Got wrong number of elements, expected {}, got {}",
            expected.len(),
            found.len()
        );
        for v in found {
            assert!(expected.contains(&v), "'{v}' not found in {expected:?}");
        }
    }
}
