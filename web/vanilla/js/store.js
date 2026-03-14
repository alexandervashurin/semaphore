/**
 * Semaphore Vanilla JS - Store
 * Простой state management с реактивностью через Proxy
 */

export class Store {
  constructor(state = {}, options = {}) {
    this.state = state;
    this.listeners = [];
    this.middlewares = options.middlewares || [];
    
    // Создаём реактивный state
    this.state = this.createProxy(state);
  }

  /**
   * Создание реактивного прокси
   * @param {Object} state - Исходное состояние
   * @returns {Proxy}
   */
  createProxy(state) {
    const self = this;
    
    return new Proxy(state, {
      get(target, key) {
        const value = target[key];
        // Рекурсивно создаём прокси для вложенных объектов
        if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
          return self.createProxy(value);
        }
        return value;
      },
      
      set(target, key, value) {
        const oldValue = target[key];
        target[key] = value;
        
        // Уведомляем подписчиков
        self.emit('change', { key, value, oldValue, type: 'set' });
        
        return true;
      },
      
      deleteProperty(target, key) {
        const oldValue = target[key];
        delete target[key];
        
        self.emit('change', { key, value: undefined, oldValue, type: 'delete' });
        
        return true;
      }
    });
  }

  /**
   * Подписка на изменения
   * @param {Function} fn - Функция обратного вызова
   * @returns {Function} - Функция отписки
   */
  subscribe(fn) {
    this.listeners.push(fn);
    
    // Возвращаем функцию отписки
    return () => {
      const index = this.listeners.indexOf(fn);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Уведомление подписчиков
   * @param {string} event - Тип события
   * @param {Object} payload - Данные события
   */
  emit(event, payload) {
    this.listeners.forEach(fn => {
      try {
        fn(event, payload, this.state);
      } catch (error) {
        console.error('Store listener error:', error);
      }
    });
  }

  /**
   * Получение значения
   * @param {string} key - Ключ
   * @returns {any}
   */
  get(key) {
    const keys = key.split('.');
    let value = this.state;
    
    for (const k of keys) {
      if (value === undefined || value === null) {
        return undefined;
      }
      value = value[k];
    }
    
    return value;
  }

  /**
   * Установка значения
   * @param {string} key - Ключ
   * @param {any} value - Значение
   */
  set(key, value) {
    const keys = key.split('.');
    const lastKey = keys.pop();
    
    let obj = this.state;
    for (const k of keys) {
      if (!(k in obj)) {
        obj[k] = {};
      }
      obj = obj[k];
    }
    
    obj[lastKey] = value;
  }

  /**
   * Мутация состояния
   * @param {string} type - Тип мутации
   * @param {Object} payload - Данные
   */
  commit(type, payload) {
    // Проходим через middleware
    const context = {
      state: this.state,
      commit: this.commit.bind(this),
      dispatch: this.dispatch.bind(this)
    };
    
    let newPayload = payload;
    for (const middleware of this.middlewares) {
      newPayload = middleware(context, type, newPayload);
    }
    
    // Применяем мутацию
    if (this.mutations && this.mutations[type]) {
      this.mutations[type](this.state, newPayload);
    }
  }

  /**
   * Dispatch action
   * @param {string} type - Тип действия
   * @param {Object} payload - Данные
   * @returns {Promise}
   */
  async dispatch(type, payload) {
    if (this.actions && this.actions[type]) {
      const context = {
        state: this.state,
        commit: this.commit.bind(this),
        dispatch: this.dispatch.bind(this),
        getters: this.getters || {}
      };
      
      return await this.actions[type](context, payload);
    }
  }

  /**
   * Сохранение состояния в localStorage
   * @param {string} key - Ключ
   */
  save(key = 'semaphore-store') {
    try {
      localStorage.setItem(key, JSON.stringify(this.state));
    } catch (error) {
      console.error('Store save error:', error);
    }
  }

  /**
   * Загрузка состояния из localStorage
   * @param {string} key - Ключ
   * @param {Object} defaultState - Состояние по умолчанию
   * @returns {Object}
   */
  load(key = 'semaphore-store', defaultState = {}) {
    try {
      const saved = localStorage.getItem(key);
      if (saved) {
        const parsed = JSON.parse(saved);
        Object.assign(this.state, parsed);
      }
    } catch (error) {
      console.error('Store load error:', error);
      return defaultState;
    }
    return this.state;
  }

  /**
   * Очистка состояния
   */
  clear() {
    Object.keys(this.state).forEach(key => {
      delete this.state[key];
    });
  }
}

export default Store;
