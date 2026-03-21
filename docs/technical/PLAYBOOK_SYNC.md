# Синхронизация Playbook из Git

## Обзор

Сервис синхронизации playbook позволяет загружать и обновлять содержимое playbook файлов напрямую из Git репозиториев.

## Возможности

- ✅ Загрузка playbook из Git репозитория
- ✅ Поддержка HTTP/HTTPS/Git протоколов
- ✅ Автоматическое определение пути к файлу
- ✅ Предварительный просмотр содержимого
- ✅ Валидация после загрузки

## API Endpoints

### Синхронизация playbook

**POST** `/api/project/{project_id}/playbooks/{id}/sync`

Синхронизирует playbook из связанного Git репозитория и обновляет содержимое в базе данных.

**Требования:**
- Playbook должен быть связан с репозиторием (`repository_id` не null)
- Репозиторий должен быть доступен

**Ответ:**
```json
{
  "id": 1,
  "project_id": 1,
  "name": "deploy.yml",
  "content": "- hosts: all\n  tasks:\n    - debug:\n        msg: Hello",
  "playbook_type": "ansible",
  "repository_id": 5,
  "updated": "2026-03-12T14:30:00Z"
}
```

**Пример запроса:**
```bash
curl -X POST http://localhost:3000/api/project/1/playbooks/1/sync \
  -H "Authorization: Bearer $TOKEN"
```

### Предварительный просмотр

**GET** `/api/project/{project_id}/playbooks/{id}/preview`

Возвращает содержимое playbook из Git без сохранения в базу данных.

**Ответ:**
```json
"- hosts: all\n  tasks:\n    - debug:\n        msg: Hello"
```

**Пример запроса:**
```bash
curl -X GET http://localhost:3000/api/project/1/playbooks/1/preview \
  -H "Authorization: Bearer $TOKEN"
```

## Как это работает

### Алгоритм синхронизации

1. **Получение playbook**
   - Загружаем playbook из БД
   - Проверяем наличие `repository_id`

2. **Получение репозитория**
   - Загружаем информацию о репозитории
   - Извлекаем URL и параметры подключения

3. **Клонирование репозитория**
   - Создаем временную директорию
   - Клонируем репозиторий через git2

4. **Чтение файла**
   - Определяем путь к файлу playbook
   - Читаем содержимое файла

5. **Обновление БД**
   - Обновляем поле `content` в базе данных
   - Обновляем timestamp `updated`

### Определение пути к файлу

Система автоматически определяет путь к файлу playbook:

1. `playbook_name` (например, "deploy.yml")
2. `playbook_name.yml` (добавляется расширение)
3. `playbook_name.yaml` (альтернативное расширение)
4. `playbooks/playbook_name` (в поддиректории)
5. `playbooks/playbook_name.yml` (в поддиректории с расширением)

Возвращается первый найденный путь.

## Примеры использования

### Создание playbook с последующей синхронизацией

```bash
# 1. Создаем playbook (с repository_id)
curl -X POST http://localhost:3000/api/project/1/playbooks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "deploy.yml",
    "content": "# Placeholder",
    "playbook_type": "ansible",
    "repository_id": 5
  }'

# 2. Синхронизируем из Git
curl -X POST http://localhost:3000/api/project/1/playbooks/1/sync \
  -H "Authorization: Bearer $TOKEN"

# 3. Проверяем результат
curl -X GET http://localhost:3000/api/project/1/playbooks/1 \
  -H "Authorization: Bearer $TOKEN"
```

### Предварительный просмотр перед синхронизацией

```bash
# Просматриваем содержимое
curl -X GET http://localhost:3000/api/project/1/playbooks/1/preview \
  -H "Authorization: Bearer $TOKEN" \
  | jq .

# Если всё хорошо - синхронизируем
curl -X POST http://localhost:3000/api/project/1/playbooks/1/sync \
  -H "Authorization: Bearer $TOKEN"
```

## Ошибки

### Playbook не связан с репозиторием

```json
{
  "error": "Playbook не связан с Git репозиторием"
}
```

**Решение:** Обновите playbook и укажите `repository_id`.

### Репозиторий недоступен

```json
{
  "error": "Git error: failed to resolve address for ..."
}
```

**Решение:** Проверьте URL репозитория и сетевое подключение.

### Файл playbook не найден

```json
{
  "error": "Файл playbook не найден по пути \"/tmp/.../deploy.yml\": No such file or directory"
}
```

**Решение:** Убедитесь, что файл существует в репозитории.

## SSH аутентификация

⚠️ **Внимание:** SSH аутентификация пока не реализована.

Для репозиториев с SSH доступом требуется дополнительная настройка:
- Интеграция с AccessKey для получения SSH ключей
- Настройка SSH agent forwarding

## Реализация

### Файлы

- `src/services/playbook_sync_service.rs` - сервис синхронизации
- `src/api/handlers/playbook.rs` - handlers (sync, preview)
- `src/api/routes.rs` - routes

### Зависимости

- `git2 = "0.20"` - работа с Git
- `tempfile = "3"` - временные директории

## Тесты

```bash
cargo test playbook_sync_service
```

## Roadmap

- [ ] SSH аутентификация через AccessKey
- [ ] Поддержка SSH agent forwarding
- [ ] Периодическая автоматическая синхронизация
- [ ] Webhook при изменении в Git
- [ ] История синхронизаций

---

**Версия:** 0.4.3  
**Дата:** 2026-03-12  
**Статус:** ✅ Реализовано
