// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::env;
use std::vec::Vec;
use cfg_expr::{Expression, Predicate};
use cfg_expr::targets::get_builtin_target_by_triple;
use serde::{Deserialize, Deserializer};
use serde::de::Error;
use semver::VersionReq;

#[derive(Deserialize, Debug)]
struct Manifest {
    package: Package,
}

#[derive(Deserialize, Debug)]
struct Package {
    metadata: Metadata,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
enum Constraint {
    #[serde(deserialize_with = "to_version_req")]
    VersionReq(VersionReq),
    Cfg(HashMap<String, VersionReq>),
}

fn to_version_req<'de, D>(deserializer: D) -> Result<VersionReq, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    VersionReq::parse(s.as_str()).map_err(D::Error::custom)
}

#[derive(Deserialize, Debug)]
struct Metadata {
    compiler_versions: HashMap<String, Constraint>,
}

fn parse(text: &str) -> Vec<(String, VersionReq)> {
    let mani: Manifest = toml::from_str(text).unwrap();
    let target = env::var("TARGET").unwrap();
    let mut ret: Vec<(String, VersionReq)> =vec![];

    mani.package.metadata.compiler_versions.iter().for_each( |(k, v)| {
        match v {
            Constraint::VersionReq(ver) => ret.push((k.clone(), ver.clone())),
            Constraint::Cfg(c) => {
                let cfg = Expression::parse(k).unwrap();
                let res = if let Some(tinfo) = get_builtin_target_by_triple(&target) {
                    cfg.eval( |p| match p {
                        Predicate::Target(tp) => tp.matches(tinfo),
                        _ => false,
                    })
                } else {
                    false
                };
                if res {
                    c.iter().for_each( |(ck, cv)| { ret.push((ck.clone(), cv.clone())); } );
                }
            },
        };
    });

    return ret;
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;
    use temp_env;

    #[test]
    fn test_basic_read() {
        let mani: Manifest = toml::from_str(r#"
            [package.metadata.compiler_versions]
            foo = "1.0.0"
        "#).unwrap();

        let v = &mani.package.metadata.compiler_versions["foo"];
        let ver = match v {
            Constraint::VersionReq(ver) => ver,
            _ => panic!("Did not get a Version"),
        };
        assert!(ver.matches(&Version::new(1, 0, 0)));
    }

    #[test]
    fn test_cfg() {
        let mani: Manifest = toml::from_str(r#"
            [package.metadata.compiler_versions.'cfg(target_os = "linux")']
            foo = "1.0.0"
        "#).unwrap();

        let v = &mani.package.metadata.compiler_versions["cfg(target_os = \"linux\")"];
        let cfg = match v {
            Constraint::Cfg(cfg) => cfg,
            _ => panic!("Did not get a Version"),
        };

        assert!(cfg.contains_key("foo"));
        assert!(cfg["foo"].matches(&Version::new(1, 0, 0)));
    }

    #[test]
    fn test_parse_cfg() {
        temp_env::with_var("TARGET", Some("x86_64-unknown-linux-gnu"), || {
            let vals = parse(r#"
                [package.metadata.compiler_versions]
                foo = "1.0.0"
                [package.metadata.compiler_versions.'cfg(target_os = "linux")']
                bar = "1.2.0"
                [package.metadata.compiler_versions.'cfg(target_os = "windows")']
                bad = "1.2.0"
            "#);

            assert_eq!(vals.len(), 2);
        });
    }
}