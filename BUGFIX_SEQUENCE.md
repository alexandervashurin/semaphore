# 🐛 Исправление: Ошибка "duplicate key value unique constraint"

## Проблема

При попытке создать новый проект в CRUD демо возникала ошибка:

```
Ошибка базы данных: error returned from database: duplicate key value 
unique constraint "project_pkey"
```

## Причина

В демо-данных (`db/postgres/init-demo.sql`) проекты вставляются с явными ID:

```sql
INSERT INTO project (id, name, ...) VALUES
(1, 'Demo Infrastructure', ...),
(2, 'Web Application Deployment', ...),
(3, 'Database Management', ...),
(4, 'Security & Compliance', ...);
```

После вставки данных PostgreSQL не автоматически обновляет последовательность `project_id_seq`, 
поэтому при попытке создать новый проект он получал ID=1 (следующее значение последовательности), 
что вызывало конфликт уникального ключа.

## Решение

Добавлен сброс последовательностей в конец файла `init-demo.sql`:

```sql
-- Сброс последовательностей для предотвращения конфликтов ID
SELECT setval('project_id_seq', (SELECT COALESCE(MAX(id), 0) FROM project) + 1, false);
SELECT setval('"user_id_seq"', (SELECT COALESCE(MAX(id), 0) FROM "user") + 1, false);
SELECT setval('template_id_seq', (SELECT COALESCE(MAX(id), 0) FROM template) + 1, false);
SELECT setval('inventory_id_seq', (SELECT COALESCE(MAX(id), 0) FROM inventory) + 1, false);
SELECT setval('repository_id_seq', (SELECT COALESCE(MAX(id), 0) FROM repository) + 1, false);
SELECT setval('environment_id_seq', (SELECT COALESCE(MAX(id), 0) FROM environment) + 1, false);
SELECT setval('access_key_id_seq', (SELECT COALESCE(MAX(id), 0) FROM access_key) + 1, false);
SELECT setval('task_id_seq', (SELECT COALESCE(MAX(id), 0) FROM task) + 1, false);
SELECT setval('schedule_id_seq', (SELECT COALESCE(MAX(id), 0) FROM schedule) + 1, false);
```

## Применение исправления

### Для новых установок

Просто сбросьте и пересоздайте базу данных:

```bash
# Остановка и удаление данных
docker-compose down -v

# Запуск с новыми данными
docker-compose up -d db

# Проверка последовательности
docker-compose exec -T db psql -U semaphore -d semaphore \
  -c "SELECT last_value FROM project_id_seq;"
# Должно вернуть: 5
```

### Для существующих установок

Если база данных уже запущена, выполните SQL команды вручную:

```bash
docker-compose exec -T db psql -U semaphore -d semaphore <<EOF
SELECT setval('project_id_seq', (SELECT COALESCE(MAX(id), 0) FROM project) + 1, false);
SELECT setval('"user_id_seq"', (SELECT COALESCE(MAX(id), 0) FROM "user") + 1, false);
SELECT setval('template_id_seq', (SELECT COALESCE(MAX(id), 0) FROM template) + 1, false);
SELECT setval('inventory_id_seq', (SELECT COALESCE(MAX(id), 0) FROM inventory) + 1, false);
SELECT setval('repository_id_seq', (SELECT COALESCE(MAX(id), 0) FROM repository) + 1, false);
SELECT setval('environment_id_seq', (SELECT COALESCE(MAX(id), 0) FROM environment) + 1, false);
SELECT setval('access_key_id_seq', (SELECT COALESCE(MAX(id), 0) FROM access_key) + 1, false);
SELECT setval('task_id_seq', (SELECT COALESCE(MAX(id), 0) FROM task) + 1, false);
SELECT setval('schedule_id_seq', (SELECT COALESCE(MAX(id), 0) FROM schedule) + 1, false);
EOF
```

## Проверка

После применения исправления проверьте:

```bash
# 1. Проверка последовательности проектов
docker-compose exec -T db psql -U semaphore -d semaphore \
  -c "SELECT last_value FROM project_id_seq;"
# Ожидается: 5

# 2. Тестирование создания проекта
./test-crud-simple.sh

# Ожидается: успешное создание проекта с ID=5
```

## Дополнительные исправления

### Обновление JavaScript для CRUD

В файле `web/public/demo-crud.js` обновлена функция `updateProject`:

```javascript
async function updateProject(projectId, projectData) {
    try {
        // Добавляем id в тело запроса
        const dataWithId = { ...projectData, id: projectId };
        await apiRequest(`/projects/${projectId}`, {
            method: 'PUT',
            body: JSON.stringify(dataWithId),
        });
        // ...
    }
}
```

Это необходимо, потому что API требует поле `id` в теле PUT запроса.

## Файлы с исправлениями

| Файл | Изменения |
|------|-----------|
| `db/postgres/init-demo.sql` | Добавлен сброс последовательностей |
| `web/public/demo-crud.js` | Обновлена функция `updateProject` |

## Статус

✅ Исправление применено  
✅ CRUD операции работают  
✅ Тесты пройдены
