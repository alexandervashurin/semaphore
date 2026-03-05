# 🐳 Запуск Semaphore UI через Docker

Полный стек Semaphore UI (frontend + backend + PostgreSQL с демо-данными) запускается одной командой.

## 🚀 Быстрый старт

```bash
# Запуск Semaphore UI
./start.sh
```

Это все, что нужно! Скрипт автоматически:
1. ✅ Проверит наличие Docker
2. ✅ Соберет frontend (если не собран)
3. ✅ Соберет Docker-образы
4. ✅ Запустит PostgreSQL с демо-данными
5. ✅ Запустит backend (Rust)
6. ✅ Предоставит доступ к веб-интерфейсу

## 📋 Доступ к системе

После запуска откройте в браузере:

- **URL**: http://localhost:3000
- **Логин**: `admin`
- **Пароль**: `admin123`

### Демо-пользователи

Включены 4 тестовых пользователя:

| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `demo123` | Admin |
| `john.doe` | `demo123` | Manager |
| `jane.smith` | `demo123` | Developer |
| `devops` | `demo123` | DevOps |

## 🛠 Команды управления

### Запуск

```bash
# Обычный запуск
./start.sh

# Запуск с пересборкой образов
./start.sh --build

# Полный сброс и запуск (удаление данных БД)
./start.sh --clean --build
```

### Остановка

```bash
# Остановка сервисов
./stop.sh

# Остановка с очисткой данных
./stop.sh --clean
```

### Просмотр логов

```bash
# Просмотр логов всех сервисов
./start.sh --logs

# Или через docker-compose
docker-compose logs -f
```

### Перезапуск

```bash
./start.sh --restart
```

## 📦 Что входит в стек

| Сервис | Образ | Порт | Описание |
|--------|-------|------|----------|
| **db** | `postgres:15-alpine` | 5432 (внутренний) | PostgreSQL с демо-данными |
| **backend** | Кастомный (Rust) | 3000 | Backend на Rust |
| **frontend-build** | `node:18-alpine` | - | Сборка frontend (одноразово) |

## 🗂 Структура

```
semaphore/
├── docker-compose.yml       # Конфигурация Docker Compose
├── Dockerfile              # Dockerfile для backend
├── start.sh                # Скрипт запуска
├── stop.sh                 # Скрипт остановки
├── web/
│   ├── Dockerfile.build    # Dockerfile для сборки frontend
│   ├── build.sh            # Скрипт сборки frontend
│   └── public/             # Скомпилированный frontend
└── db/postgres/
    └── init-demo.sql       # Демонстрационные данные
```

## 🔧 Требования

- **Docker**: 20.x или новее
- **Docker Compose**: 2.x или новее

### Установка Docker (Linux)

```bash
# Автоматическая установка
curl -fsSL https://get.docker.com | sh

# Добавить пользователя в группу docker
sudo usermod -aG docker $USER

# Перелогиньтесь
```

## 💾 Хранение данных

Данные PostgreSQL хранятся в Docker volume `postgres_data`.

### Резервное копирование

```bash
# Экспорт БД
docker-compose exec db pg_dump -U semaphore semaphore > backup.sql

# Импорт БД
docker-compose exec -T db psql -U semaphore semaphore < backup.sql
```

### Очистка данных

```bash
# Остановка с очисткой volumes
./stop.sh --clean

# Или вручную
docker-compose down -v
```

## 🔍 Диагностика

### Проверка статуса сервисов

```bash
docker-compose ps
```

### Просмотр логов

```bash
# Все сервисы
docker-compose logs -f

# Только backend
docker-compose logs -f backend

# Только БД
docker-compose logs -f db
```

### Проверка готовности БД

```bash
docker-compose exec db pg_isready -U semaphore -d semaphore
```

### Подключение к БД

```bash
# psql внутри контейнера
docker-compose exec db psql -U semaphore -d semaphore

# Или локально (если проброшен порт)
psql -h localhost -U semaphore -d semaphore
```

## ⚙️ Конфигурация

### Переменные окружения

Можно настроить через `.env` файл или переменные окружения:

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_DB_HOST` | Хост БД | `db` |
| `SEMAPHORE_DB_PORT` | Порт БД | `5432` |
| `SEMAPHORE_DB_NAME` | Имя БД | `semaphore` |
| `SEMAPHORE_DB_USER` | Пользователь БД | `semaphore` |
| `SEMAPHORE_DB_PASS` | Пароль БД | `semaphore123` |
| `SEMAPHORE_ADMIN` | Логин админа | `admin` |
| `SEMAPHORE_ADMIN_PASSWORD` | Пароль админа | `admin123` |

### Изменение порта

Отредактируйте `docker-compose.yml`:

```yaml
services:
  backend:
    ports:
      - "8080:3000"  # Измените 3000 на нужный порт
```

## 🐛 Решение проблем

### "Cannot connect to the Docker daemon"

```bash
# Проверьте, что Docker запущен
sudo systemctl status docker

# Запустите Docker
sudo systemctl start docker
```

### "Port 3000 is already in use"

```bash
# Найдите процесс на порту 3000
lsof -i :3000

# Остановите процесс или измените порт в docker-compose.yml
```

### "Database is not ready"

```bash
# Проверьте логи БД
docker-compose logs db

# Перезапустите БД
docker-compose restart db
```

### "Frontend не загружается"

```bash
# Проверьте, что frontend собран
ls -lh web/public/

# Пересоберите frontend
./web/build.sh

# Перезапустите backend
docker-compose restart backend
```

### Сброс к начальным условиям

```bash
# Полная очистка и запуск
./stop.sh --clean
./start.sh --build
```

## 📚 Дополнительная документация

- [README.md](README.md) - основная документация
- [CONFIG.md](CONFIG.md) - конфигурация
- [db/postgres/DEMO.md](db/postgres/DEMO.md) - демонстрационные данные
- [web/DOCKER_BUILD.md](web/DOCKER_BUILD.md) - сборка frontend

## 🎯 Следующие шаги

После запуска:

1. Откройте http://localhost:3000
2. Войдите как `admin` / `admin123`
3. Изучите демонстрационные проекты
4. Создайте свой первый шаблон задачи
5. Запустите задачу!

🎉 Приятной работы с Semaphore UI!
