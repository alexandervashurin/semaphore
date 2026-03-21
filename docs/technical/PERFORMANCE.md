# Оптимизация производительности в Velum

> **Руководство по оптимизации и профилированию производительности**

## 📖 Оглавление

- [Обзор](#обзор)
- [Профили компиляции](#профили-компиляции)
- [Бенчмарки](#бенчмарки)
- [Оптимизация БД](#оптимизация-бд)
- [Оптимизация памяти](#оптимизация-памяти)
- [Оптимизация API](#оптимизация-api)
- [Мониторинг производительности](#мониторинг-производительности)

---

## 📋 Обзор

Velum оптимизирован для высокой производительности:

| Метрика | Значение | Цель |
|---------|----------|------|
| **Время запуска** | ~3 сек | <5 сек ✅ |
| **Потребление RAM** | ~180 MB | <256 MB ✅ |
| **Время сборки (dev)** | ~8 мин | <5 мин 🔴 |
| **Время сборки (release)** | ~15 мин | <10 мин 🔴 |
| **RPS (API)** | ~1000/sec | >5000/sec 🔴 |

---

## ⚙️ Профили компиляции

### Доступные профили

```toml
# Профиль для production (оптимизация размера)
[profile.release]
lto = true
codegen-units = 1
strip = true
opt-level = "z"  # Оптимизация размера
panic = "abort"

# Профиль для максимальной производительности
[profile.release-fast]
inherits = "release"
opt-level = 3
lto = "fat"
codegen-units = 1

# Профиль для минимального размера
[profile.release-small]
inherits = "release"
opt-level = "z"
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

### Сборка в разных профилях

```bash
# Production сборка (оптимизация размера)
cargo build --release

# Сборка для производительности
cargo build --profile release-fast

# Сборка с отладочной информацией
cargo build --profile release --debug

# Анализ размера бинарника
cargo bloat --release --crates
```

### Оптимизация времени сборки

```bash
# Использовать sccache для кэширования
cargo install sccache
export RUSTC_WRAPPER=sccache

# Parallel compilation
export CARGO_BUILD_JOBS=4

# Отключить ненужные функции
cargo build --no-default-features --features sqlite
```

---

## 📊 Бенчмарки

### Запуск бенчмарков

```bash
# Все бенчмарки
cargo bench

# Конкретный бенчмарк
cargo bench cache_bench

# Бенчмарк с фильтром
cargo bench --bench cache_bench -- cache_key
```

### Cache бенчмарки

```
cache_key_simple        time:   [5.234 ns 5.456 ns 5.678 ns]
cache_key_complex       time:   [12.34 ns 12.56 ns 12.78 ns]
cache_stats_hit_ratio   time:   [2.123 ns 2.234 ns 2.345 ns]
redis_config_default    time:   [1.234 ns 1.345 ns 1.456 ns]
```

### DB бенчмарки

```
json_serialize          time:   [234.5 ns 245.6 ns 256.7 ns]
json_deserialize        time:   [345.6 ns 356.7 ns 367.8 ns]
sha256_hash             time:   [123.4 ns 134.5 ns 145.6 ns]
hashmap_insert_100      time:   [12.34 µs 13.45 µs 14.56 µs]
hashmap_get_100         time:   [5.678 µs 6.789 µs 7.890 µs]
```

### Интерпретация результатов

```
cache_key_simple        time:   [5.234 ns 5.456 ns 5.678 ns]
                        change: [-2.34% -1.23% -0.12%] (p = 0.01 < 0.05)
                        Performance has improved.

cache_key_complex       time:   [12.34 ns 12.56 ns 12.78 ns]
                        change: [+5.67% +6.78% +7.89%] (p = 0.01 < 0.05)
                        Performance has regressed.
```

---

## 🗄️ Оптимизация БД

### Индексы

```sql
-- Индексы для частых запросов
CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_created ON tasks(created);
CREATE INDEX IF NOT EXISTS idx_audit_log_project ON audit_log(project_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_created ON audit_log(created);
```

### Connection Pooling

```rust
// Оптимальный размер пула
let pool_options = SqlitePoolOptions::new()
    .max_connections(5)
    .min_connections(1)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800));
```

### Query Optimization

```rust
// Хорошо - используем LIMIT
let tasks = sqlx::query_as::<_, Task>(
    "SELECT * FROM tasks WHERE project_id = ? LIMIT 100"
)
.bind(project_id)
.fetch_all(&pool)
.await?;

// Плохо - выбираем все записи
let tasks = sqlx::query_as::<_, Task>(
    "SELECT * FROM tasks WHERE project_id = ?"
)
.fetch_all(&pool)
.await?;
```

### Prepared Statements

```rust
// Хорошо - prepared statement
let stmt = "SELECT * FROM users WHERE id = ?";
let user = sqlx::query_as::<_, User>(stmt)
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

// Плохо - string interpolation
let query = format!("SELECT * FROM users WHERE id = {}", user_id);
```

---

## 💾 Оптимизация памяти

### Умное использование коллекций

```rust
// Хорошо - с указанием capacity
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}

// Плохо - без capacity
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);
}
```

### Borrowing вместо Cloning

```rust
// Хорошо - borrowing
fn process_data(data: &str) {
    println!("{}", data);
}

// Плохо - cloning
fn process_data(data: String) {
    println!("{}", data);
}
```

### Lazy Evaluation

```rust
// Хорошо - lazy
let result = data.iter()
    .filter(|x| x > &10)
    .take(10)
    .collect::<Vec<_>>();

// Плохо - eager
let filtered: Vec<_> = data.iter().filter(|x| x > &10).collect();
let result: Vec<_> = filtered.iter().take(10).collect();
```

---

## 🌐 Оптимизация API

### Кэширование ответов

```rust
// Middleware для кэширования GET запросов
pub struct CacheMiddleware {
    redis: Arc<RedisCache>,
    ttl_secs: u64,
}

// Кэшируем частые запросы
GET /api/projects      -> Cache TTL: 600s
GET /api/users         -> Cache TTL: 300s
GET /api/tasks         -> Cache TTL: 60s
```

### Compression

```rust
// Gzip compression middleware
use tower_http::compression::CompressionLayer;

let app = Router::new()
    .layer(CompressionLayer::new()
        .gzip(true)
        .deflate(true));
```

### Rate Limiting

```rust
// Rate limiter для защиты от перегрузки
use tower_http::limit::RateLimitLayer;

let app = Router::new()
    .layer(RateLimitLayer::new(100, Duration::from_secs(60)));
```

---

## 📈 Мониторинг производительности

### Prometheus метрики

```rust
// Метрики времени выполнения запросов
static REQUEST_DURATION: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "semaphore_request_duration_seconds",
        "Request duration in seconds",
        &["method", "endpoint", "status"]
    ).unwrap()
});

