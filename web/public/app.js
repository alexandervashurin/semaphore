/**
 * Semaphore UI - Vanilla JavaScript Application
 */
const API_BASE = '/api';
const STORAGE_KEY = 'semaphore_token';
const USER_KEY = 'semaphore_user';
const PROJECT_KEY = 'semaphore_project_id';

const state = {
    token: localStorage.getItem(STORAGE_KEY),
    user: JSON.parse(localStorage.getItem(USER_KEY) || 'null'),
    currentProjectId: parseInt(localStorage.getItem(PROJECT_KEY) || '1'),
    projects: [],
    templates: [],
    tasks: [],
    inventories: [],
    repositories: [],
    environments: [],
    keys: []
};

const api = {
    async request(endpoint, options = {}) {
        const url = `${API_BASE}${endpoint}`;
        const headers = { 'Content-Type': 'application/json', ...options.headers };
        if (state.token) headers['Authorization'] = `Bearer ${state.token}`;
        const response = await fetch(url, { ...options, headers });
        const data = await response.json();
        if (!response.ok) throw new Error(data.error || data.message || 'Failed');
        return data;
    },
    get(endpoint) { return this.request(endpoint, { method: 'GET' }); },
    post(endpoint, body) { return this.request(endpoint, { method: 'POST', body: JSON.stringify(body) }); },
    put(endpoint, body) { return this.request(endpoint, { method: 'PUT', body: JSON.stringify(body) }); },
    delete(endpoint) { return this.request(endpoint, { method: 'DELETE' }); }
};

const auth = {
    async login(username, password) {
        try {
            const data = await api.post('/auth/login', { username, password, expire: true });
            state.token = data.token;
            state.user = data.user || { username: username, name: username, role: 'user', admin: true };
            localStorage.setItem(STORAGE_KEY, state.token);
            localStorage.setItem(USER_KEY, JSON.stringify(state.user));
            return { success: true };
        } catch (error) {
            return { success: false, error: error.message };
        }
    },
    logout() {
        state.token = null; state.user = null;
        localStorage.removeItem(STORAGE_KEY);
        localStorage.removeItem(USER_KEY);
        ui.showView('login');
    },
    isAuthenticated() { return !!state.token; }
};

