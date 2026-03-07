# 🚀 Скрипты запуска Semaphore UI

Этот документ описывает все скрипты для управления демонстрационным окружением Semaphore UI.

## 📋 Обзор скриптов

| Скрипт | Назначение |
|--------|------------|
| `start.sh` | Запуск frontend (Nginx) + PostgreSQL с демо-данными |
| `stop.sh` | Остановка Docker контейнеров |
| `cleanup.sh` | Полная очистка (контейнеры, volumes, сети, образы) |
| `demo-start.sh` | Альтернативный скрипт быстрого старта |

---

## 🎯 Быстрый старт

```bash
# 1. Запуск окружения
./start.sh

# 2. Откройте браузер
# http://localhost

# 3. Остановка (когда закончили)
./stop.sh

# 4. Полная очистка (если нужно удалить все данные)
./cleanup.sh --all
```

---

## 📖 Подробное описание

### start.sh - Запуск окружения

Запускает Docker контейнеры с PostgreSQL и Nginx (frontend).

**Использование:**
```bash
./start.sh [OPTIONS]
```

**Опции:**
| Опция | Описание |
|-------|----------|
| `--build, -b` | Пересобрать Docker образы |
| `--clean, -c` | Очистить volumes перед запуском (удалить данные БД) |
| `--stop, -s` | Остановить сервисы |
| `--restart, -r` | Перезапустить сервисы |
| `--logs, -l` | Показать логи (Ctrl+C для выхода) |
| `--backend` | Запустить backend через cargo |
| `--help, -h` | Показать справку |

**Примеры:**

```bash
# Запуск окружения
./start.sh

# Запуск с пересборкой образов
./start.sh --build

# Запуск с очисткой данных БД
./start.sh --clean

# Запуск backend в этом терминале
./start.sh --backend

# Просмотр логов
./start.sh --logs

# Остановка сервисов
./start.sh --stop

# Перезапуск сервисов
./start.sh --restart
```

**После запуска:**
- Frontend: http://localhost
- PostgreSQL: localhost:5432
- Backend (если запущен): http://localhost:3000

---

### stop.sh - Остановка сервисов

Останавливает Docker контейнеры с возможностью очистки volumes.

**Использование:**
```bash
./stop.sh [OPTIONS]
```

**Опции:**
| Опция | Описание |
|-------|----------|
| `--clean, -c` | Очистить volumes (удалить данные БД) |
| `--all, -a` | Полная очистка (остановка + volumes + сети) |
| `--dry-run` | Показать, что будет сделано (без выполнения) |
| `--force, -f` | Не запрашивать подтверждение |
| `--help, -h` | Показать справку |

**Примеры:**

```bash
# Простая остановка (контейнеры останавливаются, данные сохраняются)
./stop.sh

# Остановка с очисткой данных БД
./stop.sh --clean

# Полная очистка (контейнеры, volumes, сети)
./stop.sh --all

# Показать, что будет удалено (без фактического удаления)
./stop.sh --all --dry-run

# Полная очистка без подтверждения
./stop.sh --all --force
```

---

### cleanup.sh - Полная очистка

Скрипт для остановки и удаления Docker контейнеров, volumes, сетей и образов.

**Использование:**
```bash
./cleanup.sh [OPTIONS]
```

**Опции:**
| Опция | Описание |
|-------|----------|
| `--all, -a` | Удалить всё (контейнеры, volumes, сети, образы) |
| `--volumes, -v` | Удалить volumes (данные БД) |
| `--networks, -n` | Удалить сети |
| `--images, -i` | Удалить образы |
| `--dry-run` | Показать, что будет удалено (без удаления) |
| `--force, -f` | Не запрашивать подтверждение |
| `--help, -h` | Показать справку |

**Примеры:**

```bash
# Удалить volumes (данные БД)
./cleanup.sh --volumes

# Удалить сети
./cleanup.sh --networks

# Удалить образы
./cleanup.sh --images

# Полная очистка всего
./cleanup.sh --all

# Показать, что будет удалено (проверка)
./cleanup.sh --all --dry-run

# Полная очистка без подтверждения
./cleanup.sh --all --force
```

**⚠️ Внимание:** Этот скрипт удаляет данные безвозвратно!

---

### demo-start.sh - Автоматический запуск всего

Скрипт для **полностью автоматического** запуска CRUD демо:
1. ✅ Docker контейнеры (PostgreSQL + Frontend)
2. ✅ Backend (Rust)
3. ✅ Логирование в режиме INFO

**Использование:**
```bash
./demo-start.sh [OPTIONS]
```

**Опции:**
| Опция | Описание |
|-------|----------|
| `--stop` | Остановить все сервисы |
| `--restart` | Перезапустить все сервисы |
| `--logs` | Переключиться в режим просмотра логов |
| `--clean` | Очистить volumes перед запуском |
| `--no-backend` | Не запускать backend (только Docker) |
| `--help, -h` | Эта справка |

**Примеры:**

