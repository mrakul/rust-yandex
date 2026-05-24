// Импортируем необходимые библиотеки
use reqwasm::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

// (!) Доустановил вот это для запуска:
// > rustup target add wasm32-unknown-unknown

#[derive(Clone, PartialEq, Deserialize)]
struct Price {
    symbol: String, // Название торговой пары (например: "BTCUSDT", "ETHUSDT")
    price: String,  // Текущая цена в виде строки (используем String для точности)
}

// Создаём главный компонент приложения
// #[function_component] - макрос, который превращает функцию в компонент Yew
#[function_component(App)]
fn app() -> Html {
    // Используем хуки состояния (аналогично useState в React)

    // data - хранит список криптовалют и их цен
    // use_state инициализирует состояние пустым вектором
    let data = use_state(|| Vec::<Price>::new());

    // loading - флаг, указывающий на выполнение HTTP-запроса
    let loading = use_state(|| false);

    // Создаём callback-функцию для загрузки данных
    // Используем блок для ограничения области видимости клонированных переменных
    let fetch_data = {
        // Клонируем указатели на состояние для использования в замыкании
        let data = data.clone();
        let loading = loading.clone();

        // Callback::from преобразует замыкание в обработчик событий Yew
        Callback::from(move |_| {
            // Дополнительно клонируем для асинхронной задачи
            let data = data.clone();
            let loading = loading.clone();

            // Устанавливаем флаг загрузки в true
            loading.set(true);

            // spawn_local запускает асинхронную задачу в контексте WebAssembly
            spawn_local(async move {
                // Выполняем GET-запрос к Binance API
                // .await - приостанавливает выполнение, запрос завершится
                let resp = Request::get("https://api.binance.com/api/v3/ticker/price")
                    .send()
                    .await
                    .unwrap(); // В реальном приложении лучше обрабатывать ошибки

                // Парсим JSON-ответ в вектор структур Price
                let json: Vec<Price> = resp.json().await.unwrap();

                // Обновляем состояние с полученными данными
                data.set(json);

                // Сбрасываем флаг загрузки
                loading.set(false);
            });
        })
    };

    // Макрос html! позволяет писать JSX-подобный код на Rust
    html! {
        <div class="p-4 font-sans">
            /* Заголовок приложения */
            <h1 class="text-2xl mb-4">{"Binance Crypto Prices"}</h1>

            /* Кнопка для загрузки/обновления данных */
            <button
                onclick={fetch_data.clone()}
                class="px-4 py-2 bg-blue-600 text-white rounded"
            >
                {"Обновить"}
            </button>

            /* Условный рендеринг: проверяем состояние загрузки */
            if *loading {
                // Показываем индикатор загрузки
                <p>{"Загрузка..."}</p>
            } else {
                // Рендерим таблицу с данными
                <table class="mt-4 border-collapse">
                    /* Заголовок таблицы */
                    <tr><th>{"Пара"}</th><th>{"Цена"}</th></tr>

                    /* Итерируемся по данным и создаём строки таблицы */
                    {
                        // data.iter() - создаёт итератор по данным
                        // .take(10) - берёт только первые 10 элементов
                        // .map() - преобразует каждый элемент в HTML
                        for data.iter().take(10).map(|p| html! {
                            <tr>
                                /* Ячейка с названием торговой пары */
                                <td class="border px-2">{ &p.symbol }</td>
                                /* Ячейка с ценой */
                                <td class="border px-2">{ &p.price }</td>
                            </tr>
                        })
                    }
                </table>
            }
        </div>
    }
}

// Главная функция - точка входа приложения
fn main() {
    // Инициализируем систему логирования для отладки в браузере
    // Логи будут появляться в консоли разработчика
    wasm_logger::init(wasm_logger::Config::default());

    // Создаём рендерер Yew и монтируем компонент App в DOM
    // Renderer::<App>::new() - создаёт рендерер для компонента App
    // .render() - запускает рендеринг и привязывает компонент к HTML-странице
    yew::Renderer::<App>::new().render();
}