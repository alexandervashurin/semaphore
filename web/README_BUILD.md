# 📦 Сборка фронтенда Velum

Этот документ описывает процесс сборки frontend-части Velum.

## 📋 Обзор

Frontend Velum построен на:
- **Vue 2.6** - реактивный фреймворк
- **Vuetify 2.6** - UI-компоненты Material Design
- **Vue Router 3.5** - маршрутизация
- **Axios** - HTTP-клиент
- **vue-i18n** - интернационализация (15 языков)

## ✅ Все ресурсы локальные

После сборки **все ресурсы находятся локально** - не требуется загрузка из интернета:

| Ресурс | Расположение |
|--------|--------------|
| Шрифты Roboto | `web/src/assets/fonts/*.ttf` |
| Иконки MDI | `node_modules/@mdi/font/` (собираются в бандл) |
| Кастомные иконки | Vue-компоненты в `web/src/components/` |
| Флаги языков | `web/public/flags/*.svg` |
| Favicon | `web/public/favicon.png`, `web/public/favicon.svg` |

## 🔧 Требования

### Вариант 1: Docker (рекомендуется для сервера)

- **Docker**: 20.x или новее
- **Docker Compose**: 2.x или новее

**Преимущества:**
- ✅ Не требует установки Node.js на сервере
- ✅ Изолированная среда сборки
- ✅ Воспроизводимые результаты
- ✅ Чистая система после сборки

### Вариант 2: Node.js (для разработки)

- **Node.js**: 16.x или новее
- **npm**: 7.x или новее

## 🚀 Быстрая сборка

### Через Docker (рекомендуется)

```bash
# Перейти в директорию проекта
cd /path/to/semaphore

# Сборка frontend через Docker
./web/build.sh

# Или через Taskfile
task build:frontend
```

### Через Node.js (для разработки)

```bash
# Перейти в директорию frontend
cd web

# Установить зависимости
npm install

# Собрать production-версию
npm run build
```

После сборки:
- Vue-приложение будет в `web/public/`
- `app.js` - основной JavaScript файл
- `app.css` - основные стили
- `index.html` - точка входа

## 📦 Полная сборка проекта

### Через Taskfile (рекомендуется)

```bash
# Сборка frontend + backend
task build
```

### Пошаговая сборка

```bash
# 1. Сборка frontend
cd web
npm install
npm run build
cd ..

# 2. Сборка backend
cd rust
cargo build --release
```

## 🛠 Команды для сборки

### Docker (рекомендуется)

| Команда | Описание |
|---------|----------|
| `./web/build.sh` | Скрипт сборки через Docker |
| `task build:frontend` | Сборка через Taskfile |
| `docker-compose -f web/docker-compose.build.yml build` | Прямой вызов docker-compose |

### Node.js (для разработки)

| Команда | Описание |
|---------|----------|
| `npm run serve` | Запуск dev-сервера с hot-reload |
| `npm run build` | Production-сборка |
| `npm run build:clean` | Очистка и сборка |
| `npm run lint` | Проверка кода |
| `npm run test:unit` | Unit-тесты |

## 🌐 Настройка проксирования

В режиме разработки (`npm run serve`) frontend проксирует API-запросы на backend:

```javascript
// vue.config.js
devServer: {
  proxy: {
    '^/api': {
      target: 'http://localhost:3000',
    },
  },
}
```

## 🔧 Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_WEB_PATH` | Путь к фронтенду | `./web/public` |
| `VUE_APP_BUILD_TYPE` | Тип сборки | - |

## 📁 Структура после сборки

```
web/public/
├── index.html          # Точка входа (Vue-приложение)
├── app.js              # Основной JS бандл
├── app.css             # Основные стили
├── favicon.png         # Иконка
├── favicon.svg         # SVG иконка
├── flags/              # Флаги языков
│   ├── en.svg
│   ├── ru.svg
│   └── ...
└── js/                 # Chunk-файлы (ленивая загрузка)
    ├── chunk-vendors.js
    └── ...
```

## 🐛 Решение проблем

### Ошибка: "docker: команда не найдена"

Установите Docker:
- Linux: `curl -fsSL https://get.docker.com | sh`
- macOS: установите Docker Desktop с https://docker.com/
- Windows: установите Docker Desktop с https://docker.com/

### Ошибка: "docker-compose: команда не найдена"

Docker Compose v2 использует команду `docker compose` (без дефиса):
```bash
docker compose -f web/docker-compose.build.yml build
```

Или установите docker-compose:
- Linux: `apt-get install docker-compose-plugin`
- macOS: входит в состав Docker Desktop

### Ошибка сборки в Docker: "permission denied"

```bash
# Запуск от root (если нужно)
sudo ./web/build.sh

# Или добавьте пользователя в группу docker
sudo usermod -aG docker $USER
# Затем перелогиньтесь
```

### Ошибка: "Cannot find module"

При сборке через Docker эта ошибка невозможна, так как зависимости устанавливаются в контейнере.

Если используете Node.js напрямую:
```bash
cd web
rm -rf node_modules package-lock.json
npm install
```

### Ошибка сборки: "JavaScript heap out of memory"

При сборке в Docker увеличьте лимит памяти:
```bash
export NODE_OPTIONS="--max-old-space-size=4096"
./web/build.sh
```

### Фронтенд не загружается

1. Проверьте, что `web/public/` существует:
   ```bash
   ls -la web/public/
   ```

2. Проверьте переменную окружения:
   ```bash
   echo $SEMAPHORE_WEB_PATH
   ```

3. Пересоберите frontend:
   ```bash
   ./web/build.sh
   ```

## 📊 Размер бандлов

После сборки типичные размеры файлов:

| Файл | Размер (сжатый) |
|------|-----------------|
| `app.js` | ~500-800 KB |
| `app.css` | ~100-150 KB |
| `chunk-vendors.js` | ~300-500 KB |

**Общий размер**: ~1-1.5 MB

## 🔒 Безопасность

- Все зависимости проверяются через `npm audit`
- Исходный код минифицируется
- Source maps не включаются в production
- Нет внешних CDN-зависимостей

## 📚 Дополнительная документация

- [README.md](../README.md) - основная документация
- [CONFIG.md](../CONFIG.md) - конфигурация
- [API.md](../API.md) - API документация
