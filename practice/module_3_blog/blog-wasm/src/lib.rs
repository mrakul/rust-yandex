use wasm_bindgen::prelude::*;
use yew::prelude::*;
// Импортируем функцию info! для логирования в браузерную консоль
use log::info;

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
    // use_state - это хук, который позволяет компоненту хранить и изменять состояние
    // Он принимает замыкание, которое возвращает начальное значение (Option<String> == None)
    // То есть это вроде как объект c текущим значением и методами для его изменения
    let feedback_msg = use_state(|| None::<String>);

    // Обработчик события отправки формы регистрации (Callback, в общем)
    let on_register_submit = {
        let feedback_msg = feedback_msg.clone();
        // Здесь event == SubmitEvent
        Callback::from(move |event: SubmitEvent| {
            // Перезагрузка страницы, выключаем
            event.prevent_default();

            // Логируем сообщение в консоль браузера
            info!("Форма регистрации отправлена!");
            // Новое значение feedback_msg (Some(...)) вызывает перерендеринг компонента App
            feedback_msg.set(Some("Форма регистрации отправлена! (Еще не реализовано)".to_string()));
        })
    };

    // Можно отделить логику от общей части html!
    // Присваиваем элементу сообщения или html с сообщением, если есть. Или никакой (но надо вернуть пустой html!)
    let feedback_msg_html = if let Some(msg) = (*feedback_msg).as_ref() {
        html! {
            // <div class="success-msg">
            <div class="error-msg">
                { msg } 
            </div>
        }
    } else {
        // Нужен VNode тип
        html! {}
    };

    html! {
        <div class="container">
            <nav>
                <h1>{"Блог, WASM с использованием фреймворка Yew"}</h1>
                <div>
                </div>
            </nav>
            // (!) вставляем в разметку полученный VNode
            { feedback_msg_html }
            <div class="form-card">
                <h2>{ "Регистрация" }</h2>
                <form onsubmit={on_register_submit}>
                    <label for="reg_username">{"Имя пользователя*"}</label>
                    <input
                        type="text"             // Любой текст
                        id="reg_username"       // Идентификатор для label
                        required=true           // Обязательное
                        placeholder="Введите имя пользователя" // Подсказка внутри поля
                    />
                    <label for="reg_email">{"Email*"}</label>
                    <input
                        type="email"            // Проверится e-mail формат
                        id="reg_email"
                        required=true
                        placeholder="Введите email"
                    />
                    <label for="reg_password">{"Пароль*"}</label>
                    <input
                        type="password"     // Парольный тип
                        id="reg_password"
                        required=true
                        placeholder="Введите пароль"
                    />
                    <button class="btn-primary" type="submit">{"Зарегистрироваться"}</button>
                </form>
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    // Инициализируем логгер
    wasm_logger::init(wasm_logger::Config::default());

    // Создаем рендерер для компонента App и монтируем его в элемент с id="app" в index.html
    yew::Renderer::<App>::new().render();
}