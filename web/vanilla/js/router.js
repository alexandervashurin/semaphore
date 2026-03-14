/**
 * Semaphore Vanilla JS - Router
 * Простой роутер на базе History API
 */

export class Router {
  constructor(routes = []) {
    this.routes = routes;
    this.currentRoute = null;
    this.basePath = '';
    this.init();
  }

  /**
   * Инициализация роутера
   */
  init() {
    window.addEventListener('popstate', (e) => {
      this.loadRoute(window.location.pathname);
    });
    
    // Загружаем текущий маршрут при старте
    this.loadRoute(window.location.pathname);
  }

  /**
   * Загрузка маршрута
   * @param {string} path - Путь для загрузки
   */
  async loadRoute(path) {
    const route = this.findRoute(path);
    
    if (route) {
      this.currentRoute = route;
      const params = this.extractParams(path, route.path);
      
      // Вызываем handler маршрута
      if (route.handler) {
        await route.handler(params);
      }
      
      // Dispatch event
      window.dispatchEvent(new CustomEvent('route-changed', { 
        detail: { route, params } 
      }));
    } else {
      // 404
      this.loadRoute('/404');
    }
  }

  /**
   * Поиск маршрута по пути
   * @param {string} path - Путь для поиска
   * @returns {Object|null}
   */
  findRoute(path) {
    for (const route of this.routes) {
      if (this.matchRoute(path, route.path)) {
        return route;
      }
    }
    return null;
  }

  /**
   * Проверка соответствия пути маршруту
   * @param {string} path - Путь
   * @param {string} routePath - Путь маршрута
   * @returns {boolean}
   */
  matchRoute(path, routePath) {
    const regex = new RegExp('^' + routePath.replace(/:\w+/g, '(\\w+)') + '$');
    return regex.test(path);
  }

  /**
   * Извлечение параметров из пути
   * @param {string} path - Путь
   * @param {string} routePath - Путь маршрута
   * @returns {Object}
   */
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

  /**
   * Переход на страницу
   * @param {string} path - Путь
   * @param {Object} state - Состояние
   */
  push(path, state = {}) {
    history.pushState(state, '', path);
    this.loadRoute(path);
  }

  /**
   * Замена текущей страницы
   * @param {string} path - Путь
   * @param {Object} state - Состояние
   */
  replace(path, state = {}) {
    history.replaceState(state, '', path);
    this.loadRoute(path);
  }

  /**
   * Назад
   */
  back() {
    history.back();
  }

  /**
   * Вперёд
   */
  forward() {
    history.forward();
  }

  /**
   * Получить текущий параметр
   * @param {string} name - Имя параметра
   * @returns {string|undefined}
   */
  getParam(name) {
    return this.currentRoute?.params?.[name];
  }

  /**
   * Добавить маршрут
   * @param {string} path - Путь
   * @param {Function} handler - Обработчик
   */
  addRoute(path, handler) {
    this.routes.push({ path, handler });
  }
}

export default Router;