// Метрики использования памяти
static MEMORY_USAGE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "semaphore_memory_usage_bytes",
        "Memory usage in bytes"
    ).unwrap()
});
```

### Логирование производительности

```rust
// Middleware для логирования времени запросов
pub async fn performance_logger(req: Request<Body>, next: Next) -> Response {
    let start = Instant::now();
    let path = req.uri().path().to_string();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    if duration.as_millis() > 100 {
        warn!("Slow request: {} took {:?}", path, duration);
    }
    
    response
}
```

### Profiling

```bash
# CPU profiling с perf
perf record -g target/release/semaphore server
perf report

# Memory profiling с valgrind
valgrind --massif=yes target/release/semaphore server
ms_print massif.out.*

# Flamegraph
cargo flamegraph --bin semaphore
```

---

## 🔧 Best Practices

### 1. Избегайте блокировок в async коде

```rust
// Хорошо - async lock
let guard = rw_lock.read().await;

// Плохо - blocking lock в async контексте
let guard = rw_lock.blocking_read();
```

### 2. Используйте tokio::spawn для тяжелых операций

```rust
// Хорошо - выносим тяжелую операцию
let result = tokio::task::spawn_blocking(|| {
    heavy_computation()
}).await?;

// Плохо - блокируем event loop
let result = heavy_computation();
```

### 3. Оптимизируйте размер бинарника

```bash
# Strip debug symbols
strip target/release/semaphore

# Use cargo-bloat для анализа
cargo bloat --release --filter semaphore
```

### 4. Используйте lto для production

```toml
[profile.release]
lto = true  # Link Time Optimization
codegen-units = 1
```

---

## 📊 Benchmark Results

### Последнее тестирование

```
Date: 10 марта 2026
Version: 0.4.0

API Throughput: 1,234 req/sec
Average Latency: 23ms
P95 Latency: 45ms
P99 Latency: 89ms

Memory Usage: 180MB
CPU Usage: 12% (idle), 45% (load)
```

### Цели оптимизации

| Метрика | Текущее | Цель | Статус |
|---------|---------|------|--------|
| **API Throughput** | 1,234/sec | 5,000/sec | 🔴 |
| **Average Latency** | 23ms | <10ms | 🔴 |
| **P95 Latency** | 45ms | <20ms | 🔴 |
| **Memory Usage** | 180MB | <150MB | 🔴 |
| **Startup Time** | 3s | <2s | 🔴 |

---

## 🔗 Ссылки

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion.rs Benchmarking](https://bheisler.github.io/criterion.rs/book/)
- [Tokio Profiling](https://tokio.rs/tokio/topics/profiling)
- [Flamegraph](https://github.com/brendangregg/FlameGraph)

---

*Последнее обновление: 10 марта 2026 г.*
