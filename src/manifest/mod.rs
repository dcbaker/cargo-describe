// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

#[cfg(feature = "compiler_checks")]
mod compiler;

#[cfg(feature = "compiler_checks")]
use crate::manifest::compiler::{Condition, Constraint, parse_compiler_checks};

use serde::Deserialize;
use std::collections::HashMap;
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
        checks.compiler = parse_compiler_checks(&mani.package.metadata.toml_describe.compiler_checks);
    }

    checks
}
