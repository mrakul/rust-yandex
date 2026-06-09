# Проект "Блог"

Весь проект - в виде Workspace, `Cargo.toml` которого определяет 4 крата и содержит зависимости по всем кратам.
Зависимости подключаются через `workspace = true` в соответствующих кратах.

Проект разделён на 4 крата-подмодуля:
```toml
[workspace]
members = ["blog-server", "blog-client", "blog-cli", "blog-wasm"]
```

## Описание кратов
*Примечание: сценарии использования-проверки находятся в отдельной секции.*

### `blog-server`
backend-часть, обрабатывает входящие запросы клиента с использованием типа транспорта HTTP (фреймворк `actix-web`) / gRPC (фреймворк `tonic`).
В Protected API использует JWT (`jsonwebtoken`) для выдачи (это при регистрации/логине, разумеется), и проверки токена в формате "Bearer [токен]", токен "длится" 24 часа.
Используется СУБД PostgreSQL (`sqlx`) на Data-уровне, миграции делаются для табличек Users, Posts. Соответственно, доменный уровень содержит структуры User и Post с соответствующими impl.
На уровне Application содержит сервис авторизации `AuthService` (JWT) и `BlogService` с сценариями использования - операции с постами.
Построен на Clean Architecture (во многом по теме 3 "Rust backend" урока 1 "Введение в backend" и подсказкам задания). Проверка JWT как Middleware на уровне Presentation.
В коде много комментариев, надеюсь, это поможет ревью.

Для JWT-секрета, адреса БД, CORS используется файл с переменными окружения: `.env`
Как описано в задании, сам файл не загружен, приложен файл с инструкцией по генерации JWT-секрета: `example.env`

Оба сервера запускаются на этих адресах:
* HTTP: `http://127.0.0.1:3000`
* gRPC: `http://127.0.0.1:50051`

В отдельной секции я приведу сценарии использования/проверки с помощью `curl`/`grpcurl`.

### `blog-client`
библиотека с API для выполнения запросов по HTTP (используется зависимость `reqwest`, клиентская часть HTTP) или gRPC (`tonic` + `prost`).
Для HTTP - обёртки над `reqwest::..::RequestBuilder` / `Response`, с соответствующими установками хедеров "Authorization", "Content-Type" и вызовом API сервера.

Для gRPC используется `blog.proto` (аналогично серверу) и `build.rs` с генерацией клиентской части без trait'ов сервера, содержит обёртку над сгенерированными вызовами/структурами.

### `blog-cli`
Консольное CLI-приложение, использует библиотеку `blog-client` для проверки работы сервера.
Тип транспорта указывается в командной строке, для обработки параметров командной строки используется `clap`.
Полученный токен при регистрации и логине сохраняется в файл `.blog_token`

Формат команды:
```text
CLI-утилита для блога

Usage: blog-cli [OPTIONS] <COMMAND>

Commands:
  register  Регистрация нового пользователя
  login     Логин с получением JWT-токена в файлик
  create    Создание поста
  get       Пост по ID
  update    Обновление поста
  delete    Удалить пост по ID
  list      Список постов с limit и ofsset
  help      Print this message or the help of the given subcommand(s)

Options:
      --server <SERVER>  Адрес сервера (по умолчанию: http://127.0.0.1:3000, где должен быть HTTP)
      --grpc             Использовать gRPC транспорт вместо HTTP (адрес gRPC-сервера по умолчанию: http://127.0.0.1:50051)
  -h, --help             Print help
```
 
В отдельной секции я приведу сценарии использования/проверки для HTTP / gRPC.
 
### `blog-wasm`
frontend-часть проекта с WebAssembly, используется фреймворк Yew на основе урока 4, темы 2 "Реализация веб-интерфейса на Rust".
Если коротко, что делает Renderer Yew: в `index.html` находит секцию `<div id="app">` (куда монтировиться), создаёт App компонент (`#[function_component(App)]`) с отслеживанием DOM, отрисовывает виртуальные DOM из `html!{}` и отслеживает изменение состояний для перерисовки.
В коде много комментариев, но в общем - используется не чистый WASM, а зависимость Yew с соответствующими вызовами `use_state`, `html!` для разметки и другие.
Помимо Yew используются: `gloo_net` - для HTTP-запросов, `gloo_storage` - для хранения/чтения токена.

