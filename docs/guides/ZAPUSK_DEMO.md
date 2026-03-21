# 🚀 ЗАПУСК Velum - ПОШАГОВАЯ ИНСТРУКЦИЯ

## ⚡ Быстрый старт (1 команда)

```bash
# Запуск режима Native (SQLite, минимальные зависимости)
./semaphore.sh init native    # первый запуск
./semaphore.sh start native   # последующие запуски
```

**Доступ:** http://localhost:3000

**Учётные данные:**
| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `admin123` | Администратор |

---

## 📋 Режимы запуска

### Режим 1: Native (чистый запуск на хосте)

Всё на хосте: SQLite + Backend + Frontend. Минимальные зависимости - только Rust.

```bash
# Первый запуск с инициализацией
./semaphore.sh init native

# Запуск сервера
./semaphore.sh start native
```

**Доступ:** http://localhost:3000

**Учётные данные:**
- `admin` / `admin123`

**Полезные команды:**
```bash
./semaphore.sh stop              # Остановить backend
./semaphore.sh logs              # Просмотр логов
./semaphore.sh clean             # Удалить БД
./semaphore.sh init native       # Инициализировать БД
./semaphore.sh build             # Пересобрать backend
./semaphore.sh status            # Показать статус
```

**Когда использовать:**
- ✅ Быстрое тестирование
- ✅ Разработка без Docker
- ✅ Минимальные зависимости

---

### Режим 2: Hybrid (PostgreSQL в Docker, остальное на хосте) ⭐ Рекомендуется

PostgreSQL в Docker контейнере, Backend и Frontend на хосте.

```bash
# Первый запуск с инициализацией
./semaphore.sh init hybrid

# Запуск сервера
./semaphore.sh start hybrid
```

**Доступ:** http://localhost:3000

**Учётные данные (демо):**
- `admin` / `demo123`

**Полезные команды:**
```bash
./semaphore.sh stop              # Остановить сервисы
./semaphore.sh logs              # Просмотр логов
./semaphore.sh clean             # Очистить данные БД
./semaphore.sh init hybrid       # Инициализировать БД
./semaphore.sh build             # Пересобрать backend
./semaphore.sh status            # Показать статус
```

**Когда использовать:**
- ✅ Продакшен окружение
- ✅ Тестирование с PostgreSQL
- ✅ Быстрая разработка с полноценной БД

---

### Режим 3: Docker (всё в Docker)

Все сервисы в Docker контейнерах: Frontend + PostgreSQL + Backend.

```bash
# Запуск всех сервисов
./semaphore.sh start docker
```

**Доступ:** http://localhost

**Учётные данные (демо):**
- `admin` / `demo123`

**Полезные команды:**
```bash
./semaphore.sh stop              # Остановить сервисы
./semaphore.sh logs              # Просмотр логов
./semaphore.sh clean             # Очистить volumes
./semaphore.sh build             # Пересобрать образы
./semaphore.sh status            # Показать статус
```

**Когда использовать:**
- ✅ Полная изоляция окружения
- ✅ Продакшен развёртывание
- ✅ Тестирование в изолированной среде

---

## 🔍 Проверка работы

### Тестирование API

```bash
# Проверка доступности API
curl http://localhost:3000/api

# Вход и получение токена (для native режима)
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'

# Вход для hybrid/docker режимов
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"demo123"}'

# Использование токена
TOKEN="eyJ..."
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:3000/api/projects
```

### Проверка статуса

```bash
# Для native режима
ps aux | grep semaphore
cat logs/backend.log

# Для hybrid режима
docker ps --filter "name=semaphore-db"
./start.sh hybrid --logs

# Для docker режима
docker ps --filter "name=semaphore"
./start.sh docker --logs
```

---

## 🛑 Остановка

```bash
# Остановка native режима
./start.sh native --stop

# Остановка hybrid режима
./start.sh hybrid --stop

# Остановка docker режима
./start.sh docker --stop
```

---

## 🐛 Решение проблем

### Проблема: "Address already in use"

**Решение:**
```bash
# Остановить процессы на порту 3000
pkill -f "semaphore server"

# Или через lsof
lsof -ti:3000 | xargs kill -9

# Запустить снова
./start.sh native
```

### Проблема: "Docker не найден"

**Решение:**
```bash
# Используйте native режим (не требует Docker)
./start.sh native

# Или установите Docker
# Для Ubuntu/Debian:
sudo apt-get update
sudo apt-get install docker.io docker-compose

# Для macOS/Windows: установите Docker Desktop
```

### Проблема: Backend не подключается к БД

**Решение:**
```bash
# Для hybrid режима - проверьте PostgreSQL
docker logs semaphore-db
./start.sh hybrid --restart

# Для native режима - проверьте SQLite
ls -la data/semaphore.db
./start.sh native --clean
./start.sh native --init
```

### Проблема: Frontend не загружается

**Решение:**
```bash
# Проверьте наличие frontend файлов
ls -la web/public/

# Соберите frontend
cd web && ./build.sh

# Перезапустите
./start.sh native --stop
./start.sh native
```

### Проблема: "Permission denied"

**Решение:**
```bash
# Сделайте скрипты исполняемыми
chmod +x start.sh
chmod +x web/build.sh
```

---

## 📖 Дополнительная документация

- **README.md** - Общая информация о проекте
- **CRUD_DEMO.md** - CRUD демо руководство
- **API.md** - REST API документация
- **CONFIG.md** - Переменные окружения
- **AUTH.md** - Аутентификация и авторизация

---

## 💡 Советы

1. **Native режим** идеален для быстрой разработки и тестирования
2. **Hybrid режим** - лучший баланс для продакшена (БД в Docker, остальное на хосте)
3. **Docker режим** подходит для полной изоляции окружения
4. **Используйте --init** для первого запуска в любом режиме
5. **Проверяйте логи** при ошибках: `./start.sh <режим> --logs`

---

## 🎯 Что дальше?

После запуска:

1. Войдите в систему под `admin` / `admin123` (native) или `admin` / `demo123` (hybrid/docker)
2. Изучите CRUD демо: http://localhost:3000/demo-crud.html
3. Создайте свой первый проект
4. Настройте шаблоны и инвентари
5. Запустите задачу (task)

**Приятной работы с Velum! 🚀**
