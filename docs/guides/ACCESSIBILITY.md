# Accessibility Guide (WCAG 2.1 AA)

## Обзор

Velum соответствует рекомендациям **WCAG 2.1 AA** (Web Content Accessibility Guidelines) для обеспечения доступности интерфейса для всех пользователей, включая людей с ограниченными возможностями.

## Реализованные возможности

### 1. Mobile Responsive Design

Интерфейс адаптируется для работы на мобильных устройствах:

#### Breakpoints
- **Desktop**: > 1024px
- **Tablet**: 768px - 1024px
- **Mobile**: < 768px
- **Small Mobile**: < 480px

#### Функции
- Гамбургер-меню для мобильных устройств
- Скрываемый sidebar с overlay
- Адаптивные таблицы с горизонтальной прокруткой
- Оптимизированные модальные окна
- Сенсорные цели мин. 44x44px

#### Использование
```html
<!-- Кнопка мобильного меню добавляется автоматически -->
<button class="mobile-menu-toggle" aria-label="Открыть меню">
    <i class="fa-solid fa-bars"></i>
</button>

<!-- Overlay для затемнения фона -->
<div class="mobile-sidebar-overlay"></div>
```

### 2. WCAG 2.1 AA Compliance

#### Навигация с клавиатуры
- Все интерактивные элементы доступны через Tab
- Видимые индикаторы фокуса (3px outline)
- Поддержка focus-visible
- Skip link для пропуска навигации

#### ARIA-атрибуты
- Роли для основных областей (navigation, main)
- ARIA-label для кнопок и секций
- aria-expanded для раскрывающихся элементов
- aria-live для динамических уведомлений

#### Контрастность
- Минимальный контраст текста: 4.5:1
- Увеличенный контраст для крупного текста: 3:1
- Поддержка режима высокой контрастности ОС

#### Поддержка скринридеров
- Скрытый текст через `.sr-only`
- ARIA announcer для динамических сообщений
- Семантическая разметка

#### Примеры использования
```html
<!-- Skip link -->
<a href="#main-content" class="skip-link">Перейти к содержимому</a>

<!-- ARIA landmarks -->
<nav class="sidebar" role="navigation" aria-label="Главное меню">
<main id="main-content" role="main">

<!-- Screen reader text -->
<span class="sr-only">Только для скринридеров</span>

<!-- Announce dynamic content -->
<script>
window.announce('Данные загружены', 'polite');
</script>
```

### 3. Internationalization (i18n)

Поддержка нескольких языков: **Русский**, **English**, **中文**

#### Переключение языка
```html
<!-- Language switcher -->
<div id="lang-switcher"></div>

<script>
// Инициализация переключателя
initLanguageSwitcher('#lang-switcher');

// Программное переключение
setLanguage('en');

// Получение перевода
const text = t('nav.dashboard');
</script>
```

#### Data-атрибуты для автоперевода
```html
<h1 data-i18n="nav.dashboard">Панель управления</h1>
<input type="text" data-i18n-placeholder="search" placeholder="Поиск">
<button data-i18n-title="a11y.close-modal" title="Закрыть модальное окно">
```

#### Добавление нового языка
```javascript
translations.ja = {
    'loading': '読み込み中...',
    'error': 'エラー',
    // ... другие переводы
};
```

### 4. Keyboard Shortcuts

Горячие клавиши для ускорения работы:

#### Глобальные
| Клавиша | Действие |
|---------|----------|
| `/` | Фокус на поиск |
| `?` | Показать справку по горячим клавишам |
| `Esc` | Закрыть модальные окна |
| `n` | Новое действие (context-dependent) |
| `r` | Обновить страницу |

#### Навигация (префикс `g`)
| Клавиши | Действие |
|---------|----------|
| `g d` | Перейти на Dashboard |
| `g p` | Перейти в Projects |
| `g t` | Перейти в Templates |
| `g k` | Перейти в Kubernetes |

