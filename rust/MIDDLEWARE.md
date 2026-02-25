# Middleware Documentation

## Обзор

Модуль `middleware` предоставляет набор промежуточных слоёв для обработки HTTP-запросов в Semaphore UI.

## Доступные Middleware

### 1. Request Logger (`request_logger`)

Логирует все входящие запросы и исходящие ответы.

**Функциональность:**
- Логирует метод и путь запроса
- Измеряет время выполнения
- Логирует статус ответа
- Поддерживает X-Request-ID для трассировки

**Пример использования:**
```rust
use crate::api::middleware::request_logger;

Router::new()
    .layer(middleware::from_fn(request_logger))
```

### 2. Auth Middleware (`auth_middleware`)

Проверяет аутентификацию пользователя.

**Функциональность:**
- Проверяет заголовок `Authorization: Bearer <token>`
- Пропускает публичные пути (/api/auth/, /api/health)
- Возвращает 401 при отсутствии токена

**Пример использования:**
```rust
use crate::api::middleware::auth_middleware;

Router::new()
    .layer(middleware::from_fn_with_state(state, auth_middleware))
```

### 3. Permission Middleware (`permission_middleware`)

Проверяет права доступа пользователя к ресурсам.

**Функциональность:**
- Извлекает ID проекта из пути
- Проверяет права доступа (TODO: реализовать)
- Возвращает 403 при отсутствии прав

### 4. Request Size Limit (`request_size_limit`)

Ограничивает размер запроса.

**Функциональность:**
- Проверяет заголовок `Content-Length`
- Лимит: 10 MB
- Возвращает 413 при превышении лимита

### 5. Security Headers (`security_headers`)

Добавляет заголовки безопасности.

**Добавляемые заголовки:**
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Content-Security-Policy: default-src 'self'`
- `Strict-Transport-Security: max-age=31536000; includeSubDomains`

### 6. Timeout Middleware (`timeout_middleware`)

Устанавливает таймаут на обработку запроса.

**Функциональность:**
- Таймаут: 30 секунд
- Возвращает 504 при превышении времени

### 7. Panic Handler (`panic_handler`)

Обрабатывает паники в обработчиках запросов.

**Функциональность:**
- Перехватывает паники
- Логирует ошибку
- Возвращает 500 Internal Server Error

## Структура ответа об ошибке

```json
{
  "error": "Описание ошибки",
  "code": "CODE_ERROR",
  "details": {"key": "value"}
}
```

## Примеры кодов ошибок

| Код | Описание |
|-----|----------|
| `AUTH_REQUIRED` | Требуется аутентификация |
| `INVALID_TOKEN` | Неверный токен |
| `PAYLOAD_TOO_LARGE` | Превышен размер запроса |
| `TIMEOUT` | Превышено время ожидания |
| `PANIC` | Внутренняя паника |
| `PERMISSION_DENIED` | Доступ запрещён |

## Публичные пути

Следующие пути не требуют аутентификации:
- `/api/auth/*` - аутентификация
- `/api/health` - проверка здоровья
- `/api/version` - версия API
- `/favicon.ico` - иконка
- `/static/*` - статические файлы

## Рекомендуемый стек middleware

```rust
use crate::api::middleware::{
    request_logger,
    security_headers,
    request_size_limit,
    auth_middleware,
    permission_middleware,
    timeout_middleware,
};

let app = Router::new()
    // API routes
    .merge(api_routes())
    // Middleware (порядок важен!)
    .layer(middleware::from_fn(security_headers))
    .layer(middleware::from_fn(request_logger))
    .layer(middleware::from_fn(request_size_limit))
    .layer(middleware::from_fn_with_state(state, auth_middleware))
    .layer(middleware::from_fn_with_state(state, permission_middleware))
    .layer(middleware::from_fn(timeout_middleware))
    .with_state(state);
```

## Тестирование

Middleware включают unit-тесты:

```bash
cargo test middleware::tests
```

### Доступные тесты

- `test_error_response` - проверка структуры ErrorResponse
- `test_is_public_path` - проверка определения публичных путей
- `test_extract_project_id` - проверка извлечения ID проекта

## Расширение

Для добавления собственного middleware:

```rust
pub async fn my_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Ваша логика здесь
    
    Ok(next.run(request).await)
}
```

## Отладка

Включите детальное логирование:

```bash
RUST_LOG=debug ./target/debug/semaphore server
```

Логи middleware будут включать:
- Метод и путь запроса
- Время выполнения
- Статус ответа
- Request ID
