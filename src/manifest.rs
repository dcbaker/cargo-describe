// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use cfg_expr::targets::get_builtin_target_by_triple;
use cfg_expr::{Expression, Predicate};
use semver::{Version, VersionReq};
use serde::de::Error;
use serde::{Deserialize, Deserializer};
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

#[derive(Clone, Deserialize, Debug, PartialEq)]
pub struct Condition {
    #[serde(deserialize_with = "to_version_req")]
    version: Option<VersionReq>,
}

impl Condition {
    pub fn check(&self, rust_version: &Version) -> bool {
        match &self.version {
            Some(v) => v.matches(rust_version),
            None => false,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum Constraint {
    Condition(Condition),
    Cfg(HashMap<String, Condition>),
}

fn to_version_req<'de, D>(deserializer: D) -> Result<Option<VersionReq>, D::Error>
where
    D: Deserializer<'de>,
{
    let o: Option<String> = Deserialize::deserialize(deserializer)?;
    match o {
        Some(s) => VersionReq::parse(s.as_str())
            .map_err(D::Error::custom)
            .map(Some),
        None => Ok(None),
    }
}

#[derive(Deserialize, Debug)]
struct Metadata {
    compiler_support: HashMap<String, Constraint>,
}

pub fn parse(text: &str) -> Vec<(String, Condition)> {
    let mani: Manifest =
        toml::from_str(text).expect("Did not find a 'compiler_versions' metadata section.");
    let target = env::var("TARGET").unwrap();
    let mut ret: Vec<(String, Condition)> = vec![];

    mani.package
        .metadata
        .compiler_support
        .iter()
        .for_each(|(k, v)| {
            match v {
                Constraint::Condition(con) => ret.push((k.clone(), con.clone())),
                Constraint::Cfg(c) => {
                    let cfg = Expression::parse(k).unwrap();
                    let res = if let Some(tinfo) = get_builtin_target_by_triple(&target) {
                        cfg.eval(|p| match p {
                            Predicate::Target(tp) => tp.matches(tinfo),
                            _ => false,
                        })
                    } else {
                        false
                    };
                    if res {
                        c.iter().for_each(|(ck, cv)| {
                            ret.push((ck.clone(), cv.clone()));
                        });
                    }
                }
            };
        });

    return ret;
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[test]
    fn test_basic_read() {
        let mani: Manifest = toml::from_str(
            r#"
            [package.metadata.compiler_support]
            foo = { version = "1.0.0" }
        "#,
        )
        .unwrap();

        let v = &mani.package.metadata.compiler_support["foo"];
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
            [package.metadata.compiler_support]
            foo = { version = ">1.0.0, <2.0.0" }
        "#,
        )
        .unwrap();

        let v = &mani.package.metadata.compiler_support["foo"];
        let ver = match v {
            Constraint::Condition(ver) => ver.version.as_ref().unwrap(),
            _ => panic!("Did not get a Version"),
        };
        assert!(ver.matches(&Version::new(1, 3, 0)));
        assert!(!ver.matches(&Version::new(0, 3, 0)));
        assert!(!ver.matches(&Version::new(2, 3, 0)));
    }

    #[test]
    fn test_cfg() {
        let mani: Manifest = toml::from_str(
            r#"
            [package.metadata.compiler_support.'cfg(target_os = "linux")']
            foo = { version = "~1.0.0" }
        "#,
        )
        .unwrap();

        let v = &mani.package.metadata.compiler_support["cfg(target_os = \"linux\")"];
        let cfg = match v {
            Constraint::Cfg(cfg) => cfg,
            _ => panic!("Did not get a Version"),
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
                [package.metadata.compiler_support]
                foo = { version = "1.0.0" }
                [package.metadata.compiler_support.'cfg(target_os = "linux")']
                bar = { version = "1.2.0" }
                [package.metadata.compiler_support.'cfg(target_os = "windows")']
                bad = { version = "1.2.0" }
            "#,
            );

            assert_eq!(vals.len(), 2);
        });
    }
}
