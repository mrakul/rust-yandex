// src/main.rs

// Импортируем необходимые модули из Yew и других крейтов
use yew::prelude::*;           // Основные типы и макросы Yew
use wasm_bindgen_futures::spawn_local;  // Для выполнения асинхронных задач в WASM
use reqwasm::http::Request;    // HTTP-клиент для WebAssembly
use serde::Deserialize;        // Для десериализации JSON из API

// Структура для хранения данных о ценах с Binance API
// #[derive] автоматически реализует трейты для структуры
#[derive(Clone, PartialEq, Deserialize)]
struct Price {
    symbol: String,   // Название торговой пары (например: "BTCUSDT")
    price: String,    // Текущая цена в виде строки (для точности)
}

// Объявляем корневой компонент приложения
// #[function_component] - макрос для создания функциональных компонентов
#[function_component(App)]
fn app() -> Html {
    // Хуки состояния - аналогично useState в React
    
    // data: хранит список цен, инициализируется пустым вектором
    let data = use_state(|| Vec::<Price>::new());
    
    // filter: хранит текст фильтра для поиска по символам
    let filter = use_state(|| String::new());
    
    // loading: флаг загрузки данных
    let loading = use_state(|| false);

    // Создаем callback для загрузки данных
    let fetch_data = {
        // Клонируем указатели на состояние для использования в замыкании
        let data = data.clone();
        let loading = loading.clone();
        // Callback::from преобразует замыкание в обработчик событий Yew
        Callback::from(move |_| {
            // Дополнительное клонирование для асинхронной задачи
            let data = data.clone();
            let loading = loading.clone();
            
            // Устанавливаем флаг загрузки
            loading.set(true);
            
            // Запускаем асинхронную задачу
            spawn_local(async move {
                // Выполняем GET-запрос к Binance API
                if let Ok(resp) = Request::get("https://api.binance.com/api/v3/ticker/price")
                    .send()
                    .await 
                {
                    // Парсим JSON ответ в вектор структур Price
                    if let Ok(json) = resp.json::<Vec<Price>>().await {
                        // Обновляем состояние с новыми данными
                        data.set(json);
                    }
                }
                // Сбрасываем флаг загрузки независимо от результата
                loading.set(false);
            });
        })
    };

    // Обработчик ввода для поля фильтра
    let oninput = {
        let filter = filter.clone();
        Callback::from(move |e: InputEvent| {
            // Преобразуем event target в HTMLInputElement
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            // Обновляем состояние фильтра
            filter.set(input.value());
        })
    };

    // Рендерим JSX-подобную разметку с помощью макроса html!
    html! {
        <div class="p-4 font-sans">
            <h1>{"Binance Crypto Prices"}</h1>
            
            <div>
                // Кнопка для загрузки данных
                <button onclick={fetch_data.clone()}>{"Обновить"}</button>
                
                // Поле ввода для фильтрации
                <input
                    type="text"
                    placeholder="Фильтр по символу..."
                    value={(*filter).clone()}  // Дереференсируем и клонируем значение
                    {oninput}                  // Привязываем обработчик ввода
                />
            </div>

            // Условный рендеринг: показываем индикатор загрузки или таблицу
            if *loading {
                <p>{"Загрузка..."}</p>
            } else {
                <table>
                    <thead>
                        <tr><th>{"Пара"}</th><th>{"Цена"}</th></tr>
                    </thead>
                    <tbody>
                        {
                            // Итерация по отфильтрованным данным
                            for data.iter()
                                // Фильтруем по символу (регистронезависимо)
                                .filter(|p| p.symbol.to_lowercase().contains(&filter.to_lowercase()))
                                // Берем только первые 15 элементов для производительности
                                .take(15)
                                // Преобразуем каждую структуру Price в HTML строку таблицы
                                .map(|p| html! {
                                    <tr>
                                        <td>{ &p.symbol }</td>
                                        <td>{ &p.price }</td>
                                    </tr>
                                })
                        }
                    </tbody>
                </table>
            }
        </div>
    }
}

// Точка входа приложения
fn main() {
    // Инициализируем систему логирования для отладки в браузере
    // Логи будут появляться в консоли разработчика
    wasm_logger::init(wasm_logger::Config::default());

    // Создаём рендерер Yew и монтируем компонент App в DOM
    // Renderer::<App>::new() - создаёт рендерер для компонента App
    // .render() - запускает рендеринг и привязывает компонент к HTML-странице
    yew::Renderer::<App>::new().render();
}