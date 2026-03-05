/**
 * Semaphore UI - Vanilla JS Frontend
 */

// API Client
const API = {
    baseURL: '/api',
    token: localStorage.getItem('semaphore_token'),

    async request(endpoint, options = {}) {
        const url = `${this.baseURL}${endpoint}`;
        const headers = {
            'Content-Type': 'application/json',
            ...options.headers
        };

        if (this.token) {
            headers['Authorization'] = `Bearer ${this.token}`;
        }

        try {
            const response = await fetch(url, {
                ...options,
                headers
            });

            if (response.status === 401) {
                this.logout();
                throw new Error('Unauthorized');
            }

            let data = {};
            const text = await response.text();
            if (text) {
                try {
                    data = JSON.parse(text);
                } catch (_) {}
            }
            
            if (!response.ok) {
                throw new Error(data.error || `Request failed (${response.status})`);
            }

            return data;
        } catch (error) {
            console.error('API Error:', error);
            throw error;
        }
    },

    async get(endpoint) {
        return this.request(endpoint, { method: 'GET' });
    },

    async post(endpoint, body) {
        return this.request(endpoint, {
            method: 'POST',
            body: JSON.stringify(body)
        });
    },

    async put(endpoint, body) {
        return this.request(endpoint, {
            method: 'PUT',
            body: JSON.stringify(body)
        });
    },

    async delete(endpoint) {
        return this.request(endpoint, { method: 'DELETE' });
    },

    login(username, password) {
        return this.post('/auth/login', { username: username, password: password })
            .then(data => {
                if (data.token) {
                    this.token = data.token;
                    localStorage.setItem('semaphore_token', data.token);
                }
                return data;
            });
    },

    logout() {
        this.token = null;
        localStorage.removeItem('semaphore_token');
    },

    async health() {
        return this.get('/health');
    }
};

// UI Manager
const UI = {
    views: {
        login: document.getElementById('login-view'),
        dashboard: document.getElementById('dashboard-view')
    },

    showView(viewName) {
        Object.entries(this.views).forEach(([name, el]) => {
            el.classList.toggle('hidden', name !== viewName);
        });
    },

    showError(elementId, message) {
        const el = document.getElementById(elementId);
        if (el) {
            el.textContent = message;
            el.classList.add('visible');
        }
    },

    clearError(elementId) {
        const el = document.getElementById(elementId);
        if (el) {
            el.textContent = '';
            el.classList.remove('visible');
        }
    },

    renderList(containerId, items, renderItem) {
        const container = document.getElementById(containerId);
        if (!container) return;

        if (!items || items.length === 0) {
            container.innerHTML = `
                <div class="empty-state">
                    <p>Список пуст</p>
                </div>
            `;
            return;
        }

        container.innerHTML = items.map(renderItem).join('');
    },

    showModal(title, content) {
        document.getElementById('modal-title').textContent = title;
        document.getElementById('modal-body').innerHTML = content;
        document.getElementById('modal').classList.remove('hidden');
    },

    hideModal() {
        document.getElementById('modal').classList.add('hidden');
    }
};