От интерфейса потрясающего user experience ожидать не стоит, надеюсь на понимание, у меня в опыте сильный перекос в сторону backend (последнее, чего касался, кроме html/CSS - это JQuery лет 10 назад).
Но если коротко: слева две формы - "Регистрация" и под ней "Вход" (Логин). Справа список постов, если пользователь не авторизован.
Если пользователь авторизован - в правом верхнем углу показывается его имя.
Над табличкой постов появляется форма создания/редактирования постов. И у постов, автором которых явлется залогиненный пользователь, появляются кнопочки "Удалить" и "Редактировать" в соответствующих стилях.

Сообщение по обратной связи `feedback_msg` в зависимости от Success / Error использует разные стили, реализовано через Enum.
При действии сообщает об успешной регистрации, логине, манипуляции с постами.

Доустанавливал ещё на этапе теории помимо указанных зависимостей:
```bash
rustup target add wasm32-unknown-unknown
```

---

## Настройка окружения, зависимости

### 1. PostgreSQL
```bash
sudo apt update
sudo apt install postgresql

# Запуск и автозапуск
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Создать БД blog_db и задать пароль суперпользователю postgres
sudo -u postgres psql -c "CREATE DATABASE blog_db;"
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';"

# Войти в сессию PostgreSQL под админом или другим авторизованным для создания таблиц пользователем:
sudo -u postgres psql
```

```sql
-- Создать базу blog_db:
CREATE DATABASE blog_db;

-- Опционально, если с другим пользователем:
CREATE USER blog_user WITH PASSWORD 'your_secure_password'; 
ALTER USER blog_user CREATEDB;
GRANT ALL PRIVILEGES ON DATABASE blog_db TO blog_user;
\q
```

### 2. `.env`:
В `blog-server` загружен файл с `example.env`: необходимо указать пользователя с паролем, сгенерировать и сохранить JWT-секрет, переименовать файлик в `.env`
```ini
# Подключение к БД
DATABASE_URL=postgresql://[user]:[password]@localhost:5432/blog_db

# JWT-секрет
# Для генерации: openssl rand -base64 32
# (32 байта минимум,)
JWT_SECRET=[PASTE_YOUR_SECRET]

# Сервер
HOST=127.0.0.1
PORT=3000

# CORS
CORS_ORIGIN=http://localhost:3000
```

### 3. Для WASM:
Доустанавливал ещё на этапе теории помимо указанных зависимостей:
```bash
rustup target add wasm32-unknown-unknown
```

---

## Сценарии использования/проверки: 

Из корневой директории Workspace: лучше сразу сделать на все краты:
```bash
cargo build
```

### 1. `blog-server`:
Сборка:
```bash
cargo build -p blog-server
```

Запуск:
```bash
cd ./blog-server/
cargo run
```

Должны появиться сообщения логгера, запуск обоих серверов:
```json
{"timestamp":"2026-06-08T23:23:37.946496498Z","level":"INFO","fields":{"message":"connected to PostgreSQL"}}
{"timestamp":"2026-06-08T23:23:37.946719464Z","level":"INFO","fields":{"message":"running database migrations"}}
{"timestamp":"2026-06-08T23:23:37.949729756Z","level":"INFO","fields":{"message":"relation \"_sqlx_migrations\" already exists, skipping"}}
{"timestamp":"2026-06-08T23:23:37.954433002Z","level":"INFO","fields":{"message":"migrations completed"}}
{"timestamp":"2026-06-08T23:23:37.954582476Z","level":"INFO","fields":{"message":"🚀 HTTP-сервер запускается на http://127.0.0.1:3000"}}
{"timestamp":"2026-06-08T23:23:37.955448706Z","level":"INFO","fields":{"message":"starting 14 workers"}}
{"timestamp":"2026-06-08T23:23:37.95556372Z","level":"INFO","fields":{"message":"Actix runtime found; starting in Actix runtime"}}
{"timestamp":"2026-06-08T23:23:37.955604691Z","level":"INFO","fields":{"message":"starting service: \"actix-web-service-127.0.0.1:3000\", workers: 14, listening on: 127.0.0.1:3000"}}
{"timestamp":"2026-06-08T23:23:38.245881498Z","level":"INFO","fields":{"message":"🚀 gRPC-сервер запускается на http://127.0.0.1:50051"}}
```

