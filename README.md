# Toml Describe

Toml-describe is a rust crate to allow describing feature checks in toml, instead of open coding them in build.rs. It achieves this by providing a simple entry point to add to your build.rs, and then you describe the environment in your Cargo.toml.

## Usage

Add this snippet to your build.rs:
```rs
extern crate toml_describe;

fn main() {
    toml_describe::evaluate();
}
```

### Compiler checked CFGs

These are added to Toml in the format

```toml
[package.metadata.toml_describe.compiler_checks.CFG_NAME]
version = "1.75"
nightly_version = "1.60"
```

Using `version` will set the cfg if the compiler is at least the given version, `nightly_version` will return tru if the compiler is `nightly` and the compiler has at least that version. If both `version` and `nightly_version` are set, then the condition is true if either check succeeds.

```toml
[package.metadata.toml_describe.compiler_checks]
CFG_NAME = { can_compile = "fn function() -> bool { false }" }
```

Will cause the compiler to be invoked to compile the given code snippet. If it can be compiled then the cfg will be enabled, otherwise it wont.

If both versions and compiler checks are provided, then both must be true for the cfg to be emitted.

### Allowed CFGS

These are sent to cargo for use with the `-Z cfg-check` option. options are in the form `cfg = [...<allowed values>]`. If the CFG does not have allowed values, it should be an empty list

```toml
[package.metadata.toml_describe.allowed_cfgs]
use_foo = []
foo_backend = ["unix", "windows"]
```

With the correct flags, rustc will then warn about CFGS that are not defined, or values to those cfgs that are not in the allowed value set.
