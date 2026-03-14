# 🐳 Демонстрационное окружение Semaphore UI

**Frontend (Nginx) + PostgreSQL с демо-данными**. Backend запускается отдельно через `cargo`.

## 🚀 Быстрый старт

### 1️⃣ Запуск frontend и БД

```bash
./start.sh
```

### 2️⃣ Запуск backend (отдельно)

```bash
# Вариант 1: Через скрипт
./start.sh --backend

# Вариант 2: Напрямую
cd rust
cargo run -- server --host 0.0.0.0 --port 3000
```

### 3️⃣ Открыть браузер

- **Frontend**: http://localhost
- **Логин**: `admin`
- **Пароль**: `demo123`

---

## 📋 Полная информация

| Компонент | URL | Описание |
|-----------|-----|----------|
| **Frontend** | http://localhost | Nginx раздает Vue статику |
| **Backend API** | http://localhost:3000/api | Rust backend |
| **PostgreSQL** | localhost:5432 | БД с демо-данными |

### Демо-пользователи

| Логин | Пароль | Роль |
|-------|--------|------|
| `admin` | `demo123` | Admin |
| `john.doe` | `demo123` | Manager |
| `jane.smith` | `demo123` | Developer |
| `devops` | `demo123` | DevOps |

---

## 🛠 Команды управления

### Запуск

```bash
# Только frontend + БД
./start.sh

# Frontend + БД + Backend
./start.sh --backend

# С пересборкой образов
./start.sh --build

# С полным сбросом (удаление данных БД)
./start.sh --clean --build
```

### Остановка

```bash
# Остановка сервисов
./stop.sh

# Остановка с очисткой БД
./stop.sh --clean
```

### Логи

```bash
# Все сервисы
./start.sh --logs

# Только БД
docker-compose logs -f db

# Только frontend
docker-compose logs -f frontend
```

### Статус

```bash
# Через docker-compose
docker-compose ps

# Через скрипт
./start.sh  # покажет статус при запуске
```

---

## 📦 Что входит в стек

| Сервис | Образ | Порт | Назначение |
|--------|-------|------|------------|
| **db** | `postgres:15-alpine` | 5432 | PostgreSQL с демо-данными |
| **frontend** | `nginx:alpine` | 80 | Раздача Vue статики |

**Backend не входит в Docker** — запускается отдельно через `cargo run`.

### Почему так?

- ✅ **Backend с отладкой** — `cargo run` показывает логи в реальном времени
- ✅ **Меньше ресурсов** — не нужно собирать Docker-образ backend
- ✅ **Быстрая разработка** — изменения в коде backend применяются сразу
- ✅ **Проще деплой** — в продакшене backend запускается отдельно

---

## 🔧 Требования

- **Docker**: 20.x или новее
- **Docker Compose**: 2.x или новее
- **Rust**: 1.75+ (для запуска backend)
- **Node.js**: 16+ (опционально, для сборки frontend)

### Установка Docker

```bash
# Автоматическая установка (Linux)
curl -fsSL https://get.docker.com | sh

# Добавить пользователя в группу docker
sudo usermod -aG docker $USER

# Перелогиньтесь или выполните: newgrp docker
```

---

## 🗂 Структура файлов

```
semaphore/
├── docker-compose.yml       # Конфигурация Docker (frontend + БД)
├── nginx.conf               # Конфигурация Nginx
├── start.sh                 # Скрипт запуска
├── stop.sh                  # Скрипт остановки
├── DOCKER_DEMO.md           # Эта документация
├── rust/                    # Backend
│   └── src/
└── web/
    ├── public/              # Скомпилированный frontend
    ├── build.sh             # Сборка frontend через Docker
    └── Dockerfile.build     # Dockerfile для сборки
```

---

## 🔍 Диагностика

### Статус сервисов

```bash
docker-compose ps
```

### Проверка БД

```bash
# Проверка готовности
docker-compose exec db pg_isready -U semaphore -d semaphore

# Подключение к БД
docker-compose exec db psql -U semaphore -d semaphore

# Список таблиц
docker-compose exec db psql -U semaphore -d semaphore -c "\dt"
```

### Проверка frontend

```bash
# Проверка файлов
ls -lh web/public/

# Тест Nginx
curl http://localhost

# Проверка подключения к API
curl http://localhost/api/health
```

