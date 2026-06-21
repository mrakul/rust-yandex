# Проект "Обработчик изображений с плагинами"

Весь проект - в виде Workspace, `Cargo.toml` которого определяет 3 крата:
```toml
members = ["image_processor",
           "mirror_plugin",
           "blur_plugin"]
```

## Описание кратов
*Примечание: сценарии использования-проверки находятся в отдельной секции.*

### `image_processor`
CLI-приложение для проверки подключения плагинов-обработчиков изображений в виде динамических библиотек, написанных на Rust.
Парсит параметры командной строки, используя `clap`. 
Загрузка/сохранения изображения с использованием крата `image` для перевода в формат RGBA8-последовательность, последующей передачей сырого указателя на буфер в `unsafe`-секции плагинов.
Как и указателя на параметры обработки изображения с выбранным плагином (только тут указатель константный).

Кроме того, производит выбранного через CLI загрузку динамической библиотеки-плагина (Mirror или Blur) с помощью крата `libloader`,
как и поиск символа `proess_image` в библиотеках. В случае успешной загрузки и поиска символа вызывает функцию (это всё, разумеется, в `unsafe`-секции).
Указатель на RGBA8-буфер передаётся как мутабельный для изменения in-place `*mut u8` (сишный `uint8_t *rgba_data`), список параметров - неизменяемый - `*const c_char` (сишный `const char *params`).

### `mirror_plugin`
Экспортирует основной символ `process_image` через C ABI, без мэнглирования символов в библиотеке.
```rust
#[unsafe(no_mangle)]
pub extern "C" fn process_image
```

С вызовом парсинга параметров и применения основной логики Mirror в `unsafe`-секции для работы с сырыми указателями.
Кроме того, вызовы обёрнуты в `catch_unwind` с передачей замыкания для возможности "отловить" панику.
В случае паники выводится сообщение "Плагин mirror запаниковал!" и обработка завершается.

Параметры Mirror:
- `horizontal` - булев параметр, запрашивает горизонтальное отражение
- `vertical`   - булев параметр, запрашивает вертикальное отражение

Алгоритм незамысловатый: для горизонтального отражения просматривается левая половина изображения и меняются местами пиксели с симметричным относительно вертикальной середины. Аналогично для вертикального, но верхняя часть изображения со swap-ом с симметричным относительно горизонтальной середины.
В коде много комментариев по поводу реализации.

Unit-тесты (ниже приведу вызов): парсинг параметров и 3 теста на буфере 2x2 с горизонтальным отражением, вертикальным и обоими одновременно.

Собирается как `cdylib` - динамическая библиотека с C ABI:
```toml
crate-type = ["cdylib"]
```

### `blur_plugin`
Экспортирует основной символ `process_image` через C ABI, без мэнглирования символов в библиотеке.
```rust
#[unsafe(no_mangle)]
pub extern "C" fn process_image
```

С вызовом парсинга параметров и применения основной логики Blur в `unsafe`-секции для работы с сырыми указателями.
Кроме того, вызовы обёрнуты в `catch_unwind` с передачей замыкания для возможности "отловить" панику.
В случае паники выводится сообщение "Плагин mirror запаниковал!" и обработка завершается.

Используется алгоритм Box Blur: для каждого пикселя считается среднее значение байт RGBA в зависимости от установленного радиуса (с проверкой, чтобы не выйти за границы изображения). Значения меняются in-place в буфере `rgba_data`, но промежуточное поитерационное состояние буфера после итерации размытия
сохраняется в `image_iter_snapshot`, чтобы использовать "актуальные" пиксели на каждой итерации до изменения в `rgba_data`.
В коде много комментариев.

Параметры Blur:
- `radius`     - значение радиуса относительно текущего пикселя для усреднения по байтам RGBA
- `iterations` - количество итераций Box Blur

Unit-тесты (ниже приведу вызов): парсинг параметров и 2 теста на буфере 3x3: буфер с одинаковыми цветами - размытие не меняет значений RGBA пикселя.
И яркий пиксель посередине с нулевыми по краям (0, 0, 0, 255) => размытие приводит к изменению пикселей по краям.

Собирается как `cdylib` - динамическая библиотека с C ABI:
```toml
crate-type = ["cdylib"]
```

## Сценарии использования / воспроизведение
Всё можно делать из корня проекта, workspace.

В папке `./aux/` находится изображение для проверки - `random_png_1033x775.png`.
И файлы параметров для Mirror и Blur соответственно: `mirror_params.txt` и `blur_params.txt`

Плагины по умолчанию идут в директорию `target/debug/`: `libmirror_plugin.so` и `libblur_plugin.so`.

Сборка:
```bash
cargo build --workspace
```

Формат CLI-команд:
```bash
cargo run -p image_processor -- --help
```

Вывод:
```text
CLI для проверки применения плагинов к изображению

Usage: image_processor [OPTIONS] --input <INPUT> --output <OUTPUT> --plugin <PLUGIN> --params <PARAMS>

Options:
  -i, --input <INPUT>              Исходное изображение (путь)
  -o, --output <OUTPUT>            Путь сохранения обработанного изображения
      --plugin <PLUGIN>            Имя плагина. Предполагаются: "mirror", "blur"
      --params <PARAMS>            Путь к файлу с параметрами
      --plugin-path <PLUGIN_PATH>  Путь к директории с плагинами [default: target/debug]
  -h, --help                       Print help
```

