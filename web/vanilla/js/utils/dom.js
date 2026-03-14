/**
 * DOM Utilities
 * Вспомогательные функции для работы с DOM
 */

/**
 * Создание элемента
 * @param {string} tag - Тег элемента
 * @param {Object} options - Опции
 * @returns {HTMLElement}
 */
export function createElement(tag, options = {}) {
  const el = document.createElement(tag);
  
  if (options.className) {
    el.className = options.className;
  }
  
  if (options.id) {
    el.id = options.id;
  }
  
  if (options.attributes) {
    Object.entries(options.attributes).forEach(([key, value]) => {
      el.setAttribute(key, value);
    });
  }
  
  if (options.style) {
    Object.assign(el.style, options.style);
  }
  
  if (options.text) {
    el.textContent = options.text;
  }
  
  if (options.html) {
    el.innerHTML = options.html;
  }
  
  if (options.children) {
    options.children.forEach(child => {
      if (typeof child === 'string') {
        el.appendChild(document.createTextNode(child));
      } else if (child instanceof HTMLElement) {
        el.appendChild(child);
      }
    });
  }
  
  if (options.events) {
    Object.entries(options.events).forEach(([event, handler]) => {
      el.addEventListener(event, handler);
    });
  }
  
  if (options.dataset) {
    Object.entries(options.dataset).forEach(([key, value]) => {
      el.dataset[key] = value;
    });
  }
  
  return el;
}

/**
 * Поиск элемента
 * @param {string} selector - Селектор
 * @param {HTMLElement} context - Контекст
 * @returns {HTMLElement|null}
 */
export function $(selector, context = document) {
  return context.querySelector(selector);
}

/**
 * Поиск всех элементов
 * @param {string} selector - Селектор
 * @param {HTMLElement} context - Контекст
 * @returns {NodeList}
 */
export function $$(selector, context = document) {
  return context.querySelectorAll(selector);
}

/**
 * Удаление элемента
 * @param {HTMLElement} el - Элемент
 */
export function remove(el) {
  if (el && el.parentNode) {
    el.remove();
  }
}

/**
 * Показ элемента
 * @param {HTMLElement} el - Элемент
 */
export function show(el) {
  if (el) {
    el.style.display = '';
  }
}

/**
 * Скрытие элемента
 * @param {HTMLElement} el - Элемент
 */
export function hide(el) {
  if (el) {
    el.style.display = 'none';
  }
}

/**
 * Переключение класса
 * @param {HTMLElement} el - Элемент
 * @param {string} className - Класс
 * @param {boolean} force - Принудительно
 */
export function toggleClass(el, className, force = undefined) {
  if (el) {
    el.classList.toggle(className, force);
  }
}

/**
 * Добавление класса
 * @param {HTMLElement} el - Элемент
 * @param {string} className - Класс
 */
export function addClass(el, className) {
  if (el) {
    el.classList.add(className);
  }
}

/**
 * Удаление класса
 * @param {HTMLElement} el - Элемент
 * @param {string} className - Класс
 */
export function removeClass(el, className) {
  if (el) {
    el.classList.remove(className);
  }
}

/**
 * Проверка наличия класса
 * @param {HTMLElement} el - Элемент
 * @param {string} className - Класс
 * @returns {boolean}
 */
export function hasClass(el, className) {
  return el?.classList.contains(className) || false;
}

/**
 * Установка атрибутов
 * @param {HTMLElement} el - Элемент
 * @param {Object} attributes - Атрибуты
 */
export function setAttributes(el, attributes) {
  if (!el) return;
  Object.entries(attributes).forEach(([key, value]) => {
    el.setAttribute(key, value);
  });
}

/**
 * Получение атрибута
 * @param {HTMLElement} el - Элемент
 * @param {string} name - Имя атрибута
 * @returns {string|null}
 */
export function getAttribute(el, name) {
  return el?.getAttribute(name);
}

