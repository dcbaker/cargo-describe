// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use crate::rustc::{get_rustc, VersionData};
use cfg_expr::targets::get_builtin_target_by_triple;
use cfg_expr::{Expression, Predicate};
use semver::VersionReq;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::vec::Vec;

fn check_compiles(src: &str) -> bool {
    use std::io::Write;

    let out_dir = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let rustc = get_rustc();

    let mut cmd = match env::var("RUSTC_WRAPPER") {
        Ok(wrapper) => {
            let mut c = std::process::Command::new(wrapper);
            c.arg(rustc);
            c
        }
        Err(_) => std::process::Command::new(rustc),
    };

    cmd.arg("--crate-type=rlib")
        .arg("--emit=metadata") // use emit and crate-type to minimize the work done
        .arg("--target")
        .arg(target)
        .arg("--out-dir")
        .arg(out_dir); // Ensure that we know where this is going

    if let Ok(flags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        if !flags.is_empty() {
            for f in flags.split('\x1f') {
                cmd.arg(f);
            }
        }
    }

    let mut child = cmd
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    writeln!(child.stdin.take().unwrap(), "{}", src).unwrap();

    child.wait().unwrap().success()
}

#[derive(Clone, Deserialize, Debug, PartialEq, Default)]
pub struct Condition {
    #[serde(default)]
    pub version: Option<VersionReq>,

    #[serde(default)]
    pub nightly_version: Option<VersionReq>,

    #[serde(default)]
    pub can_compile: Option<String>,
}

impl Condition {
    #[cfg(test)]
    fn new() -> Self {
        Condition {
            ..Default::default()
        }
    }

    #[cfg(test)]
    fn set_version(mut self, v: Option<VersionReq>) -> Self {
        self.version = v;
        self
    }

    #[cfg(test)]
    fn set_nightly(mut self, v: Option<VersionReq>) -> Self {
        self.nightly_version = v;
        self
    }

    #[cfg(test)]
    fn set_can_compile(mut self, v: Option<String>) -> Self {
        self.can_compile = v;
        self
    }

    fn check_compiles(&self) -> bool {
        match &self.can_compile {
            Some(s) => check_compiles(s),
            None => true, // If there is no compile check return True
        }
    }

    fn check_version(&self, rustc: &VersionData) -> bool {
        if rustc.nightly {
            if let Some(v) = &self.nightly_version {
                return v.matches(&rustc.version);
            };
        }

        match &self.version {
            Some(v) => v.matches(&rustc.version),
            None => true,
        }
    }

