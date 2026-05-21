// 1. Генерация кода:

use std::{env, fs};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("generated.rs");

    let code = r#"
        pub const VERSION: &str = "1.0.0";
        pub fn get_config() -> &'static str { "production" }
    "#;

    fs::write(dest, code).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
} 