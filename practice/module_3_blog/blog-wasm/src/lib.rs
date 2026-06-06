use wasm_bindgen::prelude::*;
use yew::prelude::*;
// Импортируем функцию info! для логирования в браузерную консоль
use log::info;
use web_sys::HtmlInputElement;

// DTO в отдельном файлике
mod dto;
use dto::{RegisterRequest, AuthResponse, LoginRequest, CreatePostRequest};

// Для HTTP-запросов
use gloo_net::http::Request;
// Для хранения/чтения токена
use gloo_storage::{Storage, LocalStorage};

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


// Для передачи при работе с токеном
const BLOG_TOKEN_KEY:    &str = "blog_token";
const BLOG_USERNAME_KEY: &str = "blog_username";
const BLOG_USER_ID_KEY:  &str = "blog_user_id";

#[function_component(App)]
fn app() -> Html {

    // Состояния для полей регистрации
    let reg_username = use_state(|| "".to_string());
    let reg_email = use_state(|| "".to_string());
    let reg_password = use_state(|| "".to_string());

    // Состояние для полей логина
    let login_username = use_state(|| "".to_string());
    let login_password = use_state(|| "".to_string());

    // Состояние для отслеживания аутентификации
    let is_authenticated = use_state(|| {
        // Проверяем наличие токена в LocalStorage
        LocalStorage::get::<String>(BLOG_TOKEN_KEY).is_ok()
    });

    // Состояние для хранения имени пользователя (часть UserDto, можно остальное тоже докинуть)
    let current_username = use_state(|| {
        // Получить имя пользователя из LocalStorage 
        // Если аутентифицированы, пытаемся получить имя из LocalStorage
        if LocalStorage::get::<String>(BLOG_TOKEN_KEY).is_ok() {
            LocalStorage::get::<String>(BLOG_USERNAME_KEY).ok()   // Предполагаем, что имя сохраняется отдельно
        } else {
             None
        }
    });

    // Аналогично - User ID для сравнения для удаления/обновления поста
    let current_user_id = use_state(|| {
        if LocalStorage::get::<String>(BLOG_TOKEN_KEY).is_ok() {
            LocalStorage::get::<i64>(BLOG_USER_ID_KEY).ok()
        } else {
            None
        }
    });

    // use_state - это хук, который позволяет компоненту хранить и изменять состояние
    // Он принимает замыкание, которое возвращает начальное значение (Option<String> == None)
    // То есть это вроде как объект c текущим значением и методами для его изменения

    // Сообщение по нажатию кнопок, успех или ошибка, от этого зависит стиль в index.html
    let feedback_msg = use_state(|| None::<FeedbackMessage>);

    /*** Регистрация ***/

    // Асинхронная функция для вызова API регистрации
    let register_user_callback = {
        let reg_username = reg_username.clone();
        let reg_email = reg_email.clone();
        let reg_password = reg_password.clone();
        let feedback_msg = feedback_msg.clone();
        let is_authenticated = is_authenticated.clone();
        let current_username = current_username.clone();
        let current_user_id = current_user_id.clone();
        
        // Callback::from преобразует замыкание в обработчик событий Yew
        Callback::from(move |event: web_sys::SubmitEvent| {
            let reg_username = reg_username.clone();
            let reg_email = reg_email.clone();
            let reg_password = reg_password.clone();
            let feedback_msg = feedback_msg.clone();
            let is_authenticated = is_authenticated.clone();
            let current_username = current_username.clone();
            let current_user_id = current_user_id.clone();

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
                                    feedback_msg.set(Some(FeedbackMessage::Success("Вы успешно зарегистрировались (токен тоже получен)".to_string())));
                                    // (!) Вывод токена для отладки
                                    info!("Вы успешно зарегистрировались. Токен: {}", auth_resp.token);

                                    // (!) Сохраняем токен
                                    match LocalStorage::set(BLOG_TOKEN_KEY, &auth_resp.token) {
                                        Ok(()) => {
                                            info!("Токен сохранен в LocalStorage");
                                            if let Err(error) = LocalStorage::set(BLOG_USERNAME_KEY, &auth_resp.user.username) {
                                                // TODO: можно поменять на error, пока везде оставляю info!
                                                 info!("Ошибка сохранения имени пользователя: {}", error);
                                            } else {
                                                current_username.set(Some(auth_resp.user.username.clone()));
                                            }

                                            // Аналогично, сохраняем User ID
                                            LocalStorage::set(BLOG_USER_ID_KEY, &auth_resp.user.id).ok();
                                            current_user_id.set(Some(auth_resp.user.id));

                                            // Обновляем состояние аутентификации
                                            is_authenticated.set(true);
                                        }
                                        Err(error) => {
                                            feedback_msg.set(Some(FeedbackMessage::Error(format!("Ошибка сохранения токена: {}", error))));
                                            info!("Ошибка сохранения токена: {}", error);
                                        }
                                    }
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
            info!("Отправка формы регистрации...");
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

    /*** Логин и выход ***/

    let login_user_callback = {
        let login_username = login_username.clone();
        let login_password = login_password.clone();
        let feedback_msg = feedback_msg.clone();
        let is_authenticated = is_authenticated.clone();
        let current_username = current_username.clone();
        let current_user_id = current_user_id.clone();

        Callback::from(move |_: SubmitEvent| {
            let login_username = login_username.clone();
            let login_password = login_password.clone();
            let feedback_msg = feedback_msg.clone();
            let is_authenticated = is_authenticated.clone();
            let current_username = current_username.clone();
            let current_user_id = current_user_id.clone();

            // По аналогии с регистрацией (TODO: можно ли вынести в общий код)
            wasm_bindgen_futures::spawn_local(async move {
                let request_payload = LoginRequest {
                    username: (*login_username).clone(),
                    password: (*login_password).clone(),
                };

                let request = Request::post("http://127.0.0.1:3000/api/auth/login")
                    .header("Content-Type", "application/json")
                    .json(&request_payload)
                    .unwrap();

                match request.send().await {
                    Ok(response) => {
                        if response.ok() {
                            match response.json::<AuthResponse>().await {
                                Ok(auth_resp) => {
                                    feedback_msg.set(Some(FeedbackMessage::Success("Вы успешно вошли (токен получен)".to_string())));
                                    // (!) Вывод токена для отладки
                                    info!("Вы успешно вошли, получен токен: {}", auth_resp.token);

                                    // Сохраняем всё так же, как при регистрации
                                    match LocalStorage::set(BLOG_TOKEN_KEY, &auth_resp.token) {
                                        Ok(()) => {
                                            info!("Токен сохранен в LocalStorage при входе.");
                                            if let Err(e) = LocalStorage::set(BLOG_USERNAME_KEY, &auth_resp.user.username) {
                                                 info!("Ошибка сохранения имени пользователя при входе: {}", e);
                                            } else {
                                                current_username.set(Some(auth_resp.user.username.clone()));
                                            }

                                            // Аналогично, сохраняем User ID
                                            LocalStorage::set(BLOG_USER_ID_KEY, &auth_resp.user.id).ok();
                                            current_user_id.set(Some(auth_resp.user.id));

                                            // Обновляем состояние аутентификации
                                            is_authenticated.set(true);
                                        }
                                        Err(error) => {
                                            feedback_msg.set(Some(FeedbackMessage::Error(format!("Ошибка сохранения токена при входе: {}", error))));
                                            info!("Ошибка сохранения токена при входе: {}", error);
                                        }
                                    }
                                }
                                Err(error) => {
                                    feedback_msg.set(Some(FeedbackMessage::Error(format!("Ошибка парсинга ответа при входе: {}", error))));
                                    info!("Ошибка парсинга ответа при входе: {}", error);
                                }
                            }
                        } else {
                            // Обработка 4XX
                            let status = response.status();
                            let text = response.text().await.unwrap();
                            let error_msg = format!("Ошибка входа: ({}): {}", status, text);
                            
                            // Сообщение и лог
                            feedback_msg.set(Some(FeedbackMessage::Error(error_msg)));
                            info!("Ошибка входа: {}: {}", status, text);
                        }
                    }
                    Err(e) => {
                        feedback_msg.set(Some(FeedbackMessage::Error(format!("Сетевая ошибка при входе: {}", e))));
                        info!("Сетевая ошибка при входе: {}", e);
                    }
                }
            })
        })
    };

    // Обработчик события отправки формы входа
    let on_login_submit = {

        let feedback_msg = feedback_msg.clone();
        // let login_username = login_username.clone();
        // let login_password = login_password.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();    // Выкл перезагрузку страницы
            info!("Форма входа отправлена...");
            // Наверное, не нужно, подумать
            feedback_msg.set(None);

            // Вызываем коллбек
            login_user_callback.emit(event);

            // // Получаем значения из состояния
            // let username_val = (*login_username).clone();
            // let _password_val = (*login_password).clone();
            // // И лог в консоль (пароль не выводим напрямую)
            // info!("Логин: {}, Пароль: (скрыт)", username_val);
            // // Устанавливаем сообщение
            // feedback_msg.set(Some(FeedbackMessage::Success("Форма входа отправлена (Еще не реализовано)".to_string())));
        })
    };

    // Выход из сессии - удаляем токен (и что ещё есть) из LocalStorage
    let on_logout = {
        let is_authenticated = is_authenticated.clone();
        let current_username = current_username.clone();
        let feedback_msg = feedback_msg.clone();

        // (!) Тут нужен MouseEvent, чтобы в html! передать по onclick
        Callback::from(move |event: web_sys::MouseEvent| {
            // Удаляем токен и имя пользователя из LocalStorage
            LocalStorage::delete(BLOG_TOKEN_KEY);
            LocalStorage::delete(BLOG_USERNAME_KEY);
            // Обновляем состояние аутентификации
            is_authenticated.set(false);
            current_username.set(None::<String>); // Явно устанавливаем None
            // Показываем сообщение
            feedback_msg.set(Some(FeedbackMessage::Success("Вы вышли из системы.".to_string())));
            info!("Пользователь вышел из системы");
        })
    };


    /*** Посты ***/
    // Состояние для хранения списка постов (используется в Callback, )
    let posts_list = use_state(|| vec![]);
    // Состояние для индикатора загрузки, не уверен, сильно нужен или нет
    let loading_posts_indicator = use_state(|| false);

    // Список постов
    let posts_list_callback = {
        let posts = posts_list.clone();
        let loading_posts = loading_posts_indicator.clone();
        let feedback_msg = feedback_msg.clone();

        Callback::from(move |_| {
            let posts = posts.clone();
            let loading_posts = loading_posts.clone();
            let feedback_msg = feedback_msg.clone();

            wasm_bindgen_futures::spawn_local(async move {
                loading_posts.set(true);
                feedback_msg.set(None);

                // GET api/posts
                let request = Request::get("http://127.0.0.1:3000/api/posts")
                    .header("Content-Type", "application/json")
                    .build()
                    .unwrap();

                match request.send().await {
                    Ok(response) => {
                        if response.ok() {
                            match response.json::<dto::ListPostsResponse>().await {
                                Ok(posts_list_response) => {
                                    // Обновляем список
                                    posts.set(posts_list_response.posts);
                                    // info!("Список постов успешно загружен, количество: {}", posts_list_response.posts.len());
                                }
                                Err(error) => {
                                    feedback_msg.set(Some(FeedbackMessage::Error(format!("Ошибка парсинга списка постов: {}", error))));
                                    info!("Ошибка парсинга списка постов: {}", error);
                                }
                            }
                        } else {
                            // 4XX
                            let status = response.status();
                            let text = response.text().await.unwrap();
                            let error_msg = format!("Ошибка получения списка постов: ({}): {}", status, text);
                            feedback_msg.set(Some(FeedbackMessage::Error(error_msg)));
                            info!("Ошибка получения списка постов: {}: {}", status, text);
                        }
                    }
                    Err(e) => {
                        feedback_msg.set(Some(FeedbackMessage::Error(format!("Сетевая ошибка при получении постов: {}", e))));
                        info!("Сетевая ошибка при получении постов: {}", e);
                    }
                }
                loading_posts.set(false);
            })
        })
    };

    // (!) Начальная отрисовка постов: вызов через use_effect_with функции загрузки постов
    // Хитрая конструкция, надо передать () как зависимость, чтобы пущануть один раз
    {
        let posts_list_callback = posts_list_callback.clone();
        use_effect_with((), move |_| {
            posts_list_callback.emit(());
            || ()
        });
    }

    // Создание поста, состояния
    let create_title = use_state(|| "".to_string());
    let create_content = use_state(|| "".to_string());

    let create_post_callback = {
        let create_title = create_title.clone();
        let create_content = create_content.clone();
        let feedback_msg = feedback_msg.clone();
        let posts_list_callback = posts_list_callback.clone();

        // SubmitEvent
        Callback::from(move |_: web_sys::SubmitEvent| {
            let create_title = create_title.clone();
            let create_content = create_content.clone();
            let feedback_msg = feedback_msg.clone();
            let posts_list_callback = posts_list_callback.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Получаем токен из LocalStorage для защищенного запроса
                let token_from_local = match LocalStorage::get::<String>(BLOG_TOKEN_KEY) {
                    Ok(token) => token,
                    Err(_) => {
                        feedback_msg.set(Some(FeedbackMessage::Error("Токен не найден. Пожалуйста, залогиньтесь.".to_string())));
                        return;     // ()
                    }
                };

                // Заполняем DTO
                let request_payload = CreatePostRequest {
                    title: (*create_title).clone(),
                    content: (*create_content).clone(),
                };

                // POST api/posts с заголовком Authorization
                let request = Request::post("http://127.0.0.1:3000/api/posts")
                    .header("Content-Type", "application/json")
                    // (!) Формат Bearer и отправка (&format для &str)
                    .header("Authorization", &format!("Bearer {}", token_from_local))
                    .json(&request_payload)
                    .unwrap();

                match request.send().await {
                    Ok(response) => {
                        if response.ok() {
                            feedback_msg.set(Some(FeedbackMessage::Success("Пост успешно создан".to_string())));
                            // Очищаем форму
                            create_title.set("".to_string());
                            create_content.set("".to_string());
                            // (!) Обновляем список постов
                            posts_list_callback.emit(());
                        } else {
                            // 4XX
                            let status = response.status();
                            let text = response.text().await.unwrap();
                            feedback_msg.set(Some(FeedbackMessage::Error(format!("Ошибка создания поста: ({}): {}", status, text))));
                            info!("Ошибка создания поста: {}: {}", status, text);
                        }
                    }
                    Err(error) => {
                        feedback_msg.set(Some(FeedbackMessage::Error(format!("Сетевая ошибка при создании поста: {}", error))));
                        info!("Сетевая ошибка при создании поста: {}", error);
                    }
                }
            })
        })
    };

    // Submit-коллбек (Create)
    let on_create_post_submit = {
        let create_post_callback = create_post_callback.clone();
        Callback::from(move |event: web_sys::SubmitEvent| {
            event.prevent_default();

            info!("Отправка формы создания поста");
            create_post_callback.emit(event);
        })
    };


    // Удаление поста
    let delete_post_callback = {
        let feedback_msg = feedback_msg.clone();
        let posts_list_callback = posts_list_callback.clone();

        Callback::from(move |post_id: i64| {
            let feedback_msg = feedback_msg.clone();
            let posts_list_callback = posts_list_callback.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let token = match LocalStorage::get::<String>(BLOG_TOKEN_KEY) {
                    Ok(token) => token,
                    Err(_) => {
                        feedback_msg.set(Some(FeedbackMessage::Error("Нет токена".to_string())));
                        return;
                    }
                };

                let url = format!("http://127.0.0.1:3000/api/posts/{}", post_id);
                let request = Request::delete(&url)
                    .header("Authorization", &format!("Bearer {}", token))
                    .build()
                    .unwrap();

                match request.send().await {
                    Ok(response) => {
                        if response.ok() {
                            feedback_msg.set(Some(FeedbackMessage::Success("Пост удален".to_string())));
                            // Обновляем список
                            posts_list_callback.emit(());
                        } else {
                            let text = response.text().await.unwrap_or_default();
                            feedback_msg.set(Some(FeedbackMessage::Error(format!("Ошибка удаления: {}", text))));
                        }
                    }
                    Err(error) => feedback_msg.set(Some(FeedbackMessage::Error(format!("Сеть, ошибка: {}", error)))),
                }
            })
        })
    };

    /*** HTML-элементы, выделенные отдельно от общей логики html! */

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

    // Список постов тоже выножу отдельно, чтобы не забивать основную часть html!
    let posts_html = if *loading_posts_indicator {
        html! { <div class="loading">{"Загрузка постов..."}</div> }
    } else {
        html! {
            <div class="posts-container">
                // Пока ещё public API без токена
                <h2>{ "Список постов" }</h2>
                <div class="posts-list">
                    {
                        posts_list.iter().map(|post| {
                                // Проверяем по каждому, что пользователь - автор поста
                            let is_author = if let Some(uid_from_state) = *current_user_id {
                                uid_from_state == post.author_id
                            } else {
                                false
                            };

                            // По каждому создаём элемент с соощением и собираем в коллекцию с .collect::<Html>
                            html! {
                                // Здесь хитрая конструкция, поскольку добавляем кнопку, и нужно отобразить два родственных элемента: мету и кнопку.
                                // Надо обрамить всё Fragment <></> с одним root-узлом, не добавляя дополнительный элемент в DOM браузера
                                <> 
                                <div class="post-card">
                                    <h3>{ &post.title }</h3>
                                    <p>{ &post.content }</p>
                                    <div class="post-meta">
                                        <small>{ format!("ID: {}, Автор: {}", post.id, post.author_id) }</small>
                                    </div>
                                </div>
                                // (!) Кнопка удаления, по условию - логин + автор
                                if *is_authenticated && is_author {
                                    <button 
                                        class="btn-danger"
                                        style="margin-top: 10px; font-size: 0.8rem;"
                                        onclick={Callback::from({
                                            let delete_post_callback = delete_post_callback.clone();
                                            let post_id = post.id;
                                            move |_| {
                                                // Подтверждение удаления
                                                let confirmed_deletion = web_sys::window()
                                                    // Да, с обработкой ошибкой можно поработать
                                                    // .expect("Window object is not available")
                                                    .unwrap()
                                                    .confirm_with_message("Вы уверены, что хотите удалить этот пост?")
                                                    .unwrap();
                                                    // .unwrap_or(false);
                                                
                                                if confirmed_deletion {
                                                    delete_post_callback.emit(post_id);
                                                }
                                            }
                                        })}
                                    >
                                    {"Удалить"}
                                    </button>
                                }
                            </>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </div>
        }
    };

    /*** Основная HTML-часть ***/
    
    html! {
        <div class="container">
            <nav>
                <h1>{"Блог, WASM с использованием фреймворка Yew"}</h1>
                <div>
                    // Статус логина
                    if *is_authenticated {
                        if let Some(ref username) = (*current_username).as_ref() {
                             <span class="auth-status">{format!("Пользователь: {}", username)}</span>
                        } else {
                            <span class="auth-status">{"Аутентифицирован"}</span>
                        }
                        <button class="btn-secondary" onclick={on_logout.clone()}>{"Выйти"}</button>
                    } else {
                        <span class="auth-status">{"Не вошли в систему"}</span>
                    }
                </div>
            </nav>
            // (!) вставляем в разметку полученный VNode
            { feedback_msg_html }
            
            // Основной контейнер с двумя колонками: формы слева, посты справа
            <div style="display: flex; gap: 20px;">
                // Левая колонка: формы 
                <div class="forms-column" style="flex: 1; min-width: 300px;">
                    <div class="form-card">
                        <h2>{ "Регистрация" }</h2>
                        <form onsubmit={on_register_submit}>
                            <label for="reg_username">{"Имя пользователя*"}</label>
                            <input
                                type="text"             // Любой текст
                                id="reg_username"       // Идентификатор для label
                                required=true           // Обязательное
                                placeholder="Введите имя пользователя"
                                // Привязываем значение к состоянию
                                value={(*reg_username).clone()}

                                // (!) Обновляем состояние при вводе
                                oninput={Callback::from({
                                    let reg_username = reg_username.clone();
                                    
                                    move |event: web_sys::InputEvent| {
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
                                    move |event: web_sys::InputEvent| {
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
                                
                                // (!) Обновляем состояние при вводе
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

                    // Карточка формы входа
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
                                // (!) Обновляем состояние при вводе
                                oninput={Callback::from({
                                    let login_username = login_username.clone();
                                    move |event: web_sys::InputEvent| {
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
                                // (!) Обновляем состояние при вводе
                                oninput={Callback::from({
                                    let login_password = login_password.clone();
                                    move |event: web_sys::InputEvent| {
                                        let input: HtmlInputElement = event.target_unchecked_into();
                                        login_password.set(input.value());
                                    }
                                })}
                            />
                            <button class="btn-primary" type="submit">{"Войти"}</button>
                        </form>
                    </div>
                </div>

                // Справа список постов
                <div class="posts-column" style="flex: 2; min-width: 400px;">

                    // Показываем форму, только если пользователь аутентифицирован
                    if *is_authenticated {
                        <div class="form-card">
                            <h2>{ "Создать пост" }</h2>
                            // Коллбек стандартно по OnSubmit
                            <form onsubmit={on_create_post_submit}>
                                <label for="create_title">{"Заголовок*"}</label>
                                <input
                                    type="text"
                                    id="create_title"
                                    required=true
                                    placeholder="Введите заголовок"
                                    value={(*create_title).clone()}
                                    oninput={Callback::from({
                                        let create_title = create_title.clone();
                                        move |event: web_sys::InputEvent| {
                                            let input: HtmlInputElement = event.target_unchecked_into();
                                            create_title.set(input.value());
                                        }
                                    })}
                                />
                                <label for="create_content">{"Содержание*"}</label>
                                // Используем textarea для многострочного ввода
                                <textarea
                                    id="create_content"
                                    required=true
                                    placeholder="Введите содержание"
                                    rows=5  // 5 строк

                                    value={(*create_content).clone()}
                                    oninput={Callback::from({
                                        let create_content = create_content.clone();
                                        move |event: web_sys::InputEvent| {
                                            // Для textarea нужен HtmlTextAreaElement, добавил в workspace
                                            let input: web_sys::HtmlTextAreaElement = event.target_unchecked_into();
                                            create_content.set(input.value());
                                        }
                                    })}
                                />
                                <button class="btn-primary" type="submit">{"Создать пост"}</button>
                            </form>
                        </div>
                    }

                    // Список постов под формой (если пользователь может добавлять)
                    { posts_html }
                </div>
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