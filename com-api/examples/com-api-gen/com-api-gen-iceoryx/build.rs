use std::path::Path;

use abi_types_codegen::{
    build,
    config::{Config, RustOptions, Target},
};

fn main() {
    let config = Config {
        format: true,
        source: Path::new("src/vehicle_interface.types").to_owned(),
        target: Target::Rust,
        rust: RustOptions { derive_reloc: true },
    };
    build(&config).unwrap();
}
