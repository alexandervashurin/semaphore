/**
 * Semaphore Vanilla JS - API Client
 * HTTP клиент для работы с Semaphore API
 */

import axios from 'axios';

class SemaphoreAPI {
  constructor() {
    this.client = axios.create({
      baseURL: '/api',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Interceptor для добавления токена
    this.client.interceptors.request.use(config => {
      const token = localStorage.getItem('semaphore_token');
      if (token) {
        config.headers.Authorization = `Bearer ${token}`;
      }
      return config;
    });

    // Interceptor для обработки ошибок
    this.client.interceptors.response.use(
      response => response,
      error => {
        if (error.response?.status === 401) {
          // Unauthorized - очищаем токен и редиректим на login
          localStorage.removeItem('semaphore_token');
          window.location.href = '/auth/login';
        } else if (error.response?.status === 403) {
          // Forbidden
          console.error('Access denied');
        } else if (error.response?.status === 404) {
          // Not found
          console.error('Resource not found');
        } else if (error.response?.status === 500) {
          // Server error
          console.error('Server error');
        }
        return Promise.reject(error);
      }
    );

    // Кэш для GET запросов
    this.cache = new Map();
    this.cacheEnabled = true;
    this.cacheTTL = 5 * 60 * 1000; // 5 минут
  }

  /**
   * GET запрос
   * @param {string} url - URL
   * @param {Object} params - Query параметры
   * @param {Object} options - Опции
   * @returns {Promise}
   */
  async get(url, params = {}, options = {}) {
    const cacheKey = options.cache !== false 
      ? `${url}?${new URLSearchParams(params).toString()}` 
      : null;

    // Проверяем кэш
    if (cacheKey && this.cacheEnabled) {
      const cached = this.cache.get(cacheKey);
      if (cached && Date.now() - cached.timestamp < this.cacheTTL) {
        return cached.data;
      }
    }

    const response = await this.client.get(url, { params });
    const data = response.data;

    // Сохраняем в кэш
    if (cacheKey) {
      this.cache.set(cacheKey, {
        data,
        timestamp: Date.now()
      });
    }

    return data;
  }

  /**
   * POST запрос
   * @param {string} url - URL
   * @param {Object} data - Данные
   * @returns {Promise}
   */
  async post(url, data) {
    const response = await this.client.post(url, data);
    return response.data;
  }

  /**
   * PUT запрос
   * @param {string} url - URL
   * @param {Object} data - Данные
   * @returns {Promise}
   */
  async put(url, data) {
    const response = await this.client.put(url, data);
    return response.data;
  }

  /**
   * PATCH запрос
   * @param {string} url - URL
   * @param {Object} data - Данные
   * @returns {Promise}
   */
  async patch(url, data) {
    const response = await this.client.patch(url, data);
    return response.data;
  }

  /**
   * DELETE запрос
   * @param {string} url - URL
   * @returns {Promise}
   */
  async delete(url) {
    const response = await this.client.delete(url);
    return response.data;
  }

  /**
   * Очистка кэша
   */
  clearCache() {
    this.cache.clear();
  }

  // ==================== Auth API ====================

  /**
   * Логин
   * @param {string} username - Имя пользователя
   * @param {string} password - Пароль
   * @returns {Promise}
   */
  async login(username, password) {
    const response = await this.post('/auth/login', { username, password });
    if (response.token) {
      localStorage.setItem('semaphore_token', response.token);
    }
    return response;
  }

  /**
   * Логаут
   * @returns {Promise}
   */
  async logout() {
    localStorage.removeItem('semaphore_token');
    return this.post('/auth/logout', {});
  }

  /**
   * Получение текущего пользователя
   * @returns {Promise}
   */
  async getCurrentUser() {
    return this.get('/user');
  }

  // ==================== Projects API ====================

  /**
   * Получение списка проектов
   * @returns {Promise}
   */
  async getProjects() {
    return this.get('/projects');
  }

  /**
   * Получение проекта
   * @param {number} projectId - ID проекта
   * @returns {Promise}
   */
  async getProject(projectId) {
    return this.get(`/project/${projectId}`);
  }

  /**
   * Создание проекта
   * @param {Object} data - Данные проекта
   * @returns {Promise}
   */
  async createProject(data) {
    return this.post('/projects', data);
  }

  /**
   * Обновление проекта
   * @param {number} projectId - ID проекта
   * @param {Object} data - Данные
   * @returns {Promise}
   */
  async updateProject(projectId, data) {
    return this.put(`/project/${projectId}`, data);
  }

  /**
   * Удаление проекта
   * @param {number} projectId - ID проекта
   * @returns {Promise}
   */
  async deleteProject(projectId) {
    return this.delete(`/project/${projectId}`);
  }

  // ==================== Templates API ====================

  /**
   * Получение списка шаблонов
   * @param {number} projectId - ID проекта
   * @param {Object} params - Параметры
   * @returns {Promise}
   */
  async getTemplates(projectId, params = {}) {
    return this.get(`/project/${projectId}/templates`, params);
  }

  /**
   * Получение шаблона
   * @param {number} projectId - ID проекта
   * @param {number} templateId - ID шаблона
   * @returns {Promise}
   */
  async getTemplate(projectId, templateId) {
    return this.get(`/project/${projectId}/templates/${templateId}`);
  }

  /**
   * Создание шаблона
   * @param {number} projectId - ID проекта
   * @param {Object} data - Данные
   * @returns {Promise}
   */
  async createTemplate(projectId, data) {
    return this.post(`/project/${projectId}/templates`, data);
  }

  /**
   * Обновление шаблона
   * @param {number} projectId - ID проекта
   * @param {number} templateId - ID шаблона
   * @param {Object} data - Данные
   * @returns {Promise}
   */
  async updateTemplate(projectId, templateId, data) {
    return this.put(`/project/${projectId}/templates/${templateId}`, data);
  }

  /**
   * Удаление шаблона
   * @param {number} projectId - ID проекта
   * @param {number} templateId - ID шаблона
   * @returns {Promise}
   */
  async deleteTemplate(projectId, templateId) {
    return this.delete(`/project/${projectId}/templates/${templateId}`);
  }

  // ==================== Tasks API ====================

  /**
   * Получение списка задач
   * @param {number} projectId - ID проекта
   * @param {Object} params - Параметры
   * @returns {Promise}
   */
  async getTasks(projectId, params = {}) {
    return this.get(`/project/${projectId}/tasks`, params);
  }

  /**
   * Получение задачи
   * @param {number} projectId - ID проекта
   * @param {number} taskId - ID задачи
   * @returns {Promise}
   */
  async getTask(projectId, taskId) {
    return this.get(`/project/${projectId}/tasks/${taskId}`);
  }

  /**
   * Запуск задачи
   * @param {number} projectId - ID проекта
   * @param {number} templateId - ID шаблона
   * @param {Object} data - Данные
   * @returns {Promise}
   */
  async runTask(projectId, templateId, data = {}) {
    return this.post(`/project/${projectId}/templates/${templateId}/tasks`, data);
  }

  /**
   * Остановка задачи
   * @param {number} projectId - ID проекта
   * @param {number} taskId - ID задачи
   * @returns {Promise}
   */
  async stopTask(projectId, taskId) {
    return this.post(`/project/${projectId}/tasks/${taskId}/stop`);
  }

  /**
   * Получение лога задачи
   * @param {number} projectId - ID проекта
   * @param {number} taskId - ID задачи
   * @returns {Promise}
   */
  async getTaskLog(projectId, taskId) {
    return this.get(`/project/${projectId}/tasks/${taskId}/output`);
  }

  // ==================== Inventory API ====================

  async getInventories(projectId, params = {}) {
    return this.get(`/project/${projectId}/inventory`, params);
  }

  async getInventory(projectId, inventoryId) {
    return this.get(`/project/${projectId}/inventory/${inventoryId}`);
  }

  async createInventory(projectId, data) {
    return this.post(`/project/${projectId}/inventory`, data);
  }

  async updateInventory(projectId, inventoryId, data) {
    return this.put(`/project/${projectId}/inventory/${inventoryId}`, data);
  }

  async deleteInventory(projectId, inventoryId) {
    return this.delete(`/project/${projectId}/inventory/${inventoryId}`);
  }

  // ==================== Repositories API ====================

  async getRepositories(projectId, params = {}) {
    return this.get(`/project/${projectId}/repositories`, params);
  }

  async getRepository(projectId, repoId) {
    return this.get(`/project/${projectId}/repositories/${repoId}`);
  }

  async createRepository(projectId, data) {
    return this.post(`/project/${projectId}/repositories`, data);
  }

  async updateRepository(projectId, repoId, data) {
    return this.put(`/project/${projectId}/repositories/${repoId}`, data);
  }

  async deleteRepository(projectId, repoId) {
    return this.delete(`/project/${projectId}/repositories/${repoId}`);
  }

  // ==================== Environment API ====================

  async getEnvironments(projectId, params = {}) {
    return this.get(`/project/${projectId}/environment`, params);
  }

  async getEnvironment(projectId, envId) {
    return this.get(`/project/${projectId}/environment/${envId}`);
  }

  async createEnvironment(projectId, data) {
    return this.post(`/project/${projectId}/environment`, data);
  }

  async updateEnvironment(projectId, envId, data) {
    return this.put(`/project/${projectId}/environment/${envId}`, data);
  }

  async deleteEnvironment(projectId, envId) {
    return this.delete(`/project/${projectId}/environment/${envId}`);
  }

  // ==================== Keys API ====================

  async getKeys(projectId, params = {}) {
    return this.get(`/project/${projectId}/keys`, params);
  }

  async getKey(projectId, keyId) {
    return this.get(`/project/${projectId}/keys/${keyId}`);
  }

  async createKey(projectId, data) {
    return this.post(`/project/${projectId}/keys`, data);
  }

  async updateKey(projectId, keyId, data) {
    return this.put(`/project/${projectId}/keys/${keyId}`, data);
  }

  async deleteKey(projectId, keyId) {
    return this.delete(`/project/${projectId}/keys/${keyId}`);
  }

  // ==================== Team API ====================

  async getTeam(projectId, params = {}) {
    return this.get(`/project/${projectId}/team`, params);
  }

  async addTeamMember(projectId, data) {
    return this.post(`/project/${projectId}/team`, data);
  }

  async updateTeamMember(projectId, userId, data) {
    return this.put(`/project/${projectId}/team/${userId}`, data);
  }

  async removeTeamMember(projectId, userId) {
    return this.delete(`/project/${projectId}/team/${userId}`);
  }

  // ==================== Audit Log API ====================

  async getAuditLogs(projectId, params = {}) {
    return this.get(`/project/${projectId}/audit-log`, params);
  }

  async clearAuditLog(projectId) {
    return this.delete(`/project/${projectId}/audit-log/clear`);
  }

  // ==================== Analytics API ====================

  async getProjectAnalytics(projectId, params = {}) {
    return this.get(`/analytics/project/${projectId}`, params);
  }

  async getTaskStats(projectId, params = {}) {
    return this.get(`/analytics/project/${projectId}/tasks`, params);
  }

  async getUserActivity(projectId, params = {}) {
    return this.get(`/analytics/project/${projectId}/users`, params);
  }

  async getPerformanceMetrics(projectId, params = {}) {
    return this.get(`/analytics/project/${projectId}/performance`, params);
  }

  async getChartData(projectId, params = {}) {
    return this.get(`/analytics/project/${projectId}/chart`, params);
  }

  async getSlowTasks(projectId, params = {}) {
    return this.get(`/analytics/project/${projectId}/slow-tasks`, params);
  }

  // ==================== Playbook API ====================

  /**
   * Получить список playbook
   * @param {number} projectId - ID проекта
   * @returns {Promise}
   */
  async getPlaybooks(projectId) {
    return this.get(`/project/${projectId}/playbooks`);
  }

  /**
   * Получить playbook по ID
   * @param {number} projectId - ID проекта
   * @param {number} playbookId - ID playbook
   * @returns {Promise}
   */
  async getPlaybook(projectId, playbookId) {
    return this.get(`/project/${projectId}/playbooks/${playbookId}`);
  }

  /**
   * Создать playbook
   * @param {number} projectId - ID проекта
   * @param {Object} data - Данные playbook
   * @returns {Promise}
   */
  async createPlaybook(projectId, data) {
    return this.post(`/project/${projectId}/playbooks`, data);
  }

  /**
   * Обновить playbook
   * @param {number} projectId - ID проекта
   * @param {number} playbookId - ID playbook
   * @param {Object} data - Данные playbook
   * @returns {Promise}
   */
  async updatePlaybook(projectId, playbookId, data) {
    return this.put(`/project/${projectId}/playbooks/${playbookId}`, data);
  }

  /**
   * Удалить playbook
   * @param {number} projectId - ID проекта
   * @param {number} playbookId - ID playbook
   * @returns {Promise}
   */
  async deletePlaybook(projectId, playbookId) {
    return this.delete(`/project/${projectId}/playbooks/${playbookId}`);
  }

  /**
   * Синхронизировать playbook из Git
   * @param {number} projectId - ID проекта
   * @param {number} playbookId - ID playbook
   * @returns {Promise}
   */
  async syncPlaybook(projectId, playbookId) {
    return this.post(`/project/${projectId}/playbooks/${playbookId}/sync`);
  }

  /**
   * Предпросмотр playbook из Git
   * @param {number} projectId - ID проекта
   * @param {number} playbookId - ID playbook
   * @returns {Promise}
   */
  async previewPlaybook(projectId, playbookId) {
    return this.get(`/project/${projectId}/playbooks/${playbookId}/preview`);
  }

  /**
   * Запустить playbook
   * @param {number} projectId - ID проекта
   * @param {number} playbookId - ID playbook
   * @param {Object} data - Параметры запуска
   * @returns {Promise}
   */
  async runPlaybook(projectId, playbookId, data = {}) {
    return this.post(`/project/${projectId}/playbooks/${playbookId}/run`, data);
  }
}

// Экспортируем singleton
export default new SemaphoreAPI();