В VSCode на вкладке Ports можно увидеть, какие адреса/порты используются запущенным процессом `debug/blog-server`.

#### Проверка API HTTP - в отдельном терминале:

**health:**
```bash
curl http://127.0.0.1:3000/api/health
```
```json
{"status":"ok","timestamp":"2026-06-08T23:27:50.909012085Z"}
```

**Регистрация с выделением токена:**
(токен возвращаю и при регистрации, и при логине):
```bash
export TOKEN=$(curl -s -X POST http://127.0.0.1:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -v -d '{
    "username": "yandex",
    "email": "yandex@yandex.ru",
    "password": "12345678"
  }' | jq -r '.token')
```
```text
=>
*   Trying 127.0.0.1:3000...
* Connected to 127.0.0.1 (127.0.0.1) port 3000
> POST /api/auth/register HTTP/1.1
> Host: 127.0.0.1:3000
> User-Agent: curl/8.5.0
> Accept: */*
> Content-Type: application/json
> Content-Length: 93
> 
} [93 bytes data]
< HTTP/1.1 201 Created
```

Можно посмотреть токен: 
```bash
echo $TOKEN
```
  
Повторный пуск даёт HTTP 409:
```text
< HTTP/1.1 409 Conflict
< content-length: 85
```
  
**Логин:**
```bash
export TOKEN=$(curl -s -X POST http://127.0.0.1:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -v -d '{
    "username": "yandex",
    "password": "12345678"
  }' | jq -r '.token')
```
```text
=>
< HTTP/1.1 200 OK
< content-length: 296
```

Можно сверить, что токен другой: 
```bash
echo $TOKEN
```

**Список постов:**
```bash
curl -v -X GET http://127.0.0.1:3000/api/posts
```
```text
=>
< HTTP/1.1 200 OK
< content-length: 1522
< content-type: application/json
```
+ посты:
```json
{"posts":[{"id":14,"title":"Мой второй пост","content":"Содержание","author_id":10,"created_at":"2026-06-08T21:08:30.915526Z","updated_at":"2026-06-08T21:08:30.915526Z"},{"id":13,"title":"Обновлённый заголовок gRPC","content":"","author_id":10,"created_at":"2026-06-08T21:08:18.462880Z","updated_at":"2026-06-08T21:09:49.153171Z"},{"id":12,"title":"Updated Title via gRPC","content":"Updated content.","author_id":7,"created_at":"2026-06-07T20:35:17.718468Z","updated_at":"2026-06-07T20:38:35.373557Z"},
...
```

**Создание поста:**
```bash
curl -v -X POST http://127.0.0.1:3000/api/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "Изначальный пост пользователя yandex",
    "content": "Изначальный заголовок"
  }'
```
```text
> Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...
> Content-Length: 144
> 
< HTTP/1.1 201 Created
< content-length: 244
...
```
```json
{"id":19,"title":"Изначальный пост пользователя yandex","content":"Изначальный заголовок","author_id":11,"created_at":"2026-06-08T23:47:02.692320352Z","updated_at":"2026-06-08T23:47:02.69232"}
```

**Пост по ID:**
```bash
curl -v -X GET http://127.0.0.1:3000/api/posts/19
```
```text
< HTTP/1.1 200 OK
< content-length: 144
...
```
```json
{"id":10,"title":"misha","content":"Test 3","author_id":4,"created_at":"2026-06-06T06:10:38.151534Z","updated_at":"2026-06-07T04:22:49.460542Z"}
```

Если не найден - 404.

