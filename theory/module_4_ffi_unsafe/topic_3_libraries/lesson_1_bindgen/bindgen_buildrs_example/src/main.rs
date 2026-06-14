// (!) Надо заинклудить bindgen.rs, это уже результат после bindgen
include!(concat!(env!("OUT_DIR"), "/bindgen.rs")); 

// Для запуска с динамической библиотекой:
// LD_LIBRARY_PATH=. cargo run

fn main() {
    println!("Squared eleven: {}", unsafe { square(11) });
} 