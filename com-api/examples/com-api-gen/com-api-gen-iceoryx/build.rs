use std::{fs, path::Path};

use abi_types_codegen::{
    build,
    config::{Config, RustOptions, Target},
};

fn main() {
    let source = Path::new("src").join("vehicle_interface.types");
    println!("cargo::rerun-if-changed={}", source.display());

    let output_dir = Path::new("src").join("generated");
    let output = output_dir.join("vehicle_interface.rs");
    fs::create_dir_all(&output_dir).expect("create 'src/generated' directory");

    let config = Config {
        format: true,
        source,
        output: Some(output),
        target: Target::Rust,
        rust: RustOptions { derive_reloc: true },
    };
    build(&config).unwrap();
}
