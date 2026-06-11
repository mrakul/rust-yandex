

fn main() {
    // Путь к библиотеке для линковки
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu/libz.so");

    // Линковать zlib
    println!("cargo:rustc-link-lib=dylib=z");
}