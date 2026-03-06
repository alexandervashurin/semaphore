# 🎯 CRUD Демо - Шпаргалка

## 🚀 Быстрый старт

```bash
# 1. Запуск (PostgreSQL + Frontend)
./demo-start.sh

# 2. Запуск Backend (в новом терминале)
./demo-start.sh --backend

# 3. Откройте браузер
http://localhost/demo-crud.html
```

## 👤 Учетные данные

| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `demo123` | Администратор |
| `john.doe` | `demo123` | Менеджер |
| `jane.smith` | `demo123` | Менеджер |
| `devops` | `demo123` | Исполнитель |

## 📋 CRUD Операции

### Проекты (Projects)
- ✅ **Create**: POST `/api/projects`
- ✅ **Read**: GET `/api/projects`
- ✅ **Update**: PUT `/api/projects/{id}`
- ✅ **Delete**: DELETE `/api/projects/{id}`

### Шаблоны (Templates)
- ✅ **Create**: POST `/api/project/{id}/templates`
- ✅ **Read**: GET `/api/project/{id}/templates`
- ✅ **Update**: PUT `/api/project/{id}/templates/{id}`
- ✅ **Delete**: DELETE `/api/project/{id}/templates/{id}`

### Задачи (Tasks)
- ✅ **Create**: POST `/api/project/{id}/tasks`
- ✅ **Read**: GET `/api/project/{id}/tasks`

### Инвентарь (Inventory)
- ✅ **Create**: POST `/api/project/{id}/inventory`
- ✅ **Read**: GET `/api/project/{id}/inventory`
- ✅ **Update**: PUT `/api/project/{id}/inventory/{id}`
- ✅ **Delete**: DELETE `/api/project/{id}/inventory/{id}`

### Репозитории (Repositories)
- ✅ **Create**: POST `/api/project/{id}/repository`
- ✅ **Read**: GET `/api/project/{id}/repository`
- ✅ **Update**: PUT `/api/project/{id}/repository/{id}`
- ✅ **Delete**: DELETE `/api/project/{id}/repository/{id}`

### Окружения (Environments)
- ✅ **Create**: POST `/api/project/{id}/environment`
- ✅ **Read**: GET `/api/project/{id}/environment`
- ✅ **Update**: PUT `/api/project/{id}/environment/{id}`
- ✅ **Delete**: DELETE `/api/project/{id}/environment/{id}`

### Ключи доступа (Keys)
- ✅ **Create**: POST `/api/project/{id}/keys`
- ✅ **Read**: GET `/api/project/{id}/keys`
- ✅ **Update**: PUT `/api/project/{id}/keys/{id}`
- ✅ **Delete**: DELETE `/api/project/{id}/keys/{id}`

## 🔧 Команды управления

```bash
# Запуск
./demo-start.sh

# Запуск backend
./demo-start.sh --backend

# Остановка
./demo-start.sh --stop

# Перезапуск
./demo-start.sh --restart

# Просмотр логов
./demo-start.sh --logs

# Сброс данных
./demo-start.sh --reset

# Справка
./demo-start.sh --help
```

## 📖 Документация

- [CRUD_DEMO.md](CRUD_DEMO.md) - Полное руководство
- [API.md](API.md) - Документация API
- [README.md](README.md) - Общая информация

## 🐛 Решение проблем

### Backend не подключается к БД
```bash
# Проверьте PostgreSQL
docker-compose ps

# Перезапустите БД
docker-compose restart db
```

### Frontend не загружается
```bash
# Пересоберите frontend
cd web && ./build.sh

# Перезапустите контейнер
docker-compose restart frontend
```

### Сброс демо-данных
```bash
# Полный сброс
./demo-start.sh --reset
```

## 📞 Поддержка

Возникли проблемы? Проверьте:
1. Логи: `./demo-start.sh --logs`
2. Статус: `docker-compose ps`
3. Документацию: `CRUD_DEMO.md`
