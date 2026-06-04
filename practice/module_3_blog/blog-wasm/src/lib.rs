use wasm_bindgen::prelude::*;
use yew::prelude::*;
// Импортируем функцию info! для логирования в браузерную консоль
use log::info;
use web_sys::HtmlInputElement;

// DTO в отдельном файлике
mod dto;
use dto::{RegisterRequest, AuthResponse};

// Для HTTP-запросов
use gloo_net::http::Request;

// Для вывода сообщения по запросам в разном стиле CSS - Success / Error
#[derive(Clone)]
enum FeedbackMessage {
    Success(String),
    Error(String)
}


// Пока самая простая отрисовка

// (!) Доустановил вот это для запуска ещё на этапе теории:
//  > rustup target add wasm32-unknown-unknown

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

    // Состояния для полей регистрации
    let reg_username = use_state(|| "".to_string());
    let reg_email = use_state(|| "".to_string());
    let reg_password = use_state(|| "".to_string());

    // Состояние для полей логина
    let login_username = use_state(|| "".to_string());
    let login_password = use_state(|| "".to_string());


    // use_state - это хук, который позволяет компоненту хранить и изменять состояние
    // Он принимает замыкание, которое возвращает начальное значение (Option<String> == None)
    // То есть это вроде как объект c текущим значением и методами для его изменения
    let feedback_msg = use_state(|| None::<FeedbackMessage>);

    // Асинхронная функция для вызова API регистрации
    let register_user_callback = {
        let reg_username = reg_username.clone();
        let reg_email = reg_email.clone();
        let reg_password = reg_password.clone();
        let feedback_msg = feedback_msg.clone();
        
        // Callback::from преобразует замыкание в обработчик событий Yew
        Callback::from(move |event: web_sys::SubmitEvent| {
            let reg_username = reg_username.clone();
            let reg_email = reg_email.clone();
            let reg_password = reg_password.clone();
            let feedback_msg = feedback_msg.clone();
            
            // Запускаем асинхронную задачу (как в теории )
            wasm_bindgen_futures::spawn_local(async move {
                // Через DTO
                let request_payload = RegisterRequest {
                    username: (*reg_username).clone(),
                    email: (*reg_email).clone(),
                    password: (*reg_password).clone(),
                };

                // Запрос на регистрацию, public API, адрес хардкожу
                let request = Request::post("http://127.0.0.1:3000/api/auth/register")
                    .header("Content-Type", "application/json")
                    .json(&request_payload)      // Сериализуем в JSON
                    .unwrap();

                match request.send().await {
                    Ok(response) => {
                        if response.ok() {
                            match response.json::<AuthResponse>().await {
                                Ok(auth_resp) => {
                                    // Успешная регистрация
                                    feedback_msg.set(Some(FeedbackMessage::Success("Вы успешно зарегистрировались! (Токен получен)".to_string())));
                                    // (!) Вывод токена для отладки
                                    info!("Вы успешно зарегистрировались!, token: {}", auth_resp.token);

                                    // TODO: тут сохраняем токен для отладки
                                }
                                Err(error) => {
                                    feedback_msg.set(Some(FeedbackMessage::Error(format!("Ошибка парсинга  ответа: {}", error))));
                                    info!("Ошибка парсинга ответа: {}", error);
                                }
                            }
                        } else {
                            // Код возврата 4XX
                            let status = response.status();
                            let text = response.text().await.unwrap();  // Да, надо обрабатывать .unwrap()

                            feedback_msg.set(Some(FeedbackMessage::Error(format!("Ошибка регистрации: ({}): {}", status, text))));
                            info!("Ошибка регистрации: {}: {}", status, text);
                        }
                    }
                    Err(error) => {
                        // Сетеваяя ошибка
                        feedback_msg.set(Some(FeedbackMessage::Error(format!("Сетевая ошибка: {}", error))));
                        info!("Сетевая ошибка: {}", error);
                    }
                }
            })
        })
    };

    // Обработчик события отправки формы регистрации (Callback, в общем)
    let on_register_submit = {
        // Получение значений перенесено в функцию

        // Здесь event == SubmitEvent
        Callback::from(move |event: SubmitEvent| {
            // Перезагрузка страницы, выключаем
            event.prevent_default();

            // Логируем сообщение в консоль браузера
            info!("Отправка формы регистрации!");
            // Вызываем функцию по обработке 
            register_user_callback.emit(event);

            // Логируем сообщение в консоль браузера
            // info!("Форма регистрации отправлена!");
            // // Получаем значения из состояния
            // let username_val = (*reg_username_clone).clone();
            // let email_val = (*reg_email_clone).clone();
            // let _password_val = (*reg_password_clone).clone();
            // // Логируем без пароля
            // info!("Регистрация: Имя={}, Email={}, Пароль=(скрыт)", username_val, email_val);
            // // Новое значение feedback_msg (Some) вызывает перерендеринг компонента App
            // feedback_msg_clone.set(Some("Форма регистрации отправлена! (Еще не реализовано)".to_string()));
        })
    };

    // Обработчик события отправки формы входа
    let on_login_submit = {

        let feedback_msg = feedback_msg.clone();
        let login_username = login_username.clone(); // Клонируем для доступа в замыкании
        let login_password = login_password.clone(); // Клонируем для доступа в замыкании

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default(); // Предотвращаем перезагрузку
            info!("Форма входа отправлена!");
            
            // Получаем значения из состояния
            let username_val = (*login_username).clone();
            let _password_val = (*login_password).clone();
            // И лог в консоль (пароль не выводим напрямую)
            info!("Логин: {}, Пароль: (скрыт)", username_val);
            // Устанавливаем сообщение
            feedback_msg.set(Some(FeedbackMessage::Success("Форма входа отправлена! (Еще не реализовано)".to_string())));
        })
    };

    // Можно отделить логику от общей части html!
    // Присваиваем элементу сообщения или html с сообщением, если есть. Или никакой (но надо вернуть пустой html!)
    let feedback_msg_html = if let Some(feedback) = (*feedback_msg).as_ref() {
        // Через match определяем тип сообщения для соответствующего типа из CSS
        match feedback {
            FeedbackMessage::Success(text) => {
                // Успешное
                html! {
                    <div class="success-msg">
                        { text.clone() }
                    </div>
                }
            }
            FeedbackMessage::Error(text) => {
                // Ошибка
                html! {
                    <div class="error-msg">
                        { text.clone() }
                    </div>
                }
            }
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
                        required=true
                        placeholder="Введите имя пользователя"
                        // Привязываем значение к состоянию
                        value={(*reg_username).clone()}
                        // (!) Обновляем состояние при вводе
                        oninput={Callback::from({
                            let reg_username = reg_username.clone();
                            move |event: InputEvent| {
                                // Или так:
                                // let target = event.target().unwrap();
                                // Тут динамическое преобразование типов, но с проверкой на компиляции
                                // let input = target.dyn_into::<HtmlInputElement>().unwrap();

                                // Это как в теории, тема 3, урок 7
                                let input: HtmlInputElement = event.target_unchecked_into();
                                // Забираем значение (ниже для полей всё аналогично)
                                reg_username.set(input.value());
                            }
                        })}
                    />
                    <label for="reg_email">{"Email*"}</label>
                    <input
                        type="email"            // Проверится e-mail формат
                        id="reg_email"
                        required=true
                        placeholder="Введите email"
                        // Привязываем значение к состоянию
                        value={(*reg_email).clone()}
                        // (!) Обновляем состояние при вводе
                        oninput={Callback::from({
                            let reg_email = reg_email.clone();
                            move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                reg_email.set(input.value());
                            }
                        })}
                    />
                    <label for="reg_password">{"Пароль*"}</label>
                    <input
                        type="password"     // Парольный тип
                        id="reg_password"
                        required=true
                        placeholder="Введите пароль"
                        // Привязываем значение к состоянию
                        value={(*reg_password).clone()}
                        // Обновляем состояние при вводе
                        oninput={Callback::from({
                            let reg_password = reg_password.clone();
                            move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                reg_password.set(input.value());
                            }
                        })}
                    />
                    <button class="btn-primary" type="submit">{"Зарегистрироваться"}</button>
                </form>
            </div>
            <div class="form-card">
                <h2>{ "Вход" }</h2>
                <form onsubmit={on_login_submit}>
                    <label for="login_username">{"Имя пользователя*"}</label>
                    <input
                        type="text"
                        id="login_username"
                        required=true
                        placeholder="Введите имя пользователя"
                        // Привязываем значение к состоянию
                        value={(*login_username).clone()}
                        // Обновляем состояние при вводе
                        oninput={Callback::from({
                            let login_username = login_username.clone();
                            move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                login_username.set(input.value());
                            }
                        })}
                    />
                    <label for="login_password">{"Пароль*"}</label>
                    <input
                        type="password"
                        id="login_password"
                        required=true
                        placeholder="Введите пароль"
                        // Привязываем значение к состоянию
                        value={(*login_password).clone()}
                        // Обновляем состояние при вводе
                        oninput={Callback::from({
                            let login_password = login_password.clone();
                            move |event: InputEvent| {
                                let input: HtmlInputElement = event.target_unchecked_into();
                                login_password.set(input.value());
                            }
                        })}

                    />
                    <button class="btn-primary" type="submit">{"Войти"}</button>
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