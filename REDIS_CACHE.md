# Redis Кэширование в Velum

> **Высокопроизводительное кэширование на базе Redis для улучшения производительности Semaphore**

## 📖 Оглавление

- [Обзор](#обзор)
- [Архитектура](#архитектура)
- [Быстрый старт](#быстрый-старт)
- [Конфигурация](#конфигурация)
- [Использование](#использование)
- [API кэширования](#api-кэширования)
- [Инвалидация кэша](#инвалидация-кэша)
- [Метрики](#метрики)
- [Best Practices](#best-practices)

---

## 📋 Обзор

Redis кэширование в Velum предоставляет:

- **Кэширование сессий** - быстрое хранение и проверка пользовательских сессий
- **Кэширование запросов** - кэширование результатов частых запросов к БД
- **Кэширование сущностей** - проекты, задачи, шаблоны, инвентари
- **HTTP кэширование** - кэширование ответов API
- **Метрики hit/miss** - мониторинг эффективности кэша

**Преимущества:**

| Преимущество | Описание |
|--------------|----------|
| **Производительность** | Уменьшение нагрузки на БД до 90% |
| **Масштабируемость** | Поддержка кластерного режима Redis |
| **Гибкость** | Настраиваемый TTL для разных типов данных |
| **Надёжность** | Автоматическое переподключение и retry |

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                    Velum                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │              Cache Service                        │  │
│  │  ┌─────────────────┐  ┌─────────────────────────┐ │  │
│  │  │  Session Cache  │  │  Query Cache            │ │  │
│  │  │  • JWT tokens   │  │  • Projects             │ │  │
│  │  │  • User data    │  │  • Tasks                │ │  │
│  │  │  • Permissions  │  │  • Templates            │ │  │
│  │  └─────────────────┘  └─────────────────────────┘ │  │
│  │  ┌─────────────────┐  ┌─────────────────────────┐ │  │
│  │  │  HTTP Cache     │  │  Entity Cache           │ │  │
│  │  │  • GET /api/*   │  │  • Users                │ │  │
│  │  │  • X-Cache HIT  │  │  • Repositories         │ │  │
│  │  │  • X-Cache MISS │  │  • Environments         │ │  │
│  │  └─────────────────┘  └─────────────────────────┘ │  │
│  └───────────────────────────────────────────────────┘  │
│                          │                              │
│                    Redis Client                         │
│                    (Connection Manager)                 │
└──────────────────────────┼──────────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────────┐
│                      Redis Server                       │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Keys:                                            │  │
│  │  • semaphore:session:{token}                      │  │
│  │  • semaphore:user:id:{id}                         │  │
│  │  • semaphore:project:{id}                         │  │
│  │  • semaphore:http_cache:{hash}                    │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 🚀 Быстрый старт

### 1. Установка Redis

```bash
# Ubuntu/Debian
sudo apt-get install redis-server

# macOS
brew install redis

# Docker
docker run -d -p 6379:6379 redis:latest
```

### 2. Настройка Semaphore

```bash
# В .env файле
SEMAPHORE_REDIS_ENABLED=true
SEMAPHORE_REDIS_URL=redis://localhost:6379
SEMAPHORE_REDIS_PREFIX=semaphore:
SEMAPHORE_REDIS_DEFAULT_TTL=300
```

### 3. Запуск

```bash
cd rust
cargo run -- server
```

### 4. Проверка подключения

```
INFO Connecting to Redis at redis://localhost:6379
INFO Successfully connected to Redis
```

---

## ⚙️ Конфигурация

### Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_REDIS_ENABLED` | Включить кэширование | `false` |
| `SEMAPHORE_REDIS_URL` | URL подключения к Redis | `redis://localhost:6379` |
| `SEMAPHORE_REDIS_PREFIX` | Префикс для ключей | `semaphore:` |
| `SEMAPHORE_REDIS_DEFAULT_TTL` | TTL по умолчанию (сек) | `300` |
| `SEMAPHORE_REDIS_MAX_RETRIES` | Максимум попыток подключения | `3` |
| `SEMAPHORE_REDIS_CONNECTION_TIMEOUT` | Таймаут подключения (сек) | `5` |

### Конфигурационный файл (config.json)

```json
{
  "redis": {
    "url": "redis://localhost:6379",
    "prefix": "semaphore:",
    "default_ttl": 300,
    "enabled": true
  }
}
```

### Кластерный режим

```json
{
  "redis": {
    "url": "redis://node1:6379,redis://node2:6379,redis://node3:6379",
    "prefix": "semaphore:",
    "default_ttl": 300,
    "enabled": true
  }
}
```

---

## 💡 Использование

### Кэширование сессий

```rust
use crate::services::{CacheService, SessionData};

// Сохранение сессии
let session = SessionData::new(&user, 3600);
cache_service.save_session(&token, &session).await?;

// Получение сессии
let session = cache_service.get_session(&token).await?;

// Удаление сессии (logout)
cache_service.delete_session(&token).await?;

// Продление сессии
cache_service.extend_session(&token).await?;
```

### Кэширование пользователей

```rust
// Кэширование пользователя
cache_service.cache_user(&user).await?;

// Получение по ID
let user = cache_service.get_user_by_id(1).await?;

// Получение по username
let user = cache_service.get_user_by_username("admin").await?;

// Инвалидация
cache_service.invalidate_user(user.id, &user.username).await?;
```

### Кэширование проектов

```rust
// Кэширование проекта
cache_service.cache_project(project.id, &project).await?;

// Получение проекта
let project = cache_service.get_project::<Project>(1).await?;

// Инвалидация проекта и связанных данных
cache_service.invalidate_project(1).await?;
```

### Кэширование задач

```rust
// Кэширование задач проекта
cache_service.cache_project_tasks(project_id, Some("running"), &tasks).await?;

// Получение задач
let tasks = cache_service.get_project_tasks::<Vec<Task>>(
    project_id,
    Some("running")
).await?;

// Инвалидация задач проекта
cache_service.invalidate_project_tasks(project_id).await?;
```

---

## 🔌 API кэширования

### Структуры ключей

```rust
use crate::services::CacheKeys;

// Сессии
CacheKeys::session("token123")  // "session:token123"

// Пользователи
CacheKeys::user_id(1)           // "user:id:1"
CacheKeys::user_username("admin") // "user:username:admin"

// Проекты
CacheKeys::project(42)          // "project:42"
CacheKeys::project_tasks(1, Some("running")) // "project:1:tasks:running"

// Шаблоны
CacheKeys::template(1)          // "template:1"

// Инвентари
CacheKeys::inventory(1)         // "inventory:1"

// Репозитории
CacheKeys::repository(1)        // "repository:1"

// Окружения
CacheKeys::environment(1)       // "environment:1"
```

### TTL для разных типов данных

| Тип данных | TTL (сек) | Описание |
|------------|-----------|----------|
| **Сессии** | 3600 | 1 час |
| **Пользователи** | 300 | 5 минут |
| **Проекты** | 600 | 10 минут |
| **Задачи** | 60 | 1 минута |
| **Шаблоны** | 300 | 5 минут |
| **HTTP кэш** | 300 | 5 минут |

---

## 🔄 Инвалидация кэша

### Автоматическая инвалидация

Кэш автоматически инвалидируется при:

- **Создании сущности** - кэшируется новая сущность
- **Обновлении сущности** - старый кэш удаляется, новый записывается
- **Удалении сущности** - кэш удаляется

### Ручная инвалидация

```rust
// Инвалидация проекта
cache_service.invalidate_project(project_id).await?;

// Инвалидация задач
cache_service.invalidate_project_tasks(project_id).await?;

// Инвалидация пользователя
cache_service.invalidate_user(user_id, &username).await?;

// Инвалидация шаблона
cache_service.invalidate_template(template_id).await?;

// Инвалидация инвентаря
cache_service.invalidate_inventory(inventory_id).await?;
```

### Инвалидация HTTP кэша

```rust
use crate::api::middleware::cache::invalidate_http_cache;

invalidate_http_cache(&redis, "/api/projects/*").await?;
```

---

## 📊 Метрики

### Получение статистики

```rust
let stats = cache_service.get_stats().await;

println!("Hits: {}", stats.hits);
println!("Misses: {}", stats.misses);
println!("Hit Ratio: {:.2}%", stats.hit_ratio());
println!("Total Requests: {}", stats.total_requests());
println!("Errors: {}", stats.errors);
```

### Prometheus метрики

```rust
// semaphore_cache_hits_total - Всего попаданий в кэш
// semaphore_cache_misses_total - Всего промахов кэша
// semaphore_cache_errors_total - Всего ошибок
// semaphore_cache_hit_ratio - Процент попаданий
// semaphore_cache_keys_count - Количество ключей в кэше
```

### Dashboard

```json
{
  "title": "Semaphore Cache Metrics",
  "panels": [
    {
      "title": "Cache Hit Ratio",
      "targets": ["semaphore_cache_hit_ratio"]
    },
    {
      "title": "Cache Requests",
      "targets": [
        "semaphore_cache_hits_total",
        "semaphore_cache_misses_total"
      ]
    }
  ]
}
```

---

## 📚 Best Practices

### 1. Правильный выбор TTL

```rust
// Короткий TTL для часто меняющихся данных
cache_service.cache_project_tasks(id, None, &tasks, 60).await?;

// Длинный TTL для стабильных данных
cache_service.cache_project(id, &project, 600).await?;
```

### 2. Инвалидация при изменениях

```rust
// Всегда инвалидируйте кэш при обновлении
async fn update_project(&self, project: &Project) -> Result<()> {
    // Обновление в БД
    self.db.update_project(project).await?;
    
    // Инвалидация кэша
    self.cache.invalidate_project(project.id).await?;
    
    Ok(())
}
```

### 3. Graceful degradation

```rust
// Код должен работать даже если Redis недоступен
let cached_user = cache_service.get_user_by_id(id).await?;
if let Some(user) = cached_user {
    return Ok(user);
}

// Fallback к БД
let user = db.get_user_by_id(id).await?;
```

### 4. Мониторинг hit ratio

```rust
// Alert если hit ratio < 50%
if stats.hit_ratio() < 50.0 {
    warn!("Cache hit ratio is low: {:.2}%", stats.hit_ratio());
}
```

### 5. Использование для сессий

```rust
// Сессии должны иметь разумный TTL
let session_ttl = 3600; // 1 час
cache_service.save_session(&token, &session).await?;
```

---

## 🔧 Troubleshooting

### Проблема: Redis не подключается

**Решение:**
```bash
# Проверьте что Redis запущен
redis-cli ping  # Должен вернуть PONG

# Проверьте firewall
sudo ufw allow 6379/tcp
```

### Проблема: Низкий hit ratio

**Решение:**
1. Увеличьте TTL для часто запрашиваемых данных
2. Проверьте что инвалидация не происходит слишком часто
3. Добавьте кэширование для новых endpoints

### Проблема: Высокое потребление памяти

**Решение:**
```bash
# Настройте maxmemory в redis.conf
maxmemory 256mb
maxmemory-policy allkeys-lru
```

---

## 🔗 Ссылки

- [Redis Documentation](https://redis.io/documentation)
- [Redis Rust Client](https://github.com/redis-rs/redis-rs)
- [Semaphore Cache Service](rust/src/services/cache_service.rs)
- [Semaphore Cache Module](rust/src/cache.rs)

---

*Последнее обновление: 10 марта 2026 г.*
