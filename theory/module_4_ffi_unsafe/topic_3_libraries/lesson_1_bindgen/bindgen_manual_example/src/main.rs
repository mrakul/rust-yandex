// (!) Надо заинклудить bindgen.rs, это уже результат после bindgen
include!("./bindgen.rs");

// 0. Здесь не нужно ничего писать в build.rs и с Cargo.toml => links

// 1. Для генерации объявлений:
// > bindgen src/mylib.h -o src/bindgen.rs
// Можете посмотреть в файл src/bindgen.rs и увидеть там аналогичную декларацию функции, что вы писали, но сгенерированную автоматически. 

// 2. Теперь соберём C-библиотеку:>
// > gcc -c src/mylib.c -o mylib.o 
// > ar rcs libmylib.a mylib.o

// (из теории)
// > clang -c src/mylib.c -o mylib.o 

// Вместо clang можно использовать и gcc (и теоретически любой другой C компилятор), но так как Rust зависит от инфраструктуры llvm, все примеры будут именно для clang.
// Соберём и запустим программу:
// > RUSTFLAGS="-L. -l./libmylib.a" cargo run

// Вот это сработало с полным путём: возможно, он пытается открыть от workspace или от lesson_1_bindgen
// (!) RUSTFLAGS="-C link-arg=/home/m_rakul/Code/rust-yandex/theory/module_4_ffi_unsafe/topic_2_libraries/lesson_1_bindgen/bindgen_example/libmylib.a --verbose" cargo run

// RUSTFLAGS="-L. -lmylib" cargo run
// Флаг -l ожидает имя библиотеки (mylib → libmylib.a), а не путь к .o. 
// Команда RUSTFLAGS="-L. -l./mylib.o" некорректна и не сработает.
// Переменная среды RUSTFLAGS передаёт флаги линкеру в rustc. На практике надёжнее build.rs и крейт cc — к этому перейдём дальше.

fn main() {
    println!("Squared eleven: {}", unsafe { square(11) });
} 