Unit-тесты - по обоим плагинам:
```bash
cargo test
```

Вывод:
```text
running 9 tests
test tests::test_apply_blur_logic_center_affects_area ... ok
test tests::test_apply_blur_logic_uniform_no_change ... ok
test tests::test_parse_params_defaults_missing_keys ... ok
test tests::test_parse_params_invalid_format ... ok
test tests::test_parse_params_iterations_only ... ok
test tests::test_parse_params_invalid_value ... ok
test tests::test_parse_params_unknown_key ... ok
test tests::test_parse_params_radius_only ... ok
test tests::test_parse_params_1_3 ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/image_processor-c7499ef1302e0e00)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/mirror_plugin-70dba3f4911f18a0)

running 11 tests
test tests::test_apply_mirror_logic_vertical ... ok
test tests::test_parse_params_all_false ... ok
test tests::test_parse_params_defaults_missing_keys ... ok
test tests::test_parse_params_all_true ... ok
test tests::test_apply_mirror_logic_horizontal ... ok
test tests::test_parse_params_horizontal_only ... ok
test tests::test_parse_params_invalid_format ... ok
test tests::test_apply_mirror_logic_both ... ok
test tests::test_parse_params_invalid_value ... ok
test tests::test_parse_params_unknown_key ... ok
test tests::test_parse_params_vertical_only ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Неверный параметр для Mirror - будет выдана ошибка:
> "Ошибка плагина Mirror: Неподдерживаемый параметр 'horizonDal':
> Исходное изображение не меняется"

```bash
cargo run -p image_processor -- --input ./aux/random_png_1033x775.png --output ./aux/processed_image.png --plugin mirror --params ./aux/mirror_params.txt
```

Вывод:
```text
Изображение:                    ./aux/random_png_1033x775.png
Обработанное изображение:       ./aux/processed_image.png
Плагин обработал:               mirror
Файл параметров:                ./aux/mirror_params.txt
Путь к плагинам:                target/debug
Загрузка изображения: ./aux/random_png_1033x775.png
Размер изображения: 1033x775
Размер буфера: 3202300 байт
Загрузка плагина: target/debug/libmirror_plugin.so
Ошибка плагина Mirror: Неподдерживаемый параметр 'horizonDal'.
Исходное изображение не меняется
Обработанное изображение: ./aux/processed_image.png
Обработка завершена плагином: mirror
```

Неверный параметр для Blur - будет выдана ошибка:
> "Загрузка плагина: target/debug/libblur_plugin.so
> Ошибка плагина Blur: Неверное значение для пары 'iterations': 'BAD_VALUE'."

```bash
cargo run -p image_processor -- --input ./aux/random_png_1033x775.png --output ./aux/processed_image.png --plugin blur --params ./aux/blur_params.txt
```

Вывод:
```text
Изображение:                    ./aux/random_png_1033x775.png
Обработанное изображение:       ./aux/processed_image.png
Плагин обработал:               blur
Файл параметров:                ./aux/blur_params.txt
Путь к плагинам:                target/debug
Загрузка изображения: ./aux/random_png_1033x775.png
Размер изображения: 1033x775
Размер буфера: 3202300 байт
Загрузка плагина: target/debug/libblur_plugin.so
Ошибка плагина Blur: Неверное значение для пары 'iterations': 'BAD_VALUE'.
Исходное изображение не меняется
Обработанное изображение: ./aux/processed_image.png
Обработка завершена плагином: blur
```

Успешные запуски:

Mirror:
```bash
cargo run -p image_processor -- --input ./aux/random_png_1033x775.png --output ./aux/processed_image.png --plugin mirror --params ./aux/mirror_params.txt
```

Вывод:
```text
Изображение:                    ./aux/random_png_1033x775.png
Обработанное изображение:       ./aux/processed_image.png
Плагин обработал:               mirror
Файл параметров:                ./aux/mirror_params.txt
Путь к плагинам:                target/debug
Загрузка изображения: ./aux/random_png_1033x775.png
Размер изображения: 1033x775
Размер буфера: 3202300 байт
Загрузка плагина: target/debug/libmirror_plugin.so
Изображение успешно обработано плагином Mirror
Обработанное изображение: ./aux/processed_image.png
Обработка завершена плагином: mirror
```

`./aux/processed_image.png` сохранён в соответствии с запрошенными параметрами после обработки исходного изображения.

Blur:
```bash
cargo run -p image_processor -- --input ./aux/random_png_1033x775.png --output ./aux/processed_image.png --plugin blur --params ./aux/blur_params.txt
```

Вывод:
```text
Изображение:                    ./aux/random_png_1033x775.png
Обработанное изображение:       ./aux/processed_image.png
Плагин обработал:               blur
Файл параметров:                ./aux/blur_params.txt
Путь к плагинам:                target/debug
Загрузка изображения: ./aux/random_png_1033x775.png
Размер изображения: 1033x775
Размер буфера: 3202300 байт
Загрузка плагина: target/debug/libblur_plugin.so
Изображение успешно обработано плагином Blur
Обработанное изображение: ./aux/processed_image.png
Обработка завершена плагином: blur
```

`./aux/processed_image.png` сохранён в соответствии с запрошенными параметрами после обработки исходного изображения.

Кроме того, сценарии проверки наличия файлов параметров и исходного изображения.
Ошибки вида:
```text
Файл параметров './aux/blur_params.TXT' не существует
Файл изображения './aux/random_png_1033x775.pn' не существует.
```