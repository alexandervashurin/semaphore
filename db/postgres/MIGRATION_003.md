# Миграция базы данных для Playbook API

## Проблема

При запуске Playbook Run API возникали ошибки о недостающих колонках в таблицах `template` и `task`.

## Решение

Применена миграция `003_full_schema_update.sql` которая добавляет:

### Таблица `view` (создаётся если не существует)

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

### Таблица `template` (добавлены колонки)

| Колонка | Тип | Описание |
|---------|-----|----------|
| `view_id` | INTEGER | Связь с представлением |
| `build_template_id` | INTEGER | Шаблон сборки |
| `autorun` | BOOLEAN | Автозапуск |
| `allow_override_args_vars` | BOOLEAN | Переопределение аргументов |
| `allow_override_branch_in_task` | BOOLEAN | Переопределение ветки |
| `allow_inventory_in_task` | BOOLEAN | Переопределение инвентаря |
| `allow_parallel_tasks` | BOOLEAN | Параллельные задачи |
| `suppress_success_alerts` | BOOLEAN | Отключить алерты |
| `task_params` | TEXT | Параметры задачи |
| `survey_vars` | TEXT | Опрос переменных |
| `vaults` | TEXT | Хранилища секретов |

### Таблица `task` (добавлены колонки)

| Колонка | Тип | Описание |
|---------|-----|----------|
| `environment` | TEXT | Окружение |
| `limit_hosts` | VARCHAR(255) | Ограничение хостов |
| `tags` | TEXT | Теги Ansible |
| `skip_tags` | TEXT | Пропускаемые теги |
| `git_branch` | VARCHAR(255) | Ветка Git |
| `repository_id` | INTEGER | Репозиторий |
| `inventory_id` | INTEGER | Инвентарь |
| `environment_id` | INTEGER | Окружение (ID) |
| `integration_id` | INTEGER | Интеграция |
| `playbook_id` | INTEGER | Playbook (ID) |
| `schedule_id` | INTEGER | Расписание |
| `event_id` | INTEGER | Событие |
| `build_task_id` | INTEGER | Задача сборки |
| `task_args` | TEXT | Аргументы задачи |
| `version` | VARCHAR(50) | Версия |
| `repo_path` | TEXT | Путь к репозиторию |
| `playbook_content` | TEXT | Содержимое playbook |

## Применение миграции

```bash
# Автоматически через скрипт
./scripts/apply-migration.sh

# Или вручную
docker exec -i semaphore-db psql -U semaphore -d semaphore \
  < db/postgres/migrations/003_full_schema_update.sql
```

## Проверка

```bash
# Проверка структуры таблицы template
docker exec semaphore-db psql -U semaphore -d semaphore -c "\d template"

# Проверка структуры таблицы task
docker exec semaphore-db psql -U semaphore -d semaphore -c "\d task"

# Проверка структуры таблицы view
docker exec semaphore-db psql -U semaphore -d semaphore -c "\d view"
```

## Тестирование Playbook Run API

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

# Ответ: {"task_id":7,"template_id":13,"status":"waiting",...}
```

## Дата

2026-03-20
