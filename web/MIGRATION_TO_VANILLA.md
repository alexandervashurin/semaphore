# 🚀 Migration Plan: Vue.js → Vanilla JS+CSS+HTML

> **Поэтапная миграция фронтенда с Vue 2 на чистый JavaScript**

---

## 📊 Обзор миграции

### Текущее состояние
- **Vue 2.6.14** с 113 `.vue` компонентами
- **Vuetify 2.6.10** UI-фреймворк
- **Vue Router** для маршрутизации
- **Webpack** (vue-cli) для сборки
- **Требуется npm build** на этапе разработки

### Целевое состояние
- **Vanilla ES6+ JavaScript** без фреймворков
- **Vuetify CSS** (только стили) для сохранения дизайна
- **Custom Router** на базе History API
- **Gulp** для простой сборки (минификация, конкатенация)
- **Нет npm на проде** — только статические файлы

---

## 🏗️ Архитектура нового фронтенда

```
web/
├── src/
│   ├── css/
│   │   ├── main.scss          # Главный SCSS файл
│   │   ├── components/        # SCSS компоненты
│   │   │   ├── buttons.scss
│   │   │   ├── inputs.scss
│   │   │   ├── dialogs.scss
│   │   │   └── tables.scss
│   │   └── utils/
│   │       ├── variables.scss
│   │       └── mixins.scss
│   │
│   ├── js/
│   │   ├── app.js             # Точка входа
│   │   ├── router.js          # Кастомный роутер
│   │   ├── store.js           # State management
│   │   ├── api.js             # API клиент
│   │   ├── i18n.js            # Интернационализация
│   │   │
│   │   ├── components/        # JS компоненты
│   │   │   ├── buttons.js
│   │   │   ├── dialogs.js
│   │   │   ├── tables.js
│   │   │   └── forms.js
│   │   │
│   │   ├── pages/             # Страницы
│   │   │   ├── auth.js
│   │   │   ├── dashboard.js
│   │   │   ├── templates.js
│   │   │   └── ...
│   │   │
│   │   └── utils/
│   │       ├── helpers.js
│   │       └── dom.js
│   │
│   └── html/
│       ├── index.html
│       ├── auth.html
│       └── project/
│           ├── dashboard.html
│           ├── templates.html
│           └── ...
│
├── public/                    # Собранные файлы (продакшен)
│   ├── css/
│   │   └── main.min.css
│   ├── js/
│   │   └── app.min.js
│   ├── html/
│   └── assets/
│
├── gulpfile.js
└── package.json
```

---

## 📅 Этапы миграции

### Этап 1: Подготовка инфраструктуры (3 дня)

**Задачи:**
- [ ] Настроить Gulp для сборки SCSS → CSS, JS минификации
- [ ] Создать базовую структуру директорий
- [ ] Реализовать кастомный Router на History API
- [ ] Реализовать базовый Store (state management)
- [ ] Создать API клиент с axios

**Файлы:**
```javascript
// src/js/router.js
class Router {
  constructor(routes = []) {
    this.routes = routes;
    this.currentRoute = null;
    this.init();
  }

  init() {
    window.addEventListener('popstate', (e) => {
      this.loadRoute(window.location.pathname);
    });
    this.loadRoute(window.location.pathname);
  }

  async loadRoute(path) {
    const route = this.routes.find(r => this.matchRoute(path, r.path));
    if (route) {
      this.currentRoute = route;
      const params = this.extractParams(path, route.path);
      await route.handler(params);
    }
  }

  matchRoute(path, routePath) {
    const regex = new RegExp('^' + routePath.replace(/:\w+/g, '(\\w+)') + '$');
    return regex.test(path);
  }

  extractParams(path, routePath) {
    const regex = new RegExp('^' + routePath.replace(/:\w+/g, '(\\w+)') + '$');
    const matches = path.match(regex);
    const paramNames = routePath.match(/:\w+/g);
    if (!paramNames) return {};
    
    return paramNames.reduce((acc, name, index) => {
      acc[name.slice(1)] = matches[index + 1];
      return acc;
    }, {});
  }

  push(path) {
    history.pushState({}, '', path);
    this.loadRoute(path);
  }
}

export default Router;
```

```javascript
// src/js/store.js
class Store {
  constructor(state = {}) {
    this.state = new Proxy(state, {
      set: (target, key, value) => {
        target[key] = value;
        this.emit('change', { key, value });
        return true;
      }
    });
    this.listeners = [];
  }

  subscribe(fn) {
    this.listeners.push(fn);
  }

  emit(event, payload) {
    this.listeners.forEach(fn => fn(event, payload));
  }
}

export default Store;
```

