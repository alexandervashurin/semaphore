/**
 * Semaphore UI - Vanilla JavaScript Application
 * Полный CRUD для всех сущностей с расширенными формами
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
            // Backend не возвращает data.user, создаём из username
            state.user = { 
                username: username, 
                name: username, 
                email: '',
                admin: true,
                role: 'admin'
            };
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

// Модальные окна
const modal = {
    show(title, content, onSave) {
        const modalHtml = `
            <div id="modal-overlay" class="modal-overlay" onclick="if(event.target===this) modal.close()">
                <div class="modal">
                    <div class="modal-header">
                        <h3>${title}</h3>
                        <button class="btn-close" onclick="modal.close()">×</button>
                    </div>
                    <div class="modal-body">${content}</div>
                    <div class="modal-footer">
                        <button class="btn btn-secondary" onclick="modal.close()">Отмена</button>
                        <button class="btn btn-primary" id="modal-save">Сохранить</button>
                    </div>
                </div>
            </div>
        `;
        document.body.insertAdjacentHTML('beforeend', modalHtml);
        document.getElementById('modal-save').onclick = onSave;
    },
    close() {
        const overlay = document.getElementById('modal-overlay');
        if (overlay) overlay.remove();
    }
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
        submitBtn.innerHTML = '⏳ Вход...';
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
            dashboard: '📊 Дашборд', 
            projects: '📁 Проекты', 
            templates: '📋 Шаблоны', 
            tasks: '✅ Задачи', 
            inventory: '📦 Инвентарь',
            playbooks: '📜 Playbooks',
            environments: '⚙️ Переменные',
            schedules: '⏰ Расписания',
            keys: '🔐 Ключи', 
            repositories: '🗂️ Репозитории', 
            users: '👥 Пользователи'
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
            <div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Шаблон</th><th>Статус</th><th>Время</th><th>Действия</th></tr></thead><tbody id="recent-tasks"></tbody></table></div></div></div>`,
            projects: `<div class="card"><div class="card-header"><h3 class="card-title">📁 Проекты</h3></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Описание</th><th>Создан</th></tr></thead><tbody id="projects-table"></tbody></table></div></div></div>`,
            templates: `<div class="card"><div class="card-header"><h3 class="card-title">📋 Шаблоны</h3><button class="btn btn-primary btn-sm" onclick="app.createTemplate()">+ Новый шаблон</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Playbook</th><th>Тип</th><th>Действия</th></tr></thead><tbody id="templates-table"></tbody></table></div></div></div>`,
            tasks: `<div class="card"><div class="card-header"><h3 class="card-title">✅ Задачи</h3><button class="btn btn-primary btn-sm" onclick="app.createTask()">+ Новая задача</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Playbook</th><th>Статус</th><th>Время</th><th>Действия</th></tr></thead><tbody id="tasks-table"></tbody></table></div></div></div>`,
            inventory: `<div class="card"><div class="card-header"><h3 class="card-title">📦 Инвентарь</h3><button class="btn btn-primary btn-sm" onclick="app.createInventory()">+ Новый инвентарь</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Тип</th><th>Действия</th></tr></thead><tbody id="inventory-table"></tbody></table></div></div></div>`,
            playbooks: `<div class="card"><div class="card-header"><h3 class="card-title">📜 Playbooks</h3><button class="btn btn-primary btn-sm" onclick="app.createPlaybook()">+ Новый playbook</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Тип</th><th>Действия</th></tr></thead><tbody id="playbooks-table"></tbody></table></div></div></div>`,
            environments: `<div class="card"><div class="card-header"><h3 class="card-title">⚙️ Переменные</h3><button class="btn btn-primary btn-sm" onclick="app.createEnvironment()">+ Новая переменная</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Проект</th><th>Действия</th></tr></thead><tbody id="environments-table"></tbody></table></div></div></div>`,
            schedules: `<div class="card"><div class="card-header"><h3 class="card-title">⏰ Расписания</h3><button class="btn btn-primary btn-sm" onclick="app.createSchedule()">+ Новое расписание</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Cron</th><th>Активно</th><th>Действия</th></tr></thead><tbody id="schedules-table"></tbody></table></div></div></div>`,
            keys: `<div class="card"><div class="card-header"><h3 class="card-title">🔐 Ключи доступа</h3><button class="btn btn-primary btn-sm" onclick="app.createKey()">+ Новый ключ</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>Тип</th><th>Действия</th></tr></thead><tbody id="keys-table"></tbody></table></div></div></div>`,
            repositories: `<div class="card"><div class="card-header"><h3 class="card-title">🗂️ Репозитории</h3><button class="btn btn-primary btn-sm" onclick="app.createRepository()">+ Новый репозиторий</button></div><div class="card-body"><div class="table-responsive"><table class="table"><thead><tr><th>ID</th><th>Название</th><th>URL</th><th>Действия</th></tr></thead><tbody id="repositories-table"></tbody></table></div></div></div>`
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
            else if (page === 'playbooks') await this.loadPlaybooks();
            else if (page === 'environments') await this.loadEnvironments();
            else if (page === 'schedules') await this.loadSchedules();
            else if (page === 'keys') await this.loadKeys();
            else if (page === 'repositories') await this.loadRepositories();
            else if (page === 'users') await this.loadUsers();
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
        } catch (error) { this.renderEmptyTable('projects-table', 4); }
    },
    async loadTemplates() {
        try {
            if (!state.currentProjectId) { this.renderEmptyTable('templates-table', 5); return; }
            const templates = await api.get(`/project/${state.currentProjectId}/templates`);
            state.templates = Array.isArray(templates) ? templates : [];
            this.renderTemplatesTable(state.templates);
        } catch (error) { this.renderEmptyTable('templates-table', 5); }
    },
    async loadTasks() {
        try {
            if (!state.currentProjectId) { this.renderEmptyTable('tasks-table', 5); return; }
            const tasks = await api.get(`/project/${state.currentProjectId}/tasks`);
            state.tasks = Array.isArray(tasks) ? tasks : [];
            this.renderTasksTable(state.tasks);
        } catch (error) { this.renderEmptyTable('tasks-table', 5); }
    },
    async loadInventory() {
        try {
            if (!state.currentProjectId) { this.renderEmptyTable('inventory-table', 4); return; }
            const inventory = await api.get(`/project/${state.currentProjectId}/inventory`);
            state.inventories = Array.isArray(inventory) ? inventory : [];
            this.renderInventoryTable(state.inventories);
        } catch (error) { this.renderEmptyTable('inventory-table', 4); }
    },
    async loadKeys() {
        try {
            if (!state.currentProjectId) { this.renderEmptyTable('keys-table', 4); return; }
            const keys = await api.get(`/project/${state.currentProjectId}/keys`);
            state.keys = Array.isArray(keys) ? keys : [];
            this.renderKeysTable(state.keys);
        } catch (error) { this.renderEmptyTable('keys-table', 4); }
    },
    async loadRepositories() {
        try {
            if (!state.currentProjectId) { this.renderEmptyTable('repositories-table', 4); return; }
            const repos = await api.get(`/project/${state.currentProjectId}/repositories`);
            state.repositories = Array.isArray(repos) ? repos : [];
            this.renderRepositoriesTable(state.repositories);
        } catch (error) { this.renderEmptyTable('repositories-table', 4); }
    },
    updateStat(elementId, value) { const el = document.getElementById(elementId); if (el) el.textContent = value; },
    renderRecentTasks(tasks) {
        const tbody = document.getElementById('recent-tasks');
        if (!tbody) return;
        if (!tasks || tasks.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет недавних задач</p></td></tr>'; return; }
        tbody.innerHTML = tasks.map(task => `<tr><td>#${task.id}</td><td>${task.tpl_playbook || task.template_id || '-'}</td><td>${this.renderStatus(task.status)}</td><td>${this.formatDate(task.created)}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.viewTask(${task.id})">👁️</button></div></td></tr>`).join('');
    },
    renderProjectsTable(projects) {
        const tbody = document.getElementById('projects-table');
        if (!tbody) return;
        if (!projects || projects.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет проектов</p></td></tr>'; return; }
        tbody.innerHTML = projects.map(project => `<tr><td>${project.id}</td><td><strong>${project.name}</strong></td><td>${project.description || '-'}</td><td>${this.formatDate(project.created)}</td></tr>`).join('');
    },
    renderTemplatesTable(templates) {
        const tbody = document.getElementById('templates-table');
        if (!tbody) return;
        if (!templates || templates.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет шаблонов</p></td></tr>'; return; }
        tbody.innerHTML = templates.map(template => `<tr><td>${template.id}</td><td><strong>${template.name}</strong></td><td>${template.playbook || '-'}</td><td><span class="badge badge-info">${template.type || 'ansible'}</span></td><td><div class="actions"><button class="btn btn-sm btn-edit" title="Редактировать" onclick="app.editTemplate(${template.id})">✏️</button><button class="btn btn-sm btn-delete" title="Удалить" onclick="app.deleteTemplate(${template.id})">🗑️</button></div></td></tr>`).join('');
    },
    renderTasksTable(tasks) {
        const tbody = document.getElementById('tasks-table');
        if (!tbody) return;
        if (!tasks || tasks.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет задач</p></td></tr>'; return; }
        tbody.innerHTML = tasks.map(task => `<tr><td>#${task.id}</td><td>${task.tpl_playbook || task.template_id || '-'}</td><td>${this.renderStatus(task.status)}</td><td>${this.formatDate(task.created)}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.viewTask(${task.id})">👁️</button></div></td></tr>`).join('');
    },
    renderInventoryTable(inventory) {
        const tbody = document.getElementById('inventory-table');
        if (!tbody) return;
        if (!inventory || inventory.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет инвентарей</p></td></tr>'; return; }
        tbody.innerHTML = inventory.map(item => `<tr><td>${item.id}</td><td><strong>${item.name}</strong></td><td><span class="badge badge-info">${item.inventory_type || 'static'}</span></td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.editInventory(${item.id})">✏️</button><button class="btn btn-sm btn-delete" onclick="app.deleteInventory(${item.id})">🗑️</button></div></td></tr>`).join('');
    },
    renderKeysTable(keys) {
        const tbody = document.getElementById('keys-table');
        if (!tbody) return;
        if (!keys || keys.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет ключей</p></td></tr>'; return; }
        tbody.innerHTML = keys.map(key => `<tr><td>${key.id}</td><td><strong>${key.name}</strong></td><td><span class="badge badge-info">${key.type || '-'}</span></td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.editKey(${key.id})">✏️</button><button class="btn btn-sm btn-delete" onclick="app.deleteKey(${key.id})">🗑️</button></div></td></tr>`).join('');
    },
    renderRepositoriesTable(repos) {
        const tbody = document.getElementById('repositories-table');
        if (!tbody) return;
        if (!repos || repos.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет репозиториев</p></td></tr>'; return; }
        tbody.innerHTML = repos.map(repo => `<tr><td>${repo.id}</td><td><strong>${repo.name}</strong></td><td>${repo.git_url || '-'}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.editRepository(${repo.id})">✏️</button><button class="btn btn-sm btn-delete" onclick="app.deleteRepository(${repo.id})">🗑️</button></div></td></tr>`).join('');
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
    
    // === Templates CRUD ===
    async createTemplate() {
        const content = `
            <div class="form-group">
                <label>Название *</label>
                <input type="text" id="tpl-name" class="form-control" placeholder="Deploy Infrastructure">
            </div>
            <div class="form-group">
                <label>Playbook *</label>
                <input type="text" id="tpl-playbook" class="form-control" placeholder="site.yml">
            </div>
            <div class="form-group">
                <label>Описание</label>
                <textarea id="tpl-desc" class="form-control" rows="3" placeholder="Описание шаблона"></textarea>
            </div>
            <div class="form-group">
                <label>Тип</label>
                <select id="tpl-type" class="form-control">
                    <option value="ansible">Ansible</option>
                    <option value="terraform">Terraform</option>
                    <option value="shell">Shell</option>
                </select>
            </div>
            <div class="form-group">
                <label>Git ветка</label>
                <input type="text" id="tpl-branch" class="form-control" placeholder="main">
            </div>
        `;
        modal.show('Создание шаблона', content, async () => {
            const name = document.getElementById('tpl-name').value;
            const playbook = document.getElementById('tpl-playbook').value;
            if (!name || !playbook) { alert('Заполните обязательные поля'); return; }
            try { 
                await api.post(`/project/${state.currentProjectId}/templates`, { 
                    name, playbook, 
                    description: document.getElementById('tpl-desc').value,
                    type: document.getElementById('tpl-type').value,
                    app: document.getElementById('tpl-type').value,
                    git_branch: document.getElementById('tpl-branch').value || 'main'
                }); 
                modal.close();
                await ui.loadTemplates(); 
                alert('✅ Шаблон создан!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    async editTemplate(id) {
        const template = state.templates.find(t => t.id === id); if (!template) return;
        const content = `
            <div class="form-group">
                <label>Название *</label>
                <input type="text" id="tpl-name" class="form-control" value="${template.name}">
            </div>
            <div class="form-group">
                <label>Playbook *</label>
                <input type="text" id="tpl-playbook" class="form-control" value="${template.playbook || ''}">
            </div>
            <div class="form-group">
                <label>Описание</label>
                <textarea id="tpl-desc" class="form-control" rows="3">${template.description || ''}</textarea>
            </div>
            <div class="form-group">
                <label>Тип</label>
                <select id="tpl-type" class="form-control">
                    <option value="ansible" ${template.type === 'ansible' ? 'selected' : ''}>Ansible</option>
                    <option value="terraform" ${template.type === 'terraform' ? 'selected' : ''}>Terraform</option>
                    <option value="shell" ${template.type === 'shell' ? 'selected' : ''}>Shell</option>
                </select>
            </div>
            <div class="form-group">
                <label>Git ветка</label>
                <input type="text" id="tpl-branch" class="form-control" value="${template.git_branch || 'main'}">
            </div>
        `;
        modal.show('Редактирование шаблона', content, async () => {
            const name = document.getElementById('tpl-name').value;
            const playbook = document.getElementById('tpl-playbook').value;
            if (!name || !playbook) { alert('Заполните обязательные поля'); return; }
            try { 
                await api.put(`/project/${state.currentProjectId}/templates/${id}`, { 
                    name, playbook, 
                    description: document.getElementById('tpl-desc').value,
                    type: document.getElementById('tpl-type').value,
                    git_branch: document.getElementById('tpl-branch').value
                }); 
                modal.close();
                await ui.loadTemplates(); 
                alert('✅ Шаблон обновлён!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    async deleteTemplate(id) { if (!confirm('Удалить шаблон?')) return; try { await api.delete(`/project/${state.currentProjectId}/templates/${id}`); await ui.loadTemplates(); alert('✅ Шаблон удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    
    // === Tasks CRUD ===
    async createTask() {
        const templateOptions = state.templates.map(t => `<option value="${t.id}">${t.name} (${t.playbook})</option>`).join('');
        const content = `
            <div class="form-group">
                <label>Шаблон *</label>
                <select id="task-tpl" class="form-control">${templateOptions}</select>
            </div>
        `;
        modal.show('Создание задачи', content, async () => {
            const templateId = document.getElementById('task-tpl').value;
            try { 
                await api.post(`/project/${state.currentProjectId}/tasks`, { template_id: parseInt(templateId) }); 
                modal.close();
                await ui.loadTasks(); 
                alert('✅ Задача создана!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    async viewTask(id) { alert(`Задача #${id}`); },
    
    // === Inventory CRUD ===
    async createInventory() {
        const content = `
            <div class="form-group">
                <label>Название *</label>
                <input type="text" id="inv-name" class="form-control" placeholder="Production">
            </div>
            <div class="form-group">
                <label>Тип</label>
                <select id="inv-type" class="form-control">
                    <option value="static">Static</option>
                    <option value="file">File</option>
                </select>
            </div>
            <div class="form-group">
                <label>YAML инвентарь</label>
                <textarea id="inv-data" class="form-control yaml-editor" rows="10" placeholder="all:
  hosts:
    localhost:
      ansible_connection: local"></textarea>
            </div>
        `;
        modal.show('Создание инвентаря', content, async () => {
            const name = document.getElementById('inv-name').value;
            if (!name) { alert('Введите название'); return; }
            try { 
                await api.post(`/project/${state.currentProjectId}/inventory`, { 
                    name, 
                    inventory_type: document.getElementById('inv-type').value,
                    inventory_data: document.getElementById('inv-data').value
                }); 
                modal.close();
                await ui.loadInventory(); 
                alert('✅ Инвентарь создан!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    async editInventory(id) {
        const item = state.inventories.find(i => i.id === id); if (!item) return;
        const content = `
            <div class="form-group">
                <label>Название *</label>
                <input type="text" id="inv-name" class="form-control" value="${item.name}">
            </div>
            <div class="form-group">
                <label>Тип</label>
                <select id="inv-type" class="form-control">
                    <option value="static" ${item.inventory_type === 'static' ? 'selected' : ''}>Static</option>
                    <option value="file" ${item.inventory_type === 'file' ? 'selected' : ''}>File</option>
                </select>
            </div>
            <div class="form-group">
                <label>YAML инвентарь</label>
                <textarea id="inv-data" class="form-control yaml-editor" rows="10">${item.inventory_data || ''}</textarea>
            </div>
        `;
        modal.show('Редактирование инвентаря', content, async () => {
            const name = document.getElementById('inv-name').value;
            if (!name) { alert('Введите название'); return; }
            try { 
                await api.put(`/project/${state.currentProjectId}/inventory/${id}`, { 
                    name,
                    inventory_type: document.getElementById('inv-type').value,
                    inventory_data: document.getElementById('inv-data').value
                }); 
                modal.close();
                await ui.loadInventory(); 
                alert('✅ Инвентарь обновлён!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    async deleteInventory(id) { if (!confirm('Удалить инвентарь?')) return; try { await api.delete(`/project/${state.currentProjectId}/inventory/${id}`); await ui.loadInventory(); alert('✅ Инвентарь удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    
    // === Keys CRUD ===
    async createKey() {
        const content = `
            <div class="form-group">
                <label>Название *</label>
                <input type="text" id="key-name" class="form-control" placeholder="SSH Key">
            </div>
            <div class="form-group">
                <label>Тип *</label>
                <select id="key-type" class="form-control" onchange="app.toggleKeyFields()">
                    <option value="ssh">SSH Key</option>
                    <option value="login_password">Login/Password</option>
                </select>
            </div>
            <div id="ssh-fields">
                <div class="form-group">
                    <label>SSH Private Key</label>
                    <textarea id="key-ssh" class="form-control yaml-editor" rows="6" placeholder="-----BEGIN OPENSSH PRIVATE KEY-----"></textarea>
                </div>
                <div class="form-group">
                    <label>Passphrase</label>
                    <input type="text" id="key-pass" class="form-control">
                </div>
            </div>
            <div id="login-fields" style="display:none">
                <div class="form-group">
                    <label>Login</label>
                    <input type="text" id="key-login" class="form-control">
                </div>
                <div class="form-group">
                    <label>Password</label>
                    <input type="password" id="key-password" class="form-control">
                </div>
            </div>
        `;
        modal.show('Создание ключа', content, async () => {
            const name = document.getElementById('key-name').value;
            const type = document.getElementById('key-type').value;
            if (!name) { alert('Введите название'); return; }
            const payload = { name, type };
            if (type === 'ssh') {
                payload.ssh_key = document.getElementById('key-ssh').value;
                payload.ssh_passphrase = document.getElementById('key-pass').value;
            } else {
                payload.login_password_login = document.getElementById('key-login').value;
                payload.login_password_password = document.getElementById('key-password').value;
            }
            try { 
                await api.post(`/project/${state.currentProjectId}/keys`, payload); 
                modal.close();
                await ui.loadKeys(); 
                alert('✅ Ключ создан!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    toggleKeyFields() {
        const type = document.getElementById('key-type')?.value;
        if (type) {
            document.getElementById('ssh-fields').style.display = type === 'ssh' ? 'block' : 'none';
            document.getElementById('login-fields').style.display = type === 'login_password' ? 'block' : 'none';
        }
    },
    async editKey(id) {
        const key = state.keys.find(k => k.id === id); if (!key) return;
        const content = `
            <div class="form-group">
                <label>Название *</label>
                <input type="text" id="key-name" class="form-control" value="${key.name}">
            </div>
            <div class="form-group">
                <label>Тип</label>
                <select id="key-type" class="form-control" onchange="app.toggleKeyFields()">
                    <option value="ssh" ${key.type === 'ssh' ? 'selected' : ''}>SSH Key</option>
                    <option value="login_password" ${key.type === 'login_password' ? 'selected' : ''}>Login/Password</option>
                </select>
            </div>
            <div id="ssh-fields" ${key.type === 'ssh' ? '' : 'style="display:none"'}>
                <div class="form-group">
                    <label>SSH Private Key</label>
                    <textarea id="key-ssh" class="form-control yaml-editor" rows="6">${key.ssh_key || ''}</textarea>
                </div>
                <div class="form-group">
                    <label>Passphrase</label>
                    <input type="text" id="key-pass" class="form-control" value="${key.ssh_passphrase || ''}">
                </div>
            </div>
            <div id="login-fields" ${key.type === 'login_password' ? '' : 'style="display:none"'}>
                <div class="form-group">
                    <label>Login</label>
                    <input type="text" id="key-login" class="form-control" value="${key.login_password_login || ''}">
                </div>
                <div class="form-group">
                    <label>Password</label>
                    <input type="password" id="key-password" class="form-control" value="${key.login_password_password ? '••••••••' : ''}">
                </div>
            </div>
        `;
        modal.show('Редактирование ключа', content, async () => {
            const name = document.getElementById('key-name').value;
            const type = document.getElementById('key-type').value;
            if (!name) { alert('Введите название'); return; }
            const payload = { name, type };
            if (type === 'ssh') {
                payload.ssh_key = document.getElementById('key-ssh').value;
                payload.ssh_passphrase = document.getElementById('key-pass').value;
            } else {
                payload.login_password_login = document.getElementById('key-login').value;
                const pwd = document.getElementById('key-password').value;
                if (pwd) payload.login_password_password = pwd;
            }
            try { 
                await api.put(`/project/${state.currentProjectId}/keys/${id}`, payload); 
                modal.close();
                await ui.loadKeys(); 
                alert('✅ Ключ обновлён!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    async deleteKey(id) { if (!confirm('Удалить ключ?')) return; try { await api.delete(`/project/${state.currentProjectId}/keys/${id}`); await ui.loadKeys(); alert('✅ Ключ удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    
    // === Repositories CRUD ===
    async createRepository() {
        const content = `
            <div class="form-group">
                <label>Название *</label>
                <input type="text" id="repo-name" class="form-control" placeholder="My Playbooks">
            </div>
            <div class="form-group">
                <label>Git URL *</label>
                <input type="text" id="repo-url" class="form-control" placeholder="https://github.com/user/repo.git">
            </div>
            <div class="form-group">
                <label>Git тип</label>
                <select id="repo-type" class="form-control">
                    <option value="git">Git</option>
                    <option value="github">GitHub</option>
                    <option value="gitlab">GitLab</option>
                </select>
            </div>
            <div class="form-group">
                <label>Ветка</label>
                <input type="text" id="repo-branch" class="form-control" value="main">
            </div>
        `;
        modal.show('Создание репозитория', content, async () => {
            const name = document.getElementById('repo-name').value;
            const gitUrl = document.getElementById('repo-url').value;
            if (!name || !gitUrl) { alert('Заполните обязательные поля'); return; }
            try { 
                await api.post(`/project/${state.currentProjectId}/repositories`, { 
                    name, 
                    git_url: gitUrl, 
                    git_type: document.getElementById('repo-type').value,
                    git_branch: document.getElementById('repo-branch').value || 'main'
                }); 
                modal.close();
                await ui.loadRepositories(); 
                alert('✅ Репозиторий создан!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    async editRepository(id) {
        const repo = state.repositories.find(r => r.id === id); if (!repo) return;
        const content = `
            <div class="form-group">
                <label>Название *</label>
                <input type="text" id="repo-name" class="form-control" value="${repo.name}">
            </div>
            <div class="form-group">
                <label>Git URL *</label>
                <input type="text" id="repo-url" class="form-control" value="${repo.git_url}">
            </div>
            <div class="form-group">
                <label>Git тип</label>
                <select id="repo-type" class="form-control">
                    <option value="git" ${repo.git_type === 'git' ? 'selected' : ''}>Git</option>
                    <option value="github" ${repo.git_type === 'github' ? 'selected' : ''}>GitHub</option>
                    <option value="gitlab" ${repo.git_type === 'gitlab' ? 'selected' : ''}>GitLab</option>
                </select>
            </div>
            <div class="form-group">
                <label>Ветка</label>
                <input type="text" id="repo-branch" class="form-control" value="${repo.git_branch || 'main'}">
            </div>
        `;
        modal.show('Редактирование репозитория', content, async () => {
            const name = document.getElementById('repo-name').value;
            const gitUrl = document.getElementById('repo-url').value;
            if (!name || !gitUrl) { alert('Заполните обязательные поля'); return; }
            try { 
                await api.put(`/project/${state.currentProjectId}/repositories/${id}`, { 
                    name, 
                    git_url: gitUrl, 
                    git_type: document.getElementById('repo-type').value,
                    git_branch: document.getElementById('repo-branch').value
                }); 
                modal.close();
                await ui.loadRepositories(); 
                alert('✅ Репозиторий обновлён!'); 
            } catch (error) { alert('❌ Ошибка: ' + error.message); }
        });
    },
    async deleteRepository(id) { if (!confirm('Удалить репозиторий?')) return; try { await api.delete(`/project/${state.currentProjectId}/repositories/${id}`); await ui.loadRepositories(); alert('✅ Репозиторий удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } }
};

document.addEventListener('DOMContentLoaded', () => app.init());

    // === Playbooks CRUD ===
    async loadPlaybooks() {
        try {
            if (!state.currentProjectId) { this.renderEmptyTable('playbooks-table', 4); return; }
            const playbooks = await api.get(`/project/${state.currentProjectId}/inventories/playbooks`).catch(() => []);
            state.playbooks = Array.isArray(playbooks) ? playbooks : [];
            this.renderPlaybooksTable(state.playbooks);
        } catch (error) { this.renderEmptyTable('playbooks-table', 4); }
    },
    async createPlaybook() {
        const name = prompt('Название playbook (например, site.yml):'); if (!name) return;
        const content = prompt('YAML содержимое:'); if (!content) return;
        try {
            await api.post(`/project/${state.currentProjectId}/inventory`, {
                name, inventory_type: 'file', inventory_data: content
            });
            await this.loadPlaybooks();
            alert('✅ Playbook создан!');
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    renderPlaybooksTable(playbooks) {
        const tbody = document.getElementById('playbooks-table');
        if (!tbody) return;
        if (!playbooks || playbooks.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет playbooks</p></td></tr>'; return; }
        tbody.innerHTML = playbooks.map(pb => `<tr><td>${pb.id || '-'}</td><td><strong>${pb.name || pb}</strong></td><td><span class="badge badge-info">ansible</span></td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.editPlaybook('${pb.name || pb}')">✏️</button></div></td></tr>`).join('');
    },
    async editPlaybook(name) {
        const pb = state.playbooks.find(p => (p.name || p) === name); if (!pb) return;
        const content = prompt('YAML содержимое:', pb.content || pb.inventory_data || '');
        if (!content) return;
        try {
            await api.put(`/project/${state.currentProjectId}/inventory/${pb.id || pb}`, {
                inventory_data: content
            });
            await this.loadPlaybooks();
            alert('✅ Playbook обновлён!');
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    }

    // === Environments CRUD ===
    async loadEnvironments() {
        try {
            if (!state.currentProjectId) { this.renderEmptyTable('environments-table', 4); return; }
            const envs = await api.get(`/project/${state.currentProjectId}/environments`).catch(() => []);
            state.environments = Array.isArray(envs) ? envs : [];
            this.renderEnvironmentsTable(state.environments);
        } catch (error) { this.renderEmptyTable('environments-table', 4); }
    },
    async createEnvironment() {
        const name = prompt('Название переменной:'); if (!name) return;
        const json = prompt('JSON значение (например, {"key": "value"}):', '{}'); if (!json) return;
        try {
            await api.post(`/project/${state.currentProjectId}/environments`, { name, json });
            await this.loadEnvironments();
            alert('✅ Переменная создана!');
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async editEnvironment(id) {
        const env = state.environments.find(e => e.id === id); if (!env) return;
        const name = prompt('Название:', env.name); if (!name) return;
        const json = prompt('JSON значение:', env.json); if (!json) return;
        try {
            await api.put(`/project/${state.currentProjectId}/environments/${id}`, { name, json });
            await this.loadEnvironments();
            alert('✅ Переменная обновлена!');
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async deleteEnvironment(id) { if (!confirm('Удалить переменную?')) return; try { await api.delete(`/project/${state.currentProjectId}/environments/${id}`); await this.loadEnvironments(); alert('✅ Переменная удалена!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    renderEnvironmentsTable(envs) {
        const tbody = document.getElementById('environments-table');
        if (!tbody) return;
        if (!envs || envs.length === 0) { tbody.innerHTML = '<tr><td colspan="4" class="empty-state"><p>Нет переменных</p></td></tr>'; return; }
        tbody.innerHTML = envs.map(env => `<tr><td>${env.id}</td><td><strong>${env.name}</strong></td><td>${env.project_id}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.editEnvironment(${env.id})">✏️</button><button class="btn btn-sm btn-delete" onclick="app.deleteEnvironment(${env.id})">🗑️</button></div></td></tr>`).join('');
    },

    // === Schedules CRUD ===
    async loadSchedules() {
        try {
            if (!state.currentProjectId) { this.renderEmptyTable('schedules-table', 5); return; }
            const schedules = await api.get(`/project/${state.currentProjectId}/schedules`).catch(() => []);
            state.schedules = Array.isArray(schedules) ? schedules : [];
            this.renderSchedulesTable(state.schedules);
        } catch (error) { this.renderEmptyTable('schedules-table', 5); }
    },
    async createSchedule() {
        const name = prompt('Название расписания:'); if (!name) return;
        const cron = prompt('Cron выражение (например, 0 2 * * *):'); if (!cron) return;
        const templateId = prompt('ID шаблона:'); if (!templateId) return;
        try {
            await api.post(`/project/${state.currentProjectId}/schedules`, { name, cron, template_id: parseInt(templateId), active: true });
            await this.loadSchedules();
            alert('✅ Расписание создано!');
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async editSchedule(id) {
        const sch = state.schedules.find(s => s.id === id); if (!sch) return;
        const name = prompt('Название:', sch.name); if (!name) return;
        const cron = prompt('Cron выражение:', sch.cron); if (!cron) return;
        const active = confirm('Активно? (OK=да, Cancel=нет)');
        try {
            await api.put(`/project/${state.currentProjectId}/schedules/${id}`, { name, cron, active });
            await this.loadSchedules();
            alert('✅ Расписание обновлено!');
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async deleteSchedule(id) { if (!confirm('Удалить расписание?')) return; try { await api.delete(`/project/${state.currentProjectId}/schedules/${id}`); await this.loadSchedules(); alert('✅ Расписание удалено!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    renderSchedulesTable(schedules) {
        const tbody = document.getElementById('schedules-table');
        if (!tbody) return;
        if (!schedules || schedules.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет расписаний</p></td></tr>'; return; }
        tbody.innerHTML = schedules.map(sch => `<tr><td>${sch.id}</td><td><strong>${sch.name}</strong></td><td><code>${sch.cron}</code></td><td>${sch.active ? '✅' : '❌'}</td><td><div class="actions"><button class="btn btn-sm btn-edit" onclick="app.editSchedule(${sch.id})">✏️</button><button class="btn btn-sm btn-delete" onclick="app.deleteSchedule(${sch.id})">🗑️</button></div></td></tr>`).join('');
    },

    // === Users CRUD ===
    async loadUsers() {
        try {
            const users = await api.get('/users').catch(() => []);
            state.users = Array.isArray(users) ? users : [];
            this.renderUsersTable(state.users);
        } catch (error) { this.renderEmptyTable('users-table', 5); }
    },
    async createUser() {
        const username = prompt('Логин:'); if (!username) return;
        const name = prompt('Имя:'); if (!name) return;
        const email = prompt('Email:'); if (!email) return;
        const password = prompt('Пароль:'); if (!password) return;
        const admin = confirm('Администратор? (OK=да, Cancel=нет)');
        try {
            await api.post('/users', { username, name, email, password, admin });
            await this.loadUsers();
            alert('✅ Пользователь создан!');
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async editUser(id) {
        const user = state.users.find(u => u.id === id); if (!user) return;
        const name = prompt('Имя:', user.name); if (!name) return;
        const email = prompt('Email:', user.email); if (!email) return;
        try {
            await api.put(`/users/${id}`, { name, email });
            await this.loadUsers();
            alert('✅ Пользователь обновлён!');
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async deleteUser(id) { if (!confirm('Удалить пользователя?')) return; try { await api.delete(`/users/${id}`); await this.loadUsers(); alert('✅ Пользователь удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    renderUsersTable(users) {
        const tbody = document.getElementById('users-table');
        if (!tbody) return;
        if (!users || users.length === 0) { tbody.innerHTML = '<tr><td colspan="5" class="empty-state"><p>Нет пользователей</p></td></tr>'; return; }
        tbody.innerHTML = users.map(user => `<tr><td>${user.id}</td><td><strong>${user.username}</strong></td><td>${user.name}</td><td>${user.email}</td><td>${user.admin ? '<span class="badge badge-info">Admin</span>' : 'User'}</td></tr>`).join('');
    }
};

document.addEventListener('DOMContentLoaded', () => app.init());
