# 🧪 CRUD Тестирование Velum

## 📋 Обзор

Этот документ описывает тестирование CRUD операций для Velum.

## 🎯 Тестовые скрипты

### 1. `test-crud-full.sh` - Полное тестирование CRUD

Автоматический тест всех CRUD операций для основных сущностей:

**Использование:**
```bash
./test-crud-full.sh
```

**Или с кастомными параметрами:**
```bash
USERNAME=admin PASSWORD=demo123 API_URL=http://localhost:3000/api ./test-crud-full.sh
```

**Тестируемые сущности:**
- ✅ Проекты (Projects)
- ✅ Ключи доступа (Access Keys)
- ✅ Инвентари (Inventories)
- ✅ Репозитории (Repositories)
- ✅ Окружения (Environments)
- ✅ Шаблоны (Templates)

**Что проверяется:**
1. **CREATE** - Создание сущности
2. **READ** - Чтение (список и по ID)
3. **UPDATE** - Обновление сущности
4. **DELETE** - Удаление сущности

### 2. `create-test-data.sh` - Создание тестовых данных

Скрипт создаёт полный набор тестовых сущностей:

**Использование:**
```bash
./create-test-data.sh
```

**Создаваемые сущности:**
- Тестовый проект
- SSH ключ
- Репозиторий
- Инвентарь
- Окружение
- Шаблон
- Расписание
- Задача

## 📊 Статус CRUD операций

| Сущность | Create | Read | Update | Delete | Примечания |
|----------|--------|------|--------|--------|------------|
| Проекты | ✅ | ✅ | ⚠️ | ✅ | UPDATE требует все поля |
| Ключи доступа | ✅ | ✅ | ⚠️ | ✅ | |
| Инвентари | ✅ | ✅ | ⚠️ | ✅ | |
| Репозитории | ✅ | ✅ | ⚠️ | ✅ | |
| Окружения | ✅ | ✅ | ⚠️ | ✅ | |
| Шаблоны | ✅ | ✅ | ⚠️ | ⚠️ | DELETE может вернуть 500 |
| Задачи | ✅ | ✅ | N/A | ✅ | |
| Расписания | ✅ | ✅ | ✅ | ✅ | |

**Условные обозначения:**
- ✅ - Работает корректно
- ⚠️ - Работает с ограничениями
- ❌ - Не работает
- N/A - Не применимо

## 🔧 Известные проблемы

### 1. UPDATE требует все поля

**Проблема:** При обновлении сущностей необходимо передавать все поля, даже если они не меняются.

**Пример (правильно):**
```json
PUT /api/projects/1
{
  "name": "New Name",
  "alert": false,
  "max_parallel_tasks": 5,
  "type": "default",
  "alert_chat": null,
  "default_secret_storage_id": null
}
```

**Решение:** Включать все обязательные поля в запрос.

### 2. DELETE шаблона может вернуть 500

**Проблема:** Удаление шаблона, который связан с задачами или расписаниями, может вызвать ошибку.

**Решение:** Сначала удалить связанные сущности.

## 🚀 Быстрый старт тестирования

```bash
# 1. Запустить Velum
./start.sh

# 2. Запустить backend
./start.sh --backend

# 3. Запустить CRUD тесты
USERNAME=admin PASSWORD=demo123 ./test-crud-full.sh

# 4. Создать тестовые данные
USERNAME=admin PASSWORD=demo123 ./create-test-data.sh
```

## 📈 Результаты тестов

### Пример успешного вывода:
```
════════════════════════════════════════════════════════
  Результаты тестирования
════════════════════════════════════════════════════════

✓ Пройдено тестов: 20
✗ Провалено тестов: 6

════════════════════════════════════════
  ЕСТЬ ПРОВАЛЬНЫЕ ТЕСТЫ ⚠️
════════════════════════════════════════
```

## 🔍 Отладка

### Проверка доступности API:
```bash
curl http://localhost:3000/api/health
```

