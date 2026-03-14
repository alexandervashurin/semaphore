/**
 * Semaphore Vanilla JS - Main Application
 * Точка входа приложения
 * Версия: 1.0.0 (100% миграция)
 */

import Router from './router.js';
import Store from './store.js';
import api from './api.js';
import { $, $$, createElement, delegate } from './utils/dom.js';
import { alert, confirm, prompt } from './components/dialogs.js';
import { showSuccess, showError, showLoading } from './components/snackbar.js';
import DataTable from './components/tables.js';
import { TemplateForm } from './components/template-form.js';
import { InventoryForm } from './components/inventory-form.js';
import { RepositoryForm } from './components/repository-form.js';
import { EnvironmentForm } from './components/environment-form.js';
import { KeyForm } from './components/key-form.js';
import { UserForm } from './components/user-form.js';
import { PlaybookList } from './components/playbook-list.js';

// ==================== Global State ====================

const store = new Store({
  user: null,
  project: null,
  projects: [],
  systemInfo: null,
  sidebarOpen: true,
  currentProjectId: null
});

// ==================== Router ====================

const routes = [
  { path: '/auth/login', handler: handleLogin },
  { path: '/auth/logout', handler: handleLogout },
  { path: '/', handler: handleDashboard },
  { path: '/projects', handler: handleProjects },
  { path: '/project/:projectId', redirect: '/project/:projectId/history' },
  { path: '/project/:projectId/history', handler: handleHistory },
  { path: '/project/:projectId/templates', handler: handleTemplates },
  { path: '/project/:projectId/playbooks', handler: handlePlaybooks },
  { path: '/project/:projectId/inventory', handler: handleInventory },
  { path: '/project/:projectId/repositories', handler: handleRepositories },
  { path: '/project/:projectId/environment', handler: handleEnvironment },
  { path: '/project/:projectId/keys', handler: handleKeys },
  { path: '/project/:projectId/team', handler: handleTeam },
  { path: '/project/:projectId/schedule', handler: handleSchedule },
  { path: '/project/:projectId/integrations', handler: handleIntegrations },
  { path: '/project/:projectId/audit-log', handler: handleAuditLog },
  { path: '/project/:projectId/analytics', handler: handleAnalytics },
  { path: '/project/:projectId/settings', handler: handleSettings },
  { path: '/tasks', handler: handleTasks },
  { path: '/users', handler: handleUsers },
  { path: '/runners', handler: handleRunners },
  { path: '/apps', handler: handleApps },
  { path: '/tokens', handler: handleTokens },
  { path: '/404', handler: handleNotFound }
];

const router = new Router(routes);

// ==================== Modal State ====================

let activeModal = null;

function openModal(content, options = {}) {
  const modal = new alert({
    title: options.title || '',
    content: content,
    hideNoButton: options.hideNoButton || false,
    maxWidth: options.maxWidth || 'lg',
    onConfirm: options.onConfirm || (() => {}),
    onCancel: options.onCancel || (() => {})
  });
  activeModal = modal;
  return modal;
}

function closeModal() {
  if (activeModal) {
    activeModal.close();
    activeModal = null;
  }
}

// ==================== Page Handlers ====================

async function handleLogin() {
  if (localStorage.getItem('semaphore_token')) {
    try {
      await api.getCurrentUser();
      window.location.href = '/';
      return;
    } catch (e) {
      localStorage.removeItem('semaphore_token');
    }
  }
  
  const response = await fetch('/html/auth.html');
  const html = await response.text();
  document.body.innerHTML = html;
  
  const script = document.createElement('script');
  script.type = 'module';
  script.src = '/js/auth.js';
  document.body.appendChild(script);
}

async function handleLogout() {
  try {
    await api.logout();
  } catch (e) {}
  localStorage.removeItem('semaphore_token');
  window.location.href = '/auth/login';
}

