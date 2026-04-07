# 🐳 Docker Demo — Velum

**Один контейнер (Velum UI + API) + PostgreSQL**. Порт 8088.

## 🚀 Быстрый старт

### 1. Запуск Demo

```bash
docker compose -f docker-compose.demo.yml up --build -d
```

### 2. Открыть браузер

- **URL**: http://localhost:8088
- **Логин**: `admin`
- **Пароль:** `admin123`

---

## 📋 Информация

| Компонент | URL / Порт | Описание |
|-----------|-----------|----------|
| **Velum (UI + API)** | http://localhost:8088 | Комбинированный контейнер |
| **PostgreSQL** | localhost:5432 | БД с базовыми данными |

### Demo-пользователи

| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `admin123` | Admin |

> Остальные пользователи создаются через `fill-demo-data.sh` скрипт.

---

## 🛠 Команды управления

### Запуск

```bash
# Запуск с пересборкой
docker compose -f docker-compose.demo.yml up --build -d

# Запуск без пересборки
docker compose -f docker-compose.demo.yml up -d

# Запуск и просмотр логов
docker compose -f docker-compose.demo.yml up
```

### Остановка

```bash
# Остановка сервисов
docker compose -f docker-compose.demo.yml down

# Остановка с удалением volumes (данные БД)
docker compose -f docker-compose.demo.yml down -v
```

### Логи

```bash
# Логи всех сервисов
docker compose -f docker-compose.demo.yml logs -f

# Только Velum
docker compose -f docker-compose.demo.yml logs -f velum

# Только PostgreSQL
docker compose -f docker-compose.demo.yml logs -f postgres
```

---

## 🗄️ Наполнение демо-данными

Для заполнения БД тестовыми данными (проекты, шаблоны, задачи):

```bash
# Скрипт создаёт демо-данные через API
bash create-demo-data.sh
```

Или используйте `fill-demo-data.sh` для локального запуска.

---

## 🔧 Конфигурация

Переменные окружения в `docker-compose.demo.yml`:

```yaml
environment:
  VELUM_DB_DIALECT: postgres
  VELUM_DB_URL: postgres://velum:velum@postgres:5432/velum
  SEMAPHORE_WEB_PATH: /app/web/public
  SEMAPHORE_ADMIN: admin
  SEMAPHORE_ADMIN_PASSWORD: admin123
  SEMAPHORE_ACCESS_KEY_ENCRYPTION: my-32-char-encryption-key!!
```

---

## 📦 Другие Docker Compose файлы

| Файл | Описание | Порт |
|------|----------|------|
| `docker-compose.demo.yml` | Demo: Velum + PostgreSQL | 8088 |
| `docker-compose.yml` | Только PostgreSQL | — |
| `docker-compose.dev.yml` | Dev: hot-reload с cargo-watch | 3000 |
| `docker-compose.full.yml` | Полный: Frontend + Backend + PostgreSQL | 3000 |
| `docker-compose.postgres.yml` | Только PostgreSQL | 5432 |
| `docker-compose.prod.yml` | Production: с healthcheck | 3000 |
| `docker-compose.single.yml` | Единый контейнер с SQLite | 3000 |
| `docker-compose.test.yml` | Тестовые данные | 8089 |

---

## 🚀 Нативный запуск (для разработки)

Если хотите запустить backend напрямую без Docker:

```bash
# 1. Запуск PostgreSQL
docker compose -f docker-compose.postgres.yml up -d

# 2. Настройка переменных
export VELUM_DB_DIALECT=postgres
export VELUM_DB_URL=postgres://velum:velum@localhost:5432/velum

# 3. Запуск сервера
cd rust
cargo run -- server --host 0.0.0.0 --port 3000
```

**Доступ:** http://localhost:3000
