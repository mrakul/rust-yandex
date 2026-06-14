use bindgen;
use std::{env, path::PathBuf};

fn main() {
    let bindings = bindgen::builder()
        // Файл, для которого создаются байндинги
        .header("src/mylib.h")
        // Перезапуск сборки при изменении переданных файлов
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Сгенерировать байндинги
        .generate()
        .expect("Unable to generate bindings");

    // OUT_DIR задавать не надо
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Биндинги попадут сюда: ./target/debug/build/bindgen_buildrs-1327e2504a28f079/out/bindgen.rs
    bindings
        // Записать получившиеся байндинги в файл OUT_DIR/bindgen.rs
        .write_to_file(out_path.join("bindgen.rs"))
        .expect("Couldn't write bindings!");


    // Сборка статической (!) библиотеки из C-шного кода
    cc::Build::new()
        // добавить src/mylib.c в выходную библиотеку
        .file("src/mylib.c")
        // скомпилировать C-код как библиотеку libmylib.a в папке OUT_DIR 
        .compile("mylib");

    /*** Для подключения динамической библиотеки ***/

    // Предварительно создать:
    // gcc -shared -fPIC src/mylib.c -o libmylib.so

    // А теперь в build.rs уберите использование cc и добавьте в конце функции main (после генерации байндингов):
    // println!("cargo:rustc-link-search=native=.");
    // println!("cargo:rerun-if-changed=libmylib.so"); // на macOS: libmylib.dylib
    // println!("cargo:rustc-link-lib=dylib=mylib");
} 