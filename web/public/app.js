/**
 * Semaphore UI - Vanilla JavaScript Application
 * Full CRUD для всех сущностей
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
        const name = prompt('Название шаблона:'); if (!name) return;
        const playbook = prompt('Playbook (например, site.yml):', 'site.yml'); if (!playbook) return;
        try { 
            await api.post(`/project/${state.currentProjectId}/templates`, { 
                name, playbook, type: 'ansible', app: 'ansible',
                inventory_id: state.inventories[0]?.id || null,
                repository_id: state.repositories[0]?.id || null,
                environment_id: null
            }); 
            await ui.loadTemplates(); 
            alert('✅ Шаблон создан!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async editTemplate(id) {
        const template = state.templates.find(t => t.id === id); if (!template) return;
        const name = prompt('Название:', template.name); if (!name) return;
        const playbook = prompt('Playbook:', template.playbook); if (!playbook) return;
        try { 
            await api.put(`/project/${state.currentProjectId}/templates/${id}`, { name, playbook }); 
            await ui.loadTemplates(); 
            alert('✅ Шаблон обновлён!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async deleteTemplate(id) { if (!confirm('Удалить шаблон?')) return; try { await api.delete(`/project/${state.currentProjectId}/templates/${id}`); await ui.loadTemplates(); alert('✅ Шаблон удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    
    // === Tasks CRUD ===
    async createTask() {
        const templateId = prompt('ID шаблона:'); if (!templateId) return;
        try { 
            await api.post(`/project/${state.currentProjectId}/tasks`, { template_id: parseInt(templateId) }); 
            await ui.loadTasks(); 
            alert('✅ Задача создана!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async viewTask(id) { alert(`Задача #${id}`); },
    
    // === Inventory CRUD ===
    async createInventory() {
        const name = prompt('Название инвентаря:'); if (!name) return;
        try { 
            await api.post(`/project/${state.currentProjectId}/inventory`, { 
                name, inventory_type: 'static', inventory_data: 'all:\n  hosts:\n    localhost:\n      ansible_connection: local' 
            }); 
            await ui.loadInventory(); 
            alert('✅ Инвентарь создан!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async editInventory(id) {
        const item = state.inventories.find(i => i.id === id); if (!item) return;
        const name = prompt('Название:', item.name); if (!name) return;
        try { 
            await api.put(`/project/${state.currentProjectId}/inventory/${id}`, { name }); 
            await ui.loadInventory(); 
            alert('✅ Инвентарь обновлён!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async deleteInventory(id) { if (!confirm('Удалить инвентарь?')) return; try { await api.delete(`/project/${state.currentProjectId}/inventory/${id}`); await ui.loadInventory(); alert('✅ Инвентарь удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    
    // === Keys CRUD ===
    async createKey() {
        const name = prompt('Название ключа:'); if (!name) return;
        const type = prompt('Тип ключа (ssh, login_password, access_key):', 'ssh'); if (!type) return;
        try { 
            await api.post(`/project/${state.currentProjectId}/keys`, { name, type }); 
            await ui.loadKeys(); 
            alert('✅ Ключ создан!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async editKey(id) {
        const key = state.keys.find(k => k.id === id); if (!key) return;
        const name = prompt('Название:', key.name); if (!name) return;
        try { 
            await api.put(`/project/${state.currentProjectId}/keys/${id}`, { name }); 
            await ui.loadKeys(); 
            alert('✅ Ключ обновлён!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async deleteKey(id) { if (!confirm('Удалить ключ?')) return; try { await api.delete(`/project/${state.currentProjectId}/keys/${id}`); await ui.loadKeys(); alert('✅ Ключ удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } },
    
    // === Repositories CRUD ===
    async createRepository() {
        const name = prompt('Название репозитория:'); if (!name) return;
        const gitUrl = prompt('Git URL:'); if (!gitUrl) return;
        try { 
            await api.post(`/project/${state.currentProjectId}/repositories`, { name, git_url: gitUrl, git_type: 'git' }); 
            await ui.loadRepositories(); 
            alert('✅ Репозиторий создан!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async editRepository(id) {
        const repo = state.repositories.find(r => r.id === id); if (!repo) return;
        const name = prompt('Название:', repo.name); if (!name) return;
        const gitUrl = prompt('Git URL:', repo.git_url); if (!gitUrl) return;
        try { 
            await api.put(`/project/${state.currentProjectId}/repositories/${id}`, { name, git_url: gitUrl }); 
            await ui.loadRepositories(); 
            alert('✅ Репозиторий обновлён!'); 
        } catch (error) { alert('❌ Ошибка: ' + error.message); }
    },
    async deleteRepository(id) { if (!confirm('Удалить репозиторий?')) return; try { await api.delete(`/project/${state.currentProjectId}/repositories/${id}`); await ui.loadRepositories(); alert('✅ Репозиторий удалён!'); } catch (error) { alert('❌ Ошибка: ' + error.message); } }
};

document.addEventListener('DOMContentLoaded', () => app.init());
