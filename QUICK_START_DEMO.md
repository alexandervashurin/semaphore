# Быстрый старт с демонстрационным окружением

## Запуск

```bash
# Запуск PostgreSQL с демонстрационными данными и сервера Semaphore
./scripts/postgres-demo-start.sh
```

## Что происходит:

1. ✅ Запускается PostgreSQL контейнер с демонстрационными данными
2. ✅ Создаётся `.env` файл с необходимыми переменными
3. ✅ Экспортируются переменные окружения
4. ✅ Запускается сервер Semaphore (`cargo run -- server`)

## Доступ к системе

**URL:** http://localhost:3000

**Учетные данные** (пароль для всех: `demo123`):

| Логин | Имя | Роль |
|-------|-----|------|
| `admin` | Administrator | Полный доступ ко всем проектам |
| `john.doe` | John Doe | Менеджер Web Application Deployment |
| `jane.smith` | Jane Smith | Менеджер Database Management |
| `devops` | DevOps Engineer | Исполнитель задач |

## Демонстрационные данные

- **4 проекта**: Infrastructure, Web App, Database, Security
- **12 шаблонов**: Deploy, Update, Backup, Security Scan, и др.
- **5 инвентарей**: Production, Staging, Clusters
- **5 репозиториев**: Playbooks для разных задач
- **5 окружений**: Production, Staging, Config
- **4 расписания**: Daily, Weekly задачи
- **6 задач**: Выполненные, запущенные, ожидающие

## Остановка

В терминале где запущен сервер нажмите: `Ctrl+C`

```bash
# Остановить PostgreSQL
docker-compose -f docker-compose.postgres.yml down
```

## Перезапуск с очисткой

```bash
# Полная очистка и перезапуск
./scripts/postgres-demo-start.sh --clean
```

## Полезные команды

```bash
# Проверка статуса PostgreSQL
docker ps | grep semaphore_postgres

# Логи PostgreSQL
docker logs semaphore_postgres

# Подключение к БД
docker exec -it semaphore_postgres psql -U semaphore -d semaphore

# Проверка данных
docker exec semaphore_postgres psql -U semaphore -d semaphore -c "SELECT username FROM \"user\";"
docker exec semaphore_postgres psql -U semaphore -d semaphore -c "SELECT name FROM project;"
```

## Переменные окружения

Скрипт автоматически создаёт `.env` файл и экспортирует:

```bash
SEMAPHORE_DB_DIALECT=postgres
SEMAPHORE_DB_HOST=localhost
SEMAPHORE_DB_PORT=5433
SEMAPHORE_DB_USER=semaphore
SEMAPHORE_DB_PASS=semaphore_pass
SEMAPHORE_DB_NAME=semaphore
SEMAPHORE_HTTP_PORT=3000
SEMAPHORE_WEB_HOST=http://localhost:3000
RUST_LOG=info
```

## Решение проблем

### Ошибка "Connection refused"

Убедитесь что PostgreSQL запущен:
```bash
docker ps | grep semaphore_postgres
```

### Ошибка "Unauthorized"

1. Проверьте что демонстрационные данные загружены
2. Используйте правильный пароль: `demo123`
3. Перезапустите с очисткой: `./scripts/postgres-demo-start.sh --clean`

### Порт 5433 занят

Измените порт в `docker-compose.postgres.yml`:
```yaml
ports:
  - "5434:5432"  # Используйте другой порт
```

## Документация

- [DEMO.md](db/postgres/DEMO.md) - Подробная документация по демонстрационному окружению
- [README.md](README.md) - Основная документация проекта