### Авторизация:
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}'
```

### Проверка токена:
```bash
TOKEN="your-token-here"
curl http://localhost:3000/api/projects \
  -H "Authorization: Bearer $TOKEN"
```

## 📚 API Endpoints

### Проекты:
- `GET /api/projects` - Список проектов
- `POST /api/projects` - Создание проекта
- `GET /api/projects/{id}` - Получение проекта
- `PUT /api/projects/{id}` - Обновление проекта
- `DELETE /api/projects/{id}` - Удаление проекта

### Шаблоны:
- `GET /api/projects/{project_id}/templates` - Список шаблонов
- `POST /api/projects/{project_id}/templates` - Создание шаблона
- `GET /api/projects/{project_id}/templates/{id}` - Получение шаблона
- `PUT /api/projects/{project_id}/templates/{id}` - Обновление шаблона
- `DELETE /api/projects/{project_id}/templates/{id}` - Удаление шаблона

### Задачи:
- `GET /api/projects/{project_id}/tasks` - Список задач
- `POST /api/projects/{project_id}/tasks` - Создание задачи
- `GET /api/projects/{project_id}/tasks/{id}` - Получение задачи
- `DELETE /api/projects/{project_id}/tasks/{id}` - Удаление задачи

### Инвентари:
- `GET /api/projects/{project_id}/inventories` - Список инвентарей
- `POST /api/projects/{project_id}/inventories` - Создание инвентаря
- `GET /api/projects/{project_id}/inventories/{id}` - Получение инвентаря
- `PUT /api/projects/{project_id}/inventories/{id}` - Обновление инвентаря
- `DELETE /api/projects/{project_id}/inventories/{id}` - Удаление инвентаря

### Репозитории:
- `GET /api/projects/{project_id}/repositories` - Список репозиториев
- `POST /api/projects/{project_id}/repositories` - Создание репозитория
- `GET /api/projects/{project_id}/repositories/{id}` - Получение репозитория
- `PUT /api/projects/{project_id}/repositories/{id}` - Обновление репозитория
- `DELETE /api/projects/{project_id}/repositories/{id}` - Удаление репозитория

### Окружения:
- `GET /api/projects/{project_id}/environments` - Список окружений
- `POST /api/projects/{project_id}/environments` - Создание окружения
- `GET /api/projects/{project_id}/environments/{id}` - Получение окружения
- `PUT /api/projects/{project_id}/environments/{id}` - Обновление окружения
- `DELETE /api/projects/{project_id}/environments/{id}` - Удаление окружения

### Ключи доступа:
- `GET /api/projects/{project_id}/keys` - Список ключей
- `POST /api/projects/{project_id}/keys` - Создание ключа
- `GET /api/projects/{project_id}/keys/{id}` - Получение ключа
- `PUT /api/projects/{project_id}/keys/{id}` - Обновление ключа
- `DELETE /api/projects/{project_id}/keys/{id}` - Удаление ключа

## 💡 Советы

1. **Всегда используйте токен авторизации** - большинство endpoints требуют аутентификации
2. **Проверяйте HTTP статус коды**:
   - `200 OK` - Успех
   - `201 Created` - Ресурс создан
   - `204 No Content` - Ресурс удалён
   - `400 Bad Request` - Ошибка в запросе
   - `401 Unauthorized` - Требуется авторизация
   - `403 Forbidden` - Нет прав
   - `404 Not Found` - Ресурс не найден
   - `500 Internal Server Error` - Ошибка сервера

3. **Очищайте тестовые данные** после тестирования
4. **Используйте демо-учётные данные**:
   - Логин: `admin`
   - Пароль: `demo123`

## 📖 Дополнительная документация

- [API.md](API.md) - Полная документация API
- [SCRIPTS.md](SCRIPTS.md) - Скрипты запуска
- [CRUD_DEMO.md](CRUD_DEMO.md) - CRUD демо руководство