async function handleDashboard() {
  await loadLayout();
  
  const projects = await api.getProjects();
  store.state.projects = projects;
  
  const content = $('#page-content');
  if (!content) return;
  
  if (projects.length === 0) {
    content.innerHTML = `
      <div class="text-h4 mb-4">Добро пожаловать в Semaphore</div>
      <p class="mb-4">У вас пока нет проектов. Создайте первый проект, чтобы начать работу.</p>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-project-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Создать проект
      </button>
    `;
    
    $('#create-project-btn')?.addEventListener('click', () => {
      router.push('/project/new');
    });
  } else {
    content.innerHTML = `
      <div class="text-h4 mb-4">Проекты</div>
      <div class="v-row">
        ${projects.map(p => `
          <div class="v-col-4">
            <div class="v-card" style="padding: 16px; cursor: pointer;" data-project-id="${p.id}">
              <div class="text-h6 mb-2">${escapeHtml(p.name)}</div>
              <p class="text-body-2" style="color: rgba(0,0,0,0.6);">
                ${escapeHtml(p.description || 'Нет описания')}
              </p>
            </div>
          </div>
        `).join('')}
      </div>
    `;

    $$('.v-card[data-project-id]', content).forEach(card => {
      card.addEventListener('click', () => {
        router.push(`/project/${card.dataset.projectId}/history`);
      });
    });
  }
}

async function handleProjects() {
  await loadLayout();
  handleDashboard();
}

async function handleHistory(params) {
  await loadLayout(params.projectId);
  store.state.currentProjectId = params.projectId;
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">История задач</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="run-task-btn">
        <i class="v-icon mdi mdi-play"></i>
        Запустить задачу
      </button>
    </div>
    <div id="tasks-table"></div>
  `;
  
  const tasks = await api.getTasks(params.projectId);
  
  const tableContainer = $('#tasks-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Шаблон', value: 'template_name' },
        { text: 'Статус', value: 'status', format: formatTaskStatus },
        { text: 'Дата', value: 'created', format: (v) => formatDate(v) },
        { text: 'Длительность', value: 'end', format: (v, item) => formatDuration(item.start, v) }
      ],
      data: tasks || [],
      onRowClick: (item) => {
        router.push(`/project/${params.projectId}/tasks/${item.id}`);
      }
    });
  }
}

async function handleTemplates(params) {
  await loadLayout(params.projectId);
  store.state.currentProjectId = params.projectId;
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Шаблоны</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-template-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Создать шаблон
      </button>
    </div>
    <div id="templates-table"></div>
  `;
  
  const templates = await api.getTemplates(params.projectId);
  
  const tableContainer = $('#templates-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'Playbook', value: 'playbook' },
        { text: 'Окружение', value: 'environment_name' },
        { text: 'Инвентарь', value: 'inventory_name' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: templates || [],
      actions: [
        { 
          icon: 'mdi mdi-play', 
          tooltip: 'Запустить',
          handler: (item) => runTemplate(params.projectId, item.id)
        },
        { 
          icon: 'mdi mdi-pencil', 
          tooltip: 'Редактировать',
          handler: (item) => openTemplateForm(params.projectId, item.id)
        },
        { 
          icon: 'mdi mdi-delete', 
          tooltip: 'Удалить',
          handler: (item) => deleteTemplate(params.projectId, item.id)
        }
      ]
    });
  }
  
  $('#create-template-btn')?.addEventListener('click', () => {
    openTemplateForm(params.projectId, null);
  });
}

async function handlePlaybooks(params) {
  await loadLayout(params.projectId);
  store.state.currentProjectId = params.projectId;

  const content = $('#page-content');
  if (!content) return;

  // Получаем информацию о пользователе для проверки прав
  const user = store.state.user || {};
  const isAdmin = user.role === 'admin';
  const canManage = isAdmin || user.permissions?.manageProjectResources;
  const canRun = isAdmin || user.permissions?.runTemplate;

  const playbookList = new PlaybookList({
    projectId: params.projectId,
    container: '#page-content',
    canManage: canManage,
    canRun: canRun
  });

  await playbookList.init();

  // Сохраняем ссылку для очистки
  window.__playbookList = playbookList;
}

async function handleInventory(params) {
  await loadLayout(params.projectId);
  store.state.currentProjectId = params.projectId;
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Инвентари</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-inventory-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить инвентарь
      </button>
    </div>
    <div id="inventory-table"></div>
  `;
  
  const inventories = await api.getInventories(params.projectId);
  
  const tableContainer = $('#inventory-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'Тип', value: 'type', format: (v) => v === 'file' ? 'Файл' : 'Статический' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: inventories || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => openInventoryForm(params.projectId, item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteInventory(params.projectId, item.id) }
      ]
    });
  }
  
  $('#create-inventory-btn')?.addEventListener('click', () => {
    openInventoryForm(params.projectId, null);
  });
}

async function handleRepositories(params) {
  await loadLayout(params.projectId);
  store.state.currentProjectId = params.projectId;
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Репозитории</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-repo-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить репозиторий
      </button>
    </div>
    <div id="repositories-table"></div>
  `;
  
  const repos = await api.getRepositories(params.projectId);
  
  const tableContainer = $('#repositories-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'URL', value: 'url' },
        { text: 'Ветка', value: 'branch' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: repos || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => openRepositoryForm(params.projectId, item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteRepo(params.projectId, item.id) }
      ]
    });
  }
  
  $('#create-repo-btn')?.addEventListener('click', () => {
    openRepositoryForm(params.projectId, null);
  });
}

