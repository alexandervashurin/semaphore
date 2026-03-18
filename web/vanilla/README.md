# Semaphore Vanilla JS Frontend

> Фронтенд на чистом JavaScript без необходимости запускать `npm build` на проде

---

## 📖 Обзор

Эта директория содержит альтернативную версию фронтенда Velum на **чистом JS+CSS+HTML** без использования Vue.js.

### Преимущества

- ✅ **Нет npm на проде** — собираем локально, копируем готовые файлы
- ✅ **Простая сборка** — Gulp для минификации CSS/JS
- ✅ **Сохранён дизайн** — Vuetify-стили через чистый CSS
- ✅ **Лёгкая миграция** — постепенный переход со старого фронтенда
- ✅ **ES6 модули** — современный JavaScript без фреймворков

---

## 🚀 Быстрый старт

### Установка зависимостей

```bash
cd web
npm install
```

### Запуск разработки

```bash
# Сборка и watch режим
npm run vanilla:dev

# Или через gulp напрямую
gulp
```

### Сборка для продакшена

```bash
# Минификация и копирование в public/
npm run vanilla:build

# Или через gulp
gulp build
```

### Локальный сервер для тестирования

```bash
npm run vanilla:serve
```

---

## 📁 Структура проекта

```
vanilla/
├── css/
│   ├── main.scss              # Главный SCSS файл
│   ├── components/
│   │   ├── buttons.scss       # Кнопки
│   │   ├── inputs.scss        # Поля ввода
│   │   ├── dialogs.scss       # Диалоги
│   │   └── tables.scss        # Таблицы
│   └── utils/
│       ├── variables.scss     # CSS переменные
│       └── mixins.scss        # SCSS миксины
│
├── js/
│   ├── app.js                 # Точка входа
│   ├── router.js              # Роутер
│   ├── store.js               # State management
│   ├── api.js                 # API клиент
│   ├── components/
│   │   ├── dialogs.js         # Диалоги
│   │   └── tables.js          # Таблицы
│   ├── pages/                 # Страницы (будущие)
│   └── utils/
│       ├── dom.js             # DOM утилиты
│       └── helpers.js         # Helper функции
│
├── html/
│   ├── index.html             # Главный layout
│   ├── auth.html              # Страница входа
│   └── project/               # Страницы проекта
│
└── assets/                    # Статические файлы
```

---

## 🧩 Компоненты

### UI Компоненты

| Компонент | Описание | Статус |
|-----------|----------|--------|
| Buttons | Кнопки (contained, text, outlined, icon) | ✅ Готово |
| Inputs | Text fields, select, checkbox, switch | ✅ Готово |
| Dialogs | Modal, alert, confirm, prompt | ✅ Готово |
| Tables | DataTable с сортировкой и пагинацией | ✅ Готово |
| Cards | Карточки | ✅ Готово |
| App Bar | Верхняя панель | ✅ Готово |
| Navigation Drawer | Боковое меню | ✅ Готово |

### JS Модули

| Модуль | Описание | Статус |
|--------|----------|--------|
| Router | History API роутер | ✅ Готово |
| Store | Reactive state management | ✅ Готово |
| API | Axios клиент с интерцепторами | ✅ Готово |
| DOM Utils | Вспомогательные функции | ✅ Готово |
| Helpers | Утилиты (format, validate, etc) | ✅ Готово |

---

## 📄 Страницы

| Страница | Маршрут | Статус |
|----------|---------|--------|
| Login | `/auth/login` | ✅ Готово |
| Dashboard | `/` | ✅ Готово |
| Projects | `/projects` | ✅ Готово |
| History | `/project/:id/history` | ✅ Готово |
| Templates | `/project/:id/templates` | ✅ Готово |
| Inventory | `/project/:id/inventory` | ✅ Готово |
| Repositories | `/project/:id/repositories` | ✅ Готово |
| Environment | `/project/:id/environment` | ✅ Готово |
| Keys | `/project/:id/keys` | ✅ Готово |
| Team | `/project/:id/team` | ✅ Готово |
| Schedule | `/project/:id/schedule` | 📅 Запланировано |
| Integrations | `/project/:id/integrations` | 📅 Запланировано |
| Audit Log | `/project/:id/audit-log` | 📅 Запланировано |
| Analytics | `/project/:id/analytics` | 📅 Запланировано |
| Settings | `/project/:id/settings` | 📅 Запланировано |

---

## 🔧 Использование

### Роутер

```javascript
import router from './router.js';

// Переход на страницу
router.push('/project/1/templates');

// Замена текущей страницы
router.replace('/auth/login');

// Назад
router.back();
```

### Store

```javascript
import store from './store.js';

// Подписка на изменения
store.subscribe((event, payload, state) => {
  console.log('State changed:', event, payload);
});

// Получение значения
const user = store.get('user');

// Установка значения
store.set('user', { id: 1, name: 'Admin' });

// Сохранение в localStorage
store.save();

// Загрузка из localStorage
store.load();
```

