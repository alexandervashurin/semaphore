# 🔒 Rate Limiting в Velum API

> **Версия:** 1.0  
> **Дата:** 2026-03-30

---

## 📋 Обзор

Velum использует **Rate Limiting middleware** для защиты API от злоупотреблений и DDoS атак.

### Возможности

- ✅ Разные лимиты для разных endpoints (API, Auth, Sensitive)
- ✅ HTTP заголовки `X-RateLimit-*` для клиентов
- ✅ Автоматическая очистка старых записей (каждые 10 минут)
- ✅ Извлечение IP из `X-Forwarded-For` заголовка
- ✅ Поддержка burst mode для API

---

## ⚙️ Конфигурация

### Типы Rate Limiter

| Тип | Лимит | Период | Burst | Применение |
|-----|-------|--------|-------|------------|
| **API** | 100 req | 60 сек | 20 | Обычные API запросы |
| **Auth** | 5 req | 60 сек | — | Логин, регистрация |
| **Sensitive** | 10 req | 60 сек | — | Чувствительные операции |

### Пример конфигурации

```rust
use crate::api::middleware::rate_limiter::{RateLimiter, RateLimitConfig};

// API rate limiter
let api_limiter = RateLimiter::for_api();
// 100 запросов в минуту, burst 20

// Auth rate limiter
let auth_limiter = RateLimiter::for_auth();
// 5 запросов в минуту (защита от brute force)

// Sensitive operations
let sensitive_limiter = RateLimiter::for_sensitive();
// 10 запросов в минуту

// Custom configuration
let custom_limiter = RateLimiter::new(RateLimitConfig {
    max_requests: 50,
    period_secs: 60,
    burst_size: Some(10),
});
```

---

## 📡 HTTP Заголовки

### Ответ сервера

Все ответы API включают заголовки rate limiting:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
```

### При превышении лимита

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
Retry-After: 60

{
  "error": "Rate limit exceeded. Try again later.",
  "retry_after": 60
}
```

---

## 🔐 Защищённые Endpoints

### Auth Endpoints (5 req/min)

```
POST /api/auth/login
POST /api/auth/recovery
POST /api/auth/totp/start
POST /api/auth/totp/confirm
```

### API Endpoints (100 req/min)

Все остальные endpoints API.

### Sensitive Operations (10 req/min)

```
POST /api/users/{id}/password
DELETE /api/audit-log/clear
POST /api/project/{id}/backup
```

---

## 🧪 Тестирование

### Запуск тестов

```bash
cd rust && cargo test api::middleware::rate_limiter
```

### Примеры тестов

```rust
#[tokio::test]
async fn test_rate_limiter_allows_within_limit() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 5,
        period_secs: 60,
        burst_size: None,
    });

    for i in 0..5 {
        assert!(limiter.is_allowed("test").await);
    }

    assert!(!limiter.is_allowed("test").await);
}

#[tokio::test]
async fn test_rate_limiter_get_remaining() {
    let limiter = RateLimiter::new(RateLimitConfig {
        max_requests: 10,
        period_secs: 60,
    });

    assert_eq!(limiter.get_remaining("test").await, 10);
    
    limiter.is_allowed("test").await;
    limiter.is_allowed("test").await;
    
    assert_eq!(limiter.get_remaining("test").await, 8);
}
```

---

## 📊 Мониторинг

### Метрики Prometheus

```
# Запросов отклонено (429)
http_requests_rejected_total{reason="rate_limit"}

# Текущий лимит
rate_limiter_limit{endpoint="api"} 100
rate_limiter_limit{endpoint="auth"} 5

# Оставшиеся запросы
rate_limiter_remaining{ip="192.168.1.1"} 95
```

### Логирование

```rust
// Включение логирования очистки
RUST_LOG=debug ./velum server

// Пример лога
DEBUG RateLimiter: cleaned up 150 stale entries
```

---

## 🛡️ Рекомендации

### Для клиентов

1. **Следите за заголовками**
   ```python
   response = requests.get(url)
   remaining = int(response.headers.get('X-RateLimit-Remaining', 100))
   if remaining < 10:
       time.sleep(60)  # Подождите восстановления лимита
   ```

2. **Используйте экспоненциальную задержку**
   ```python
   import time
   
   for attempt in range(5):
       response = requests.get(url)
       if response.status_code == 429:
           retry_after = int(response.headers.get('Retry-After', 60))
           time.sleep(retry_after * (2 ** attempt))
       else:
           break
   ```

3. **Кэшируйте ответы**
   ```python
   from functools import lru_cache
   
   @lru_cache(maxsize=100)
   def get_data(endpoint):
       return api.get(endpoint)
   ```

### Для сервера

1. **Настройте лимиты под вашу нагрузку**
   ```rust
   // Для high-load систем
   RateLimitConfig {
       max_requests: 1000,
       period_secs: 60,
       burst_size: Some(100),
   }
   ```

2. **Мониторьте отклонённые запросы**
   ```bash
   # grep "429" в логах
   tail -f logs/backend.log | grep "429"
   ```

3. **Используйте whitelist для доверенных IP**
   ```rust
   if is_trusted_ip(&ip) {
       return next.run(req).await;
   }
   ```

---

## 🔧 Расширение

### Добавление нового типа лимитера

```rust
impl RateLimiter {
    pub fn for_export() -> Self {
        Self::new(RateLimitConfig {
            max_requests: 3,      // 3 экспорта в минуту
            period_secs: 60,
            burst_size: None,
        })
    }
}
```

### Применение к новому endpoint

```rust
// В routes.rs
use crate::api::middleware::rate_limiter::app_sensitive_rate_limit;

.route("/api/sensitive", post(app_sensitive_rate_limit))
```

---

## 📈 Статистика

| Показатель | Значение |
|------------|----------|
| **Тестов** | 4 passed ✅ |
| **Лимит API** | 100 req/min |
| **Лимит Auth** | 5 req/min |
| **Очистка** | каждые 10 мин |
| **Заголовки** | X-RateLimit-* |

---

## 🔗 Ссылки

- [Исходный код](../rust/src/api/middleware/rate_limiter.rs)
- [API Документация](./API.md)
- [Безопасность](./SECURITY.md)