```javascript
// src/js/api.js
import axios from 'axios';

class API {
  constructor() {
    this.client = axios.create({
      baseURL: '/api',
      headers: { 'Content-Type': 'application/json' }
    });

    this.client.interceptors.request.use(config => {
      const token = localStorage.getItem('semaphore_token');
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    });

    this.client.interceptors.response.use(
      response => response,
      error => {
        if (error.response?.status === 401) {
          localStorage.removeItem('semaphore_token');
          window.location.href = '/auth/login';
        }
        return Promise.reject(error);
      }
    );
  }

  async get(url, params = {}) {
    const response = await this.client.get(url, { params });
    return response.data;
  }

  async post(url, data) {
    const response = await this.client.post(url, data);
    return response.data;
  }

  async put(url, data) {
    const response = await this.client.put(url, data);
    return response.data;
  }

  async delete(url) {
    const response = await this.client.delete(url);
    return response.data;
  }
}

export default new API();
```

---

### Этап 2: Базовые UI компоненты (4 дня)

**Задачи:**
- [ ] Кнопки (button, icon-button, loading-button)
- [ ] Поля ввода (text, password, select, textarea)
- [ ] Диалоги (modal, confirm, alert)
- [ ] Таблицы (data-table, sortable, paginatable)
- [ ] Формы (validation, error handling)

**Пример компонента:**
```javascript
// src/js/components/dialogs.js
export class Dialog {
  constructor(options = {}) {
    this.title = options.title || '';
    this.content = options.content || '';
    this.onConfirm = options.onConfirm || (() => {});
    this.onCancel = options.onCancel || (() => {});
  }

  show() {
    const dialog = document.createElement('div');
    dialog.className = 'v-dialog__scrim';
    dialog.innerHTML = `
      <div class="v-dialog v-dialog--active" role="dialog">
        <div class="v-card">
          <div class="v-card__title">
            <span class="v-card__title-text">${this.title}</span>
          </div>
          <div class="v-card__text">${this.content}</div>
          <div class="v-card__actions">
            <button class="v-btn v-btn--text" data-action="cancel">Отмена</button>
            <button class="v-btn v-btn--contained v-btn--primary" data-action="confirm">OK</button>
          </div>
        </div>
      </div>
    `;

    dialog.querySelector('[data-action="confirm"]').onclick = () => {
      this.onConfirm();
      this.close();
    };

    dialog.querySelector('[data-action="cancel"]').onclick = () => {
      this.onCancel();
      this.close();
    };

    document.body.appendChild(dialog);
    this.dialog = dialog;
  }

  close() {
    if (this.dialog) {
      this.dialog.remove();
    }
  }
}
```

---

### Этап 3: Страница аутентификации (2 дня)

**Задачи:**
- [ ] Создать HTML шаблон login
- [ ] Реализовать форму входа
- [ ] Обработка JWT токена
- [ ] Роут на главную страницу

**Файлы:**
```html
<!-- src/html/auth.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Login - Semaphore</title>
  <link rel="stylesheet" href="/css/main.min.css">
</head>
<body>
  <div id="app">
    <div class="auth-container">
      <div class="v-card auth-card">
        <div class="v-card__title">
          <h1 class="text-h5">Login to Semaphore</h1>
        </div>
        <div class="v-card__text">
          <form id="login-form">
            <div class="v-text-field">
              <label>Username</label>
              <input type="text" name="username" required>
            </div>
            <div class="v-text-field">
              <label>Password</label>
              <input type="password" name="password" required>
            </div>
            <button type="submit" class="v-btn v-btn--contained v-btn--primary v-btn--block">
              Login
            </button>
          </form>
        </div>
      </div>
    </div>
  </div>
  <script type="module" src="/js/pages/auth.js"></script>
</body>
</html>
```

```javascript
// src/js/pages/auth.js
import api from '../api.js';

document.getElementById('login-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const form = e.target;
  const username = form.username.value;
  const password = form.password.value;

  try {
    const response = await api.post('/auth/login', { username, password });
    localStorage.setItem('semaphore_token', response.token);
    window.location.href = '/';
  } catch (error) {
    alert('Invalid credentials');
  }
});
```

---

### Этап 4: Главная страница проекта (5 дней)