**Изменение поста:**
```bash
curl -X PUT http://127.0.0.1:3000/api/posts/19 \
  -v -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "title": "Изменённый заголовок",
    "content": "Изменённый контент"
  }'
```
```text
=> 
> Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...
> Content-Length: 114
> 
< HTTP/1.1 200 OK
...
```
```json
{"id":19,"title":"Изменённый заголовок","content":"Изменённый контент","author_id":11,"created_at":"2026-06-08T23:47:02.695347Z","updated_at":"2026-06-08T23:51:24.054682020Z"}
```

Если чужой - `< HTTP/1.1 403 Forbidden`.

**Удаление:**
```bash
curl -v -X DELETE http://127.0.0.1:3000/api/posts/19 -H "Authorization: Bearer $TOKEN"
```
```text
> Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...
> 
< HTTP/1.1 204 No Content
```

Повторный: 404.

#### gRPC-сервер:
Тоже внутри `./blog-server/` (для доступа к `proto/blog.proto`)

**Регистрация:**
```bash
grpcurl -plaintext -proto proto/blog.proto -d '{
    "username": "yandex3",
    "email": "yandex3@yandex.ru",
    "password": "12345678"
  }' localhost:50051 blog.BlogService/Register
```
```json
=>
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "13",
    "username": "yandex3",
    "email": "yandex3@yandex.ru",
    "createdAt": "2026-06-09T00:03:28.987522118+00:00"
  }
}
```

Повторный:
```bash
grpcurl -plaintext -proto proto/blog.proto -d '{
    "username": "yandex3",
    "email": "yandex3@yandex.ru",
    "password": "12345678"
  }' localhost:50051 blog.BlogService/Register
```
```text
=> 
ERROR:
  Code: AlreadyExists
  Message: Пользователь уже существует
```

**Логин:**
```bash
grpcurl -plaintext -proto proto/blog.proto \
  -d '{
    "username": "yandex3",
    "password": "12345678"
  }' \
  localhost:50051 blog.BlogService/Login
```
```json
=>
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "13",
    "username": "yandex3",
    "email": "yandex3@yandex.ru",
    "createdAt": "2026-06-09T00:03:28.990491+00:00"
  }
}
```

Можно сохранить токен (уже другой :) ):
```bash
export TOKEN=$(grpcurl -plaintext -proto proto/blog.proto \
  -d '{
    "username": "yandex3",
    "password": "12345678"
  }' \
  localhost:50051 blog.BlogService/Login 2>/dev/null | jq -r '.token')
```

**Список постов:**
```bash
grpcurl -plaintext -proto proto/blog.proto \
  -d '{
    "limit": 10,
    "offset": 0
  }' \
  localhost:50051 blog.BlogService/ListPosts
```
```json
=>
{
  "posts": [
    {
      "id": "18",
      "title": "Изначальный пост пользователя yandex",
      "content": "Изначальный заголовок",
      "authorId": "11",
      "createdAt": "2026-06-08T23:46:23.338855+00:00",
      "updatedAt": "2026-06-08T23:46:23.338855+00:00"
    },
...
    {
      "id": "11",
      "title": "misha",
      "content": "Изменить ещё раз 123",
      "authorId": "4",
      "createdAt": "2026-06-06T06:20:32.829888+00:00",
      "updatedAt": "2026-06-07T04:40:44.061393+00:00"
    },
    {
      "id": "5",
      "title": "Ещё одно от misha3",
      "content": "Ещё одно сообщение от misha3",
      "authorId": "6",
      "createdAt": "2026-06-05T14:51:53.617755+00:00",
      "updatedAt": "2026-06-05T14:51:53.617755+00:00"
    }
  ]
}
```

**Создание поста:**
```bash
grpcurl -plaintext -proto proto/blog.proto \
  -H "authorization: Bearer $TOKEN" \
  -d '{
    "title": "Пост через gRPC",
    "content": "Текст через gRPC"  
  }' \
  localhost:50051 blog.BlogService/CreatePost
```
```json
{
  "post": {
    "id": "21",
    "title": "Пост через gRPC",
    "content": "Текст через gRPC",
    "authorId": "13",
    "createdAt": "2026-06-09T00:15:55.382799576+00:00",
    "updatedAt": "2026-06-09T00:15:55.382799576+00:00"
  }
}
```

