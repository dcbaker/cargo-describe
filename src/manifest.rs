// SPDX-License-Identifier: MIT

use crate::rustc::VersionData;
use cfg_expr::targets::get_builtin_target_by_triple;
use cfg_expr::{Expression, Predicate};
use semver::VersionReq;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::vec::Vec;

#[derive(Deserialize, Debug)]
struct Manifest {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    metadata: Metadata,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    toml_describe: TomlDescribe,
}

#[derive(Deserialize, Debug)]
struct TomlDescribe {
    #[cfg(feature = "compiler_checks")]
    compiler_checks: HashMap<String, Constraint>,
}

#[cfg(feature = "compiler_checks")]
#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct Condition {
    #[serde(default)]
    version: Option<VersionReq>,

    #[serde(default)]
    nightly_version: Option<VersionReq>,
}

impl Condition {
    #[cfg(test)]
    fn new() -> Self {
        Condition {
            version: None,
            nightly_version: None,
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

    pub fn check(&self, rustc: &VersionData) -> bool {
        if rustc.nightly {
            return match &self.nightly_version {
                Some(v) => v.matches(&rustc.version),
                None => false,
            };
        }

        match &self.version {
            Some(v) => v.matches(&rustc.version),
            None => false,
        }
    }
}

#[cfg(feature = "compiler_checks")]
#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum Constraint {
    Cfg(HashMap<String, Condition>),
    Condition(Condition),
}

#[cfg(feature = "compiler_checks")]
fn parse_compiler_checks(description: &TomlDescribe) -> Vec<(String, Condition)> {
    let target = env::var("TARGET").expect("TARGET environment variable is unset");
    let mut ret: Vec<(String, Condition)> = vec![];

    description.compiler_checks.iter().for_each(|(k, v)| {
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
                    c.iter().for_each(|(ck, cv)| {
                        ret.push((ck.clone(), cv.clone()));
                    });
                }
            }
        };
    });

    ret
}

#[derive(Default)]
pub struct Checks {
    #[cfg(feature = "compiler_checks")]
    pub compiler: Vec<(String, Condition)>,
}

impl Checks {
    fn new() -> Self {
        Checks {
            ..Default::default()
        }
    }
}

pub fn parse(text: &str) -> Checks {
    let mani: Manifest =
        toml::from_str(text).expect("Did not find a 'toml_describe' metadata section.");
    let mut checks = Checks::new();
    if cfg!(feature = "compiler_checks") {
        checks.compiler = parse_compiler_checks(&mani.package.metadata.toml_describe);
    }

    checks
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;
    use semver::Version;

    #[test]
    #[cfg(feature = "compiler_checks")]
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
    #[cfg(feature = "compiler_checks")]
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
    #[cfg(feature = "compiler_checks")]
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
    #[cfg(feature = "compiler_checks")]
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
    #[cfg(feature = "compiler_checks")]
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
    #[cfg(feature = "compiler_checks")]
    fn test_condition_check_version_match() {
        let c = Condition::new().set_version(VersionReq::parse(">= 1").ok());
        assert!(c.check(&version_data("1.0.0")));
    }

    #[test]
    #[cfg(feature = "compiler_checks")]
    fn test_condition_check_version_not_match() {
        let c = Condition::new().set_version(VersionReq::parse("< 1").ok());
        assert!(!c.check(&version_data("1.0.0")));
    }
}