async function handleEnvironment(params) {
  await loadLayout(params.projectId);
  store.state.currentProjectId = params.projectId;
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Окружения</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-env-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить окружение
      </button>
    </div>
    <div id="environment-table"></div>
  `;
  
  const envs = await api.getEnvironments(params.projectId);
  
  const tableContainer = $('#environment-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'JSON', value: 'json', format: (v) => v ? 'Да' : 'Нет' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: envs || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => openEnvironmentForm(params.projectId, item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteEnv(params.projectId, item.id) }
      ]
    });
  }
  
  $('#create-env-btn')?.addEventListener('click', () => {
    openEnvironmentForm(params.projectId, null);
  });
}

async function handleKeys(params) {
  await loadLayout(params.projectId);
  store.state.currentProjectId = params.projectId;
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Ключи доступа</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-key-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить ключ
      </button>
    </div>
    <div id="keys-table"></div>
  `;
  
  const keys = await api.getKeys(params.projectId);
  
  const tableContainer = $('#keys-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Название', value: 'name' },
        { text: 'Тип', value: 'type', format: (v) => v === 'ssh' ? 'SSH' : 'Логин/пароль' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: keys || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => openKeyForm(params.projectId, item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteKey(params.projectId, item.id) }
      ]
    });
  }
  
  $('#create-key-btn')?.addEventListener('click', () => {
    openKeyForm(params.projectId, null);
  });
}

async function handleTeam(params) {
  await loadLayout(params.projectId);
  store.state.currentProjectId = params.projectId;
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Команда</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="add-member-btn">
        <i class="v-icon mdi mdi-account-plus"></i>
        Добавить участника
      </button>
    </div>
    <div id="team-table"></div>
  `;
  
  const team = await api.getTeam(params.projectId);
  
  const tableContainer = $('#team-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'user_id' },
        { text: 'Имя', value: 'username' },
        { text: 'Роль', value: 'role' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: team || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => editTeamMember(params.projectId, item.user_id) },
        { icon: 'mdi mdi-delete', handler: (item) => removeTeamMember(params.projectId, item.user_id) }
      ]
    });
  }
}

async function handleSchedule(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Расписание</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить расписание
      </button>
    </div>
    <div id="schedule-table"></div>
  `;
}

async function handleIntegrations(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Интеграции</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-plus"></i>
        Добавить интеграцию
      </button>
    </div>
    <div id="integrations-table"></div>
  `;
}

async function handleAuditLog(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Audit Log</h1>
      <button class="v-btn v-btn--contained v-btn--primary">
        <i class="v-icon mdi mdi-filter"></i>
        Фильтры
      </button>
    </div>
    <div id="audit-log-table"></div>
  `;
  
  const logs = await api.getAuditLogs(params.projectId);
  
  const tableContainer = $('#audit-log-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Действие', value: 'action' },
        { text: 'Объект', value: 'object_name' },
        { text: 'Пользователь', value: 'username' },
        { text: 'Дата', value: 'created', format: (v) => formatDate(v) }
      ],
      data: logs?.records || []
    });
  }
}

