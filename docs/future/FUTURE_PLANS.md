# FUTURE_PLANS.md — Планы на будущее

> **Дата составления:** 2026-03-18
> **Статус проекта на момент написания:** v0.1.0 released, 99%+ feature parity с Go-оригиналом
> **Репозиторий:** https://github.com/tnl-o/semarust

---

## Содержание

1. [Безопасность и зависимости](#1-безопасность-и-зависимости)
2. [Качество кода и тесты](#2-качество-кода-и-тесты)
3. [Незавершённые функции (TODO в коде)](#3-незавершённые-функции-todo-в-коде)
4. [Новые возможности](#4-новые-возможности)
5. [Производительность](#5-производительность)
6. [Фронтенд](#6-фронтенд)
7. [Инфраструктура и DevOps](#7-инфраструктура-и-devops)
8. [Документация](#8-документация)
9. [Монетизация и PRO-функции](#9-монетизация-и-pro-функции)
10. [Приоритетная дорожная карта](#10-приоритетная-дорожная-карта)

---

## 1. Безопасность и зависимости

### 🔴 Высокий приоритет

#### Уязвимые зависимости (cargo audit)
| Пакет | Версия | CVSS | Проблема | Действие |
|-------|--------|------|----------|---------|
| `quinn-proto` | 0.11.13 | 8.7 | Memory safety | Обновить до 0.11.14+ |
| `wasmtime` | 41.x | 6.x | Multiple CVEs | Обновить до последней |
| `protobuf` | старая | 5.x | DoS | Обновить |

```bash
cargo audit          # посмотреть текущее состояние
cargo update         # попробовать автообновление
```

#### RSA Marvin Attack (`rsa` crate 0.9.x)
- Timing sidechannel в RSA PKCS#1 v1.5
- Патча нет. Задействован только в MySQL TLS-пути
- **Рекомендация:** Отключить RSA, перейти на ECDSA для TLS-сертификатов

#### Отзыв токена
GitHub PAT, использованный в сеансе 2026-03-18, необходимо отозвать.
**Немедленно отозвать**: https://github.com/settings/tokens и создать новый.

### 🟠 Средний приоритет

- Добавить `cargo audit` в CI pipeline (`.github/workflows/rust.yml`)
- Настроить Dependabot для автоматических PR на обновление зависимостей
- Включить `cargo deny` для контроля лицензий и уязвимостей

---

## 2. Качество кода и тесты

### Покрытие тестами (текущее ~67%, цель 80%+)

#### Где добавить тесты:

**`rust/src/services/task_runner/`** — практически без тестов:
- `logging.rs` — тест записи лога в БД, WebSocket трансляция
- `lifecycle.rs` — тест полного жизненного цикла задачи (start → run → finish)
- `hooks.rs` — тест pre/post хуков с мок-HTTP сервером
- `details.rs` — тест загрузки template/inventory/repository деталей

**`rust/src/db/sql/managers/`** — сложные SQL запросы без тестов:
- Тесты пагинации (offset/limit)
- Тесты фильтрации и сортировки
- Тесты транзакций при конкурентном доступе

**`rust/src/api/handlers/`** — тесты HTTP-уровня:
- Integration tests через `axum::test`
- Тест авторизации (401 без токена, 403 без прав)
- Тест rate limiting

#### Инструменты:
```bash
# Установить tarpaulin для покрытия
cargo install cargo-tarpaulin

# Запуск с отчётом покрытия
cargo tarpaulin --out Html --output-dir coverage/
```

### Clippy улучшения
Сейчас 0 warnings — поддерживать. Добавить строгие линты в `Cargo.toml`:
```toml
[lints.clippy]
pedantic = "warn"
nursery = "warn"
```

### Мутационное тестирование
```bash
cargo install cargo-mutants
cargo mutants --file src/services/task_pool.rs
```

---

## 3. Незавершённые функции (TODO в коде)

### Telegram Bot (`rust/src/services/telegram_bot/mod.rs`)
Полностью заглушен (строки 17, 24, 36, 43).

Что нужно реализовать:
- Отправка алертов об ошибках задач
- Уведомления о завершении деплоя
- Команды управления задачами через бота (`/status`, `/run`, `/stop`)
- Авторизация через Telegram (опционально)

```rust
// Пример интеграции:
use teloxide::prelude::*;
// Добавить в Cargo.toml: teloxide = { version = "0.12", features = ["macros"] }
```

### SSH Key Installation (`rust/src/services/access_key_installation_service.rs`)
5 стабов (строки 224–261) для:
- Установки SSH-ключа в `~/.ssh/`
- Запуска `ssh-agent` и добавления ключа
- Очистки после задачи
- Поддержки passphrase через `ssh-askpass`

### Exporter/Importer (`rust/src/services/exporter_entities.rs`)
T-BE-15 — восстановление пользователей и проектов из бэкапа (строки 37, 80).
Нужно решить async/sync конфликт в dead code.

### Scheduler Cron Validation (`rust/src/services/scheduler_pool.rs`)
Строки 57, 68 — проверка корректности cron-выражения перед сохранением:
```rust
// Добавить: cron = "0.12" в зависимости
use cron::Schedule;
fn validate_cron(expr: &str) -> Result<()> {
    expr.parse::<Schedule>().map_err(|e| Error::Validation(e.to_string()))?;
    Ok(())
}
```

### Alert Signature (`rust/src/services/alert.rs:348`)
Добавить HMAC-подпись к webhook-уведомлениям (как в GitHub Webhooks):
```
X-Semaphore-Signature: sha256=<hmac>
```

### Session Management (`rust/src/api/auth.rs:30,45`)
- Реализовать blacklist для отозванных JWT токенов (Redis или in-memory)
- Logout должен инвалидировать токен немедленно, а не ждать expiry

### OIDC Email в AuthConfig (`rust/src/api/handlers/oidc.rs:298`)
Добавить email claim в конфиг OIDC провайдера для корректного маппинга пользователей.

---

## 4. Новые возможности

### 4.1 Кросс-платформенные бинарники (CI)

Сейчас только Windows x64. Нужны:
- Linux x64 (основная целевая платформа)
- Linux ARM64 (Raspberry Pi, облачные ARM)
- macOS x64/ARM64

Добавить в `.github/workflows/rust.yml`:
```yaml
strategy:
  matrix:
    include:
      - os: ubuntu-latest, target: x86_64-unknown-linux-musl
      - os: ubuntu-latest, target: aarch64-unknown-linux-musl
      - os: macos-latest,  target: x86_64-apple-darwin
      - os: macos-latest,  target: aarch64-apple-darwin
      - os: windows-latest, target: x86_64-pc-windows-msvc
```

### 4.2 Runner Agents (распределённое выполнение)

Go-оригинал поддерживает remote runners. У нас есть заглушки в `rust/src/services/runners/`.
Нужно реализовать:
- Runner регистрация через API (токен)
- Heartbeat / health check
- Назначение задач конкретному runner
- WebSocket-туннель для логов с remote runner

### 4.3 Kubernetes-нативный Runner

Запуск задач в Kubernetes Job/Pod:
- CRD для TaskRun
- Operator-паттерн через `kube-rs` (уже в зависимостях!)
- Автоскейлинг на основе очереди задач

### 4.4 HashiCorp Vault интеграция

Хранение секретов в Vault вместо зашифрованных полей в БД:
- `vault-rs` клиент
- Dynamic secrets (AWS, PostgreSQL)
- Lease renewal для долгих задач

### 4.5 Secret Rotation

Автоматическая ротация SSH-ключей и токенов:
- Интеграция с Let's Encrypt для TLS
- Rotation policy (по времени, по количеству использований)
- Уведомления при истечении

### 4.6 Multi-tenancy

Полная изоляция между организациями:
- Схема с `organization_id` во всех таблицах
- Row-level security в PostgreSQL
- Отдельные encryption keys per-org

### 4.7 AI-интеграции

- Генерация playbook из описания на естественном языке (Claude API)
- Анализ ошибок запусков — объяснение и предложение fix
- Предиктивный мониторинг — предсказание сбоев по паттернам логов

---

## 5. Производительность

### Текущие узкие места

#### БД
- SQLite — однопоточные записи, узкое место при >10 параллельных задачах
- Нет connection pooling тюнинга для PostgreSQL
- Нет query caching (Redis)

**Рекомендации:**
```toml
# sqlx pool settings
[database]
max_connections = 20
min_connections = 5
acquire_timeout_secs = 30
```

#### WebSocket
- Текущий broadcast — O(n) по количеству подписчиков
- При >1000 одновременных клиентов нужен pub/sub (Redis, NATS)

#### Task Queue
- Очередь in-memory — теряется при рестарте
- Нет persistence для задач в очереди

**Решение:** Использовать PostgreSQL как queue backend (pg_notify + LISTEN/NOTIFY), или Redis Streams.

### Бенчмарки

Добавить criterion benchmarks:
```bash
cargo install cargo-criterion
# bench/task_pool.rs — измерить throughput
```

Цели:
- API response < 10ms (p99)
- Task queue enqueue < 1ms
- WebSocket broadcast < 5ms для 1000 клиентов

---

## 6. Фронтенд

### 6.1 Vue 3 (опционально)

Текущий фронтенд — Vanilla JS. Если захочется вернуться к фреймворку:
- Vue 3 + Composition API + TypeScript
- Pinia вместо Vuex
- Vite для сборки
- shadcn-vue или Vuetify 3 (как в Go-оригинале)

### 6.2 Улучшения текущего Vanilla JS

- Виртуализация длинных списков задач (Virtual Scroll)
- Offline mode / Service Worker для кэширования
- PWA — установка как десктопное приложение
- Dark mode (CSS variables уже частично готовы)
- Keyboard shortcuts (Vim-like navigation)
- i18n — интернационализация (сейчас только русский/английский)

### 6.3 Mobile

- Адаптивный дизайн (сейчас минимальный)
- Touch-события для свайпов
- Push-уведомления через Web Push API

---

## 7. Инфраструктура и DevOps

### 7.1 Docker улучшения

```dockerfile
# Multi-stage build с минимальным образом
FROM scratch  # или distroless
COPY target/release/semaphore /semaphore
EXPOSE 3000
ENTRYPOINT ["/semaphore"]
```

Текущий образ ~1.5GB (Rust build deps), цель — <50MB.

### 7.2 Kubernetes Helm Chart

```yaml
# helm/semaphore/values.yaml
replicaCount: 3
database:
  type: postgresql
  existingSecret: semaphore-db-credentials
ingress:
  enabled: true
  className: nginx
```

### 7.3 Мониторинг

**Prometheus метрики** (endpoint уже готов `/metrics`), нужны:
- `semaphore_tasks_total{status="success|error|running"}`
- `semaphore_task_duration_seconds{template_id}`
- `semaphore_queue_depth{project_id}`
- `semaphore_websocket_connections`

**Grafana dashboards** — создать JSON для:
- Task Execution Overview
- System Health (CPU, Memory, DB latency)
- Alert Dashboard

**Loki + Promtail** — централизованный сбор структурированных логов (tracing уже настроен).

### 7.4 High Availability

- Active-Passive failover с PostgreSQL
- Distributed lock через Redis (сейчас in-memory Mutex)
- Sticky sessions для WebSocket (или перейти на Redis pub/sub)

### 7.5 CI/CD улучшения

```yaml
# .github/workflows/rust.yml — добавить:
- name: Security audit
  run: cargo audit --deny warnings

- name: Coverage report
  run: cargo tarpaulin --out Xml

- name: Upload to Codecov
  uses: codecov/codecov-action@v3

- name: Build multi-platform
  uses: docker/build-push-action@v5
  with:
    platforms: linux/amd64,linux/arm64
```

---

## 8. Документация

### 8.1 User Guide (для конечных пользователей)
- Быстрый старт (5 минут до первого деплоя)
- Работа с шаблонами
- Настройка inventory
- Управление SSH-ключами
- Расписания и вебхуки

### 8.2 Admin Guide (для администраторов)
- Установка (Docker, bare-metal, Kubernetes)
- Настройка PostgreSQL в production
- LDAP/OIDC интеграция
- Backup и Restore
- Мониторинг и алерты
- Troubleshooting

### 8.3 API Reference
- OpenAPI 3.0 спецификация (сгенерировать из utoipa/aide)
- GraphQL schema docs
- WebSocket protocol описание

### 8.4 Developer Guide
- Архитектурные решения (ADR)
- Как добавить новый API endpoint
- Как добавить новый тип task runner
- Contribution guide

### 8.5 Migration Guide (Go → Rust)
- Как перенести данные из Go-версии Semaphore
- Совместимость схем БД
- Различия в поведении

---

## 9. Монетизация и PRO-функции

Текущий код содержит заглушки для PRO подписки (`api/auth.rs:87,107`).

### Возможные PRO-функции:
- **SAML SSO** — корпоративная авторизация
- **Audit Trail** — детальные логи с экспортом (SIEM интеграция)
- **Advanced RBAC** — кастомные роли с fine-grained permissions
- **Multi-region** — репликация между датацентрами
- **SLA guarantees** — гарантированное время отклика
- **Priority support** — dedicated support channel
- **White-label** — ребрендинг для ISV

### Лицензирование:
- Рассмотреть AGPL → BSL (Business Source License) как Hashicorp
- Или MIT core + проприетарные enterprise плагины

---

## 10. Приоритетная дорожная карта

### Q1 2026 (апрель — уже сейчас)
- [ ] Отозвать скомпрометированный GitHub токен
- [ ] Обновить уязвимые зависимости (`quinn-proto`, `wasmtime`)
- [ ] Добавить `cargo audit` в CI
- [ ] Linux x64 билд в GitHub Releases
- [ ] Реализовать Telegram Bot уведомления

### Q2 2026
- [ ] Увеличить покрытие тестами до 80%+
- [ ] Docker образ <50MB (FROM scratch)
- [ ] Helm Chart для Kubernetes
- [ ] Prometheus + Grafana dashboard
- [ ] Кросс-платформенные бинарники (Linux, macOS, Windows)

### Q3 2026
- [ ] Remote Runners (распределённое выполнение)
- [ ] HashiCorp Vault интеграция
- [ ] User Guide + Admin Guide документация
- [ ] OpenAPI спецификация
- [ ] Redis-backed task queue (persistence)

### Q4 2026
- [ ] Kubernetes-нативный Runner (Operator)
- [ ] Multi-tenancy (организации)
- [ ] Vue 3 фронтенд (если Vanilla JS недостаточно)
- [ ] PRO-функции MVP
- [ ] SaaS деплой (cloud.semarust.io?)

---

## Быстрые победы (< 1 дня каждая)

| Задача | Файл | Эффект |
|--------|------|--------|
| Отозвать старый токен | GitHub Settings | Безопасность |
| `cargo audit` в CI | `.github/workflows/rust.yml` | Безопасность |
| Cron validation | `scheduler_pool.rs:57,68` | UX |
| Alert HMAC подпись | `alert.rs:348` | Безопасность |
| JWT logout blacklist | `auth.rs:30,45` | Безопасность |
| Dependabot config | `.github/dependabot.yml` | Автоматизация |
| Linux x64 release build | CI matrix | Distribution |

---

*Документ составлен 2026-03-18. Обновлять по мере реализации.*
