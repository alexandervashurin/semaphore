# Semaphore Desktop Application

> **Нативное desktop приложение для Semaphore UI на базе Tauri**

## 📖 Оглавление

- [Обзор](#обзор)
- [Возможности](#возможности)
- [Установка](#установка)
- [Использование](#использование)
- [Сборка](#сборка)
- [Архитектура](#архитектура)

---

## 📋 Обзор

Semaphore Desktop — это нативное desktop приложение для управления Semaphore UI, построенное на базе фреймворка Tauri.

**Преимущества:**

| Преимущество | Описание |
|--------------|----------|
| **Нативный UI** | Интеграция с системным треем и уведомлениями |
| **Безопасность** | Изолированное выполнение, нет доступа к системе без разрешения |
| **Производительность** | Минимальное потребление памяти (~50 MB) |
| **Кроссплатформенность** | Linux, Windows, macOS из одной кодовой базы |
| **Малый размер** | Бинарник ~15 MB (vs 150+ MB для Electron) |

---

## ✨ Возможности

### Основные

- ✅ Подключение к Semaphore серверу
- ✅ Просмотр проектов и задач
- ✅ Запуск задач из трея
- ✅ Системные уведомления о задачах
- ✅ Автообновление статуса задач

### Системная интеграция

- ✅ Системный трей иконка
- ✅ Уведомления через ОС
- ✅ Горячие клавиши
- ✅ Автозапуск

---

## 🚀 Установка

### Требования

- **OS:** Linux / Windows 10+ / macOS 10.15+
- **RAM:** 512 MB
- **Disk:** 100 MB

### Linux (DEB)

```bash
# Скачать .deb пакет
wget https://github.com/alexandervashurin/semaphore/releases/download/v0.4.0/semaphore-desktop_0.4.0_amd64.deb

# Установить
sudo dpkg -i semaphore-desktop_0.4.0_amd64.deb

# Запустить
semaphore-desktop
```

### Linux (RPM)

```bash
# Скачать .rpm пакет
wget https://github.com/alexandervashurin/semaphore/releases/download/v0.4.0/semaphore-desktop-0.4.0-1.x86_64.rpm

# Установить
sudo rpm -i semaphore-desktop-0.4.0-1.x86_64.rpm

# Запустить
semaphore-desktop
```

### macOS

```bash
# Скачать .dmg
wget https://github.com/alexandervashurin/semaphore/releases/download/v0.4.0/Semaphore.UI_0.4.0_x64.dmg

# Смонтировать и перетащить в Applications
hdiutil attach Semaphore.UI_0.4.0_x64.dmg
cp -r /Volumes/Semaphore\ UI /Applications/
```

### Windows

```powershell
# Скачать .msi
wget https://github.com/alexandervashurin/semaphore/releases/download/v0.4.0/Semaphore.UI_0.4.0_x64_en-US.msi

# Установить
msiexec /i Semaphore.UI_0.4.0_x64_en-US.msi
```

---

## 💡 Использование

### Первое подключение

1. Запустите приложение
2. Введите URL сервера (например, `http://localhost:3000`)
3. Введите API токен (можно получить в настройках Semaphore)
4. Нажмите "Подключиться"

### Системный трей

После подключения приложение сворачивается в системный трей:

- **Левый клик** — открыть окно
- **Правый клик** — контекстное меню
  - Показать
  - Проверить обновления
  - Выход

### Уведомления

Приложение отправляет уведомления при:

- Завершении задачи
- Ошибке подключения
- Обновлении статуса

---

## 🔨 Сборка

### Требования для сборки

```bash
# Node.js 18+
node --version

# Rust 1.70+
rustc --version

# Tauri CLI
cargo install tauri-cli
```

### Установка зависимостей

```bash
cd desktop

# Frontend зависимости
npm install

# Tauri зависимости
cd src-tauri
cargo build
```

### Development режим

```bash
# Запуск в режиме разработки
npm run tauri dev
```

### Production сборка

```bash
# Сборка всех таргетов
npm run tauri build

# Сборка для конкретной платформы
npm run tauri build -- --target x86_64-unknown-linux-gnu
npm run tauri build -- --target x86_64-pc-windows-msvc
npm run tauri build -- --target x86_64-apple-darwin
```

### Результаты сборки

```
desktop/
├── src-tauri/
│   └── target/
│       └── release/
│           ├── bundle/
│           │   ├── deb/
│           │   │   └── semaphore-desktop_0.4.0_amd64.deb
│           │   ├── rpm/
│           │   │   └── semaphore-desktop-0.4.0-1.x86_64.rpm
│           │   ├── dmg/
│           │   │   └── Semaphore.UI_0.4.0_x64.dmg
│           │   └── msi/
│           │       └── Semaphore.UI_0.4.0_x64_en-US.msi
│           └── semaphore-desktop
```

---

## 🏗️ Архитектура

### Структура проекта

```
desktop/
├── src/                    # Frontend (Vue.js + Vite)
│   ├── index.html
│   └── main.js
├── src-tauri/              # Backend (Rust + Tauri)
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── package.json
└── vite.config.js
```

### Tauri Commands

| Command | Описание |
|---------|----------|
| `connect_to_server` | Подключение к Semaphore серверу |
| `disconnect_from_server` | Отключение от сервера |
| `get_connection_state` | Получить состояние подключения |
| `get_projects` | Получить список проектов |
| `get_recent_tasks` | Получить последние задачи |
| `run_task` | Запустить задачу |
| `send_notification` | Отправить системное уведомление |
| `open_external_link` | Открыть ссылку в браузере |

### Безопасность

- **CSP:** Content Security Policy для защиты от XSS
- **Isolation:** Изолированный процесс для WebView
- **Permissions:** Гранулярный контроль доступа к API

---

## 🔧 Конфигурация

### tauri.conf.json

```json
{
  "app": {
    "windows": [{
      "title": "Semaphore UI",
      "width": 1200,
      "height": 800,
      "minWidth": 800,
      "minHeight": 600
    }]
  },
  "bundle": {
    "icon": ["icons/32x32.png", "icons/128x128.png"],
    "category": "DeveloperTool"
  }
}
```

### Переменные окружения

```bash
# URL сервера по умолчанию
SEMAPHORE_DESKTOP_DEFAULT_URL=http://localhost:3000

# Автозапуск
SEMAPHORE_DESKTOP_AUTO_START=true

# Минимизировать при запуске
SEMAPHORE_DESKTOP_START_MINIMIZED=true
```

---

## 📊 Метрики

### Размер бинарника

| Платформа | Размер | Сжатый |
|-----------|--------|--------|
| **Linux DEB** | 45 MB | 18 MB |
| **Windows MSI** | 52 MB | 22 MB |
| **macOS DMG** | 48 MB | 20 MB |

### Потребление памяти

| Состояние | RAM | CPU |
|-----------|-----|-----|
| **Idle** | 45 MB | <1% |
| **Active** | 65 MB | 2-5% |
| **Loading** | 85 MB | 10-15% |

---

## 🐛 Troubleshooting

### Проблема: Не подключается к серверу

**Решение:**
```bash
# Проверьте доступность сервера
curl http://localhost:3000/api/health

# Проверьте CORS настройки сервера
# Добавьте desktop app в разрешённые origins
```

### Проблема: Нет уведомлений

**Решение:**
```bash
# Linux: установите libnotify
sudo apt-get install libnotify-dev

# macOS: проверьте разрешения
System Preferences → Notifications → Semaphore UI

# Windows: проверьте настройки уведомлений
Settings → System → Notifications
```

### Проблема: Белый экран при запуске

**Решение:**
```bash
# Очистите кэш
rm -rf desktop/dist
npm run build

# Пересоберите
npm run tauri build
```

---

## 🔗 Ссылки

- [Tauri Documentation](https://tauri.app/)
- [Vue.js Documentation](https://vuejs.org/)
- [Vite Documentation](https://vitejs.dev/)
- [Semaphore UI](https://github.com/alexandervashurin/semaphore)

---

*Последнее обновление: 10 марта 2026 г.*
