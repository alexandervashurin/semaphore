# 🐳 Сборка frontend через Docker

## Быстрый старт

### Сборка frontend (не требует Node.js)

```bash
# Из корня проекта
./web/build.sh
```

### Или через Taskfile

```bash
# Сборка только frontend
task build:frontend

# Полная сборка (frontend + backend)
task build
```

## Что происходит

1. Скрипт `./web/build.sh`:
   - Проверяет наличие Docker и Docker Compose
   - Запускает контейнер `node:18-alpine`
   - Устанавливает зависимости (`npm ci`)
   - Собирает Vue-приложение (`npm run build`)
   - Копирует результат в `web/public/`

2. После сборки:
   ```
   web/public/
   ├── index.html    # Vue-приложение
   ├── app.js        # ~500-800 KB
   ├── app.css       # ~100-150 KB
   ├── flags/        # Языковые флаги
   └── favicon.*     # Иконки
   ```

## Требования

- **Docker**: 20.x или новее
- **Docker Compose**: 2.x или новее (или `docker compose` v2)

### Установка Docker (Linux)

```bash
# Автоматическая установка
curl -fsSL https://get.docker.com | sh

# Добавить пользователя в группу docker
sudo usermod -aG docker $USER

# Перелогиньтесь или выполните:
newgrp docker

# Проверка
docker --version
docker compose version
```

## Ручная сборка через Docker

```bash
# Прямой вызов docker-compose
cd web
docker-compose -f docker-compose.build.yml build

# Или docker compose v2
docker compose -f docker-compose.build.yml build
```

## Проверка результата

```bash
# Проверка файлов
ls -lh web/public/*.js web/public/*.css

# Проверка размера
du -sh web/public/
```

## Запуск приложения

```bash
# Из корня проекта
cd rust
cargo run -- server

# Frontend доступен на http://localhost:3000
```

## Преимущества Docker-сборки

| Характеристика | Docker | Node.js на хосте |
|----------------|--------|------------------|
| **Требования** | Только Docker | Node.js + npm |
| **Чистота системы** | ✅ Изолированно | ❌ Зависимости в системе |
| **Воспроизводимость** | ✅ 100% | ⚠️ Зависит от версии |
| **Для продакшена** | ✅ Рекомендуется | ❌ Не рекомендуется |
| **Для разработки** | ⚠️ Медленнее | ✅ Быстрее (hot-reload) |

## Решение проблем

### "docker: команда не найдена"

```bash
# Установите Docker
curl -fsSL https://get.docker.com | sh
```

### "docker-compose: команда не найдена"

```bash
# Docker Compose v2 использует команду без дефиса
docker compose -f web/docker-compose.build.yml build
```

### "permission denied"

```bash
# Добавьте пользователя в группу docker
sudo usermod -aG docker $USER
# Перелогиньтесь
```

## Документация

- [web/README_BUILD.md](web/README_BUILD.md) - полное руководство
- [README.md](../README.md) - основная документация
