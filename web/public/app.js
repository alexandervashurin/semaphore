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
                if (!url.includes('/auth/login')) {
                    this.logout();
                }
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

    createProject(data) {
        return this.post('/projects', data);
    },

    updateProject(id, data) {
        return this.put('/projects/' + id, data);
    },

    deleteProject(id) {
        return this.delete('/projects/' + id);
    },

    // Users
    getUsers() {
        return this.get('/users');
    },

    getUser(id) {
        return this.get('/users/' + id);
    },

    createUser(data) {
        return this.post('/users', data);
    },

    updateUser(id, data) {
        return this.put('/users/' + id, data);
    },

    deleteUser(id) {
        return this.delete('/users/' + id);
    },

    // Inventory
    getInventories(projectId) {
        return this.get('/project/' + projectId + '/inventory');
    },

    getInventory(projectId, id) {
        return this.get('/project/' + projectId + '/inventory/' + id);
    },

    createInventory(projectId, data) {
        return this.post('/project/' + projectId + '/inventory', data);
    },

    updateInventory(projectId, id, data) {
        return this.put('/project/' + projectId + '/inventory/' + id, data);
    },

    deleteInventory(projectId, id) {
        return this.delete('/project/' + projectId + '/inventory/' + id);
    },

    // Environment
    getEnvironments(projectId) {
        return this.get('/project/' + projectId + '/environment');
    },

    getEnvironment(projectId, id) {
        return this.get('/project/' + projectId + '/environment/' + id);
    },

    createEnvironment(projectId, data) {
        return this.post('/project/' + projectId + '/environment', data);
    },

    updateEnvironment(projectId, id, data) {
        return this.put('/project/' + projectId + '/environment/' + id, data);
    },

    deleteEnvironment(projectId, id) {
        return this.delete('/project/' + projectId + '/environment/' + id);
    },

    // Repositories
    getRepositories(projectId) {
        return this.get('/project/' + projectId + '/repositories');
    },

    getRepository(projectId, id) {
        return this.get('/project/' + projectId + '/repositories/' + id);
    },

    createRepository(projectId, data) {
        return this.post('/project/' + projectId + '/repositories', data);
    },

    updateRepository(projectId, id, data) {
        return this.put('/project/' + projectId + '/repositories/' + id, data);
    },

    deleteRepository(projectId, id) {
        return this.delete('/project/' + projectId + '/repositories/' + id);
    },

    // Keys
    getKeys(projectId) {
        return this.get('/project/' + projectId + '/keys');
    },

    getKey(projectId, id) {
        return this.get('/project/' + projectId + '/keys/' + id);
    },

    createKey(projectId, data) {
        return this.post('/project/' + projectId + '/keys', data);
    },

    updateKey(projectId, id, data) {
        return this.put('/project/' + projectId + '/keys/' + id, data);
    },

    deleteKey(projectId, id) {
        return this.delete('/project/' + projectId + '/keys/' + id);
    },

    // Templates
    getTemplates(projectId) {
        return this.get('/project/' + projectId + '/templates');
    },

    getTemplate(projectId, id) {
        return this.get('/project/' + projectId + '/templates/' + id);
    },

    createTemplate(projectId, data) {
        return this.post('/project/' + projectId + '/templates', data);
    },

    updateTemplate(projectId, id, data) {
        return this.put('/project/' + projectId + '/templates/' + id, data);
    },

    deleteTemplate(projectId, id) {
        return this.delete('/project/' + projectId + '/templates/' + id);
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
        const toast = document.createElement('div');
        toast.className = `toast alert-${type}`;
        toast.textContent = message;
        document.body.appendChild(toast);
        setTimeout(() => {
            toast.style.transition = 'opacity 0.3s ease';
            toast.style.opacity = '0';
            setTimeout(() => toast.remove(), 300);
        }, 3500);
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

// ==================== Sidebar ====================

const SIDEBAR_ITEMS = [
    { href: 'index.html',        icon: '◈',  label: 'Dashboard',    noId: true },
    { href: 'project.html',      icon: '⬡',  label: 'Обзор' },
    { href: 'templates.html',    icon: '▦',  label: 'Шаблоны' },
    { href: 'inventory.html',    icon: '≡',  label: 'Инвентарь' },
    { href: 'environments.html', icon: '⊕',  label: 'Окружения' },
    { href: 'repositories.html', icon: '⌥',  label: 'Репозитории' },
    { href: 'keys.html',         icon: '⚿',  label: 'Ключи' },
    { href: 'schedules.html',    icon: '◷',  label: 'Расписания' },
    { href: 'analytics.html',    icon: '◑',  label: 'Аналитика' },
    { href: 'webhooks.html',     icon: '⇌',  label: 'Webhooks' },
    { href: 'playbooks.html',    icon: '▶',  label: 'Playbooks' },
    { href: 'team.html',         icon: '👥', label: 'Команда' },
];

function renderSidebar() {
    const sidebar = document.querySelector('.sidebar');
    if (!sidebar) return;

    const urlParams = new URLSearchParams(window.location.search);
    const projectId = urlParams.get('id');
    const currentPage = location.pathname.split('/').pop() || 'index.html';

    const navItems = SIDEBAR_ITEMS.map(item => {
        const href = (item.noId || !projectId)
            ? item.href
            : item.href + '?id=' + projectId;
        const isActive = currentPage === item.href ? 'class="active"' : '';
        return `<li><a href="${href}" ${isActive}><span class="nav-icon">${item.icon}</span>${item.label}</a></li>`;
    }).join('');

    sidebar.innerHTML = `
        <div class="sidebar-logo">
            <div class="sidebar-logo-dot"><img src="/logo.svg" alt=""></div>
            <h2>Semaphore</h2>
        </div>
        <div class="sidebar-section">Навигация</div>
        <ul class="sidebar-nav">${navItems}</ul>
        <div class="sidebar-footer">
            <button class="btn btn-logout" onclick="api.logout()" style="width:100%;justify-content:center;">
                Выйти
            </button>
        </div>
    `;

    // Mobile: inject hamburger button into .main-header and wire up overlay
    const header = document.querySelector('.main-header');
    if (header && !header.querySelector('.hamburger')) {
        const btn = document.createElement('button');
        btn.className = 'hamburger';
        btn.setAttribute('aria-label', 'Меню');
        btn.innerHTML = '<span></span><span></span><span></span>';
        header.prepend(btn);

        const overlay = document.createElement('div');
        overlay.className = 'sidebar-overlay';
        document.body.appendChild(overlay);

        const openSidebar = () => { sidebar.classList.add('open'); overlay.classList.add('open'); };
        const closeSidebar = () => { sidebar.classList.remove('open'); overlay.classList.remove('open'); };

        btn.addEventListener('click', openSidebar);
        overlay.addEventListener('click', closeSidebar);
    }
}

document.addEventListener('DOMContentLoaded', renderSidebar);

// ==================== Export ====================

window.api = api;
window.ui = ui;
window.checkAuth = checkAuth;
window.escapeHtml = escapeHtml;
window.formatDate = formatDate;
