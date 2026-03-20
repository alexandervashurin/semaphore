# Исправление ошибок базы данных для Playbook API

## Проблема

При запуске Playbook Run API через веб-интерфейс возникали ошибки:

```
Error: Ошибка базы данных: column "view_id" of relation "template" does not exist
Error: Ошибка базы данных: column "build_template_id" of relation "template" does not exist
Error: Ошибка базы данных: column "environment" of relation "task" does not exist
...
```

## Причина

Схема базы данных PostgreSQL не содержала всех необходимых колонок для поддержки полного функционала Playbook API и Playbook Run.

## Решение

### 1. Применена миграция `003_full_schema_update.sql`

```bash
# Применить миграцию
docker exec -i semaphore-db psql -U semaphore -d semaphore \
  < db/postgres/migrations/003_full_schema_update.sql
```

### 2. Перезапущен сервер

```bash
./start-server.sh restart
```

## Добавленные таблицы и колонки

### Таблица `view` (новая)

Представления для группировки шаблонов в UI.

```sql
CREATE TABLE "view" (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL,
    position INTEGER DEFAULT 0,
    created TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Таблица `template` (11 новых колонок)

| Колонка | Тип | По умолчанию | Описание |
|---------|-----|--------------|----------|
| `view_id` | INTEGER | NULL | Связь с представлением |
| `build_template_id` | INTEGER | NULL | Шаблон сборки (CI/CD) |
| `autorun` | BOOLEAN | false | Автозапуск при commit |
| `allow_override_args_vars` | BOOLEAN | false | Переопределение аргументов |
| `allow_override_branch_in_task` | BOOLEAN | false | Переопределение ветки Git |
| `allow_inventory_in_task` | BOOLEAN | false | Переопределение инвентаря |
| `allow_parallel_tasks` | BOOLEAN | false | Параллельные задачи |
| `suppress_success_alerts` | BOOLEAN | false | Отключить уведомления об успехе |
| `task_params` | TEXT | NULL | Параметры задачи |
| `survey_vars` | TEXT | NULL | Опрос переменных |
| `vaults` | TEXT | NULL | Хранилища секретов |

### Таблица `task` (18 новых колонок)

| Колонка | Тип | Описание |
|---------|-----|----------|
| `environment` | TEXT | Окружение (переменные) |
| `limit_hosts` | VARCHAR(255) | Ограничение хостов (--limit) |
| `tags` | TEXT | Теги Ansible (--tags) |
| `skip_tags` | TEXT | Пропускаемые теги (--skip-tags) |
| `git_branch` | VARCHAR(255) | Ветка Git |
| `repository_id` | INTEGER | Репозиторий (ID) |
| `inventory_id` | INTEGER | Инвентарь (ID) |
| `environment_id` | INTEGER | Окружение (ID) |
| `integration_id` | INTEGER | Интеграция (ID) |
| `playbook_id` | INTEGER | Playbook (ID) |
| `schedule_id` | INTEGER | Расписание (ID) |
| `event_id` | INTEGER | Событие (ID) |
| `build_task_id` | INTEGER | Задача сборки (ID) |
| `task_args` | TEXT | Аргументы командной строки |
| `version` | VARCHAR(50) | Версия |
| `repo_path` | TEXT | Путь к репозиторию |
| `playbook_content` | TEXT | Содержимое playbook |
| `event_id` | INTEGER | Событие (ID) |

## Проверка

### 1. Проверка структуры БД

```bash
# Проверка таблицы template
docker exec semaphore-db psql -U semaphore -d semaphore -c "\d template"

# Проверка таблицы task
docker exec semaphore-db psql -U semaphore -d semaphore -c "\d task"

# Проверка таблицы view
docker exec semaphore-db psql -U semaphore -d semaphore -c "\d view"
```

### 2. Проверка Playbook Run API

```bash
# Получить токен
TOKEN=$(curl -s -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}' | jq -r '.token')

# Запустить playbook
curl -s -X POST http://localhost:3000/api/project/1/playbooks/2/run \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{}'

# Ожидаемый ответ:
# {"task_id":8,"template_id":13,"status":"waiting","message":"Задача создана..."}
```

### 3. Проверка в веб-интерфейсе

1. Открыть http://localhost:3000
2. Войти: `admin` / `demo123`
3. Перейти в проект → Playbooks
4. Нажать ▶ (Run) на любом playbook
5. Задача должна создаться без ошибок

## Статус

✅ **Исправлено** (2026-03-20)

- [x] Таблица `view` создана
- [x] Колонки в `template` добавлены
- [x] Колонки в `task` добавлены
- [x] Индексы созданы
- [x] Playbook Run API работает
- [x] Веб-интерфейс работает без ошибок

## Файлы

| Файл | Описание |
|------|----------|
| `db/postgres/migrations/003_full_schema_update.sql` | Полная миграция схемы |
| `db/postgres/MIGRATION_003.md` | Документация миграции |
| `db/postgres/PLAYBOOK_DB_FIX.md` | Этот файл |

## Примечания

- Миграция идемпотентна (можно применять多次льно)
- Все колонки добавляются через `ADD COLUMN IF NOT EXISTS`
- Индексы создаются через `CREATE INDEX IF NOT EXISTS`
- Таблица `view` создаётся через `CREATE TABLE IF NOT EXISTS`

## Дата

2026-03-20