async function handleAnalytics(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="text-h4 mb-4">Аналитика</div>
    <div class="v-row">
      <div class="v-col-3">
        <div class="v-card" style="padding: 16px;">
          <div class="text-caption">Всего задач</div>
          <div class="text-h3">0</div>
        </div>
      </div>
      <div class="v-col-3">
        <div class="v-card" style="padding: 16px;">
          <div class="text-caption">Успешных</div>
          <div class="text-h3" style="color: #4caf50;">0</div>
        </div>
      </div>
      <div class="v-col-3">
        <div class="v-card" style="padding: 16px;">
          <div class="text-caption">Проваленных</div>
          <div class="text-h3" style="color: #f44336;">0</div>
        </div>
      </div>
      <div class="v-col-3">
        <div class="v-card" style="padding: 16px;">
          <div class="text-caption">Процент успеха</div>
          <div class="text-h3">0%</div>
        </div>
      </div>
    </div>
  `;
}

async function handleSettings(params) {
  await loadLayout(params.projectId);
  $('#page-content').innerHTML = `
    <div class="text-h4 mb-4">Настройки проекта</div>
    <div class="v-card" style="padding: 24px; max-width: 600px;">
      <div class="v-text-field">
        <input type="text" id="project-name" placeholder=" ">
        <label for="project-name">Название проекта</label>
      </div>
      <div class="v-text-field">
        <textarea id="project-description" placeholder=" " style="width: 100%; min-height: 80px;"></textarea>
        <label for="project-description">Описание</label>
      </div>
      <button class="v-btn v-btn--contained v-btn--primary">Сохранить</button>
    </div>
  `;
}

async function handleTasks() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Все задачи</div>';
}

async function handleUsers() {
  await loadLayout();
  
  const content = $('#page-content');
  if (!content) return;
  
  content.innerHTML = `
    <div class="d-flex justify-space-between align-center mb-4">
      <h1 class="text-h4">Пользователи</h1>
      <button class="v-btn v-btn--contained v-btn--primary" id="create-user-btn">
        <i class="v-icon mdi mdi-plus"></i>
        Создать пользователя
      </button>
    </div>
    <div id="users-table"></div>
  `;
  
  const users = await api.get('/users');
  
  const tableContainer = $('#users-table');
  if (tableContainer) {
    new DataTable(tableContainer, {
      headers: [
        { text: 'ID', value: 'id' },
        { text: 'Имя', value: 'username' },
        { text: 'Email', value: 'email' },
        { text: 'Роль', value: 'admin', format: (v) => v ? 'Админ' : 'Пользователь' },
        { text: '', value: 'actions', sortable: false }
      ],
      data: users || [],
      actions: [
        { icon: 'mdi mdi-pencil', handler: (item) => openUserForm(item.id) },
        { icon: 'mdi mdi-delete', handler: (item) => deleteUser(item.id) }
      ]
    });
  }
  
  $('#create-user-btn')?.addEventListener('click', () => {
    openUserForm(null);
  });
}

async function handleRunners() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Раннеры</div>';
}

async function handleApps() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Приложения</div>';
}

async function handleTokens() {
  await loadLayout();
  $('#page-content').innerHTML = '<div class="text-h4">Токены API</div>';
}

async function handleNotFound() {
  await loadLayout();
  $('#page-content').innerHTML = `
    <div class="text-center" style="padding: 48px;">
      <div class="text-h1">404</div>
      <p class="text-h6">Страница не найдена</p>
      <button class="v-btn v-btn--contained v-btn--primary" onclick="history.back()">
        Назад
      </button>
    </div>
  `;
}

// ==================== Form Handlers ====================

function openTemplateForm(projectId, templateId) {
  const container = createElement('div');
  const form = new TemplateForm(container, {
    projectId,
    templateId,
    onSave: () => {
      showSuccess(templateId ? 'Шаблон обновлён' : 'Шаблон создан');
      closeModal();
      handleTemplates({ projectId });
    },
    onCancel: () => closeModal()
  });
  
  openModal(container.innerHTML, {
    title: templateId ? 'Редактирование шаблона' : 'Создание шаблона',
    maxWidth: 'lg',
    onCancel: closeModal
  });
}

function openInventoryForm(projectId, inventoryId) {
  const container = createElement('div');
  const form = new InventoryForm(container, {
    projectId,
    inventoryId,
    onSave: () => {
      showSuccess(inventoryId ? 'Инвентарь обновлён' : 'Инвентарь создан');
      closeModal();
      handleInventory({ projectId });
    },
    onCancel: () => closeModal()
  });
  
  openModal(container.innerHTML, {
    title: inventoryId ? 'Редактирование инвентаря' : 'Создание инвентаря',
    maxWidth: 'lg',
    onCancel: closeModal
  });
}

function openRepositoryForm(projectId, repositoryId) {
  const container = createElement('div');
  const form = new RepositoryForm(container, {
    projectId,
    repositoryId,
    onSave: () => {
      showSuccess(repositoryId ? 'Репозиторий обновлён' : 'Репозиторий создан');
      closeModal();
      handleRepositories({ projectId });
    },
    onCancel: () => closeModal()
  });
  
  openModal(container.innerHTML, {
    title: repositoryId ? 'Редактирование репозитория' : 'Создание репозитория',
    maxWidth: 'lg',
    onCancel: closeModal
  });
}

function openEnvironmentForm(projectId, environmentId) {
  const container = createElement('div');
  const form = new EnvironmentForm(container, {
    projectId,
    environmentId,
    onSave: () => {
      showSuccess(environmentId ? 'Окружение обновлено' : 'Окружение создано');
      closeModal();
      handleEnvironment({ projectId });
    },
    onCancel: () => closeModal()
  });
  
  openModal(container.innerHTML, {
    title: environmentId ? 'Редактирование окружения' : 'Создание окружения',
    maxWidth: 'lg',
    onCancel: closeModal
  });
}

function openKeyForm(projectId, keyId) {
  const container = createElement('div');
  const form = new KeyForm(container, {
    projectId,
    keyId,
    onSave: () => {
      showSuccess(keyId ? 'Ключ обновлён' : 'Ключ создан');
      closeModal();
      handleKeys({ projectId });
    },
    onCancel: () => closeModal()
  });
  
  openModal(container.innerHTML, {
    title: keyId ? 'Редактирование ключа' : 'Создание ключа',
    maxWidth: 'lg',
    onCancel: closeModal
  });
}

function openUserForm(userId) {
  const container = createElement('div');
  const form = new UserForm(container, {
    userId,
    onSave: () => {
      showSuccess(userId ? 'Пользователь обновлён' : 'Пользователь создан');
      closeModal();
      handleUsers();
    },
    onCancel: () => closeModal()
  });
  
  openModal(container.innerHTML, {
    title: userId ? 'Редактирование пользователя' : 'Создание пользователя',
    maxWidth: 'md',
    onCancel: closeModal
  });
}

// ==================== Action Handlers ====================

function runTemplate(projectId, templateId) {
  confirm({
    title: 'Запуск задачи',
    content: `Вы уверены, что хотите запустить шаблон #${templateId}?`,
    yesButtonText: 'Запустить'
  }).then((result) => {
    if (result) {
      const loading = showLoading('Запуск задачи...');
      api.runTask(projectId, templateId, {})
        .then(() => {
          loading.close();
          showSuccess('Задача запущена');
        })
        .catch((error) => {
          loading.close();
          showError('Ошибка запуска: ' + error.message);
        });
    }
  });
}