### API

```javascript
import api from './api.js';

// GET запрос
const projects = await api.getProjects();

// POST запрос
const project = await api.createProject({ name: 'My Project' });

// PUT запрос
await api.updateProject(1, { name: 'Updated' });

// DELETE запрос
await api.deleteProject(1);

// Логин
await api.login('admin', 'password');

// Логаут
await api.logout();
```

### Dialogs

```javascript
import { alert, confirm, prompt } from './components/dialogs.js';

// Alert
alert({
  title: 'Успех',
  content: 'Операция выполнена'
});

// Confirm
const result = await confirm({
  title: 'Подтверждение',
  content: 'Вы уверены?'
});

if (result) {
  // Пользователь подтвердил
}

// Prompt
const value = await prompt({
  title: 'Ввод',
  label: 'Введите значение',
  defaultValue: ''
});
```

### DataTable

```javascript
import DataTable from './components/tables.js';

const table = new DataTable('#container', {
  headers: [
    { text: 'ID', value: 'id' },
    { text: 'Name', value: 'name' },
    { text: 'Status', value: 'status' }
  ],
  data: [
    { id: 1, name: 'Item 1', status: 'active' },
    { id: 2, name: 'Item 2', status: 'inactive' }
  ],
  sortable: true,
  pagination: true,
  itemsPerPage: 10,
  onRowClick: (item) => {
    console.log('Row clicked:', item);
  }
});

// Обновление данных
table.setData(newData);

// Получение выбранных
const selected = table.getSelected();
```

---

## 🎨 Стили

### CSS Переменные

```css
:root {
  --v-primary-base: #3f51b5;
  --v-secondary-base: #ff4081;
  --v-success-base: #4caf50;
  --v-error-base: #f44336;
  --v-warning-base: #ff9800;
  --v-info-base: #2196f3;
}
```

### Классы

```html
<!-- Кнопки -->
<button class="v-btn v-btn--contained v-btn--primary">Primary</button>
<button class="v-btn v-btn--text">Text</button>
<button class="v-btn v-btn--outlined">Outlined</button>
<button class="v-btn v-btn--icon"><i class="mdi mdi-heart"></i></button>

<!-- Поля ввода -->
<div class="v-text-field">
  <input type="text" id="name" required placeholder=" ">
  <label for="name">Name</label>
</div>

<!-- Карточки -->
<div class="v-card">
  <div class="v-card__title">Title</div>
  <div class="v-card__text">Content</div>
  <div class="v-card__actions">
    <button class="v-btn v-btn--text">Action</button>
  </div>
</div>

<!-- Таблицы -->
<table class="v-data-table">
  <thead>...</thead>
  <tbody>...</tbody>
</table>
```

---

## 🔄 Миграция с Vue

### Поэтапный план

1. **Этап 1** (готово) - Базовая инфраструктура
2. **Этап 2** (готово) - UI компоненты
3. **Этап 3** (готово) - Страница аутентификации
4. **Этап 4** (готово) - Роутинг и основные страницы
5. **Этап 5** (в процессе) - CRUD операции
6. **Этап 6** (запланировано) - Дополнительные функции
7. **Этап 7** (запланировано) - Тестирование

### Сравнение подходов

| Vue | Vanilla JS |
|-----|------------|
| `v-if` | `element.style.display` |
| `v-for` | `array.forEach()` + `createElement` |
| `v-model` | `input.value` + event listener |
| `@click` | `element.addEventListener('click')` |
| `computed` | Функция-геттер |
| `watch` | `store.subscribe()` |
| `router.push` | `router.push()` (аналогично) |
| `axios` | `api.get/post/put/delete` |

---

## 🧪 Тестирование

### Ручное тестирование

1. Запустите сервер:
   ```bash
   npm run vanilla:serve
   ```

2. Откройте `http://localhost:8080/html/auth.html`

3. Проверьте:
   - Вход с невалидными данными
   - Вход с валидными данными
   - Навигацию по меню
   - CRUD операции

### Автоматические тесты (будущее)

```javascript
// tests/vanilla/auth.test.js
describe('Auth Page', () => {
  it('should show error on invalid credentials', async () => {
    // ...
  });
  
  it('should redirect on successful login', async () => {
    // ...
  });
});
```

---

## 📝 Changelog

### 0.1.0 (13 марта 2026)

- ✅ Базовая структура проекта
- ✅ CSS компоненты (buttons, inputs, dialogs, tables)
- ✅ JS модули (router, store, api, utils)
- ✅ Страница аутентификации
- ✅ Главный layout с навигацией
- ✅ Gulp сборка

---

## 📚 Ресурсы

- [MDN Web Docs](https://developer.mozilla.org/)
- [Vuetify CSS Classes](https://v2.vuetifyjs.com/)
- [Gulp Documentation](https://gulpjs.com/)

---

*Последнее обновление: 13 марта 2026 г.*