const ui = {
    elements: {},
    init() {
        this.elements = {
            loginView: document.getElementById('login-view'),
            dashboardView: document.getElementById('dashboard-view'),
            loginForm: document.getElementById('login-form'),
            loginError: document.getElementById('login-error'),
            pageTitle: document.getElementById('page-title'),
            pageContent: document.getElementById('page-content'),
            navItems: document.querySelectorAll('.nav-item'),
            menuToggle: document.getElementById('menu-toggle'),
            sidebar: document.querySelector('.sidebar'),
            logoutBtn: document.getElementById('logout-btn'),
            userName: document.getElementById('user-name'),
            userRole: document.getElementById('user-role'),
            userAvatar: document.getElementById('user-avatar'),
            projectSelector: document.getElementById('project-selector')
        };
        this.bindEvents();
        auth.isAuthenticated() ? this.showDashboard() : this.showView('login');
    },
    bindEvents() {
        this.elements.loginForm?.addEventListener('submit', async (e) => {
            e.preventDefault();
            await this.handleLogin();
        });
        this.elements.navItems.forEach(item => {
            item.addEventListener('click', (e) => {
                e.preventDefault();
                this.navigate(item.dataset.page);
            });
        });
        this.elements.menuToggle?.addEventListener('click', () => {
            this.elements.sidebar?.classList.toggle('collapsed');
        });
        this.elements.logoutBtn?.addEventListener('click', () => auth.logout());
        this.elements.projectSelector?.addEventListener('change', (e) => {
            state.currentProjectId = parseInt(e.target.value);
            localStorage.setItem(PROJECT_KEY, state.currentProjectId.toString());
            this.loadDashboard();
        });
    },
    async handleLogin() {
        const username = document.getElementById('username')?.value;
        const password = document.getElementById('password')?.value;
        const errorEl = this.elements.loginError;
        const submitBtn = this.elements.loginForm.querySelector('button[type="submit"]');
        const originalText = submitBtn.innerHTML;
        submitBtn.innerHTML = '⏳';
        submitBtn.disabled = true;
        const result = await auth.login(username, password);
        submitBtn.innerHTML = originalText;
        submitBtn.disabled = false;
        result.success ? this.showDashboard() : this.showError(errorEl, result.error);
    },
    showView(view) {
        ['login', 'dashboard'].forEach(v => {
            const el = document.getElementById(`${v}-view`);
            if (el) el.classList.toggle('active', v === view);
        });
    },
    showDashboard() {
        this.showView('dashboard');
        this.updateUserInfo();
        this.navigate('dashboard');
        this.loadDashboard();
    },
    updateUserInfo() {
        if (state.user) {
            if (this.elements.userName) this.elements.userName.textContent = state.user.name || state.user.username;
            if (this.elements.userRole) this.elements.userRole.textContent = state.user.admin ? 'Администратор' : 'Пользователь';
            if (this.elements.userAvatar) this.elements.userAvatar.textContent = (state.user.name || state.user.username || 'U')[0].toUpperCase();
        }
    },
    navigate(page) {
        this.elements.navItems.forEach(item => item.classList.toggle('active', item.dataset.page === page));
        const titles = { 
            dashboard: 'Дашборд', 
            projects: 'Проекты', 
            templates: 'Шаблоны', 
            tasks: 'Задачи', 
            inventory: 'Инвентарь', 
            environment: 'Переменные', 
            keys: 'Ключи', 
            repositories: 'Репозитории', 
            integrations: 'Интеграции', 
            users: 'Пользователи', 
            settings: 'Настройки' 
        };
        if (this.elements.pageTitle) this.elements.pageTitle.textContent = titles[page] || 'Страница';
        this.showPage(page);
    },
    showPage(page) {
        document.querySelectorAll('.page').forEach(p => p.classList.remove('active'));
        let pageEl = document.getElementById(`page-${page}`);
        if (!pageEl) pageEl = this.createPage(page);
        pageEl.classList.add('active');
    },
    createPage(page) {
        const container = this.elements.pageContent;
        const pageEl = document.createElement('div');
        pageEl.id = `page-${page}`;
        pageEl.className = 'page';
        pageEl.innerHTML = this.getPageContent(page);
        container?.appendChild(pageEl);
        this.loadPageData(page);
        return pageEl;
    },
    getPageContent(page) {
        const contents = {
            dashboard: `<div class="stats-grid">
                <div class="stat-card"><div class="stat-icon" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);">📁</div><div class="stat-details"><span class="stat-value" id="stat-projects">0</span><span class="stat-label">Проектов</span></div></div>
                <div class="stat-card"><div class="stat-icon" style="background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);">📋</div><div class="stat-details"><span class="stat-value" id="stat-templates">0</span><span class="stat-label">Шаблонов</span></div></div>
                <div class="stat-card"><div class="stat-icon" style="background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);">✅</div><div class="stat-details"><span class="stat-value" id="stat-tasks">0</span><span class="stat-label">Задач</span></div></div>
                <div class="stat-card"><div class="stat-icon" style="background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);">👥</div><div class="stat-details"><span class="stat-value" id="stat-users">0</span><span class="stat-label">Пользователей</span></div></div>
            </div>
            <div class="card"><div class="card-header"><h3 class="card-title">📊 Последние задачи</h3></div>
            <div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Проект</th><th>Шаблон</th><th>Статус</th><th>Время</th><th>Действия</th></tr></thead><tbody id="recent-tasks"></tbody></table></div></div></div>`,
            projects: `<div class="card"><div class="card-header"><h3 class="card-title">📁 Проекты</h3></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Описание</th><th>Создан</th><th>Действия</th></tr></thead><tbody id="projects-table"></tbody></table></div></div></div>`,
            templates: `<div class="card"><div class="card-header"><h3 class="card-title">📋 Шаблоны</h3></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Playbook</th><th>Тип</th><th>Действия</th></tr></thead><tbody id="templates-table"></tbody></table></div></div></div>`,
            tasks: `<div class="card"><div class="card-header"><h3 class="card-title">✅ Задачи</h3></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Шаблон</th><th>Статус</th><th>Время</th><th>Действия</th></tr></thead><tbody id="tasks-table"></tbody></table></div></div></div>`,
            inventory: `<div class="card"><div class="card-header"><h3 class="card-title">📦 Инвентарь</h3></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Тип</th><th>Действия</th></tr></thead><tbody id="inventory-table"></tbody></table></div></div></div>`,
            keys: `<div class="card"><div class="card-header"><h3 class="card-title">🔐 Ключи доступа</h3></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Тип</th><th>Действия</th></tr></thead><tbody id="keys-table"></tbody></table></div></div></div>`,
            repositories: `<div class="card"><div class="card-header"><h3 class="card-title">🗂️ Репозитории</h3></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>URL</th><th>Действия</th></tr></thead><tbody id="repositories-table"></tbody></table></div></div></div>`
        };
        return contents[page] || `<h2>${page}</h2>`;
    },
    async loadPageData(page) {
        try {
            if (page === 'dashboard') await this.loadDashboard();
            else if (page === 'projects') await this.loadProjects();
            else if (page === 'templates') await this.loadTemplates();
            else if (page === 'tasks') await this.loadTasks();
            else if (page === 'inventory') await this.loadInventory();
            else if (page === 'keys') await this.loadKeys();
            else if (page === 'repositories') await this.loadRepositories();
        } catch (error) { console.error(`Error loading ${page}:`, error); }
    },
    async loadDashboard() {
        try {
            const [projects, templates, tasks] = await Promise.all([
                api.get('/projects').catch(() => []),
                state.currentProjectId ? api.get(`/project/${state.currentProjectId}/templates`).catch(() => []) : [],
                state.currentProjectId ? api.get(`/project/${state.currentProjectId}/tasks`).catch(() => []) : []
            ]);
            state.projects = Array.isArray(projects) ? projects : [];
            state.templates = Array.isArray(templates) ? templates : [];
            state.tasks = Array.isArray(tasks) ? tasks : [];
            
            // Обновляем селектор проектов
            this.updateProjectSelector();
            
            this.updateStat('stat-projects', state.projects.length);
            this.updateStat('stat-templates', state.templates.length);
            this.updateStat('stat-tasks', state.tasks.length);
            this.updateStat('stat-users', 1);
            this.renderRecentTasks(state.tasks.slice(0, 5));
        } catch (error) { console.error('Error loading dashboard:', error); }
    },
    updateProjectSelector() {
        const selector = this.elements.projectSelector;
        if (!selector) return;
        selector.innerHTML = state.projects.map(p => 
            `<option value="${p.id}" ${p.id === state.currentProjectId ? 'selected' : ''}>${p.name}</option>`
        ).join('');
    },
    async loadProjects() {
        try {
            const projects = await api.get('/projects');
            state.projects = Array.isArray(projects) ? projects : [];
            this.renderProjectsTable(state.projects);
        } catch (error) { this.renderEmptyTable('projects-table', 5); }
    },
    async loadTemplates() {
        try {
            if (!state.currentProjectId) {
                this.renderEmptyTable('templates-table', 5);
                return;
            }
            const templates = await api.get(`/project/${state.currentProjectId}/templates`);
            state.templates = Array.isArray(templates) ? templates : [];
            this.renderTemplatesTable(state.templates);
        } catch (error) { this.renderEmptyTable('templates-table', 5); }
    },
    async loadTasks() {
        try {
            if (!state.currentProjectId) {
                this.renderEmptyTable('tasks-table', 5);
                return;
            }
            const tasks = await api.get(`/project/${state.currentProjectId}/tasks`);
            state.tasks = Array.isArray(tasks) ? tasks : [];
            this.renderTasksTable(state.tasks);
        } catch (error) { this.renderEmptyTable('tasks-table', 5); }
    },
    async loadInventory() {
        try {
            if (!state.currentProjectId) {
                this.renderEmptyTable('inventory-table', 4);
                return;
            }
            const inventory = await api.get(`/project/${state.currentProjectId}/inventory`);
            state.inventories = Array.isArray(inventory) ? inventory : [];
            this.renderInventoryTable(state.inventories);
        } catch (error) { this.renderEmptyTable('inventory-table', 4); }
    },
    async loadKeys() {
        try {
            if (!state.currentProjectId) {
                this.renderEmptyTable('keys-table', 4);
                return;
            }
            const keys = await api.get(`/project/${state.currentProjectId}/keys`);
            state.keys = Array.isArray(keys) ? keys : [];
            this.renderKeysTable(state.keys);
        } catch (error) { this.renderEmptyTable('keys-table', 4); }
    },
    async loadRepositories() {
        try {
            if (!state.currentProjectId) {
                this.renderEmptyTable('repositories-table', 4);
                return;
            }
            const repos = await api.get(`/project/${state.currentProjectId}/repositories`);
            state.repositories = Array.isArray(repos) ? repos : [];
            this.renderRepositoriesTable(state.repositories);
        } catch (error) { this.renderEmptyTable('repositories-table', 4); }
    },
    updateStat(elementId, value) { const el = document.getElementById(elementId); if (el) el.textContent = value; },
    renderRecentTasks(tasks) {
        const tbody = document.getElementById('recent-tasks');
        if (!tbody) return;
        if (!tasks || tasks.length === 0) { tbody.innerHTML = '<tr><td colspan="6" class="empty-state"><p>Нет недавних задач</p></td></tr>'; return; }
        tbody.innerHTML = tasks.map(task => `<tr><td>#${task.id || task.ID || '-'}</td><td>${task.project_name || task.project_id || '-'}</td><td>${task.template_name || task.tpl_playbook || '-'}</td><td>${this.renderStatus(task.status)}</td><td>${this.formatDate(task.created)}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.viewTask(${task.id || task.ID})">👁️</button></div></td></tr>`).join('');
    },
    renderProjectsTable(projects) {
        const tbody = document.getElementById('projects-table');
        if (!tbody) return;
        if (!projects || projects.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет проектов</p></td></tr>'; return; }
        tbody.innerHTML = projects.map(project => `<tr><td>${project.id || project.ID || '-'}</td><td><strong>${project.name || project.Name || '-'}</strong></td><td>${project.description || project.Description || '-'}</td><td>${this.formatDate(project.created)}</td><td><div class="actions"></div></td></tr>`).join('');
    },
    renderTemplatesTable(templates) {
        const tbody = document.getElementById('templates-table');
        if (!tbody) return;
        if (!templates || templates.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет шаблонов</p></td></tr>'; return; }
        tbody.innerHTML = templates.map(template => `<tr><td>${template.id || template.ID || '-'}</td><td><strong>${template.name || template.Name || '-'}</strong></td><td>${template.playbook || '-'}</td><td>${template.type || template.tpl_type || 'ansible'}</td><td><div class="actions"></div></td></tr>`).join('');
    },
    renderTasksTable(tasks) {
        const tbody = document.getElementById('tasks-table');
        if (!tbody) return;
        if (!tasks || tasks.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет задач</p></td></tr>'; return; }
        tbody.innerHTML = tasks.map(task => `<tr><td>#${task.id || task.ID || '-'}</td><td>${task.tpl_playbook || task.template_id || '-'}</td><td>${this.renderStatus(task.status)}</td><td>${this.formatDate(task.created)}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.viewTask(${task.id || task.ID})">👁️</button></div></td></tr>`).join('');
    },
    renderInventoryTable(inventory) {
        const tbody = document.getElementById('inventory-table');
        if (!tbody) return;
        if (!inventory || inventory.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет инвентарей</p></td></tr>'; return; }
        tbody.innerHTML = inventory.map(item => `<tr><td>${item.id || '-'}</td><td><strong>${item.name || '-'}</strong></td><td>${item.inventory_type || 'static'}</td><td><div class="actions"></div></td></tr>`).join('');
    },
    renderKeysTable(keys) {
        const tbody = document.getElementById('keys-table');
        if (!tbody) return;
        if (!keys || keys.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет ключей</p></td></tr>'; return; }
        tbody.innerHTML = keys.map(key => `<tr><td>${key.id || '-'}</td><td><strong>${key.name || '-'}</strong></td><td>${key.type || '-'}</td><td><div class="actions"></div></td></tr>`).join('');
    },
    renderRepositoriesTable(repos) {
        const tbody = document.getElementById('repositories-table');
        if (!tbody) return;
        if (!repos || repos.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет репозиториев</p></td></tr>'; return; }
        tbody.innerHTML = repos.map(repo => `<tr><td>${repo.id || '-'}</td><td><strong>${repo.name || '-'}</strong></td><td>${repo.git_url || '-'}</td><td><div class="actions"></div></td></tr>`).join('');
    },
    renderStatus(status) {
        const statusMap = { 
            'success': { class: 'status-success', icon: '✅', label: 'Успех' }, 
            'failed': { class: 'status-danger', icon: '❌', label: 'Ошибка' }, 
            'running': { class: 'status-info', icon: '⏳', label: 'Выполняется' }, 
            'waiting': { class: 'status-warning', icon: '⏸️', label: 'Ожидание' }, 
            'pending': { class: 'status-warning', icon: '⏸️', label: 'Ожидание' } 
        };
        const s = statusMap[status] || { class: 'status-info', icon: 'ℹ️', label: status || 'Unknown' };
        return `<span class="status ${s.class}">${s.icon} ${s.label}</span>`;
    },
    formatDate(dateString) { if (!dateString) return '-'; try { const date = new Date(dateString); return date.toLocaleDateString('ru-RU', { day: '2-digit', month: '2-digit', year: '2-digit', hour: '2-digit', minute: '2-digit' }); } catch { return dateString; } },
    showError(element, message) { if (element) { element.textContent = message; element.style.display = 'block'; setTimeout(() => { element.style.display = 'none'; }, 5000); } },
    renderEmptyTable(tableId, cols) {
        const tbody = document.getElementById(tableId);
        if (tbody) tbody.innerHTML = `<tr><td colspan="${cols}" class="empty-state"><p>Нет данных</p></td></tr>`;
    }
};

const app = {
    init() { ui.init(); },
    async viewTask(id) { alert(`Задача #${id}`); }
};

document.addEventListener('DOMContentLoaded', () => app.init());
