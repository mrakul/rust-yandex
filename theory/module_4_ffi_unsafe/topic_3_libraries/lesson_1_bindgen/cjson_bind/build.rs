
use bindgen;
use cc;
use std::{env, path::PathBuf, process::Command};

fn main() {
    // 1. Ask GCC where its internal include directory is
    // gcc prints something like: /usr/lib/gcc/x86_64-linux-gnu/12/include
    let gcc_include_output = Command::new("gcc")
        .arg("-print-file-name=include")
        .output()
        .expect("Failed to execute gcc");

    let gcc_include_path = String::from_utf8(gcc_include_output.stdout)
        .expect("Invalid UTF-8 from gcc")
        .trim()
        .to_string();

    // 2. Создаём builder с параметрами: хедер, путь к стандартным инклудам
    let mut builder = bindgen::Builder::default()
        .header("cJSON/cJSON.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    if !gcc_include_path.is_empty() && gcc_include_path != "include" {
        builder = builder.clang_arg(format!("-I{}", gcc_include_path));
    }
    // Also add standard system includes as a fallback
    builder = builder.clang_arg("-I/usr/include");


    // 3. Сгенерировать Биндинги из cJSON.h
    let bindings = builder
        .generate()
        .expect("Не удалось создать биндинги");

    // Записать в bindgen.rs в OUT_DIR, стандартно
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindgen.rs"))
        .expect("Couldn't write bindings!");

    // Собрать библиотеку
    cc::Build::new()
        // добавить src/cJSON.c в выходную библиотеку
        .file("cJSON/cJSON.c")
        // скомпилировать C-код как библиотеку libcJSON.a в папке OUT_DIR
        .compile("cJSON");
}