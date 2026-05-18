# Lesson 3.2.3 — Bank API

Решение практики: HTTP API для банковских операций на Actix Web.

## Возможности

- Создание счёта с начальным балансом.
- Получение баланса.
- Внесение и вывод средств.
- Перевод между счетами с проверкой остатка.

## Запуск

```bash
cargo run
```

Сервер слушает `http://127.0.0.1:8080`. Проверить можно через `curl`:

```bash
curl -X POST http://127.0.0.1:8080/api/accounts -H "Content-Type: application/json" \
  -d '{"id":1,"initial":10000}'
```

┌─────────────────────────────────────────────────────────────────┐
│                        Presentation Layer                       │
│  • API endpoints (handlers.rs)                                  │
│  • DTOs (data transfer objects)                                 │
│  • HTTP request/response handling                              │
├─────────────────────────────────────────────────────────────────┤
│                      Application Layer                          │
│  • Use cases (bank_service.rs)                                  │
│  • Orchestrates domain logic + repositories                     │
│  • Business workflow coordination                               │
├─────────────────────────────────────────────────────────────────┤
│                         Domain Layer                            │
│  • Business entities (Account, Transfer)                        │
│  • Business rules (validation, constraints)                     │
│  • Value objects (Amount)                                       │
├─────────────────────────────────────────────────────────────────┤
│                         Data Layer                              │
│  • Repository trait (account_repository.rs)                     │
│  • Data access implementation (InMemoryAccountRepository)       │
│  • Database/Storage abstraction                                 │
└─────────────────────────────────────────────────────────────────┘

## Примеры использования:
### 1. Health check
curl http://localhost:8080/health
# {"status":"ok"}

### 2. Create account
curl -X POST http://localhost:8080/accounts -H "Content-Type: application/json" -d '{"id": 1, "initial": 1000}'
# {"id":1,"balance":1000}

### 3. Get account
curl http://localhost:8080/accounts/1
# {"id":1,"balance":1000}

### 4. Deposit money
curl -X POST http://localhost:8080/accounts/1/deposit -H "Content-Type: application/json" -d '{"amount": 500}'
# {"id":1,"balance":1500}

### 5. Withdraw money
curl -X POST http://localhost:8080/accounts/1/withdraw -H "Content-Type: application/json" -d '{"amount": 200}'
# {"id":1,"balance":1300}

### 6. Transfer money
curl -X POST http://localhost:8080/transfers -H "Content-Type: application/json" -d '{"from": 1, "to": 2, "amount": 100}'
# {"status":"transferred"}