**Создание поста с неверным токеном:**
```bash
grpcurl -plaintext -proto proto/blog.proto \
  -H "authorization: Bearer BAD_STRING" \
  -d '{
    "title": "Пост через gRPC",
    "content": "Текст через gRPC"  
  }' \
  localhost:50051 blog.BlogService/CreatePost
```
```text
ERROR:
  Code: Unauthenticated
  Message: Неверный токен
```

**Список постов с limit и offset:**
```bash
grpcurl -plaintext -proto proto/blog.proto \
  -d '{
    "limit": 5,
    "offset": 9
  }' \
  localhost:50051 blog.BlogService/ListPosts
```
```json
{
  "posts": [
    {
      "id": "11",
      "title": "misha",
      "content": "Изменить ещё раз 123",
      "authorId": "4",
      "createdAt": "2026-06-06T06:20:32.829888+00:00",
      "updatedAt": "2026-06-07T04:40:44.061393+00:00"
    },
    // 3 поста
    ...
      "id": "10",
    ...
      "id": "5",
    ...
      "id": "3",
    ...
    {
      "id": "2",
      "title": "My First Blog Post",
      "content": "This is the content of my first post. Hello, world!",
      "authorId": "2",
      "createdAt": "2026-05-30T23:58:43.622518+00:00",
      "updatedAt": "2026-05-30T23:58:43.622518+00:00"
    }
  ],
  "total": "14",
  "limit": 5,
  "offset": 9
}
```

**Получение поста по ID:**
```bash
grpcurl -plaintext -proto proto/blog.proto \
  -d '{
    "id": 23       
  }' \
  localhost:50051 blog.BlogService/GetPost
```
```json
{
  "post": {
    "id": "23",
    "title": "Пост через gRPC",
    "content": "Текст через gRPC",
    "authorId": "13",
    "createdAt": "2026-06-09T00:16:30.318226+00:00",
    "updatedAt": "2026-06-09T00:16:30.318226+00:00"
  }
}
```

**Обновление поста:**
```bash
grpcurl -plaintext -proto proto/blog.proto \
  -H "authorization: Bearer $TOKEN" \
  -d '{
    "id": 23,
    "title": "Обновлённый контекнт через gRPC",
    "content": "Обновлённый текст через gRPC."
  }' \
  localhost:50051 blog.BlogService/UpdatePost
```
```json
=>
{
  "post": {
    "id": "23",
    "title": "Обновлённый контекнт через gRPC",
    "content": "Обновлённый текст через gRPC.",
    "authorId": "13",
    "createdAt": "2026-06-09T00:16:30.318226+00:00",
    "updatedAt": "2026-06-09T00:21:30.526221523+00:00"
  }
}
```
*Если не автор:
```bash
// 
  localhost:50051 blog.BlogService/UpdatePost
ERROR:
  Code: PermissionDenied
  Message: Вы не автор 

```

**Удаление поста:**
```bash
grpcurl -plaintext -proto proto/blog.proto \
  -H "authorization: Bearer $TOKEN" \
  -d '{
    "id": 23       
  }' \
  localhost:50051 blog.BlogService/DeletePost
```
```json
=>
{
  "success": true
}
```

### 2. CLI:
При запущенном сервере (см. пункт 1).

Полный список команд, которые проверял, аналогично - покрытие API публичного и protected.
Ошибочные варианты проверял.
Токен сохраняется в файлик и используется в последующих запросах, как описано выше.