// Page Handlers
const Pages = {
    currentProjectId: null,

    async loadProjects() {
        try {
            const projects = await API.get('/projects');
            UI.renderList('projects-list', projects, project => `
                <div class="list-item">
                    <div class="list-item-info">
                        <h4>${this.escapeHtml(project.name)}</h4>
                        <p>ID: ${project.id}</p>
                    </div>
                    <div class="list-item-actions">
                        <button class="btn btn-secondary" onclick="Pages.selectProject(${project.id})">Открыть</button>
                        <button class="btn btn-danger" onclick="Pages.deleteProject(${project.id})">Удалить</button>
                    </div>
                </div>
            `);
        } catch (error) {
            console.error('Failed to load projects:', error);
        }
    },

    selectProject(projectId) {
        this.currentProjectId = projectId;
        this.loadProjectResources(projectId);
        alert(`Выбран проект ${projectId}. Загрузка ресурсов...`);
    },

    async loadProjectResources(projectId) {
        // Load templates
        try {
            const templates = await API.get(`/projects/${projectId}/templates`);
            UI.renderList('templates-list', templates, tpl => `
                <div class="list-item">
                    <div class="list-item-info">
                        <h4>${this.escapeHtml(tpl.name)}</h4>
                        <p>${this.escapeHtml(tpl.description || 'Без описания')}</p>
                    </div>
                    <div class="list-item-actions">
                        <button class="btn btn-success" onclick="Pages.runTemplate(${projectId}, ${tpl.id})">Запустить</button>
                        <button class="btn btn-secondary" onclick="Pages.editTemplate(${projectId}, ${tpl.id})">Изменить</button>
                    </div>
                </div>
            `);
        } catch (error) {
            UI.renderList('templates-list', [], null);
        }

        // Load tasks
        try {
            const tasks = await API.get(`/projects/${projectId}/tasks`);
            UI.renderList('tasks-list', tasks, task => `
                <div class="list-item">
                    <div class="list-item-info">
                        <h4>Задача #${task.id}</h4>
                        <p>Статус: <span class="status-badge status-${task.status}">${task.status}</span></p>
                    </div>
                </div>
            `);
        } catch (error) {
            UI.renderList('tasks-list', [], null);
        }

        // Load inventory
        try {
            const inventory = await API.get(`/projects/${projectId}/inventories`);
            UI.renderList('inventory-list', inventory, inv => `
                <div class="list-item">
                    <div class="list-item-info">
                        <h4>${this.escapeHtml(inv.name)}</h4>
                        <p>Тип: ${inv.inventory_type}</p>
                    </div>
                    <div class="list-item-actions">
                        <button class="btn btn-secondary" onclick="Pages.editInventory(${projectId}, ${inv.id})">Изменить</button>
                        <button class="btn btn-danger" onclick="Pages.deleteInventory(${projectId}, ${inv.id})">Удалить</button>
                    </div>
                </div>
            `);
        } catch (error) {
            UI.renderList('inventory-list', [], null);
        }

        // Load repositories
        try {
            const repos = await API.get(`/projects/${projectId}/repositories`);
            UI.renderList('repositories-list', repos, repo => `
                <div class="list-item">
                    <div class="list-item-info">
                        <h4>${this.escapeHtml(repo.name)}</h4>
                        <p>${this.escapeHtml(repo.git_url || repo.url || '')}</p>
                    </div>
                    <div class="list-item-actions">
                        <button class="btn btn-secondary" onclick="Pages.editRepository(${projectId}, ${repo.id})">Изменить</button>
                        <button class="btn btn-danger" onclick="Pages.deleteRepository(${projectId}, ${repo.id})">Удалить</button>
                    </div>
                </div>
            `);
        } catch (error) {
            UI.renderList('repositories-list', [], null);
        }

        // Load environments
        try {
            const envs = await API.get(`/projects/${projectId}/environments`);
            UI.renderList('environments-list', envs, env => `
                <div class="list-item">
                    <div class="list-item-info">
                        <h4>${this.escapeHtml(env.name)}</h4>
                        <p>JSON: ${env.json ? 'Да' : 'Нет'}</p>
                    </div>
                    <div class="list-item-actions">
                        <button class="btn btn-secondary" onclick="Pages.editEnvironment(${projectId}, ${env.id})">Изменить</button>
                        <button class="btn btn-danger" onclick="Pages.deleteEnvironment(${projectId}, ${env.id})">Удалить</button>
                    </div>
                </div>
            `);
        } catch (error) {
            UI.renderList('environments-list', [], null);
        }

        // Load keys
        try {
            const keys = await API.get(`/projects/${projectId}/keys`);
            UI.renderList('keys-list', keys, key => `
                <div class="list-item">
                    <div class="list-item-info">
                        <h4>${this.escapeHtml(key.name)}</h4>
                        <p>Тип: ${key.type}</p>
                    </div>
                    <div class="list-item-actions">
                        <button class="btn btn-secondary" onclick="Pages.editKey(${projectId}, ${key.id})">Изменить</button>
                        <button class="btn btn-danger" onclick="Pages.deleteKey(${projectId}, ${key.id})">Удалить</button>
                    </div>
                </div>
            `);
        } catch (error) {
            UI.renderList('keys-list', [], null);
        }
    },

    async deleteProject(projectId) {
        if (!confirm('Вы уверены, что хотите удалить этот проект?')) return;
        try {
            await API.delete(`/projects/${projectId}`);
            this.loadProjects();
        } catch (error) {
            alert('Ошибка при удалении: ' + error.message);
        }
    },

    runTemplate(projectId, templateId) {
        const content = `
            <form id="run-task-form">
                <div class="form-group">
                    <label>Параметры (JSON)</label>
                    <textarea id="task-params" placeholder='{"key": "value"}'></textarea>
                </div>
            </form>
            <div class="modal-actions">
                <button class="btn btn-secondary" onclick="UI.hideModal()">Отмена</button>
                <button class="btn btn-success" onclick="Pages.submitTask(${projectId}, ${templateId})">Запустить</button>
            </div>
        `;
        UI.showModal('Запуск задачи', content);
    },

    async submitTask(projectId, templateId) {
        const paramsText = document.getElementById('task-params').value;
        let params = {};
        if (paramsText) {
            try {
                params = JSON.parse(paramsText);
            } catch (e) {
                alert('Неверный формат JSON');
                return;
            }
        }

        try {
            await API.post(`/projects/${projectId}/tasks`, {
                template_id: templateId,
                parameters: params
            });
            UI.hideModal();
            this.loadProjectResources(projectId);
            alert('Задача создана');
        } catch (error) {
            alert('Ошибка: ' + error.message);
        }
    },

    // Placeholder edit/delete functions
    editTemplate(pid, tid) { alert(`Edit template ${tid}`); },
    editInventory(pid, iid) { alert(`Edit inventory ${iid}`); },
    deleteInventory(pid, iid) { if(confirm('Delete?')) { alert('Deleted'); } },
    editRepository(pid, rid) { alert(`Edit repository ${rid}`); },
    deleteRepository(pid, rid) { if(confirm('Delete?')) { alert('Deleted'); } },
    editEnvironment(pid, eid) { alert(`Edit environment ${eid}`); },
    deleteEnvironment(pid, eid) { if(confirm('Delete?')) { alert('Deleted'); } },
    editKey(pid, kid) { alert(`Edit key ${kid}`); },
    deleteKey(pid, kid) { if(confirm('Delete?')) { alert('Deleted'); } },

    escapeHtml(text) {
        if (!text) return '';
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
};

// Navigation
const Navigation = {
    init() {
        document.querySelectorAll('.nav-link').forEach(link => {
            link.addEventListener('click', (e) => {
                e.preventDefault();
                const page = e.target.dataset.page;
                this.showPage(page);
            });
        });

        // Modal close
        document.querySelector('.modal-close').addEventListener('click', () => UI.hideModal());
        document.querySelector('.modal-overlay').addEventListener('click', () => UI.hideModal());
    },

    showPage(pageName) {
        document.querySelectorAll('.page').forEach(page => {
            page.classList.toggle('hidden', page.id !== `${pageName}-page`);
        });

        document.querySelectorAll('.nav-link').forEach(link => {
            link.classList.toggle('active', link.dataset.page === pageName);
        });
    }
};

// Main App
const App = {
    async init() {
        // Setup event listeners
        document.getElementById('login-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.handleLogin();
        });

        document.getElementById('logout-btn').addEventListener('click', () => {
            this.handleLogout();
        });

        document.getElementById('add-project-btn').addEventListener('click', () => {
            this.showAddProjectModal();
        });

        document.getElementById('add-repository-btn')?.addEventListener('click', () => {
            App.showAddRepositoryModal();
        });
        document.getElementById('add-environment-btn')?.addEventListener('click', () => {
            App.showAddEnvironmentModal();
        });
        document.getElementById('add-inventory-btn')?.addEventListener('click', () => {
            App.showAddInventoryModal();
        });
        document.getElementById('add-template-btn')?.addEventListener('click', () => {
            App.showAddTemplateModal();
        });
        document.getElementById('add-key-btn')?.addEventListener('click', () => {
            App.showAddKeyModal();
        });

        Navigation.init();

        // Check if already logged in
        if (API.token) {
            try {
                await API.health();
                this.showDashboard();
            } catch (error) {
                UI.showView('login');
            }
        } else {
            UI.showView('login');
        }
    },

    async handleLogin() {
        const username = document.getElementById('username').value;
        const password = document.getElementById('password').value;

        try {
            await API.login(username, password);
            UI.clearError('login-error');
            this.showDashboard();
        } catch (error) {
            UI.showError('login-error', error.message || 'Ошибка входа');
        }
    },

    handleLogout() {
        API.logout();
        UI.showView('login');
        document.getElementById('username').value = '';
        document.getElementById('password').value = '';
    },

    showDashboard() {
        UI.showView('dashboard');
        Pages.loadProjects();
    },

    showAddProjectModal() {
        const content = `
            <form id="add-project-form">
                <div class="form-group">
                    <label>Название проекта</label>
                    <input type="text" id="project-name" required>
                </div>
            </form>
            <div class="modal-actions">
                <button class="btn btn-secondary" onclick="UI.hideModal()">Отмена</button>
                <button class="btn btn-primary" onclick="App.submitAddProject()">Создать</button>
            </div>
        `;
        UI.showModal('Новый проект', content);
    },

    async submitAddProject() {
        const name = document.getElementById('project-name').value;
        if (!name) return;

        try {
            const project = await API.post('/projects', { name });
            UI.hideModal();
            Pages.loadProjects();
            alert(`Проект "${name}" создан (ID: ${project.id})`);
        } catch (error) {
            alert('Ошибка: ' + error.message);
        }
    },

    showAddRepositoryModal() {
        if (!Pages.currentProjectId) {
            alert('Сначала выберите проект');
            return;
        }
        const content = `
            <form id="add-repository-form">
                <div class="form-group">
                    <label>Название</label>
                    <input type="text" id="repo-name" required>
                </div>
                <div class="form-group">
                    <label>Git URL</label>
                    <input type="text" id="repo-git-url" placeholder="https://github.com/..." required>
                </div>
            </form>
            <div class="modal-actions">
                <button class="btn btn-secondary" onclick="UI.hideModal()">Отмена</button>
                <button class="btn btn-primary" onclick="App.submitAddRepository()">Создать</button>
            </div>
        `;
        UI.showModal('Новый репозиторий', content);
    },

    async submitAddRepository() {
        const name = document.getElementById('repo-name').value;
        const git_url = document.getElementById('repo-git-url').value;
        if (!name || !git_url) return;
        try {
            await API.post(`/projects/${Pages.currentProjectId}/repositories`, { name, git_url });
            UI.hideModal();
            Pages.loadProjectResources(Pages.currentProjectId);
        } catch (error) {
            alert('Ошибка: ' + error.message);
        }
    },

    showAddEnvironmentModal() {
        if (!Pages.currentProjectId) {
            alert('Сначала выберите проект');
            return;
        }
        const content = `
            <form id="add-environment-form">
                <div class="form-group">
                    <label>Название</label>
                    <input type="text" id="env-name" required>
                </div>
                <div class="form-group">
                    <label>JSON переменные</label>
                    <textarea id="env-json" placeholder='{"KEY": "value"}'></textarea>
                </div>
            </form>
            <div class="modal-actions">
                <button class="btn btn-secondary" onclick="UI.hideModal()">Отмена</button>
                <button class="btn btn-primary" onclick="App.submitAddEnvironment()">Создать</button>
            </div>
        `;
        UI.showModal('Новое окружение', content);
    },

    async submitAddEnvironment() {
        const name = document.getElementById('env-name').value;
        const json = document.getElementById('env-json').value || '{}';
        if (!name) return;
        try {
            await API.post(`/projects/${Pages.currentProjectId}/environments`, { name, json });
            UI.hideModal();
            Pages.loadProjectResources(Pages.currentProjectId);
        } catch (error) {
            alert('Ошибка: ' + error.message);
        }
    },

    showAddInventoryModal() {
        if (!Pages.currentProjectId) {
            alert('Сначала выберите проект');
            return;
        }
        const content = `
            <form id="add-inventory-form">
                <div class="form-group">
                    <label>Название</label>
                    <input type="text" id="inv-name" required>
                </div>
                <div class="form-group">
                    <label>Тип</label>
                    <select id="inv-type">
                        <option value="static">Static</option>
                        <option value="dynamic">Dynamic</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Данные (JSON/YAML)</label>
                    <textarea id="inv-data" placeholder="localhost"></textarea>
                </div>
            </form>
            <div class="modal-actions">
                <button class="btn btn-secondary" onclick="UI.hideModal()">Отмена</button>
                <button class="btn btn-primary" onclick="App.submitAddInventory()">Создать</button>
            </div>
        `;
        UI.showModal('Новый инвентарь', content);
    },

    async submitAddInventory() {
        const name = document.getElementById('inv-name').value;
        const inventory_type = document.getElementById('inv-type').value;
        const inventory_data = document.getElementById('inv-data').value || '{}';
        if (!name) return;
        try {
            await API.post(`/projects/${Pages.currentProjectId}/inventories`, { name, inventory_type, inventory_data });
            UI.hideModal();
            Pages.loadProjectResources(Pages.currentProjectId);
        } catch (error) {
            alert('Ошибка: ' + error.message);
        }
    },

    showAddTemplateModal() {
        if (!Pages.currentProjectId) {
            alert('Сначала выберите проект');
            return;
        }
        const content = `
            <form id="add-template-form">
                <div class="form-group">
                    <label>Название</label>
                    <input type="text" id="tpl-name" required>
                </div>
                <div class="form-group">
                    <label>Playbook</label>
                    <input type="text" id="tpl-playbook" placeholder="playbook.yml" required>
                </div>
            </form>
            <div class="modal-actions">
                <button class="btn btn-secondary" onclick="UI.hideModal()">Отмена</button>
                <button class="btn btn-primary" onclick="App.submitAddTemplate()">Создать</button>
            </div>
        `;
        UI.showModal('Новый шаблон', content);
    },

    async submitAddTemplate() {
        const name = document.getElementById('tpl-name').value;
        const playbook = document.getElementById('tpl-playbook').value;
        if (!name || !playbook) return;
        try {
            await API.post(`/projects/${Pages.currentProjectId}/templates`, { name, playbook });
            UI.hideModal();
            Pages.loadProjectResources(Pages.currentProjectId);
        } catch (error) {
            alert('Ошибка: ' + error.message);
        }
    },

    showAddKeyModal() {
        if (!Pages.currentProjectId) {
            alert('Сначала выберите проект');
            return;
        }
        const content = `
            <form id="add-key-form">
                <div class="form-group">
                    <label>Название</label>
                    <input type="text" id="key-name" required>
                </div>
                <div class="form-group">
                    <label>Тип</label>
                    <select id="key-type">
                        <option value="ssh">SSH</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Ключ (приватный)</label>
                    <textarea id="key-data" placeholder="-----BEGIN OPENSSH PRIVATE KEY-----"></textarea>
                </div>
            </form>
            <div class="modal-actions">
                <button class="btn btn-secondary" onclick="UI.hideModal()">Отмена</button>
                <button class="btn btn-primary" onclick="App.submitAddKey()">Создать</button>
            </div>
        `;
        UI.showModal('Новый ключ доступа', content);
    },

    async submitAddKey() {
        const name = document.getElementById('key-name').value;
        const type = document.getElementById('key-type').value;
        const key = document.getElementById('key-data').value;
        if (!name || !key) return;
        try {
            await API.post(`/projects/${Pages.currentProjectId}/keys`, { name, type, key });
            UI.hideModal();
            Pages.loadProjectResources(Pages.currentProjectId);
        } catch (error) {
            alert('Ошибка: ' + error.message);
        }
    }
};

// Start app when DOM is ready
document.addEventListener('DOMContentLoaded', () => App.init());