```bash
# Полный запуск (Docker + Backend + логирование)
./demo-start.sh

# Запуск только Docker сервисов (без backend)
./demo-start.sh --no-backend

# Запуск с очисткой данных БД
./demo-start.sh --clean

# Остановка всех сервисов
./demo-start.sh --stop

# Перезапуск всех сервисов
./demo-start.sh --restart

# Просмотр логов в реальном времени
./demo-start.sh --logs
```

**После запуска:**
- Frontend: http://localhost/demo-crud.html
- Backend API: http://localhost:3000/api
- Swagger: http://localhost:3000/swagger
- Лог скрипта: `logs/demo-start.log`
- Лог backend: `logs/backend.log`

---

## 🔄 Типовые сценарии использования

### Сценарий 1: Быстрый запуск всего (рекомендуется)

```bash
# Запуск Docker + Backend + логирование
./demo-start.sh

# Открыть браузер: http://localhost/demo-crud.html

# Когда закончили - остановка
./demo-start.sh --stop
```

### Сценарий 2: Первый запуск с очисткой

```bash
# Проверка зависимостей
docker --version
docker-compose --version
cargo --version

# Запуск с очисткой старых данных
./demo-start.sh --clean

# Открыть браузер: http://localhost/demo-crud.html
```

### Сценарий 3: Раздельный запуск компонентов

```bash
# Запуск только Docker сервисов (PostgreSQL + Frontend)
./demo-start.sh --no-backend

# В другом терминале - ручной запуск backend
cd rust
cargo run -- server --host 0.0.0.0 --port 3000
```

### Сценарий 4: Отладка и логи

```bash
# Запуск и просмотр логов в реальном времени
./demo-start.sh --logs

# В другом терминале - перезапуск
./demo-start.sh --restart

# Просмотр конкретных логов
tail -f logs/demo-start.log    # лог скрипта
tail -f logs/backend.log       # лог backend
docker-compose logs -f db      # лог БД
docker-compose logs -f frontend # лог frontend
```

### Сценарий 5: Полная очистка и перезапуск

```bash
# Остановка всего
./demo-start.sh --stop

# Полная очистка через cleanup.sh
./cleanup.sh --all

# Запуск заново
./demo-start.sh --clean
```

---

## 🛠️ Полезные команды Docker

```bash
# Статус контейнеров
docker-compose ps

# Логи контейнеров
docker-compose logs -f
docker-compose logs db      # логи БД
docker-compose logs frontend # логи frontend

# Перезапуск конкретного сервиса
docker-compose restart db
docker-compose restart frontend

# Остановка с удалением volumes
docker-compose down -v

# Просмотр volumes
docker volume ls

# Просмотр сетей
docker network ls
```

---

## 🐛 Решение проблем

### Контейнер не запускается

```bash
# Проверить логи
./start.sh --logs

# Перезапустить
./start.sh --restart

# Очистить и запустить заново
./start.sh --clean
```

### Port already in use

```bash
# Найти процесс на порту
lsof -i :80
lsof -i :5432
lsof -i :3000

# Остановить процесс
kill -9 <PID>

# Или остановить все контейнеры
./stop.sh --all
```

### Проблемы с правами доступа

```bash
# Сделать скрипты исполняемыми
chmod +x start.sh stop.sh cleanup.sh demo-start.sh
```

### Backend не подключается к БД

```bash
# Проверить переменную окружения
echo $SEMAPHORE_DB_URL
# Должно быть: postgres://semaphore:semaphore_pass@localhost:5432/semaphore

# Проверить доступность БД
docker-compose exec db pg_isready -U semaphore -d semaphore

# Перезапустить БД
docker-compose restart db
```

---

## 📁 Структура файлов

```
semaphore/
├── start.sh              # Основной скрипт запуска
├── stop.sh               # Скрипт остановки
├── cleanup.sh            # Скрипт полной очистки
├── demo-start.sh         # Альтернативный быстрый старт
├── docker-compose.yml    # Основная конфигурация Docker
├── docker-compose.postgres.yml  # PostgreSQL конфиг
├── scripts/
│   ├── run-postgres.sh   # Запуск с PostgreSQL
│   ├── run-mysql.sh      # Запуск с MySQL
│   └── run-sqlite.sh     # Запуск с SQLite
└── db/
    └── postgres/
        ├── init-demo.sql      # Демо-данные
        └── init-pghba.sh      # Настройка pg_hba
```

---

## 💡 Советы

1. **Используйте `--dry-run`** перед очисткой, чтобы понять, что будет удалено
2. **Сохраняйте данные** - используйте `./stop.sh` вместо `./cleanup.sh`, если хотите сохранить данные БД
3. **Проверяйте логи** при ошибках: `./start.sh --logs`
4. **Backend отдельно** - запускайте backend в отдельном терминале через `./start.sh --backend`
5. **Очищайте аккуратно** - `./cleanup.sh --all` удаляет все данные безвозвратно

---

## 📚 Дополнительная документация

- [ЗАПУСК_ДЕМО.md](ЗАПУСК_ДЕМО.md) - Подробная инструкция по запуску демо
- [CRUD_DEMO.md](CRUD_DEMO.md) - Руководство по CRUD демо
- [API.md](API.md) - Документация API
- [README.md](README.md) - Общая информация о проекте
