// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use serde::Deserialize;
use semver::VersionReq;

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
    compiler_versions: HashMap<String, VersionReq>,
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

        assert!(mani.package.metadata.compiler_versions["foo"].matches(&Version::new(1, 0, 0)));
    }
}