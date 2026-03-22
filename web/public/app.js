/**
 * Velum - Core JavaScript
 * Чистый JS без зависимостей
 */

// ==================== Конфигурация ====================

const API_BASE = '/api';
const STORAGE_KEY = 'velum_token';
const USER_KEY = 'velum_user';
const THEME_KEY = 'velum_theme';
const LANG_KEY = 'velum_lang';
const PROJECT_KEY = 'velum_last_project';

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
    async request(urlOrMethod, urlOrOptions = {}, payload) {
        // Support both signatures:
        //   new: request(url, options?)
        //   old: request(method, url, payload?)
        const HTTP_METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD'];
        let url, options;
        if (typeof urlOrMethod === 'string' && HTTP_METHODS.includes(urlOrMethod.toUpperCase())) {
            url = urlOrOptions;
            options = { method: urlOrMethod.toUpperCase() };
            if (payload !== undefined && payload !== null) {
                options.body = JSON.stringify(payload);
            }
        } else {
            url = urlOrMethod;
            options = urlOrOptions || {};
        }

        // Strip double /api prefix (pages that build URLs with API_BASE constant)
        if (typeof url === 'string' && url.startsWith(API_BASE + '/')) {
            url = url.slice(API_BASE.length);
        }

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
        // Save user info for sidebar display
        const userInfo = { username: data.username || data.name || username };
        localStorage.setItem(USER_KEY, JSON.stringify(userInfo));
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

    leaveProject(projectId) {
        return this.delete('/project/' + projectId + '/me');
    },

    getProjectStats(projectId) {
        return this.get('/project/' + projectId + '/stats');
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

    getRepositoryBranches(projectId, id) {
        return this.get('/project/' + projectId + '/repositories/' + id + '/branches');
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
    },

    // Tasks
    getTasks(projectId) {
        return this.get('/project/' + projectId + '/tasks');
    },

    getTask(projectId, id) {
        return this.get('/project/' + projectId + '/tasks/' + id);
    },

    getTaskOutput(projectId, id) {
        return this.get('/project/' + projectId + '/tasks/' + id + '/output');
    },

    runTask(projectId, templateId) {
        return this.post('/project/' + projectId + '/tasks', { template_id: templateId });
    },

    stopTask(projectId, id) {
        return this.post('/project/' + projectId + '/tasks/' + id + '/stop', {});
    },

    confirmTask(projectId, id) {
        return this.post('/project/' + projectId + '/tasks/' + id + '/confirm', {});
    },

    rejectTask(projectId, id) {
        return this.post('/project/' + projectId + '/tasks/' + id + '/reject', {});
    },

    getTaskRawOutput(projectId, id) {
        return this.get('/project/' + projectId + '/tasks/' + id + '/output');
    },

    getTaskStages(projectId, id) {
        return this.get('/project/' + projectId + '/tasks/' + id + '/stages');
    },

    // Playbook runs
    getPlaybookRuns(projectId, playbookId) {
        return this.get('/project/' + projectId + '/playbooks/' + playbookId + '/runs');
    },

    getPlaybookStats(projectId, playbookId) {
        return this.get('/project/' + projectId + '/playbooks/' + playbookId + '/stats');
    },

    runPlaybook(projectId, playbookId, data) {
        return this.post('/project/' + projectId + '/playbooks/' + playbookId + '/run', data || {});
    },

    // API Tokens
    getTokens() {
        return this.get('/user/tokens');
    },

    createToken(data) {
        return this.post('/user/tokens', data);
    },

    deleteToken(id) {
        return this.delete('/user/tokens/' + id);
    },

    // TOTP
    startTotp() {
        return this.post('/auth/totp/start', {});
    },

    confirmTotp(code) {
        return this.post('/auth/totp/confirm', { code });
    },

    disableTotp(recoveryCode) {
        return this.post('/auth/totp/disable', { recovery_code: recoveryCode });
    },

    // Runners
    getRunners() {
        return this.get('/runners');
    },

    createRunner(data) {
        return this.post('/runners', data);
    },

    updateRunner(id, data) {
        return this.put('/runners/' + id, data);
    },

    deleteRunner(id) {
        return this.delete('/runners/' + id);
    },

    toggleRunnerActive(id) {
        return this.post('/runners/' + id + '/active', {});
    },

    // Apps
    getApps() {
        return this.get('/apps');
    },

    updateApp(id, data) {
        return this.put('/apps/' + id, data);
    },

    toggleAppActive(id) {
        return this.post('/apps/' + id + '/active', {});
    },

    // Schedules
    getSchedules(projectId) {
        return this.get('/project/' + projectId + '/schedules');
    },

    createSchedule(projectId, data) {
        return this.post('/project/' + projectId + '/schedules', data);
    },

    updateSchedule(projectId, id, data) {
        return this.put('/project/' + projectId + '/schedules/' + id, data);
    },

    deleteSchedule(projectId, id) {
        return this.delete('/project/' + projectId + '/schedules/' + id);
    },

    // Webhooks / Integrations
    getIntegrations(projectId) {
        return this.get('/project/' + projectId + '/integrations');
    },

    createIntegration(projectId, data) {
        return this.post('/project/' + projectId + '/integrations', data);
    },

    updateIntegration(projectId, id, data) {
        return this.put('/project/' + projectId + '/integrations/' + id, data);
    },

    deleteIntegration(projectId, id) {
        return this.delete('/project/' + projectId + '/integrations/' + id);
    },

    getIntegrationAliases(projectId) {
        return this.get('/project/' + projectId + '/integrations/aliases');
    },

    createIntegrationAlias(projectId, data) {
        return this.post('/project/' + projectId + '/integrations/aliases', data);
    },

    deleteIntegrationAlias(projectId, id) {
        return this.delete('/project/' + projectId + '/integrations/aliases/' + id);
    },

    // Events / Activity
    getProjectEvents(projectId) {
        return this.get('/project/' + projectId + '/events');
    },

    getAllEvents() {
        return this.get('/events');
    },

    // Roles
    getRoles(projectId) {
        return this.get('/project/' + projectId + '/roles');
    },

    createRole(projectId, data) {
        return this.post('/project/' + projectId + '/roles', data);
    },

    updateRole(projectId, id, data) {
        return this.put('/project/' + projectId + '/roles/' + id, data);
    },

    deleteRole(projectId, id) {
        return this.delete('/project/' + projectId + '/roles/' + id);
    },

    // Secret Storages
    getSecretStorages(projectId) {
        return this.get('/project/' + projectId + '/secret_storages');
    },

    createSecretStorage(projectId, data) {
        return this.post('/project/' + projectId + '/secret_storages', data);
    },

    updateSecretStorage(projectId, id, data) {
        return this.put('/project/' + projectId + '/secret_storages/' + id, data);
    },

    deleteSecretStorage(projectId, id) {
        return this.delete('/project/' + projectId + '/secret_storages/' + id);
    },

    syncSecretStorage(projectId, id) {
        return this.post('/project/' + projectId + '/secret_storages/' + id + '/sync', {});
    },

    // Notifications
    testNotification(projectId) {
        return this.post('/project/' + projectId + '/notifications/test', {});
    },

    // Cache
    clearCache() {
        return this.delete('/cache');
    },

    clearProjectCache(projectId) {
        return this.delete('/project/' + projectId + '/cache');
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
    { href: 'index.html',        icon: 'fa-solid fa-house',          label: 'Dashboard',    noId: true },
    { href: 'global_tasks.html', icon: 'fa-solid fa-list-check',     label: 'Все задачи',   noId: true },
    { href: 'project.html',      icon: 'fa-solid fa-folder-open',    label: 'Обзор' },
    { href: 'templates.html',    icon: 'fa-solid fa-file-code',      label: 'Шаблоны' },
    { href: 'history.html',      icon: 'fa-solid fa-clock-rotate-left', label: 'История задач' },
    { href: 'activity.html',     icon: 'fa-solid fa-chart-line',     label: 'Активность' },
    { href: 'inventory.html',    icon: 'fa-solid fa-server',         label: 'Инвентарь' },
    { href: 'environments.html', icon: 'fa-solid fa-leaf',           label: 'Окружения' },
    { href: 'repositories.html', icon: 'fa-brands fa-git-alt',       label: 'Репозитории' },
    { href: 'keys.html',         icon: 'fa-solid fa-key',            label: 'Ключи' },
    { href: 'schedules.html',    icon: 'fa-solid fa-calendar-days',  label: 'Расписания' },
    { href: 'analytics.html',    icon: 'fa-solid fa-chart-bar',      label: 'Аналитика' },
    { href: 'webhooks.html',     icon: 'fa-solid fa-plug',           label: 'Webhooks' },
    { href: 'notifications.html',   icon: 'fa-solid fa-bell',           label: 'Уведомления' },
    { href: 'credential_types.html', icon: 'fa-solid fa-id-card',      label: 'Типы ключей',  noId: true },
    { href: 'playbooks.html',    icon: 'fa-solid fa-scroll',         label: 'Playbooks' },
    { href: 'workflow.html',     icon: 'fa-solid fa-diagram-project', label: 'Workflows' },
    { href: 'drift.html',        icon: 'fa-solid fa-radar',           label: 'Drift Detection' },
    { href: 'snapshots.html',    icon: 'fa-solid fa-clock-rotate-left', label: 'Rollback' },
    { href: 'marketplace.html',  icon: 'fa-solid fa-store',           label: 'Marketplace', noId: true },
    { href: 'costs.html',        icon: 'fa-solid fa-dollar-sign',    label: 'Стоимость TF' },
    { href: 'team.html',         icon: 'fa-solid fa-users',          label: 'Команда' },
    { href: 'runners.html',      icon: 'fa-solid fa-bolt',           label: 'Runners',    noId: true },
    { href: 'apps.html',         icon: 'fa-solid fa-puzzle-piece',   label: 'Apps',       noId: true },
    { href: 'ldap_groups.html',  icon: 'fa-solid fa-sitemap',        label: 'LDAP Groups', noId: true },
    { href: 'mcp.html',          icon: 'fa-solid fa-robot',          label: 'MCP / AI',   noId: true },
];

function renderSidebar() {
    const sidebar = document.querySelector('.sidebar');
    if (!sidebar) return;

    const urlParams = new URLSearchParams(window.location.search);
    const urlProjectId = urlParams.get('id');
    const currentPage = location.pathname.split('/').pop() || 'index.html';

    let currentTheme = localStorage.getItem(THEME_KEY) || 'light';
    if (currentTheme !== 'light' && currentTheme !== 'dark') {
        currentTheme = 'light';
    }
    document.body.classList.toggle('theme-dark', currentTheme === 'dark');

    let currentLang = localStorage.getItem(LANG_KEY) || 'ru';
    if (!['ru', 'en'].includes(currentLang)) {
        currentLang = 'ru';
    }

    const storedUser = (() => { try { return JSON.parse(localStorage.getItem(USER_KEY) || 'null'); } catch { return null; } })();
    const userName = storedUser?.username || storedUser?.name || '';

    let storedProjectId = localStorage.getItem(PROJECT_KEY) || null;

    const projectIdForLinks = urlProjectId || storedProjectId;

    const navItems = SIDEBAR_ITEMS.map(item => {
        const href = (item.noId || !projectIdForLinks)
            ? item.href
            : item.href + '?id=' + projectIdForLinks;
        const isActive = currentPage === item.href ? 'class="active"' : '';
        return `<li><a href="${href}" ${isActive}><i class="nav-icon ${item.icon}"></i>${item.label}</a></li>`;
    }).join('');

    sidebar.innerHTML = `
        <div class="sidebar-logo">
            <div class="sidebar-logo-dot"><img src="/logo.png" alt=""></div>
            <h2>Velum</h2>
        </div>
        <div class="sidebar-section">
            <span>Проект</span>
        </div>
        <div class="sidebar-project" data-current-page="${escapeHtml(currentPage)}">
            <select class="form-control sidebar-project-select">
                <option value="">Загрузка проектов...</option>
            </select>
        </div>
        <div class="sidebar-section">Навигация</div>
        <ul class="sidebar-nav">${navItems}</ul>
        <div class="sidebar-footer">
            ${userName ? `
            <div class="sidebar-user">
                <div class="sidebar-user-avatar">${userName[0].toUpperCase()}</div>
                <div class="sidebar-user-info">
                    <div class="sidebar-user-name">${escapeHtml(userName)}</div>
                </div>
            </div>` : ''}
            <div class="sidebar-footer-controls">
                <button type="button" class="sidebar-theme-toggle" title="${currentTheme === 'dark' ? 'Светлая тема' : 'Тёмная тема'}">
                    <i class="fa-solid ${currentTheme === 'dark' ? 'fa-sun' : 'fa-moon'}"></i>
                </button>
                <select class="sidebar-lang-select">
                    <option value="ru"${currentLang === 'ru' ? ' selected' : ''}>RU</option>
                    <option value="en"${currentLang === 'en' ? ' selected' : ''}>EN</option>
                </select>
                <a href="tokens.html" class="sidebar-icon-btn" title="API Токены"><i class="fa-solid fa-key"></i></a>
                <a href="users.html" class="sidebar-icon-btn" title="Аккаунт"><i class="fa-solid fa-user-gear"></i></a>
                <button class="sidebar-icon-btn" type="button" onclick="api.logout()" title="Выйти"><i class="fa-solid fa-right-from-bracket"></i></button>
            </div>
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

    const projectSelect = sidebar.querySelector('.sidebar-project-select');
    if (projectSelect) {
        api.getProjects()
            .then(projects => {
                if (!Array.isArray(projects) || projects.length === 0) {
                    projectSelect.innerHTML = '<option value="">Нет проектов</option>';
                    return;
                }
                let effectiveProjectId = urlProjectId || storedProjectId || String(projects[0].id);
                localStorage.setItem(PROJECT_KEY, effectiveProjectId);

                projectSelect.innerHTML = projects.map(p => `
                    <option value="${p.id}"${String(p.id) === String(effectiveProjectId) ? ' selected' : ''}>
                        ${escapeHtml(p.name || 'Проект ' + p.id)}
                    </option>
                `).join('');

                projectSelect.addEventListener('change', () => {
                    const selectedId = projectSelect.value || '';
                    if (!selectedId) return;
                    localStorage.setItem(PROJECT_KEY, selectedId);
                    const page = projectSelect.closest('.sidebar-project')?.dataset.currentPage || 'project.html';
                    if (page === 'index.html' || page === 'global_tasks.html' || page === 'apps.html' || page === 'runners.html') {
                        window.location.href = `project.html?id=${encodeURIComponent(selectedId)}`;
                    } else {
                        window.location.href = `${page}?id=${encodeURIComponent(selectedId)}`;
                    }
                });
            })
            .catch(() => {
                projectSelect.innerHTML = '<option value="">Ошибка загрузки проектов</option>';
            });
    }

    const themeToggle = sidebar.querySelector('.sidebar-theme-toggle');
    if (themeToggle) {
        themeToggle.addEventListener('click', () => {
            const current = localStorage.getItem(THEME_KEY) || 'light';
            const next = current === 'dark' ? 'light' : 'dark';
            localStorage.setItem(THEME_KEY, next);
            document.body.classList.toggle('theme-dark', next === 'dark');
            const icon = themeToggle.querySelector('i');
            if (icon) icon.className = `fa-solid ${next === 'dark' ? 'fa-sun' : 'fa-moon'}`;
            themeToggle.title = next === 'dark' ? 'Светлая тема' : 'Тёмная тема';
        });
    }

    const langSelect = sidebar.querySelector('.sidebar-lang-select');
    if (langSelect) {
        langSelect.addEventListener('change', () => {
            const value = langSelect.value;
            localStorage.setItem(LANG_KEY, value);
        });
    }
}

document.addEventListener('DOMContentLoaded', renderSidebar);

// ==================== Export ====================

window.api = api;
window.ui = ui;
window.checkAuth = checkAuth;
window.escapeHtml = escapeHtml;
window.formatDate = formatDate;