#### Kubernetes (префикс `k`)
| Клавиши | Действие |
|---------|----------|
| `k p` | Pods |
| `k d` | Deployments |
| `k e` | Events |

#### Добавление горячих клавиш
```javascript
keyboardShortcuts['x'] = () => {
    // Ваше действие
};

keyboardShortcuts['g x'] = () => {
    // Действие по префиксу
    navigateTo('/your-page');
};
```

## CSS Classes Reference

### Accessibility
| Класс | Описание |
|-------|----------|
| `.skip-link` | Ссылка для пропуска навигации |
| `.sr-only` | Скрытый текст для скринридеров |
| `.wcag-touch-target` | Мин. размер 44x44px |
| `.wcag-contrast-large` | Увеличенный контраст |
| `.alert-wcag-*` | Доступные alert-боксы |
| `.form-group-wcag` | Доступные формы |
| `.kbd-shortcut` | Бейджи горячих клавиш |

### Mobile
| Класс | Описание |
|-------|----------|
| `.mobile-menu-toggle` | Кнопка гамбургер-меню |
| `.mobile-sidebar-overlay` | Overlay для sidebar |
| `.sidebar.mobile-open` | Открытый sidebar на мобильном |

## Best Practices

### 1. Формы
```html
<div class="form-group-wcag">
    <label for="email">
        Email
        <span class="required-indicator">*</span>
    </label>
    <input type="email" id="email" required aria-required="true">
    <div class="error-message" role="alert">
        <i class="fa-solid fa-circle-exclamation"></i>
        Введите корректный email
    </div>
</div>
```

### 2. Уведомления
```html
<div class="alert-wcag alert-wcag-success" role="alert">
    <i class="fa-solid fa-circle-check" aria-hidden="true"></i>
    <span>Данные успешно сохранены</span>
</div>

<div class="alert-wcag alert-wcag-error" role="alert">
    <i class="fa-solid fa-circle-exclamation" aria-hidden="true"></i>
    <span>Ошибка при сохранении</span>
</div>
```

### 3. Кнопки действий
```html
<button class="btn btn-primary wcag-touch-target" aria-label="Создать новый проект">
    <i class="fa-solid fa-plus" aria-hidden="true"></i>
    <span>Новый проект</span>
</button>
```

### 4. Таблицы
```html
<div class="data-table-wrapper" role="region" aria-label="Список подов" tabindex="0">
    <table class="data-table">
        <caption class="sr-only">Kubernetes Pods</caption>
        <thead>
            <tr>
                <th scope="col">Name</th>
                <th scope="col">Namespace</th>
                <th scope="col">Status</th>
                <th scope="col">Actions</th>
            </tr>
        </thead>
        <tbody>
            <!-- rows -->
        </tbody>
    </table>
</div>
```

## Тестирование доступности

### Инструменты
1. **Lighthouse** (Chrome DevTools)
2. **axe DevTools** (browser extension)
3. **WAVE** (webaim.org/resources/wave/)
4. **NVDA/JAWS** (скринридеры для Windows)
5. **VoiceOver** (скринридер для macOS)

### Чек-лист
- [ ] Навигация с клавиатуры работает
- [ ] Видимый фокус на всех интерактивных элементах
- [ ] Skip link присутствует и работает
- [ ] ARIA-атрибуты корректны
- [ ] Контрастность соответствует WCAG AA
- [ ] Формы имеют label
- [ ] Ошибки форм объявляются скринридерам
- [ ] Мобильное меню работает
- [ ] Сенсорные цели ≥ 44x44px
- [ ] Переключение языков работает

## Поддержка специальных возможностей ОС

### Reduced Motion
```css
@media (prefers-reduced-motion: reduce) {
    /* Анимации отключаются автоматически */
}
```

### High Contrast Mode
```css
@media (prefers-contrast: high) {
    /* Контраст увеличивается автоматически */
}
```

## Ресурсы

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [MDN Accessibility](https://developer.mozilla.org/en-US/docs/Web/Accessibility)
