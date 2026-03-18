# Конфигурация Velum

## 🎯 Демонстрационное окружение

Для быстрого старта используйте демонстрационное окружение с готовыми данными:

```bash
# Запуск PostgreSQL с демонстрационными данными
./scripts/postgres-demo-start.sh
```

**Переменные окружения для demo:**
```bash
SEMAPHORE_DB_URL=postgres://semaphore:semaphore_pass@localhost:5433/semaphore
SEMAPHORE_HTTP_PORT=3000
SEMAPHORE_WEB_HOST=http://localhost:3000
RUST_LOG=info
```

**Доступ к системе:**
- URL: http://localhost:3000
- Логин: `admin`, `john.doe`, `jane.smith`, `devops`
- Пароль: `demo123` (для всех)

📖 **Подробная документация**: [db/postgres/DEMO.md](db/postgres/DEMO.md)

---

## Переменные окружения

### Основные

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_WEB_HOST` | Хост веб-интерфейса (для генерации URL) | - |
| `SEMAPHORE_HTTP_PORT` | Порт HTTP-сервера | 3000 |
| `SEMAPHORE_CONFIG` | Путь к файлу конфигурации | - |

### База данных

**ВАЖНО:** Для PostgreSQL и MySQL необходимо использовать `SEMAPHORE_DB_URL`. 
Отдельные переменные `SEMAPHORE_DB_HOST`, `SEMAPHORE_DB_USER` и т.д. **НЕ работают** в Rust версии!

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_URL` | **Обязательно для PostgreSQL/MySQL!** Connection string для БД | - |
| `SEMAPHORE_DB_DIALECT` | Тип БД: `sqlite`, `mysql`, `postgres` | sqlite |
| `SEMAPHORE_DB_PATH` | Путь к файлу БД (для SQLite) | /tmp/semaphore.db |

**Примеры connection string:**
```bash
# PostgreSQL
postgres://semaphore:semaphore_pass@localhost:5432/semaphore

# PostgreSQL (через Docker, порт 5433)
postgres://semaphore:semaphore_pass@localhost:5433/semaphore

# MySQL
mysql://semaphore:semaphore_pass@localhost:3306/semaphore

# SQLite
sqlite:///tmp/semaphore.db
```

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

### Docker (PostgreSQL с demo-данными)

```bash
# Запуск с демонстрационными данными
./scripts/postgres-demo-start.sh

# Или вручную
docker run -p 3000:3000 \
  -e SEMAPHORE_DB_URL="postgres://semaphore:semaphore_pass@localhost:5433/semaphore" \
  -e SEMAPHORE_HTTP_PORT=3000 \
  velum/velum:rust
```

### Docker (SQLite)

```bash
docker run -p 3000:3000 \
  -e SEMAPHORE_DB_DIALECT=sqlite \
  -e SEMAPHORE_DB_PATH=/var/lib/semaphore/semaphore.db \
  -e SEMAPHORE_ADMIN=admin \
  -e SEMAPHORE_ADMIN_PASSWORD=changeme \
  velum/velum:rust
```

### Docker (PostgreSQL)

```bash
docker run -p 3000:3000 \
  -e SEMAPHORE_DB_DIALECT=postgres \
  -e SEMAPHORE_DB_HOST=postgres \
  -e SEMAPHORE_DB_PORT=5432 \
  -e SEMAPHORE_DB_USER=semaphore \
  -e SEMAPHORE_DB_PASS=secret \
  -e SEMAPHORE_DB_NAME=semaphore \
  velum/velum:rust
```

### Systemd

```ini
[Unit]
Description=Velum (Rust)
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
