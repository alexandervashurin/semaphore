/**
 * Semaphore UI - Core JavaScript
 * Чистый JS без зависимостей
 */

// ==================== Конфигурация ====================

const API_BASE = '/api';
const STORAGE_KEY = 'semaphore_token';
const USER_KEY = 'semaphore_user';

// ==================== Утилиты ====================

function $(selector) {
    return document.querySelector(selector);
}

function $$(selector) {
    return document.querySelectorAll(selector);
}

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function formatDate(dateStr) {
    if (!dateStr) return '—';
    const date = new Date(dateStr);
    return date.toLocaleDateString('ru-RU', {
        day: 'numeric',
        month: 'long',
        year: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    });
}

function formatRelativeTime(dateStr) {
    if (!dateStr) return '—';
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now - date;
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);

    if (minutes < 1) return 'только что';
    if (minutes < 60) return minutes + ' мин. назад';
    if (hours < 24) return hours + ' ч. назад';
    if (days < 7) return days + ' дн. назад';
    return formatDate(dateStr);
}

// ==================== API ====================

const api = {
    async request(url, options = {}) {
        const token = localStorage.getItem(STORAGE_KEY);
        const headers = {
            'Content-Type': 'application/json',
            ...options.headers
        };

        if (token) {
            headers['Authorization'] = 'Bearer ' + token;
        }

        try {
            const response = await fetch(API_BASE + url, {
                ...options,
                headers
            });

            if (response.status === 401) {
                this.logout();
                throw new Error('Не авторизован');
            }

            const data = await response.json();

            if (!response.ok) {
                throw new Error(data.error || data.message || 'Ошибка');
            }

            return data;
        } catch (error) {
            console.error('API Error:', error);
            throw error;
        }
    },

    async get(url) {
        return this.request(url);
    },

    async post(url, data) {
        return this.request(url, {
            method: 'POST',
            body: JSON.stringify(data)
        });
    },

    async put(url, data) {
        return this.request(url, {
            method: 'PUT',
            body: JSON.stringify(data)
        });
    },

    async delete(url) {
        return this.request(url, {
            method: 'DELETE'
        });
    },

    // Auth
    async login(username, password) {
        const data = await this.post('/auth/login', { username, password });
        localStorage.setItem(STORAGE_KEY, data.token);
        return data;
    },

    logout() {
        localStorage.removeItem(STORAGE_KEY);
        localStorage.removeItem(USER_KEY);
        window.location.href = '/login.html';
    },

    // Projects
    getProjects() {
        return this.get('/projects');
    },

    getProject(id) {
        return this.get('/projects/' + id);
    },

    // Playbooks
    getPlaybooks(projectId) {
        return this.get('/project/' + projectId + '/playbooks');
    },

    getPlaybook(projectId, id) {
        return this.get('/project/' + projectId + '/playbooks/' + id);
    },

    createPlaybook(projectId, data) {
        return this.post('/project/' + projectId + '/playbooks', data);
    },

    updatePlaybook(projectId, id, data) {
        return this.put('/project/' + projectId + '/playbooks/' + id, data);
    },

    deletePlaybook(projectId, id) {
        return this.delete('/project/' + projectId + '/playbooks/' + id);
    },

    syncPlaybook(projectId, id) {
        return this.post('/project/' + projectId + '/playbooks/' + id + '/sync');
    },

    // Templates
    getTemplates(projectId) {
        return this.get('/project/' + projectId + '/templates');
    },

    // Inventory
    getInventories(projectId) {
        return this.get('/project/' + projectId + '/inventory');
    },

    // Environment
    getEnvironments(projectId) {
        return this.get('/project/' + projectId + '/environment');
    },

    // Repositories
    getRepositories(projectId) {
        return this.get('/project/' + projectId + '/repositories');
    },

    // Keys
    getKeys(projectId) {
        return this.get('/project/' + projectId + '/keys');
    }
};

// ==================== UI Компоненты ====================

const ui = {
    showLoading(container) {
        container.innerHTML = `
            <div class="loading">
                <div class="loading-spinner"></div>
                <p>Загрузка...</p>
            </div>
        `;
    },

    showEmpty(container, icon, title, text) {
        container.innerHTML = `
            <div class="empty-state">
                <div class="empty-state-icon">${icon}</div>
                <h3>${title}</h3>
                <p>${text || ''}</p>
            </div>
        `;
    },

    showError(container, message) {
        container.innerHTML = `
            <div class="alert alert-danger">
                ${escapeHtml(message)}
            </div>
        `;
    },

    showAlert(message, type = 'info') {
        const alert = document.createElement('div');
        alert.className = `alert alert-${type}`;
        alert.textContent = message;
        alert.style.position = 'fixed';
        alert.style.top = '20px';
        alert.style.right = '20px';
        alert.style.zIndex = '9999';
        alert.style.minWidth = '300px';
        document.body.appendChild(alert);
        setTimeout(() => alert.remove(), 5000);
    },

    confirm(title, text) {
        return new Promise((resolve) => {
            const modal = document.createElement('div');
            modal.className = 'modal-overlay';
            modal.innerHTML = `
                <div class="modal">
                    <div class="modal-header">
                        <h2>${escapeHtml(title)}</h2>
                    </div>
                    <p>${escapeHtml(text)}</p>
                    <div class="modal-footer">
                        <button class="btn" id="cancel-btn">Отмена</button>
                        <button class="btn btn-danger" id="confirm-btn">Удалить</button>
                    </div>
                </div>
            `;
            document.body.appendChild(modal);

            $('#cancel-btn').onclick = () => {
                modal.remove();
                resolve(false);
            };

            $('#confirm-btn').onclick = () => {
                modal.remove();
                resolve(true);
            };
        });
    },

    showModal(title, content) {
        return new Promise((resolve) => {
            const modal = document.createElement('div');
            modal.className = 'modal-overlay';
            modal.innerHTML = `
                <div class="modal">
                    <div class="modal-header">
                        <h2>${escapeHtml(title)}</h2>
                    </div>
                    <div id="modal-content"></div>
                    <div class="modal-footer">
                        <button class="btn" id="close-modal-btn">Закрыть</button>
                    </div>
                </div>
            `;
            document.body.appendChild(modal);
            $('#modal-content').innerHTML = content;

            $('#close-modal-btn').onclick = () => {
                modal.remove();
                resolve();
            };
        });
    }
};

// ==================== Auth Check ====================

function checkAuth() {
    const token = localStorage.getItem(STORAGE_KEY);
    if (!token && !window.location.pathname.includes('login.html')) {
        window.location.href = '/login.html';
        return null;
    }
    return token;
}

// ==================== Export ====================

window.api = api;
window.ui = ui;
window.checkAuth = checkAuth;
window.escapeHtml = escapeHtml;
window.formatDate = formatDate;
