## Lesson 3.2.6 — Bank API Postgres

Продолжение Bank API: вместо in-memory хранилища используется PostgreSQL через `sqlx`, добавлен сервис актуальных курсов валют.

### Подготовка

1. Создай `.env` на основе `env.example`.
2. Подними PostgreSQL и создай базу `bank_api`.
MR: здесь может потребоваться такая команда
sudo -u postgres psql -c "CREATE DATABASE bank_db;"

3. Запусти миграции автоматически при старте (`sqlx::migrate!()`).

### Запуск

```bash
cargo run
```

Сервер слушает `http://127.0.0.1:8080`. Все маршруты `/api/*` сохранены (регистрация, логин, операции со счётами) и теперь читают/пишут реальные данные в БД. Endpoint `/api/exchange/{from}/{to}` берёт курсы с внешнего API.

# Запуск проекта

> Создайте базу данных PostgreSQL:
# Создать БД и задать пароль суперпользователю postgres
## Убедитесь, что PostgreSQL запущен
## Варианты: 

createdb bank_db

sudo -u postgres psql -c "CREATE DATABASE bank_db;"
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';" 

(?): psql -U postgres -c "CREATE DATABASE bank_db;" 

## Установите переменную окружения:

> export DATABASE_URL="postgresql://postgres:postgres@localhost/bank_db" 

## Соберите проект:

> cargo build 

## Запустите сервер:

> cargo run 

## Миграции применятся автоматически при старте — создадутся таблицы accounts и users в реальной БД.

> Проверьте, что данные сохраняются в БД:

## После создания счёта через API, проверьте в БД:
psql -U postgres -d bank_db -c "SELECT * FROM accounts;"

// Для m_rakul:
psql -d bank_db -c "SELECT * FROM accounts;"

## После депозита проверьте обновление:
psql -U postgres -d bank_db -c "SELECT id, balance, updated_at FROM accounts WHERE id = 1;" 

Убедитесь, что все операции с аккаунтами сохраняются в реальную PostgreSQL БД и переживут перезапуск сервера.

# Запуск команд

## Сначала используется внешнее API

1. Регистрация пользователя:
> Запрос:
curl -X POST http://127.0.0.1:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"123"}'

> Ответ:
{"email":"alice@example.com","user_id":"f3683148-1558-43c6-a13c-6dee1b8a66e1"}

2. Авторизация с получением токена
> Если неверный пароль - unathorized:
> Запрос:
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"SecurePass123!"}'
> Ответ:
{"error":"unauthorized"}


Верный пароль:
> Запрос:
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"123"}'
> Ответ:
{"access_token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJmMzY4MzE0OC0xNTU4LTQzYzYtYTEzYy02ZGVlMWI4YTY2ZTEiLCJleHAiOjE3NzkwNjM5MDksImlhdCI6MTc3OTA2MDMwOX0.g-1HQpIYa3BVzfsY8lfrRo__cV_SbFIOI9sSzU6ji2w"}

> Вот так можно сохранить токен в переменную:
TOKEN=$(curl -s -X POST http://127.0.0.1:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"alice@example.com","password":"123"}' \
  | jq -r .access_token)

Проверка:
echo $TOKEN

## Проверка в БД в табличке users:
psql -d bank_db -c "SELECT id, email, created_at FROM users WHERE email = 'alice@example.com';"

                  id                  |       email       |          created_at           
--------------------------------------+-------------------+-------------------------------
 f3683148-1558-43c6-a13c-6dee1b8a66e1 | alice@example.com | 2026-05-18 02:23:30.039974+03
(1 row)


# После регистрации и логина

## Создание счёта с подстановкой токена:
curl -v -X POST http://127.0.0.1:8080/api/accounts \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"initial": 1000}'

## Создание счёта
curl -X POST http://127.0.0.1:8080/api/accounts \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"initial_balance": 1000}'


## Получение баланса
curl http://127.0.0.1:8080/api/accounts/1 \
  -H "Authorization: Bearer YOUR_TOKEN"

## Получение курса валют (реальный API, возвращает актуальные курсы)
curl http://127.0.0.1:8080/api/exchange/USD/EUR \
  -H "Authorization: Bearer YOUR_TOKEN"
> Ответ: {"from":"USD","to":"EUR","rate":0.865}

curl http://127.0.0.1:8080/api/exchange/USD/RUB \
  -H "Authorization: Bearer YOUR_TOKEN"
> Ответ: {"from":"USD","to":"RUB","rate":80.95} 

