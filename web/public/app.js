/**
 * Semaphore UI - Vanilla JavaScript Application
 */
const API_BASE = '/api';
const STORAGE_KEY = 'semaphore_token';
const USER_KEY = 'semaphore_user';

const state = {
    token: localStorage.getItem(STORAGE_KEY),
    user: JSON.parse(localStorage.getItem(USER_KEY) || 'null'),
    projects: [],
    templates: [],
    tasks: []
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
            // Создаём объект user из username, если нет данных
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
            userAvatar: document.getElementById('user-avatar')
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
        const titles = { dashboard: 'Дашборд', projects: 'Проекты', templates: 'Шаблоны', tasks: 'Задачи', inventory: 'Инвентарь', environment: 'Переменные', keys: 'Ключи', repositories: 'Репозитории', integrations: 'Интеграции', users: 'Пользователи', settings: 'Настройки' };
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
            <div class="card"><div class="card-header"><h3 class="card-title">📊 Последние задачи</h3><button class="btn btn-primary btn-sm" onclick="app.showCreateTaskModal()">+ Новая задача</button></div>
            <div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Проект</th><th>Шаблон</th><th>Статус</th><th>Время</th><th>Действия</th></tr></thead><tbody id="recent-tasks"></tbody></table></div></div></div>`,
            projects: `<div class="card"><div class="card-header"><h3 class="card-title">📁 Проекты</h3><button class="btn btn-primary btn-sm" onclick="app.showCreateProjectModal()">+ Новый проект</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Описание</th><th>Создан</th><th>Действия</th></tr></thead><tbody id="projects-table"></tbody></table></div></div></div>`,
            templates: `<div class="card"><div class="card-header"><h3 class="card-title">📋 Шаблоны</h3><button class="btn btn-primary btn-sm" onclick="app.showCreateTemplateModal()">+ Новый шаблон</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Проект</th><th>Тип</th><th>Действия</th></tr></thead><tbody id="templates-table"></tbody></table></div></div></div>`,
            tasks: `<div class="card"><div class="card-header"><h3 class="card-title">✅ Задачи</h3><button class="btn btn-primary btn-sm" onclick="app.showCreateTaskModal()">+ Новая задача</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Проект</th><th>Шаблон</th><th>Статус</th><th>Время</th><th>Действия</th></tr></thead><tbody id="tasks-table"></tbody></table></div></div></div>`
        };
        return contents[page] || `<h2>${page}</h2>`;
    },
    async loadPageData(page) {
        try {
            if (page === 'dashboard') await this.loadDashboard();
            else if (page === 'projects') await this.loadProjects();
            else if (page === 'templates') await this.loadTemplates();
            else if (page === 'tasks') await this.loadTasks();
        } catch (error) { console.error(`Error loading ${page}:`, error); }
    },
    async loadDashboard() {
        try {
            const [projects, templates, tasks] = await Promise.all([
                api.get('/projects').catch(() => []),
                api.get('/templates').catch(() => []),
                api.get('/tasks').catch(() => [])
            ]);
            state.projects = Array.isArray(projects) ? projects : [];
            state.templates = Array.isArray(templates) ? templates : [];
            state.tasks = Array.isArray(tasks) ? tasks : [];
            this.updateStat('stat-projects', state.projects.length);
            this.updateStat('stat-templates', state.templates.length);
            this.updateStat('stat-tasks', state.tasks.length);
            this.updateStat('stat-users', 1);
            this.renderRecentTasks(state.tasks.slice(0, 5));
        } catch (error) { console.error('Error loading dashboard:', error); }
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
            const templates = await api.get('/templates');
            state.templates = Array.isArray(templates) ? templates : [];
            this.renderTemplatesTable(state.templates);
        } catch (error) { this.renderEmptyTable('templates-table', 5); }
    },
    async loadTasks() {
        try {
            const tasks = await api.get('/tasks');
            state.tasks = Array.isArray(tasks) ? tasks : [];
            this.renderTasksTable(state.tasks);
        } catch (error) { this.renderEmptyTable('tasks-table', 5); }
    },
    updateStat(elementId, value) { const el = document.getElementById(elementId); if (el) el.textContent = value; },
    renderRecentTasks(tasks) {
        const tbody = document.getElementById('recent-tasks');
        if (!tbody) return;
        if (!tasks || tasks.length === 0) { tbody.innerHTML = '<tr><td colspan="6" class="empty-state"><p>Нет недавних задач</p></td></tr>'; return; }
        tbody.innerHTML = tasks.map(task => `<tr><td>#${task.id || task.ID || '-'}</td><td>${task.project_name || task.project_id || '-'}</td><td>${task.template_name || task.template_id || '-'}</td><td>${this.renderStatus(task.status)}</td><td>${this.formatDate(task.created)}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.viewTask(${task.id || task.ID})">👁️</button></div></td></tr>`).join('');
    },
    renderProjectsTable(projects) {
        const tbody = document.getElementById('projects-table');
        if (!tbody) return;
        if (!projects || projects.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет проектов</p></td></tr>'; return; }
        tbody.innerHTML = projects.map(project => `<tr><td>${project.id || project.ID || '-'}</td><td><strong>${project.name || project.Name || '-'}</strong></td><td>${project.description || project.Description || '-'}</td><td>${this.formatDate(project.created)}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.editProject(${project.id || project.ID})">✏️</button><button class="btn btn-sm btn-delete" onclick="app.deleteProject(${project.id || project.ID})">🗑️</button></div></td></tr>`).join('');
    },
    renderTemplatesTable(templates) {
        const tbody = document.getElementById('templates-table');
        if (!tbody) return;
        if (!templates || templates.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет шаблонов</p></td></tr>'; return; }
        tbody.innerHTML = templates.map(template => `<tr><td>${template.id || template.ID || '-'}</td><td><strong>${template.name || template.Name || '-'}</strong></td><td>${template.project_id || '-'}</td><td>${template.type || 'playbook'}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.editTemplate(${template.id || template.ID})">✏️</button><button class="btn btn-sm btn-delete" onclick="app.deleteTemplate(${template.id || template.ID})">🗑️</button></div></td></tr>`).join('');
    },
    renderTasksTable(tasks) {
        const tbody = document.getElementById('tasks-table');
        if (!tbody) return;
        if (!tasks || tasks.length === 0) { tbody.innerHTML = '<tr><td colspan="6" class="empty-state"><p>Нет задач</p></td></tr>'; return; }
        tbody.innerHTML = tasks.map(task => `<tr><td>#${task.id || task.ID || '-'}</td><td>${task.project_id || '-'}</td><td>${task.template_id || '-'}</td><td>${this.renderStatus(task.status)}</td><td>${this.formatDate(task.created)}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.viewTask(${task.id || task.ID})">👁️</button></div></td></tr>`).join('');
    },
    renderStatus(status) {
        const statusMap = { 'success': { class: 'status-success', icon: '✅', label: 'Успех' }, 'failed': { class: 'status-danger', icon: '❌', label: 'Ошибка' }, 'running': { class: 'status-info', icon: '⏳', label: 'Выполняется' }, 'waiting': { class: 'status-warning', icon: '⏸️', label: 'Ожидание' }, 'pending': { class: 'status-warning', icon: '⏸️', label: 'Ожидание' } };
        const s = statusMap[status] || { class: 'status-info', icon: 'ℹ️', label: status || 'Unknown' };
        return `<span class="status ${s.class}">${s.icon} ${s.label}</span>`;
    },
    formatDate(dateString) { if (!dateString) return '-'; try { const date = new Date(dateString); return date.toLocaleDateString('ru-RU', { day: '2-digit', month: '2-digit', year: '2-digit', hour: '2-digit', minute: '2-digit' }); } catch { return dateString; } },
    showError(element, message) { if (element) { element.textContent = message; element.style.display = 'block'; setTimeout(() => { element.style.display = 'none'; }, 5000); } }
};

const app = {
    init() { ui.init(); },
    async showCreateProjectModal() {
        const name = prompt('Название проекта:'); if (!name) return;
        const description = prompt('Описание:') || '';
        try { await api.post('/projects', { name, description }); await ui.loadProjects(); await ui.loadDashboard(); alert('Проект создан!'); } catch (error) { alert('Ошибка: ' + error.message); }
    },
    async editProject(id) {
        const project = state.projects.find(p => (p.id || p.ID) === id); if (!project) return;
        const name = prompt('Название:', project.name || project.Name); if (!name) return;
        const description = prompt('Описание:', project.description || project.Description || '') || '';
        try { await api.put(`/projects/${id}`, { name, description }); await ui.loadProjects(); alert('Проект обновлён!'); } catch (error) { alert('Ошибка: ' + error.message); }
    },
    async deleteProject(id) { if (!confirm('Удалить проект?')) return; try { await api.delete(`/projects/${id}`); await ui.loadProjects(); alert('Проект удалён!'); } catch (error) { alert('Ошибка: ' + error.message); } },
    async showCreateTemplateModal() {
        const name = prompt('Название шаблона:'); if (!name) return;
        const projectId = prompt('ID проекта:'); if (!projectId) return;
        try { await api.post('/templates', { name, project_id: parseInt(projectId), playbook: 'playbook.yml', type: 'playbook' }); await ui.loadTemplates(); alert('Шаблон создан!'); } catch (error) { alert('Ошибка: ' + error.message); }
    },
    async editTemplate(id) {
        const template = state.templates.find(t => (t.id || t.ID) === id); if (!template) return;
        const name = prompt('Название:', template.name || template.Name); if (!name) return;
        try { await api.put(`/templates/${id}`, { name }); await ui.loadTemplates(); alert('Шаблон обновлён!'); } catch (error) { alert('Ошибка: ' + error.message); }
    },
    async deleteTemplate(id) { if (!confirm('Удалить шаблон?')) return; try { await api.delete(`/templates/${id}`); await ui.loadTemplates(); alert('Шаблон удалён!'); } catch (error) { alert('Ошибка: ' + error.message); } },
    async showCreateTaskModal() {
        const templateId = prompt('ID шаблона:'); if (!templateId) return;
        try { await api.post('/tasks', { template_id: parseInt(templateId) }); await ui.loadTasks(); alert('Задача создана!'); } catch (error) { alert('Ошибка: ' + error.message); } },
    async viewTask(id) { alert(`Задача #${id}`); }
};

document.addEventListener('DOMContentLoaded', () => app.init());
