use wasm_bindgen::prelude::*;
use yew::prelude::*;

// Пока самая простая торисовка

// Запуск как из урока 2, темы 3 теории: 
// cd blog-wasm
// wasm-pack build --target web

// [INFO]: ⬇️  Installing wasm-bindgen...
// [INFO]: Optimizing wasm binaries with `wasm-opt`...
// [INFO]: Optional fields missing from Cargo.toml: 'description', 'repository', and 'license'. These are not necessary, but recommended
// [INFO]: ✨   Done in 1m 04s
// [INFO]: 📦   Your wasm pkg is ready to publish at /home/m_rakul/Code/rust-yandex/practice/module_3_blog/blog-wasm/pkg.

// И пускануть сервер:
// python3 -m http.server 8000

#[function_component(App)]
fn app() -> Html {
    html! {
        <div class="container">
            <nav><h1>{"Блог, WASM с использованием фреймворка Yew"}</h1></nav>
            <div class="form-card">
                <h2>{"Yew запущен!"}</h2>
                <p>{"Проверка стиля, отрисовка с помощью WASM."}</p>
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}