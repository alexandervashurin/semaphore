# Конфигурация Velum

## 🎯 Быстрый старт

### Demo (SQLite, одна команда)

```bash
./velum.sh init dev && ./velum.sh start dev
```

**Доступ:** http://localhost:3000 · **Логин:** `admin` / **Пароль:** `admin123`

### Docker Demo

```bash
docker compose -f docker-compose.demo.yml up --build -d
```

**Доступ:** http://localhost:8088 · **Логин:** `admin` / **Пароль:** `admin123`

---

## Переменные окружения

> ⚠️ **Важно:** Проект использует **два префикса** переменных окружения:
> - `VELUM_*` — настройки БД, логирования, runner
> - `SEMAPHORE_*` — настройки аутентификации, LDAP, OIDC, веб-пути

### База данных

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `VELUM_DB_DIALECT` | Тип БД: `sqlite`, `postgres` | `sqlite` |
| `VELUM_DB_URL` | Строка подключения PostgreSQL | — |
| `VELUM_DB_PATH` | Путь к SQLite файлу | `./data/semaphore.db` |

**Примеры connection string:**
```bash
# PostgreSQL
VELUM_DB_URL=postgres://velum:password@localhost:5432/velum

# SQLite (по умолчанию)
VELUM_DB_DIALECT=sqlite
```

### Аутентификация и администратор

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_ADMIN` | Имя пользователя администратора | `admin` |
| `SEMAPHORE_ADMIN_PASSWORD` | Пароль администратора | `admin123` |
| `SEMAPHORE_ADMIN_EMAIL` | Email администратора | `admin@localhost` |
| `SEMAPHORE_ADMIN_NAME` | Полное имя администратора | `Administrator` |
| `SEMAPHORE_JWT_SECRET` | Секрет для JWT токенов | `secret` |
| `SEMAPHORE_ACCESS_KEY_ENCRYPTION` | AES-256 ключ шифрования ключей (32 символа) | — |

### Веб-интерфейс и пути

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_WEB_PATH` | Путь к статическим файлам UI | `./web/public` |
| `SEMAPHORE_TMP_PATH` | Временная папка для задач | `/tmp/velum` |

### Логирование

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `RUST_LOG` | Уровень логирования | `info` |
| `VELUM_LOG_FILE` | Путь к файлу логов | — |

Уровни логирования: `trace`, `debug`, `info`, `warn`, `error`

### LDAP (опционально)

| Переменная | Описание |
|------------|----------|
| `SEMAPHORE_LDAP_ENABLE` | Включить LDAP (`true`/`false`) |
| `SEMAPHORE_LDAP_HOST` | LDAP сервер |
| `SEMAPHORE_LDAP_PORT` | Порт (389 или 636 для LDAPS) |
| `SEMAPHORE_LDAP_SEARCH_DN` | DN для поиска пользователей |
| `SEMAPHORE_LDAP_SEARCH_BIND_DN` | DN для bind |
| `SEMAPHORE_LDAP_SEARCH_BIND_PASSWORD` | Пароль bind |
| `SEMAPHORE_LDAP_DN` | DN шаблон |

### OIDC (опционально)

| Переменная | Описание |
|------------|----------|
| `SEMAPHORE_OIDC_ENABLE` | Включить OIDC (`true`/`false`) |
| `SEMAPHORE_OIDC_{NAME}_CLIENT_ID` | Client ID |
| `SEMAPHORE_OIDC_{NAME}_CLIENT_SECRET` | Client Secret |
| `SEMAPHORE_OIDC_{NAME}_REDIRECT_URL` | Redirect URL |
| `SEMAPHORE_OIDC_{NAME}_SCOPES` | OAuth scopes |
| `SEMAPHORE_OIDC_{NAME}_AUTO_DISCOVERY` | Auto-discovery URL |
| `SEMAPHORE_AUTH_EMAIL_LOGIN_ENABLED` | Включить email login в UI |

### Runner режим

| Переменная | Описание |
|------------|----------|
| `VELUM_RUNNER_TOKEN` | Токен раннера |
| `VELUM_SERVER_URL` | URL сервера Velum |

---

## Примеры конфигурации

### Docker Compose (PostgreSQL)

```yaml
# docker-compose.yml
services:
  db:
    image: postgres:17-alpine
    environment:
      POSTGRES_USER: velum
      POSTGRES_PASSWORD: velum
      POSTGRES_DB: velum

  velum:
    build: .
    ports:
      - "3000:3000"
    environment:
      VELUM_DB_DIALECT: postgres
      VELUM_DB_URL: postgres://velum:velum@db:5432/velum
      SEMAPHORE_WEB_PATH: /app/web/public
      SEMAPHORE_JWT_SECRET: my-secret-key-32-chars-long!!
      SEMAPHORE_ADMIN: admin
      SEMAPHORE_ADMIN_PASSWORD: admin123
      SEMAPHORE_ACCESS_KEY_ENCRYPTION: my-32-char-encryption-key!!
```

### Systemd

```ini
# /etc/systemd/system/velum.service
[Unit]
Description=Velum DevOps Automation
After=network.target postgresql.service

[Service]
Type=notify
User=velum
Group=velum
Environment="VELUM_DB_DIALECT=postgres"
Environment="VELUM_DB_URL=postgres://velum:password@localhost:5432/velum"
Environment="SEMAPHORE_WEB_PATH=/usr/share/velum/web/public"
Environment="SEMAPHORE_JWT_SECRET=<YOUR_JWT_SECRET>"
Environment="SEMAPHORE_ADMIN=admin"
Environment="SEMAPHORE_ADMIN_PASSWORD=<ADMIN_PASSWORD>"
Environment="SEMAPHORE_ACCESS_KEY_ENCRYPTION=<32_CHAR_KEY>"
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/velum server --host 127.0.0.1 --port 3000
Restart=on-failure
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

---

## CLI команды

```bash
# Запуск сервера
velum server --host 0.0.0.0 --port 3000

# Создание пользователя
velum user add --username admin --name "Admin" --email admin@example.com --password admin123 --admin

# Инициализация БД (миграции)
velum setup

# Проверка версии
velum version

# Health check
velum healthcheck --url http://127.0.0.1:3000/healthz
```

---

## Поддерживаемые БД

| БД | Статус | Примечание |
|---|---|---|
| SQLite | ✅ Demo/Dev | По умолчанию, быстрый старт |
| PostgreSQL 13+ | ✅ Prod-ready | Рекомендован для продакшена |
| MySQL 8+ | ⚠️ В коде, не тестировался | Альтернативная поддержка |