### Проверка backend

```bash
# Проверка доступности
curl http://localhost:3000/api/health

# Логи backend (если запущен через cargo)
# Смотрите в терминале, где запущен cargo
```

---

## 💾 Хранение данных

Данные PostgreSQL хранятся в Docker volume `postgres_data`.

### Резервное копирование

```bash
# Экспорт БД
docker-compose exec db pg_dump -U semaphore semaphore > backup.sql

# Импорт БД
docker-compose exec -T db psql -U semaphore semaphore < backup.sql
```

### Сброс данных

```bash
# Остановка с очисткой volumes
./stop.sh --clean

# Запуск с чистой БД
./start.sh
```

---

## ⚙️ Конфигурация

### Переменные окружения БД

| Переменная | Значение |
|------------|----------|
| `POSTGRES_DB` | semaphore |
| `POSTGRES_USER` | semaphore |
| `POSTGRES_PASSWORD` | semaphore_pass |
| `POSTGRES_HOST` | localhost |
| `POSTGRES_PORT` | 5432 |

### Переменные окружения backend

**ВАЖНО:** Для Rust версии необходимо использовать `SEMAPHORE_DB_URL` вместо отдельных переменных!

```bash
export SEMAPHORE_DB_URL="postgres://semaphore:semaphore_pass@localhost:5432/semaphore"
export SEMAPHORE_WEB_PATH=./web/public
```

### Изменение порта frontend

Отредактируйте `docker-compose.yml`:

```yaml
services:
  frontend:
    ports:
      - "8080:80"  # Измените 80 на 8080
```

### Изменение порта backend

В `nginx.conf` измените `proxy_pass`:

```nginx
location /api/ {
    proxy_pass http://host.docker.internal:3001/api/;  # Было :3000
}
```

---

## 🐛 Решение проблем

### "Cannot connect to the Docker daemon"

```bash
# Проверьте статус Docker
sudo systemctl status docker

# Запустите Docker
sudo systemctl start docker
```

### "Port 80 is already in use"

```bash
# Найдите процесс на порту 80
sudo lsof -i :80

# Остановите процесс или измените порт в docker-compose.yml
```

### "Port 5432 is already in use"

```bash
# Найдите процесс на порту 5432
sudo lsof -i :5432

# Остановите PostgreSQL на хосте или измените порт в docker-compose.yml
```

### "Backend не подключается к БД"

1. Проверьте переменные окружения:
   ```bash
   echo $SEMAPHORE_DB_HOST
   echo $SEMAPHORE_DB_PORT
   ```

2. Проверьте доступность БД:
   ```bash
   docker-compose exec db pg_isready -U semaphore -d semaphore
   ```

3. Проверьте логи БД:
   ```bash
   docker-compose logs db
   ```

### "Frontend показывает ошибку подключения к API"

1. Убедитесь, что backend запущен:
   ```bash
   curl http://localhost:3000/api/health
   ```

2. Проверьте логи backend

3. Проверьте `nginx.conf`:
   ```bash
   cat nginx.conf | grep proxy_pass
   ```

### "Nginx возвращает 502 Bad Gateway"

Backend не запущен или недоступен:

```bash
# Проверьте backend
curl http://localhost:3000/api/health

# Перезапустите backend
pkill semaphore
cd rust && cargo run -- server
```

### "Ошибки сборки frontend"

```bash
# Очистите и пересоберите
cd web
rm -rf public/app.js public/app.css public/js
./build.sh
```

---

## 📚 Дополнительная документация

- [README.md](README.md) — основная документация проекта
- [QUICK_START.md](QUICK_START.md) — краткая шпаргалка
- [db/postgres/DEMO.md](db/postgres/DEMO.md) — описание демо-данных
- [web/DOCKER_BUILD.md](web/DOCKER_BUILD.md) — сборка frontend через Docker
- [CONFIG.md](CONFIG.md) — конфигурация Semaphore

---

## 🎯 Следующие шаги

1. ✅ Запустите `./start.sh`
2. ✅ Запустите backend: `./start.sh --backend`
3. ✅ Откройте http://localhost
4. ✅ Войдите как `admin` / `demo123`
5. ✅ Изучите демонстрационные проекты
6. ✅ Создайте свой первый шаблон
7. ✅ Запустите задачу!

🎉 Приятной работы с Semaphore UI!
