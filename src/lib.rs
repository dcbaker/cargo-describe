// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
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
    Platform(HashMap<String, VersionReq>),
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

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

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
}