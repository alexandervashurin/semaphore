# Конфигурация Semaphore UI

## Переменные окружения

### Основные

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_WEB_HOST` | Хост веб-интерфейса (для генерации URL) | - |
| `SEMAPHORE_HTTP_PORT` | Порт HTTP-сервера | 3000 |
| `SEMAPHORE_CONFIG` | Путь к файлу конфигурации | - |

### База данных

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_DIALECT` | Тип БД: `bolt`, `sqlite`, `mysql`, `postgres` | bolt |
| `SEMAPHORE_DB_PATH` | Путь к файлу БД (для bolt/sqlite) | - |
| `SEMAPHORE_DB_HOST` | Хост БД (для mysql/postgres) | localhost |
| `SEMAPHORE_DB_PORT` | Порт БД | 3306 (MySQL), 5432 (PostgreSQL) |
| `SEMAPHORE_DB_USER` | Пользователь БД | - |
| `SEMAPHORE_DB_PASS` | Пароль БД | - |
| `SEMAPHORE_DB_NAME` | Имя базы данных | semaphore |

### Администратор

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_ADMIN` | Имя пользователя администратора | admin |
| `SEMAPHORE_ADMIN_PASSWORD` | Пароль администратора | changeme |
| `SEMAPHORE_ADMIN_NAME` | Полное имя администратора | Administrator |
| `SEMAPHORE_ADMIN_EMAIL` | Email администратора | admin@localhost |

### Логирование

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `RUST_LOG` | Уровень логирования | info |
| `SEMAPHORE_LOG_FILE` | Путь к файлу логов | - |

### Режим раннера

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_RUNNER` | Запуск в режиме раннера | false |
| `SEMAPHORE_RUNNER_TOKEN` | Токен раннера | - |
| `SEMAPHORE_SERVER_URL` | URL сервера (для раннера) | - |

## Примеры конфигурации

### Docker (SQLite)

```bash
docker run -p 3000:3000 \
  -e SEMAPHORE_DB_DIALECT=sqlite \
  -e SEMAPHORE_DB_PATH=/var/lib/semaphore/semaphore.db \
  -e SEMAPHORE_ADMIN=admin \
  -e SEMAPHORE_ADMIN_PASSWORD=changeme \
  semaphoreui/semaphore:rust
```

### Docker (MySQL)

```bash
docker run -p 3000:3000 \
  -e SEMAPHORE_DB_DIALECT=mysql \
  -e SEMAPHORE_DB_HOST=mysql \
  -e SEMAPHORE_DB_PORT=3306 \
  -e SEMAPHORE_DB_USER=semaphore \
  -e SEMAPHORE_DB_PASS=secret \
  -e SEMAPHORE_DB_NAME=semaphore \
  semaphoreui/semaphore:rust
```

### Systemd

```ini
[Unit]
Description=Semaphore UI (Rust)
After=network.target

[Service]
Type=simple
User=semaphore
Environment="SEMAPHORE_DB_DIALECT=sqlite"
Environment="SEMAPHORE_DB_PATH=/var/lib/semaphore/semaphore.db"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/semaphore server
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

## Формат конфигурационного файла

Конфигурационный файл в формате JSON:

```json
{
  "web_host": "https://semaphore.example.com",
  "http_port": 3000,
  "db_dialect": "sqlite",
  "db_path": "/var/lib/semaphore/semaphore.db",
  "log_level": "info",
  "max_parallel_tasks": 10
}
```
