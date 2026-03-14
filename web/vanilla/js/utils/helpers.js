/**
 * Helper Utilities
 * Различные вспомогательные функции
 */

/**
 * Глубокое слияние объектов
 * @param {Object} target - Цель
 * @param {...Object} sources - Источники
 * @returns {Object}
 */
export function merge(target, ...sources) {
  if (!sources.length) return target;
  
  const source = sources.shift();
  
  if (isObject(target) && isObject(source)) {
    for (const key in source) {
      if (isObject(source[key])) {
        if (!target[key]) {
          Object.assign(target, { [key]: {} });
        }
        merge(target[key], source[key]);
      } else {
        Object.assign(target, { [key]: source[key] });
      }
    }
  }
  
  return merge(target, ...sources);
}

/**
 * Проверка на объект
 * @param {any} item - Элемент
 * @returns {boolean}
 */
export function isObject(item) {
  return (item && typeof item === 'object' && !Array.isArray(item));
}

/**
 * Проверка на пустое значение
 * @param {any} value - Значение
 * @returns {boolean}
 */
export function isEmpty(value) {
  if (value == null) return true;
  if (typeof value === 'string') return value.trim() === '';
  if (Array.isArray(value)) return value.length === 0;
  if (typeof value === 'object') return Object.keys(value).length === 0;
  return false;
}

/**
 * Глубокое клонирование
 * @param {any} obj - Объект
 * @returns {any}
 */
export function deepClone(obj) {
  if (obj === null || typeof obj !== 'object') return obj;
  if (obj instanceof Date) return new Date(obj.getTime());
  if (obj instanceof Array) return obj.map(item => deepClone(item));
  if (obj instanceof Object) {
    const clonedObj = new obj.constructor();
    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        clonedObj[key] = deepClone(obj[key]);
      }
    }
    return clonedObj;
  }
}

/**
 * Безопасное получение вложенного свойства
 * @param {Object} obj - Объект
 * @param {string} path - Путь
 * @param {any} defaultValue - Значение по умолчанию
 * @returns {any}
 */
export function get(obj, path, defaultValue = undefined) {
  const keys = path.split('.');
  let result = obj;
  
  for (const key of keys) {
    if (result === null || result === undefined) {
      return defaultValue;
    }
    result = result[key];
  }
  
  return result !== undefined ? result : defaultValue;
}

/**
 * Безопасная установка вложенного свойства
 * @param {Object} obj - Объект
 * @param {string} path - Путь
 * @param {any} value - Значение
 */
export function set(obj, path, value) {
  const keys = path.split('.');
  const lastKey = keys.pop();
  
  let current = obj;
  for (const key of keys) {
    if (!(key in current)) {
      current[key] = {};
    }
    current = current[key];
  }
  
  current[lastKey] = value;
}

/**
 * Форматирование строки с параметрами
 * @param {string} str - Строка
 * @param {Object} params - Параметры
 * @returns {string}
 */
export function format(str, params) {
  return str.replace(/{(\w+)}/g, (match, key) => {
    return params[key] !== undefined ? params[key] : match;
  });
}

/**
 * Транслитерация
 * @param {string} str - Строка
 * @returns {string}
 */
export function transliterate(str) {
  const ru = 'А-Яа-яЁё'.split('');
  const en = 'A-YA-YA-YA-YA-YA-YA'.split('-');
  
  return str.split('').map(char => {
    const index = ru.indexOf(char);
    return index !== -1 ? en[index] : char;
  }).join('');
}

/**
 * Валидация email
 * @param {string} email - Email
 * @returns {boolean}
 */
export function isValidEmail(email) {
  const re = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return re.test(email);
}

/**
 * Валидация URL
 * @param {string} url - URL
 * @returns {boolean}
 */
export function isValidUrl(url) {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

/**
 * Обрезка строки
 * @param {string} str - Строка
 * @param {number} length - Длина
 * @param {string} suffix - Суффикс
 * @returns {string}
 */
export function truncate(str, length = 100, suffix = '...') {
  if (!str) return '';
  if (str.length <= length) return str;
  return str.substring(0, length - suffix.length) + suffix;
}

/**
 * Capitalize первая буква
 * @param {string} str - Строка
 * @returns {string}
 */
export function capitalize(str) {
  if (!str) return '';
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Slugify строка
 * @param {string} str - Строка
 * @returns {string}
 */
export function slugify(str) {
  return str
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, '')
    .replace(/[\s_-]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

/**
 * Парсинг query параметров
 * @param {string} queryString - Query строка
 * @returns {Object}
 */
export function parseQuery(queryString) {
  const query = {};
  const pairs = (queryString[0] === '?' ? queryString.substr(1) : queryString).split('&');
  
  for (const pair of pairs) {
    const [key, value] = pair.split('=');
    query[decodeURIComponent(key || '')] = decodeURIComponent(value || '');
  }
  
  return query;
}

/**
 * Сериализация query параметров
 * @param {Object} params - Параметры
 * @returns {string}
 */
export function stringifyQuery(params) {
  return Object.entries(params)
    .filter(([_, value]) => value !== undefined && value !== null)
    .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`)
    .join('&');
}

/**
 * Группировка массива
 * @param {Array} array - Массив
 * @param {string|Function} key - Ключ
 * @returns {Object}
 */
export function groupBy(array, key) {
  return array.reduce((result, item) => {
    const groupKey = typeof key === 'function' ? key(item) : item[key];
    if (!result[groupKey]) {
      result[groupKey] = [];
    }
    result[groupKey].push(item);
    return result;
  }, {});
}

/**
 * Уникализация массива
 * @param {Array} array - Массив
 * @returns {Array}
 */
export function unique(array) {
  return [...new Set(array)];
}

/**
 * Сортировка массива
 * @param {Array} array - Массив
 * @param {string} key - Ключ
 * @param {string} order - Порядок (asc/desc)
 * @returns {Array}
 */
export function sortBy(array, key, order = 'asc') {
  return [...array].sort((a, b) => {
    const aVal = a[key];
    const bVal = b[key];
    
    if (aVal < bVal) return order === 'asc' ? -1 : 1;
    if (aVal > bVal) return order === 'asc' ? 1 : -1;
    return 0;
  });
}

/**
 * Фильтрация массива по поиску
 * @param {Array} array - Массив
 * @param {string} query - Запрос
 * @param {Array<string>} fields - Поля
 * @returns {Array}
 */
export function filterBySearch(array, query, fields = []) {
  if (!query) return array;
  
  const lowerQuery = query.toLowerCase();
  
  return array.filter(item => {
    if (fields.length === 0) {
      return JSON.stringify(item).toLowerCase().includes(lowerQuery);
    }
    
    return fields.some(field => {
      const value = item[field];
      return String(value).toLowerCase().includes(lowerQuery);
    });
  });
}

/**
 * Chunk массива
 * @param {Array} array - Массив
 * @param {number} size - Размер
 * @returns {Array}
 */
export function chunk(array, size = 1) {
  const result = [];
  for (let i = 0; i < array.length; i += size) {
    result.push(array.slice(i, i + size));
  }
  return result;
}

/**
 * Sleep
 * @param {number} ms - Миллисекунды
 * @returns {Promise}
 */
export function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Retry функция
 * @param {Function} fn - Функция
 * @param {number} retries - Попытки
 * @param {number} delay - Задержка
 * @returns {Promise}
 */
export async function retry(fn, retries = 3, delay = 1000) {
  try {
    return await fn();
  } catch (error) {
    if (retries === 0) throw error;
    await sleep(delay);
    return retry(fn, retries - 1, delay);
  }
}