function deleteTemplate(projectId, templateId) {
  confirm({
    title: 'Удаление шаблона',
    content: `Вы уверены, что хотите удалить шаблон #${templateId}?`,
    yesButtonText: 'Удалить'
  }).then((result) => {
    if (result) {
      api.deleteTemplate(projectId, templateId)
        .then(() => {
          showSuccess('Шаблон удалён');
          handleTemplates({ projectId });
        })
        .catch((error) => {
          showError('Ошибка удаления: ' + error.message);
        });
    }
  });
}

function deleteInventory(projectId, id) {
  confirm({
    title: 'Удаление инвентаря',
    content: `Удалить инвентарь #${id}?`,
    yesButtonText: 'Удалить'
  }).then((result) => {
    if (result) {
      api.deleteInventory(projectId, id)
        .then(() => {
          showSuccess('Инвентарь удалён');
          handleInventory({ projectId });
        })
        .catch((error) => {
          showError('Ошибка удаления: ' + error.message);
        });
    }
  });
}

function deleteRepo(projectId, id) {
  confirm({
    title: 'Удаление репозитория',
    content: `Удалить репозиторий #${id}?`,
    yesButtonText: 'Удалить'
  }).then((result) => {
    if (result) {
      api.deleteRepository(projectId, id)
        .then(() => {
          showSuccess('Репозиторий удалён');
          handleRepositories({ projectId });
        })
        .catch((error) => {
          showError('Ошибка удаления: ' + error.message);
        });
    }
  });
}

function deleteEnv(projectId, id) {
  confirm({
    title: 'Удаление окружения',
    content: `Удалить окружение #${id}?`,
    yesButtonText: 'Удалить'
  }).then((result) => {
    if (result) {
      api.deleteEnvironment(projectId, id)
        .then(() => {
          showSuccess('Окружение удалено');
          handleEnvironment({ projectId });
        })
        .catch((error) => {
          showError('Ошибка удаления: ' + error.message);
        });
    }
  });
}