/**
 * Удаление атрибута
 * @param {HTMLElement} el - Элемент
 * @param {string} name - Имя атрибута
 */
export function removeAttribute(el, name) {
  if (el) {
    el.removeAttribute(name);
  }
}

/**
 * Очистка элемента
 * @param {HTMLElement} el - Элемент
 */
export function clear(el) {
  if (el) {
    el.innerHTML = '';
  }
}

/**
 * Вставка HTML
 * @param {HTMLElement} el - Элемент
 * @param {string} html - HTML
 * @param {string} position - Позиция
 */
export function insertHTML(el, html, position = 'beforeend') {
  if (el) {
    el.insertAdjacentHTML(position, html);
  }
}

/**
 * Делегирование событий
 * @param {HTMLElement} el - Элемент
 * @param {string} eventType - Тип события
 * @param {string} selector - Селектор
 * @param {Function} handler - Обработчик
 */
export function delegate(el, eventType, selector, handler) {
  el.addEventListener(eventType, (e) => {
    const target = e.target.closest(selector);
    if (target && el.contains(target)) {
      handler.call(target, e);
    }
  });
}

/**
 * Создание debounce функции
 * @param {Function} func - Функция
 * @param {number} wait - Задержка
 * @returns {Function}
 */
export function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

/**
 * Создание throttle функции
 * @param {Function} func - Функция
 * @param {number} limit - Лимит
 * @returns {Function}
 */
export function throttle(func, limit) {
  let inThrottle;
  return function(...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}

/**
 * Форматирование даты
 * @param {Date|string|number} date - Дата
 * @param {string} format - Формат
 * @returns {string}
 */
export function formatDate(date, format = 'LL') {
  const d = new Date(date);
  if (isNaN(d.getTime())) return '—';
  
  const options = {
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  };
  
  return d.toLocaleDateString('ru-RU', options);
}

/**
 * Форматирование времени
 * @param {Date|string|number} date - Дата
 * @returns {string}
 */
export function formatTime(date) {
  const d = new Date(date);
  if (isNaN(d.getTime())) return '—';
  
  return d.toLocaleTimeString('ru-RU', { 
    hour: '2-digit', 
    minute: '2-digit' 
  });
}

/**
 * Форматирование даты и времени
 * @param {Date|string|number} date - Дата
 * @returns {string}
 */
export function formatDateTime(date) {
  const d = new Date(date);
  if (isNaN(d.getTime())) return '—';
  
  return d.toLocaleString('ru-RU', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  });
}

/**
 * Относительное время
 * @param {Date|string|number} date - Дата
 * @returns {string}
 */
export function timeAgo(date) {
  const d = new Date(date);
  const now = new Date();
  const diff = now - d;
  
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  const weeks = Math.floor(days / 7);
  const months = Math.floor(days / 30);
  const years = Math.floor(days / 365);
  
  if (years > 0) return `${years} г. назад`;
  if (months > 0) return `${months} мес. назад`;
  if (weeks > 0) return `${weeks} нед. назад`;
  if (days > 0) return `${days} дн. назад`;
  if (hours > 0) return `${hours} ч. назад`;
  if (minutes > 0) return `${minutes} мин. назад`;
  return 'только что';
}

/**
 * Форматирование размера
 * @param {number} bytes - Байты
 * @returns {string}
 */
export function formatSize(bytes) {
  if (bytes === 0) return '0 B';
  
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Генерация уникального ID
 * @returns {string}
 */
export function generateId() {
  return Math.random().toString(36).substring(2, 15) + 
         Math.random().toString(36).substring(2, 15);
}

/**
 * Копирование в буфер
 * @param {string} text - Текст
 * @returns {Promise<boolean>}
 */
export async function copyToClipboard(text) {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (err) {
    console.error('Copy failed:', err);
    return false;
  }
}

/**
 * Скачивание файла
 * @param {Blob} blob - Blob
 * @param {string} filename - Имя файла
 */
export function downloadFile(blob, filename) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