    pub fn check(&self, rustc: &VersionData) -> bool {
        self.check_compiles() && self.check_version(rustc)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Constraint {
    Cfg(HashMap<String, Condition>),
    Condition(Condition),
}

pub fn parse_compiler_checks(
    compiler_checks: &HashMap<String, Constraint>,
) -> Vec<(String, Condition)> {
    let target = env::var("TARGET").expect("TARGET environment variable is unset");
    let mut ret: Vec<(String, Condition)> = vec![];

    for (k, v) in compiler_checks.iter() {
        match v {
            Constraint::Condition(con) => ret.push((k.clone(), con.clone())),
            Constraint::Cfg(c) => {
                let cfg = Expression::parse(k)
                    .expect(format!("Invalid cfg expression: {}", k.to_string()).as_str());
                let res = match get_builtin_target_by_triple(&target) {
                    Some(tinfo) => cfg.eval(|p| match p {
                        Predicate::Target(tp) => tp.matches(tinfo),
                        _ => panic!("Invalid CFG expression: {}", &target),
                    }),
                    None => panic!("Invalid CFG expression: {}", &target),
                };
                if res {
                    for (ck, cv) in c.iter() {
                        ret.push((ck.clone(), cv.clone()));
                    }
                }
            }
        };
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{parse, Manifest};
    use semver::Version;
    use temp_env;

    #[test]
    fn test_basic_read() {
        let mani: Manifest = toml::from_str(
            r#"
            [package.metadata.toml_describe.compiler_checks]
            foo = { version = "1.0.0" }
        "#,
        )
        .unwrap();

        let v = &mani.package.metadata.toml_describe.compiler_checks["foo"];
        let ver = match v {
            Constraint::Condition(ver) => ver.version.as_ref().unwrap(),
            _ => panic!("Did not get a Version"),
        };
        assert!(ver.matches(&Version::new(1, 0, 0)));
    }

    #[test]
    fn test_multiple_constraints() {
        let mani: Manifest = toml::from_str(
            r#"
            [package.metadata.toml_describe.compiler_checks]
            foo = { version = ">1.0.0, <2.0.0" }
        "#,
        )
        .unwrap();

        let v = &mani.package.metadata.toml_describe.compiler_checks["foo"];
        let ver = match v {
            Constraint::Condition(ver) => ver.version.as_ref().unwrap(),
            _ => panic!("Did not get a Version"),
        };
        assert!(ver.matches(&Version::new(1, 3, 0)));
        assert!(!ver.matches(&Version::new(0, 3, 0)));
        assert!(!ver.matches(&Version::new(2, 3, 0)));
    }

    #[test]
    fn test_parses_nightly() {
        let mani: Manifest = toml::from_str(
            r#"
            [package.metadata.toml_describe.compiler_checks]
            foo = { nightly_version = ">1.0.0, <2.0.0" }
        "#,
        )
        .unwrap();

        let v = &mani.package.metadata.toml_describe.compiler_checks["foo"];
        let ver = match v {
            Constraint::Condition(ver) => ver.nightly_version.as_ref().unwrap(),
            _ => panic!("Did not get a nightly_version"),
        };
        assert!(ver.matches(&Version::new(1, 3, 0)));
        assert!(!ver.matches(&Version::new(0, 3, 0)));
        assert!(!ver.matches(&Version::new(2, 3, 0)));
    }

    #[test]
    fn test_parses_can_compile() {
        let mani: Manifest = toml::from_str(
            r#"
            [package.metadata.toml_describe.compiler_checks]
            foo = { can_compile = "fn foo() -> { false }" }
        "#,
        )
        .unwrap();

        let v = &mani.package.metadata.toml_describe.compiler_checks["foo"];
        let check = match v {
            Constraint::Condition(ver) => ver.can_compile.as_ref().unwrap(),
            _ => panic!("Did not get a can_compile value"),
        };
        assert_eq!(check, "fn foo() -> { false }");
    }

    #[test]
    fn test_cfg() {
        let mani: Manifest = toml::from_str(
            r#"
            [package.metadata.toml_describe.compiler_checks.'cfg(target_os = "linux")']
            foo = { version = "~1.0.0" }
        "#,
        )
        .unwrap();

        let v = &mani.package.metadata.toml_describe.compiler_checks["cfg(target_os = \"linux\")"];
        let cfg = match v {
            Constraint::Cfg(cfg) => cfg,
            _ => panic!("Got a Condition instead of a CFG"),
        };

        assert!(cfg.contains_key("foo"));
        assert!(cfg["foo"]
            .version
            .as_ref()
            .unwrap()
            .matches(&Version::new(1, 0, 9)));
    }

    #[test]
    fn test_parse_cfg() {
        temp_env::with_var("TARGET", Some("x86_64-unknown-linux-gnu"), || {
            let vals = parse(
                r#"
                [package.metadata.toml_describe.compiler_checks]
                foo = { version = "1.0.0" }
                [package.metadata.toml_describe.compiler_checks.'cfg(target_os = "linux")']
                bar = { version = "1.2.0" }
                [package.metadata.toml_describe.compiler_checks.'cfg(target_os = "windows")']
                bad = { version = "1.2.0" }
            "#,
            );

            assert_eq!(vals.compiler.len(), 2);
        });
    }

    #[test]
    #[should_panic(expected = "Invalid CFG expression: x86_65-unknown-freax-gna")]
    fn test_invalid_cfg() {
        temp_env::with_var("TARGET", Some("x86_65-unknown-freax-gna"), || {
            parse(
                r#"
                [package.metadata.toml_describe.compiler_checks.'cfg(target_os = "linux")']
                bar = { version = "1.2.0" }
            "#,
            );
        });
    }

    fn version_data(v: &str) -> VersionData {
        VersionData::new(Version::parse(&v).unwrap(), false)
    }

    #[test]
    fn test_condition_check_version_match() {
        let c = Condition::new().set_version(VersionReq::parse(">= 1").ok());
        assert!(c.check(&version_data("1.0.0")));
    }

    #[test]
    fn test_condition_check_version_not_match() {
        let c = Condition::new().set_version(VersionReq::parse("< 1").ok());
        assert!(!c.check(&version_data("1.0.0")));
    }

    #[test]
    fn test_condition_compiles() {
        temp_env::with_vars([
            ("TARGET", Some("x86_64-unknown-linux-gnu")),
            ("OUT_DIR", Some("test-output")),
        ], || {
            let c =
                Condition::new().set_can_compile(Some("fn foo() -> bool { false }".to_string()));
            assert!(c.check(&version_data("1.0.0")));
        });
    }

    #[test]
    fn test_condition_compiles_fails() {
        temp_env::with_vars([
            ("TARGET", Some("x86_64-unknown-linux-gnu")),
            ("OUT_DIR", Some("test-output")),
        ], || {
            let c =
                Condition::new().set_can_compile(Some("fn foo() -> boolean { false }".to_string()));
            assert!(!c.check(&version_data("1.0.0")));
        });
    }
}
