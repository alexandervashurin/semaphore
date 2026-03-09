# 🗺️ Дорожная карта проекта Semaphore UI (Rust)

> **Последнее обновление:** 9 марта 2026 г.
> **Статус:** Q2-Q3 2026 завершены

---

## 📚 Содержание

1. [Стек технологий](#стек-технологий)
2. [Архитектура](#архитектура)
3. [Запуск одним Docker-контейнером](#запуск-одним-docker-контейнером)
4. [План разработки](#план-разработки)
5. [Статус функций](#статус-функций)

---

## 🛠️ Стек технологий

### Backend (Rust)

| Категория | Технология | Версия | Назначение |
|-----------|-----------|--------|------------|
| **Язык** | Rust | 1.75+ | Основной язык backend |
| **Веб-фреймворк** | Axum | 0.8 | HTTP сервер, роутинг, middleware |
| **Асинхронность** | Tokio | 1.x | Async runtime |
| **База данных** | SQLx | 0.8 | Асинхронный SQL клиент |
| **БД (поддержка)** | PostgreSQL, MySQL, SQLite | - | Хранение данных |
| **Аутентификация** | JWT, bcrypt, RSA | 9.3, 0.17, 0.9 | Токены, пароли, ключи |
| **Логирование** | tracing, tracing-subscriber | 0.1, 0.3 | Структурированное логирование |
| **Валидация** | validator | 0.20 | Валидация данных |
| **CLI** | clap | 4.5 | Командная строка |
| **Серализация** | serde, serde_json | 1.0 | JSON обработка |
| **Время** | chrono | 0.4 | Работа с датой/временем |
| **Email** | lettre | 0.11 | Отправка писем |
| **OAuth2/OIDC** | oauth2 | 5.0 | Внешняя аутентификация |
| **Git** | git2 | 0.20 | Работа с Git репозиториями |
| **SSH** | ssh2 | 0.9 | SSH подключения |
| **Сжатие** | flate2 | 1.1 | Gzip compression |
| **Хэширование** | sha2, sha1, md-5, hmac | 0.10 | Криптография |
| **TOTP** | base32, otp | 0.5 | 2FA коды |
| **KV-хранилище** | sled | 0.34 | Встроенное key-value хранилище |

### Frontend (Vue.js)

| Категория | Технология | Версия | Назначение |
|-----------|-----------|--------|------------|
| **Фреймворк** | Vue.js | 2.6.14 | UI фреймворк |
| **UI библиотека** | Vuetify | 2.6.10 | Material Design компоненты |
| **HTTP клиент** | Axios | 1.13.5 | API запросы |
| **Роутинг** | Vue Router | 3.5.4 | Навигация |
| **Интерnationalization** | vue-i18n | 8.18.2 | Многоязычность |
| **Графики** | Chart.js, vue-chartjs | 3.8.0 | Визуализация данных |
| **Терминал** | ansi_up | 6.0.6 | Подсветка ANSI кодов |
| **Cron** | cron-parser | 5.3.0 | Парсинг cron выражений |
| **Дата/время** | dayjs | 1.11.13 | Работа с датой |
| **Сборка** | Vue CLI | 5.0.6 | Build toolchain |
| **Препроцессор** | Sass | 1.32.12 | CSS препроцессор |
| **Линтинг** | ESLint, Prettier | 7.x, 3.x | Качество кода |

### DevOps и инфраструктура

| Категория | Технология | Версия | Назначение |
|-----------|-----------|--------|------------|
| **Контейнеризация** | Docker | 20.x+ | Изоляция среды |
| **Оркестрация** | Docker Compose | 2.x+ | Мультиконтейнерный запуск |
| **Web сервер** | Nginx | Alpine | Раздача статики, reverse proxy |
| **База данных** | PostgreSQL | 15-alpine | Основная БД |
| **CI/CD** | GitHub Actions | - | Автоматизация |
| **Документация** | Swagger/OpenAPI | 3.0 | API документация |

### Инструменты разработки

| Инструмент | Назначение |
|-----------|------------|
| **Taskfile** | Управление задачами (альтернатива Make) |
| **renovate** | Автоматическое обновление зависимостей |
| **dotenvy** | Управление переменными окружения |

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                    Semaphore UI                          │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │   Frontend  │    │   Backend   │    │   Database  │ │
│  │  Vue.js 2.6 │◄──►│  Rust Axum  │◄──►│ PostgreSQL  │ │
│  │  Vuetify    │    │  0.8 Tokio  │    │   MySQL     │ │
│  │  Nginx      │    │  SQLx 0.8   │    │   SQLite    │ │
│  └─────────────┘    └─────────────┘    └─────────────┘ │
└─────────────────────────────────────────────────────────┘
         │                   │                   │
         ▼                   ▼                   ▼
  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
  │  Static     │    │  API REST   │    │  Persistent │
  │  Files      │    │  WebSocket  │    │  Storage    │
  └─────────────┘    └─────────────┘    └─────────────┘
```

### Структура проекта

```
rust_semaphore/
├── rust/                    # Backend на Rust
│   ├── src/
│   │   ├── api/            # HTTP handlers (CRUD)
│   │   ├── db/             # Модели и репозитории БД
│   │   ├── service/        # Бизнес-логика
│   │   ├── auth/           # Аутентификация и авторизация
│   │   ├── mailer/         # Email уведомления
│   │   ├── crypto/         # Шифрование и ключи
│   │   └── lib.rs          # Основной модуль
│   ├── Cargo.toml          # Rust зависимости
│   └── data/               # Локальные данные (SQLite)
├── web/                     # Frontend на Vue.js
│   ├── src/                # Исходный код Vue
│   ├── public/             # Скомпилированная статика
│   └── package.json        # Node.js зависимости
├── db/                      # Скрипты БД
│   └── postgres/
│       ├── init.sql        # Схема БД
│       └── init-demo.sql   # Демо-данные
├── deployment/              # Docker конфигурации
│   ├── single/             # Единый контейнер
│   └── compose/            # Docker Compose
├── Dockerfile              # Основной Docker образ
├── docker-compose.yml      # Compose для разработки
└── README.md               # Документация
```

---

## 🐳 Запуск одним Docker-контейнером

### Вариант 1: Единый образ (рекомендуется для production)

```dockerfile
# deployment/single/Dockerfile
FROM rust:1.75-slim AS backend-builder

WORKDIR /app
COPY rust/Cargo.toml rust/Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && rm -rf src target

COPY rust/ ./
RUN cargo build --release

FROM node:18-alpine AS frontend-builder
WORKDIR /web
COPY web/package*.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 nginx && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -m -u 1000 semaphore

COPY --from=backend-builder /app/target/release/semaphore /usr/local/bin/
COPY --from=frontend-builder /web/dist /var/www/html
COPY deployment/single/nginx.conf /etc/nginx/sites-enabled/default
COPY db/postgres/init.sql /docker-entrypoint-initdb.d/

RUN chown -R semaphore:semaphore /var/www/html /var/log/nginx /var/lib/nginx && \
    chmod -R 755 /var/www/html

WORKDIR /app
USER semaphore

EXPOSE 80

ENV SEMAPHORE_DB_URL="sqlite://data/semaphore.db"
ENV SEMAPHORE_WEB_PATH="/var/www/html"

CMD ["sh", "-c", "semaphore server --host 0.0.0.0 --port 3000 & nginx -g 'daemon off;'"]
```

### Вариант 2: SQLite + встроенный веб-сервер (минималистичный)

```bash
# Запуск без внешних зависимостей
docker run -d \
  --name semaphore \
  -p 3000:3000 \
  -v semaphore_data:/app/data \
  -e SEMAPHORE_DB_URL="sqlite://data/semaphore.db" \
  ghcr.io/alexandervashurin/semaphore:latest
```

### Вариант 3: Docker Compose (один сервис)

```yaml
# docker-compose.single.yml
version: '3.8'

services:
  semaphore:
    image: ghcr.io/alexandervashurin/semaphore:latest
    container_name: semaphore
    restart: unless-stopped
    ports:
      - "80:80"
    volumes:
      - semaphore_data:/app/data
      - semaphore_config:/app/config
    environment:
      - SEMAPHORE_DB_URL=sqlite://data/semaphore.db
      - SEMAPHORE_WEB_PATH=/var/www/html
      - SEMAPHORE_HOST=0.0.0.0
      - SEMAPHORE_PORT=80
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost/health"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  semaphore_data:
  semaphore_config:
```

### Быстрый старт

```bash
# 1. Сборка образа
docker build -t semaphore:latest .

# 2. Запуск
docker run -d \
  --name semaphore \
  -p 80:80 \
  -v $(pwd)/data:/app/data \
  semaphore:latest

# 3. Проверка
curl http://localhost/health

# 4. Остановка
docker stop semaphore && docker rm semaphore
```

---

## 📋 План разработки

### ✅ Завершено (Q1 2026)

- [x] Миграция с Go на Rust
- [x] Базовая аутентификация (JWT + bcrypt)
- [x] CRUD операции для основных сущностей
- [x] Поддержка PostgreSQL, MySQL, SQLite
- [x] Vue.js frontend с Vuetify
- [x] Docker контейнеризация
- [x] WebSocket для real-time обновлений
- [x] Email уведомления (lettre)
- [x] OAuth2/OIDC интеграция
- [x] SSH подключения (ssh2)
- [x] Git интеграция (git2)
- [x] TOTP 2FA аутентификация
- [x] LDAP аутентификация

### ✅ Завершено (Q2 2026)

- [x] Единый Docker контейнер (all-in-one)
- [x] Оптимизация размера образа (musl, distroless) — до 92% экономии
- [x] Webhook интеграции — 5 типов (Generic, Slack, Teams, Discord, Telegram)
- [x] Audit log с расширенным поиском — 50+ типов событий
- [x] Расширенная аналитика и дашборды — метрики, статистика, графики

### ✅ Завершено (Q3 2026)

- [x] Плагин система — базовая архитектура, хуки, менеджер
- [x] 6 типов плагинов: TaskExecutor, Notification, Storage, Auth, ApiExtension, Hook
- [x] 40+ системных хуков для событий
- [x] Менеджер плагинов с зависимостями
- [x] Документация PLUGINS.md

### 🔄 В работе (Q4 2026)

- [ ] GraphQL API (опционально)
- [ ] Telegram Bot API
- [ ] Prometheus метрики
- [ ] WASM загрузчик плагинов

### 📅 Запланировано (Q4 2026 - Q1 2027)

- [ ] Кластерный режим работы
- [ ] Горизонтальное масштабирование
- [ ] Redis кэширование
- [ ] gRPC API для внутренних сервисов
- [ ] Мобильное приложение (React Native / Flutter)
- [ ] Desktop приложение (Tauri)
- [ ] Интеграция с Kubernetes
- [ ] Terraform провайдер
- [ ] Grafana дашборды
- [ ] Distributed tracing (OpenTelemetry)

### 🔮 Будущее (2027+)

- [ ] AI ассистент для генерации playbook'ов
- [ ] Автоматическое тестирование инфраструктуры
- [ ] Visual pipeline editor
- [ ] Marketplace шаблонов и интеграций
- [ ] Multi-tenant режим с изоляцией
- [ ] Serverless execution mode

---

## 📊 Статус функций

### Ядро

| Функция | Статус | Описание |
|--------|--------|----------|
| **Аутентификация** | ✅ Готово | JWT, сессии, 2FA TOTP |
| **Авторизация** | ✅ Готово | RBAC, роли, разрешения |
| **LDAP** | ✅ Готово | Интеграция с LDAP/AD |
| **OAuth2/OIDC** | ✅ Готово | Внешние провайдеры |
| **API** | ✅ Готово | REST + WebSocket |
| **База данных** | ✅ Готово | PostgreSQL, MySQL, SQLite |

### Управление задачами

| Функция | Статус | Описание |
|--------|--------|----------|
| **Playbook** | ✅ Готово | Ansible playbook задачи |
| **Terraform** | ✅ Готово | Terraform plan/apply |
| **PowerShell** | ✅ Готово | PowerShell скрипты |
| **Bash** | ✅ Готово | Bash скрипты |
| **Расписание** | ✅ Готово | Cron выражения |
| **Очереди** | ✅ Готово | Приоритеты, лимиты |

### Уведомления

| Функция | Статус | Описание |
|--------|--------|----------|
| **Email** | ✅ Готово | SMTP, шаблоны |
| **Webhook** | ✅ Готово | Generic, Slack, Teams, Discord, Telegram |
| **Telegram** | ✅ Готово | Webhook уведомления |
| **Slack** | ✅ Готово | Incoming webhooks |

### Инфраструктура

| Функция | Статус | Описание |
|--------|--------|----------|
| **Docker** | ✅ Готово | Одиночный контейнер |
| **Docker Compose** | ✅ Готово | Multi-container |
| **Docker Alpine** | ✅ Готово | Оптимизированный (~60 MB) |
| **Docker Distroless** | ✅ Готово | Безопасный (~35 MB) |
| **Kubernetes** | 📅 Запланировано | Helm chart, operator |
| **Systemd** | ✅ Готово | Service unit |

### Аналитика

| Функция | Статус | Описание |
|--------|--------|----------|
| **Project Stats** | ✅ Готово | Статистика проектов |
| **Task Metrics** | ✅ Готово | Метрики задач |
| **User Activity** | ✅ Готово | Активность пользователей |
| **Performance** | ✅ Готово | Метрики производительности |
| **System Status** | ✅ Готово | Статус системы |
| **Charts** | ✅ Готово | Временные ряды |
| **Audit Log** | ✅ Готово | 50+ типов событий |

### Плагины

| Функция | Статус | Описание |
|--------|--------|----------|
| **Plugin System** | ✅ Готово | Базовая архитектура |
| **Task Executor** | ✅ Готово | Кастомные исполнители |
| **Notification** | ✅ Готово | Провайдеры уведомлений |
| **Storage** | ✅ Готово | Провайдеры хранилищ |
| **Auth** | ✅ Готово | Провайдеры аутентификации |
| **API Extension** | ✅ Готово | Расширения API |
| **Hook System** | ✅ Готово | 40+ системных хуков |
| **Plugin Manager** | ✅ Готово | Менеджер с зависимостями |

---

## 🎯 Метрики качества

| Метрика | Цель | Текущее | Достижение |
|--------|------|---------|------------|
| **Покрытие тестами** | >80% | ~65% | 81% |
| **Время сборки** | <5 мин | ~8 мин | 🔴 |
| **Размер образа (standard)** | <100 MB | ~450 MB | 🔴 |
| **Размер образа (slim)** | <200 MB | ~180 MB | ✅ |
| **Размер образа (alpine)** | <100 MB | ~60 MB | ✅ |
| **Размер образа (distroless)** | <50 MB | ~35 MB | ✅ |
| **Время запуска** | <5 сек | ~3 сек | ✅ |
| **Потребление RAM** | <256 MB | ~180 MB | ✅ |
| **Frontend сборка** | ✅ Работает | Vue.js 2.6.14 | ✅ |
| **Backend API** | ✅ Работает | Порт 3000 | ✅ |
| **Database** | ✅ Работает | PostgreSQL 5432 | ✅ |

---

## 🚀 Текущий статус запуска

### Рабочие сервисы (Март 2026)

```bash
# Backend (Rust + встроенный frontend)
✅ http://localhost:3000

# Database (PostgreSQL)
✅ localhost:5432 (semaphore/semaphore_pass)

# Frontend (nginx)
🔄 Требуется настройка
```

### Команды запуска

```bash
# 1. Запуск БД и frontend
docker-compose up -d

# 2. Сборка frontend
cd web && npm install && npm run build

# 3. Запуск backend
cd rust && cargo run -- server --host 0.0.0.0 --port 3000

# 4. Доступ к UI
http://localhost:3000
```

---

## 📞 Контакты

- **GitHub:** https://github.com/alexandervashurin/semaphore
- **Email:** alexandervashurin@yandex.ru
- **Документация:** 
  - [API.md](API.md) — API документация
  - [AUTH.md](AUTH.md) — Аутентификация
  - [CONFIG.md](CONFIG.md) — Конфигурация
  - [AUDIT_LOG.md](AUDIT_LOG.md) — Audit Log API
  - [WEBHOOK.md](WEBHOOK.md) — Webhook интеграции
  - [IMAGE_OPTIMIZATION.md](IMAGE_OPTIMIZATION.md) — Оптимизация образов
  - [ANALYTICS.md](ANALYTICS.md) — Аналитика и дашборды
  - [PLUGINS.md](PLUGINS.md) — Плагин система
  - [SINGLE_CONTAINER.md](SINGLE_CONTAINER.md) — Единый контейнер

---

*Последнее обновление: 9 марта 2026 г.*

---

## 📝 История изменений

### Q3 2026 (Март)

**✅ Завершённые задачи:**

6. **Плагин система**
   - Базовая архитектура плагинов
   - 6 типов плагинов
   - 40+ системных хуков
   - Менеджер плагинов с зависимостями
   - `plugins/base.rs`, `plugins/hooks.rs`
   - `PLUGINS.md`

### Q2 2026 (Март)

**✅ Завершённые задачи:**

1. **Единый Docker контейнер**
   - `deployment/single/Dockerfile`
   - `docker-compose.single.yml`
   - `SINGLE_CONTAINER.md`

2. **Оптимизация размера образа**
   - `Dockerfile.alpine` (~60 MB, -87%)
   - `Dockerfile.distroless` (~35 MB, -92%)
   - `Dockerfile.slim` (~180 MB, -60%)
   - `scripts/build-optimized-images.sh`
   - `IMAGE_OPTIMIZATION.md`

3. **Audit Log с расширенным поиском**
   - 50+ типов событий
   - 14 типов объектов
   - 4 уровня важности
   - Расширенная фильтрация
   - `models/audit_log.rs`
   - `db/sql/audit_log.rs`
   - `api/handlers/audit_log.rs`
   - `AUDIT_LOG.md`

4. **Webhook интеграции**
   - 5 типов webhook (Generic, Slack, Teams, Discord, Telegram)
   - Автоматические повторные попытки
   - Кастомные заголовки
   - Секретная аутентификация
   - `services/webhook.rs`
   - `models/webhook.rs`
   - `WEBHOOK.md`

5. **Расширенная аналитика и дашборды**
   - Статистика проектов
   - Метрики задач
   - Активность пользователей
   - Производительность
   - Временные ряды
   - `models/analytics.rs`
   - `ANALYTICS.md`