function deleteKey(projectId, id) {
  confirm({
    title: 'Удаление ключа',
    content: `Удалить ключ #${id}?`,
    yesButtonText: 'Удалить'
  }).then((result) => {
    if (result) {
      api.deleteKey(projectId, id)
        .then(() => {
          showSuccess('Ключ удалён');
          handleKeys({ projectId });
        })
        .catch((error) => {
          showError('Ошибка удаления: ' + error.message);
        });
    }
  });
}

function deleteUser(userId) {
  confirm({
    title: 'Удаление пользователя',
    content: `Удалить пользователя #${userId}?`,
    yesButtonText: 'Удалить'
  }).then((result) => {
    if (result) {
      api.delete(`/users/${userId}`)
        .then(() => {
          showSuccess('Пользователь удалён');
          handleUsers();
        })
        .catch((error) => {
          showError('Ошибка удаления: ' + error.message);
        });
    }
  });
}

function editTeamMember(projectId, userId) {
  alert({
    title: 'Редактирование участника',
    content: `Участник #${userId}`
  });
}

function removeTeamMember(projectId, userId) {
  confirm({
    title: 'Удаление участника',
    content: `Удалить участника #${userId} из команды?`
  }).then((result) => {
    if (result) {
      api.removeTeamMember(projectId, userId)
        .then(() => {
          showSuccess('Участник удалён');
          handleTeam({ projectId });
        })
        .catch((error) => {
          showError('Ошибка удаления: ' + error.message);
        });
    }
  });
}

// ==================== Helper Functions ====================

async function loadLayout(projectId = null) {
  const response = await fetch('/html/index.html');
  const html = await response.text();
  document.body.innerHTML = html;
  
  initLayout(projectId);
  
  try {
    const user = await api.getCurrentUser();
    store.state.user = user;
    $('#username-display').textContent = user.username || user.name || 'Пользователь';
  } catch (e) {
    console.error('Failed to load user:', e);
  }
}

function initLayout(projectId) {
  const menuToggle = $('#menu-toggle');
  const navDrawer = $('#nav-drawer');
  const mainContent = $('#main-content');
  
  menuToggle?.addEventListener('click', () => {
    store.state.sidebarOpen = !store.state.sidebarOpen;
    if (store.state.sidebarOpen) {
      navDrawer.style.display = '';
      mainContent.classList.remove('main-content--no-drawer');
    } else {
      navDrawer.style.display = 'none';
      mainContent.classList.add('main-content--no-drawer');
    }
  });
  
  $('#logout-btn')?.addEventListener('click', (e) => {
    e.preventDefault();
    handleLogout();
  });
  
  const currentPath = window.location.pathname;
  $$('.v-list-item').forEach(item => {
    const route = item.dataset.route;
    if (route && currentPath.startsWith(route.replace(/:\w+/g, '\\w+'))) {
      item.classList.add('v-list-item--active');
    }
  });
}

function formatTaskStatus(status) {
  const colors = {
    success: 'success',
    failed: 'error',
    running: 'info',
    waiting: 'warning'
  };
  const labels = {
    success: 'Успешно',
    failed: 'Ошибка',
    running: 'Выполняется',
    waiting: 'Ожидание'
  };
  const color = colors[status] || '';
  return `<span class="v-chip v-chip--${color}">${labels[status] || status}</span>`;
}

function formatDate(date) {
  if (!date) return '—';
  return new Date(date).toLocaleString('ru-RU', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  });
}

function formatDuration(start, end) {
  if (!start) return '—';
  const s = new Date(start);
  const e = end ? new Date(end) : new Date();
  const diff = e - s;
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  
  if (hours > 0) {
    return `${hours}ч ${minutes % 60}м`;
  } else if (minutes > 0) {
    return `${minutes}м ${seconds % 60}с`;
  }
  return `${seconds}с`;
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

// ==================== Init ====================

(async () => {
  const token = localStorage.getItem('semaphore_token');
  if (token) {
    try {
      await api.getCurrentUser();
      router.loadRoute(window.location.pathname);
    } catch (e) {
      localStorage.removeItem('semaphore_token');
      router.loadRoute('/auth/login');
    }
  } else {
    router.loadRoute('/auth/login');
  }
})();
