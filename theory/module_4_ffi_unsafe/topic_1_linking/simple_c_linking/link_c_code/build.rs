

fn main() {
    // Создание библиотеки add из файла add.c
    cc::Build::new()
        .file("src/add.c")
        .compile("add");

    // Просьба cargo перезапустить скрипт, если add.c изменился
    println!("cargo:rerun-if-changed=src/add.c");
}