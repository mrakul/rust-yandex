## Lesson 3.2.5 — Bank API Security

Продолжение Bank API: добавлены защита CORS, JWT‑авторизация, Argon2 для паролей, кастомные middleware и ошибки.

### Запуск

1. Создай `.env` или экспортируй переменные:
   ```
   HOST=127.0.0.1
   PORT=8080
   JWT_SECRET=dev_super_secret_change_me_please
   CORS_ORIGINS=http://localhost:3000
   ```
2. Запусти сервер:
   ```bash
   cargo run
   ```

### Что реализовано

- `RequestIdMiddleware`, `TimingMiddleware`, `JwtAuthMiddleware`.
- Регистрация и логин с Argon2 + JWT.
- Все банковские действия защищены токеном, `AuthenticatedUser` берётся из extensions.
- Гибкий CORS и security headers.
- Единая система ошибок `BankError` → JSON.


## Маршруты API

### Публичные маршруты (без JWT):
GET  /api/health                 # Health check
POST /api/auth/register           # Регистрация (новый)
POST /api/auth/login             # Логин с паролем (новый)
POST /api/auth/token             # Получение JWT (из 3.2.4)

### Защищённые маршруты (с JWT):
POST /api/accounts                # Создание счёта (защитить)
GET  /api/accounts/{id}           # Получение баланса (защитить)
POST /api/accounts/{id}/deposit   # Пополнение (защитить)
POST /api/accounts/{id}/withdraw  # Снятие (защитить)
POST /api/transfers               # Перевод (защитить) 

## Примеры запросов
### Регистрация:

curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure123"}' 

### Логин:

curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure123"}'
# Ответ: {"access_token": "eyJhbGc..."} 

# Создание счёта с JWT:

TOKEN="eyJhbGc..." # из ответа логина

curl -X POST http://localhost:8080/api/accounts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"id": 1, "initial": 1000}' 

# Полный флоу: регистрация → логин → создание счёта:

### 1. Регистрация
curl -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure123"}'
### Ответ: {"id": "uuid-...", "email": "user@example.com"}

### 2. Логин
LOGIN_RESPONSE=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "secure123"}')
echo $LOGIN_RESPONSE
### Ответ: {"access_token": "eyJhbGc..."}

### 3. Извлечение токена
TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.access_token')
echo "Token: $TOKEN"

### 4. Создание счёта с JWT
curl -X POST http://localhost:8080/api/accounts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"id": 1, "initial": 1000}'
### Ответ: {"id": 1}

### 5. Проверка баланса
curl -X GET http://localhost:8080/api/accounts/1 \
  -H "Authorization: Bearer $TOKEN"
### Ответ: {"id": 1, "balance": 1000}

### 6. Попытка доступа без токена (должна вернуть 401)
curl -X GET http://localhost:8080/api/accounts/1
### Ответ: {"error": "missing bearer"} 

Тестирование CORS:

### Запрос с разрешённого origin (должен пройти)
curl -H "Origin: http://localhost:3000" \
     -H "Access-Control-Request-Method: POST" \
     -X OPTIONS http://localhost:8080/api/accounts

### Запрос с неразрешённого origin (должен быть заблокирован)
curl -H "Origin: https://evil.com" \
     -X POST http://localhost:8080/api/accounts 

# Пример .env-файла:

###  Логирование
RUST_LOG=actix_web=info,bank_api=debug

### CORS
ALLOWED_ORIGINS=http://localhost:3000,https://myapp.com

### JWT
JWT_SECRET=your-super-secret-key-change-in-production

### Порт сервера
PORT=8080 

## Security headers (рекомендуется):

.use(
  actix_web::middleware::DefaultHeaders::new()
    .add(("X-Content-Type-Options", "nosniff"))
    .add(("Referrer-Policy", "no-referrer"))
    .add(("Permissions-Policy", "geolocation=()"))
    .add(("Cross-Origin-Opener-Policy", "same-origin"))
) 

## Из задания
1. CORS-настройка:

    Настройте CORS middleware с whitelist origins из переменных окружения.
    Разрешите только методы: GET, POST, PUT, DELETE.
    Разрешите заголовки: Content-Type, Authorization.
    Установите max_age = 3600.
    Для тестирования: попробуйте отправить запрос с неразрешённого origin и убедитесь, что он блокируется.

2. JWT-авторизация:

    Создайте middleware для JWT-авторизации.
    Реализуйте экстрактор AuthenticatedUser для извлечения user_id из токена.
    Защитите маршруты создания счёта, переводов, депозитов и снятий JWT middleware.
    Оставьте /api/health и /api/auth/token публичными.
    Добавьте endpoint POST /api/auth/register для регистрации с паролем.

3. Безопасное хранение паролей:

    Реализуйте регистрацию пользователя с хешированием пароля через argon2.
    Добавьте проверку пароля при логине (endpoint POST /api/auth/login).
    Храните хеши паролей, никогда не храните исходные пароли.
    При успешном логине возвращайте JWT-токен.

4. Защита от SQL-инъекций:

    Убедитесь, что все запросы к БД параметризованы.
    Продемонстрируйте разницу между безопасным и небезопасным кодом (в комментариях).

5. CSRF-токены (опционально, задание со звёздочкой):

    Добавьте middleware для генерации CSRF-токенов.
    Реализуйте проверку CSRF для всех POST/PUT/DELETE-запросов.
    Используйте экстрактор CsrfToken для автоматической проверки.

## Структура проекта
Расширьте существующую структуру из предыдущего урока:

bank-api-clean-arch/
├── Cargo.toml
├── .env
└── src/
    ├── main.rs
    ├── domain/
    │   ├── mod.rs
    │   ├── error.rs              # Расширить: добавить Validation, NotFound, Unauthorized, Internal
    │   └── user.rs               # Новый: модель User
    ├── application/
    │   ├── mod.rs
    │   ├── bank_service.rs
    │   └── auth_service.rs       # Новый: сервис авторизации (register, login)
    ├── data/
    │   ├── mod.rs
    │   ├── account_repository.rs
    │   └── user_repository.rs    # Новый: репозиторий пользователей (InMemoryUserRepository)
    ├── presentation/
    │   ├── mod.rs
    │   ├── handlers.rs           # Добавить: register, login handlers
    │   ├── dto.rs                 # Добавить: RegisterRequest, LoginRequest
    │   ├── middleware.rs          # Добавить: JwtAuthMiddleware
    │   └── auth.rs                # Обновить: AuthenticatedUser (из extensions)
    └── infrastructure/
        ├── mod.rs
        ├── config.rs
        ├── logging.rs
        └── security.rs            # Расширить: Argon2 (hash_password, verify_password), JWT (generate_token с String) 

Важно:

    В domain/error.rs добавьте новые варианты в DomainError: Validation, NotFound, Unauthorized, Internal.
    В main.rs добавьте инициализацию user_repository и auth_service, подключите JwtAuthMiddleware перед другими middleware.
    В security.rs используйте rand_core::OsRng (не argon2::password_hash::rand_core::OsRng).
    В security.rs функция generate_token должна принимать user_id: &str (не u32).

