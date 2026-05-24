// src/lib.rs

use wasm_bindgen::prelude::*;

// Сборка проекта с помощью wasm-pack:
// > wasm-pack build --target web

// На выходе должно быть такое:

// [INFO]: ⬇️  Installing wasm-bindgen...
// [INFO]: Optimizing wasm binaries with `wasm-opt`...
// [INFO]: Optional fields missing from Cargo.toml: 'description', 'repository', and 'license'. These are not necessary, but recommended
// [INFO]: ✨   Done in 18.46s
// [INFO]: 📦   Your wasm pkg is ready to publish at /home/m_rakul/Code/rust-yandex/theory/module_3/topic_4_cli_frontend/lesson_3_web_assembly/pkg.


// Запустите локальный сервер (например, python3 -m http.server) и откройте http://localhost:8000/demo_multiple.html в браузере.
// Вы должны увидеть следующий текст: «Привет, Мир! Rust говорит тебе: добро пожаловать в WebAssembly».

// Указываем, что эту функцию можно вызывать из JS
// #[wasm_bindgen]
// pub fn greet(name: &str) -> String {
//     format!("Привет, {name}! Rust говорит тебе: добро пожаловать в WebAssembly.")
// }

// src/lib.rs

#[wasm_bindgen]
pub fn greet(name: &str, count: u32) -> String {
    let mut result = String::new();
    for i in 0..count {
        result.push_str(&format!("{}. Привет, {}! Rust в WebAssembly.\n", i + 1, name));
    }
    result
} 