**Команды для проверки: HTTP и gRPC попарно**
```bash
# Регистрация
cargo run -p blog-cli -- register --username "misha" --email "misha@misha.com" --password "12345678"
cargo run -p blog-cli -- --grpc register --username "misha2" --email "misha2@misha.com" --password "12345678"

# Логин
cargo run -p blog-cli -- login --username "misha" --password "12345678"
cargo run -p blog-cli -- --grpc login --username "misha2" --password "12345678"

# Создать пост
cargo run -p blog-cli -- create --title "Мой первый пост" --content "Содержание"
cargo run -p blog-cli -- --grpc create --title "Мой второй пост" --content "Содержание"

# Получить пост
cargo run -p blog-cli -- get --id 1
cargo run -p blog-cli -- --grpc get --id 2

# Обновить пост, только заголовок 
cargo run -p blog-cli -- update --id 1 --title "Обновлённый заголовок HTTP"
cargo run -p blog-cli -- --grpc update --id 1 --title "Обновлённый заголовок gRPC"

# Обновить пост, только содержимое 
cargo run -p blog-cli -- update --id 1 --content "Обновлённое содержание HTTP"
cargo run -p blog-cli -- --grpc update --id 1 --content "Обновлённое содержание gRPC"

# Удалить пост
cargo run -p blog-cli -- delete --id 1
cargo run -p blog-cli -- --grpc delete --id 1

# Список с limit и offset
cargo run -p blog-cli -- list --limit 2 --offset 1
cargo run -p blog-cli -- --grpc list --limit 2 --offset 1

# Дефлотные значения
cargo run -p blog-cli -- list
cargo run -p blog-cli -- --grpc list
```

Приведу несколько выводов выборочно:
**Регистрация**
```bash
cargo run -p blog-cli -- register --username "misha5" --email "misha@misha.com" --password "12345678"
```
```text
=>
Пользователь успешно зарегистрирован
ID пользователя: 14
Имя: misha5
Токен сохранён в .blog_token
```
**Создание поста**
```bash
cargo run -p blog-cli -- create --title "Мой первый пост" --content "Содержание"
```
```text
=>
Пост создан
ID поста: 24
Заголовок: Мой первый пост
ID автора: 14
Создан: 2026-06-09 00:32:19.746647245 UTC
```
**Обновление поста**
```bash
cargo run -p blog-cli -- --grpc update --id 1 --title "Обновлённый заголовок gRPC"
```
```text
=>
Пост успешно обновлён!
ID поста: 24
Заголовок: Обновлённый заголовок gRPC
Содержимое: Содержание
Время: 2026-06-09 00:32:57.854720540 UTC
```

**Список постов**
```bash
cargo run -p blog-cli -- --grpc list
```
```text
Постов: 14
Выведены: 1-10, Limit: 10
- #25: Test (пользователь #13)
- #24: Обновлённый заголовок gRPC (пользователь #14)
- #22: Пост через gRPC (пользователь #13)
- #21: Пост через gRPC (пользователь #13)
- #20: My First gRPC Post (пользователь #13)
- #18: Изначальный пост пользователя yandex (пользователь #11)
- #14: Мой второй пост (пользователь #10)
- #13: Обновлённый заголовок gRPC (пользователь #10)
- #12: Updated Title via gRPC (пользователь #7)
- #11: misha (пользователь #4)
```


### 3. WASM (frontend):
Описание frontend приведено в описании крата.
Единственное замечание, как и описано в теории и задании, что тестируется HTTP-часть сервера.

Для запуска:
Сборка:
```bash
cd ./blog-wasm/
wasm-pack build --target web
```
```text
=>
// [INFO]: ⬇️  Installing wasm-bindgen...
// [INFO]: Optimizing wasm binaries with `wasm-opt`...
// [INFO]: Optional fields missing from Cargo.toml: 'description', 'repository', and 'license'. These are not necessary, but recommended
// [INFO]: ✨   Done in 1m 04s
// [INFO]: 📦   Your wasm pkg is ready to publish at /home/m_rakul/Code/rust-yandex/practice/module_3_blog/blog-wasm/pkg.
```

Запуск сервера:
```bash
python3 -m http.server 8000
```

Открыть в браузере / VSCode.

При логине выводится сообщение в стиле Success "Вы успешно вошли (токен получен)".
Если пользователя не существует/ неверный пароль при входи - HTTP 401: Ошибка входа: (401): `{"error":"unauthorized"}`