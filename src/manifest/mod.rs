// Copyright © 2023 Dylan Baker
// SPDX-License-Identifier: MIT

#[cfg(feature = "compiler_checks")]
mod compiler;

#[cfg(feature = "compiler_checks")]
use crate::manifest::compiler::{parse_compiler_checks, Condition, Constraint};

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
    #[serde(default)]
    compiler_checks: HashMap<String, Constraint>,

    #[cfg(feature = "cfg_checks")]
    #[serde(default)]
    pub allowed_cfgs: HashMap<String, Vec<String>>,
}

#[derive(Default)]
pub struct Checks {
    #[cfg(feature = "compiler_checks")]
    pub compiler: Vec<(String, Condition)>,

    #[cfg(feature = "cfg_checks")]
    pub cfgs: Vec<(String, Option<Vec<String>>)>,
}

impl Checks {
    fn new() -> Self {
        Checks {
            ..Default::default()
        }
    }
}

pub fn parse(text: &str) -> Checks {
    let mani: Manifest = toml::from_str(text).unwrap();
    let mut checks = Checks::new();
    #[cfg(feature = "compiler_checks")]
    {
        checks.compiler =
            parse_compiler_checks(&mani.package.metadata.toml_describe.compiler_checks);
    }
    #[cfg(feature = "cfg_checks")]
    {
        let mut cfgs = Vec::<(String, Option<Vec<String>>)>::new();
        for (k, v) in mani.package.metadata.toml_describe.allowed_cfgs {
            let ret = match v.is_empty() {
                true => None,
                false => Some(v.clone()),
            };

            cfgs.push((k.clone(), ret));
        }
        checks.cfgs = cfgs;
    }

    checks
}
