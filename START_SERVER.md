# Запуск сервера Velum

## Быстрый старт

```bash
# Запуск (PostgreSQL + сервер)
./start-server.sh start

# Проверка статуса
./start-server.sh status

# Остановка
./start-server.sh stop

# Просмотр логов
./start-server.sh logs
```

## Команды

| Команда | Описание |
|---------|----------|
| `start` | Запустить PostgreSQL и сервер Velum |
| `stop` | Остановить сервер и PostgreSQL |
| `restart` | Перезапустить сервер и PostgreSQL |
| `status` | Показать статус сервисов |
| `logs` | Показать логи сервера (tail -f) |
| `clean` | Остановить всё и удалить временные файлы |

## Примеры использования

### Запуск после клонирования репозитория

```bash
# 1. Запустить сервер
./start-server.sh start

# 2. Открыть в браузере
# http://localhost:3000

# 3. Войти с учётными данными demo
# Логин: admin
# Пароль: demo123
```

### Просмотр логов

```bash
# В реальном времени
./start-server.sh logs

# Или напрямую
tail -f velum-server.log
```

### Перезапуск после изменений в коде

```bash
./start-server.sh restart
```

## Конфигурация

Файл `.env` (создаётся автоматически при первом запуске):

```bash
# Velum - PostgreSQL
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@localhost:5432/semaphore
SEMAPHORE_WEB_PATH=/home/alex/Документы/программирование/github/semaphore/web/public
SEMAPHORE_TMP_PATH=/tmp/semaphore
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3000
RUST_LOG=info
```

## Требования

- **Docker** + **Docker Compose** — для PostgreSQL
- **Rust** (cargo) — для сборки и запуска сервера
- **curl** — для проверки health endpoint

## Проверка работы

```bash
# Health check
curl http://localhost:3000/api/health

# Ответ: OK
```

## Доступные endpoints

| URL | Описание |
|-----|----------|
| http://localhost:3000 | Web UI |
| http://localhost:3000/api | API base |
| http://localhost:3000/api/health | Health check |
| http://localhost:3000/api/auth/login | Login |
| http://localhost:3000/api/projects | Projects |
| http://localhost:3000/api/project/1/playbooks | Playbooks API |

## Учётные данные (demo)

| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `demo123` | Администратор |
| `john.doe` | `demo123` | Менеджер |
| `jane.smith` | `demo123` | Менеджер |
| `devops` | `demo123` | Исполнитель |

## Логи

- **Сервер:** `velum-server.log` в корне проекта
- **PostgreSQL:** `docker logs semaphore-db`

## Остановка и очистка

```bash
# Корректная остановка
./start-server.sh stop

# Полная очистка (остановка + удаление временных файлов)
./start-server.sh clean
```

## Решение проблем

### Сервер не запускается

```bash
# Проверить статус
./start-server.sh status

# Посмотреть логи
./start-server.sh logs

# Перезапустить
./start-server.sh restart
```

### PostgreSQL не запускается

```bash
# Остановить и запустить заново
docker compose down
docker compose up -d db

# Проверить статус
docker ps | grep postgres
```

### Порт 3000 занят

Измените порт в `.env`:

```bash
SEMAPHORE_TCP_ADDRESS=0.0.0.0:3001
```

Затем перезапустите:

```bash
./start-server.sh restart
```

## Альтернативный запуск (без скрипта)

```bash
# 1. Запустить PostgreSQL
docker compose up -d db

# 2. Запустить сервер
cd rust
cargo run --release -- server --host 0.0.0.0 --port 3000
```
