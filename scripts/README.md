# Скрипты запуска Velum

Эта директория содержит скрипты для запуска сервера Semaphore с различными базами данных.

## 🎯 Быстрый старт

### Демонстрационное окружение (рекомендуется для знакомства)

```bash
# Запуск PostgreSQL с готовыми демонстрационными данными
./scripts/postgres-demo-start.sh
```

**Что включено:**
- ✅ Полная схема БД (16 таблиц)
- ✅ 4 пользователя (пароль: `demo123`)
- ✅ 4 проекта
- ✅ 12 шаблонов задач
- ✅ 5 инвентарей
- ✅ 5 репозиториев
- ✅ 5 окружений
- ✅ 4 расписания
- ✅ 6 задач

**Доступ к системе:**
- URL: http://localhost:3000
- Логин: `admin`, `john.doe`, `jane.smith`, `devops`
- Пароль: `demo123` (для всех)

📖 **Подробная документация**: [db/postgres/DEMO.md](../db/postgres/DEMO.md)

---

## Скрипты

### SQLite (рекомендуется для тестирования)

```bash
./scripts/run-sqlite.sh
```

### SQLite (тестовая БД в /tmp)

```bash
./scripts/run-test.sh
```

### MySQL

```bash
# С настройками по умолчанию
./scripts/run-mysql.sh

# С кастомными настройками
export SEMAPHORE_DB_HOST=db.example.com
export SEMAPHORE_DB_PORT=3307
export SEMAPHORE_DB_USER=myuser
export SEMAPHORE_DB_PASS=mypassword
export SEMAPHORE_DB_NAME=mydb
./scripts/run-mysql.sh
```

### PostgreSQL

```bash
# С настройками по умолчанию
./scripts/run-postgres.sh

# С кастомными настройками
export SEMAPHORE_DB_HOST=db.example.com
export SEMAPHORE_DB_PORT=5433
export SEMAPHORE_DB_USER=myuser
export SEMAPHORE_DB_PASS=mypassword
export SEMAPHORE_DB_NAME=mydb
./scripts/run-postgres.sh

# Запуск с демонстрационными данными (рекомендуется)
./scripts/postgres-demo-start.sh

# Перезапуск с очисткой данных
./scripts/postgres-demo-start.sh --clean
```

## Переменные окружения

### Общие

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_WEB_PATH` | Путь к frontend | `./web/public` |

### SQLite

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_PATH` | Путь к файлу БД | `/var/lib/semaphore/semaphore.db` |

### MySQL

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_HOST` | Хост MySQL | `localhost` |
| `SEMAPHORE_DB_PORT` | Порт MySQL | `3306` |
| `SEMAPHORE_DB_USER` | Пользователь MySQL | `semaphore` |
| `SEMAPHORE_DB_PASS` | Пароль MySQL | `semaphore` |
| `SEMAPHORE_DB_NAME` | Имя базы данных | `semaphore` |

### PostgreSQL

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_HOST` | Хост PostgreSQL | `localhost` |
| `SEMAPHORE_DB_PORT` | Порт PostgreSQL | `5432` |
| `SEMAPHORE_DB_USER` | Пользователь PostgreSQL | `semaphore` |
| `SEMAPHORE_DB_PASS` | Пароль PostgreSQL | `semaphore` |
| `SEMAPHORE_DB_NAME` | Имя базы данных | `semaphore` |

## Создание пользователя

После первого запуска создайте администратора:

```bash
cd rust
cargo run -- user add \
    --username admin \
    --name "Administrator" \
    --email admin@localhost \
    --password admin123 \
    --admin
```

**Примечание:** При использовании демонстрационного окружения (`postgres-demo-start.sh`) пользователь уже создан!

## Тестовый доступ

### Демонстрационное окружение (PostgreSQL)

- URL: http://localhost:3000
- Логин: `admin`, `john.doe`, `jane.smith`, `devops`
- Пароль: `demo123` (для всех)

### Тестовая БД (SQLite)

- Логин: `admin`
- Пароль: `admin123`

Frontend доступен по адресу: http://localhost:3000