**Задачи:**
- [ ] Dashboard с меню проекта
- [ ] Навигация по разделам (History, Templates, Inventory, etc)
- [ ] Базовый layout с sidebar/header

---

### Этап 5: CRUD страницы (10 дней)

**Приоритетные страницы:**
1. Templates (список, создание, редактирование)
2. Inventory
3. Repositories
4. Environment
5. Keys
6. Team

---

### Этап 6: Дополнительные функции (5 дней)

- [ ] Task log viewer с ANSI цветами
- [ ] Charts и статистика
- [ ] WebSocket обновления
- [ ] Интеграции

---

### Этап 7: Тестирование и полировка (3 дня)

- [ ] Кроссбраузерность
- [ ] Производительность
- [ ] Доступность (a11y)
- [ ] Документация

---

## 🔧 Gulp конфигурация

```javascript
// gulpfile.js
const gulp = require('gulp');
const sass = require('gulp-sass')(require('sass'));
const terser = require('gulp-terser');
const concat = require('gulp-concat');
const cleanCss = require('gulp-clean-css');
const rename = require('gulp-rename');

// Пути
const paths = {
  scss: 'src/css/**/*.scss',
  js: 'src/js/**/*.js',
  html: 'src/html/**/*.html',
  dest: 'public/'
};

// SCSS → CSS
function styles() {
  return gulp.src('src/css/main.scss')
    .pipe(sass().on('error', sass.logError))
    .pipe(cleanCss())
    .pipe(rename('main.min.css'))
    .pipe(gulp.dest(paths.dest + 'css'));
}

// JS минификация
function scripts() {
  return gulp.src('src/js/app.js')
    .pipe(terser())
    .pipe(rename('app.min.js'))
    .pipe(gulp.dest(paths.dest + 'js'));
}

// Копирование HTML
function html() {
  return gulp.src('src/html/**/*.html')
    .pipe(gulp.dest(paths.dest + 'html'));
}

// Копирование ассетов
function assets() {
  return gulp.src('src/assets/**/*')
    .pipe(gulp.dest(paths.dest + 'assets'));
}

// Watch
function watch() {
  gulp.watch(paths.scss, styles);
  gulp.watch(paths.js, scripts);
  gulp.watch(paths.html, html);
}

// Задачи
gulp.task('styles', styles);
gulp.task('scripts', scripts);
gulp.task('html', html);
gulp.task('assets', assets);
gulp.task('watch', watch);

// Build
gulp.task('build', gulp.parallel(styles, scripts, html, assets));

// Default
gulp.task('default', gulp.series('build', 'watch'));
```

---

## 📦 package.json зависимости

```json
{
  "name": "semaphore-vanilla",
  "version": "1.0.0",
  "scripts": {
    "dev": "gulp",
    "build": "gulp build",
    "serve": "http-server public -p 8080"
  },
  "dependencies": {
    "axios": "^1.13.5",
    "dayjs": "^1.11.13",
    "ansi_up": "^6.0.6"
  },
  "devDependencies": {
    "gulp": "^5.0.0",
    "gulp-sass": "^5.0.0",
    "sass": "~1.32.12",
    "gulp-terser": "^2.0.0",
    "gulp-concat": "^2.6.1",
    "gulp-clean-css": "^4.3.0",
    "gulp-rename": "^2.0.0",
    "http-server": "^14.0.0"
  }
}
```

---

## 🎯 Критерии успеха

- [ ] **Нет npm на проде** — только статические файлы
- [ ] **Сохранён Vuetify-дизайн** — визуально идентично
- [ ] **Все CRUD операции** работают
- [ ] **Роутинг** между страницами
- [ ] **Аутентификация** через JWT
- [ ] **WebSocket** для real-time обновлений
- [ ] **i18n** поддержка

---

## 📊 Timeline

| Этап | Длительность | Статус |
|------|--------------|--------|
| 1. Инфраструктура | 3 дня | 📅 Запланировано |
| 2. UI компоненты | 4 дня | 📅 Запланировано |
| 3. Аутентификация | 2 дня | 📅 Запланировано |
| 4. Главная страница | 5 дней | 📅 Запланировано |
| 5. CRUD страницы | 10 дней | 📅 Запланировано |
| 6. Доп. функции | 5 дней | 📅 Запланировано |
| 7. Тестирование | 3 дня | 📅 Запланировано |

**Итого:** ~32 рабочих дня (~6-7 недель)

---

*Последнее обновление: 13 марта 2026 г.*
