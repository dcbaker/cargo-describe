// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use serde::Deserialize;

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
    compiler_versions: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_read() {
        let mani: Manifest = toml::from_str(r#"
            [package.metadata.compiler_versions]
            foo = "bar"
        "#).unwrap();

        assert_eq!(mani.package.metadata.compiler_versions["foo"], "bar");
    }
}