/**
 * Playbook List Component
 * Страница списка Playbook'ов
 */

import api from '../api.js';
import { $, $$, createElement, delegate } from '../utils/dom.js';
import { DataTable } from './tables.js';
import { confirm } from './dialogs.js';
import { showSuccess, showError } from './snackbar.js';
import { PlaybookForm } from './playbook-form.js';

export class PlaybookList {
  constructor(options = {}) {
    this.projectId = options.projectId;
    this.container = options.container || '#page-content';
    this.canManage = options.canManage || false;
    this.canRun = options.canRun || false;

    this.playbooks = null;
    this.dataTable = null;
    this.editModal = null;
    this.currentPlaybookId = null;

    this.init();
  }

  /**
   * Инициализация
   */
  async init() {
    await this.loadPlaybooks();
    this.render();
  }

  /**
   * Загрузка playbook'ов
   */
  async loadPlaybooks() {
    try {
      this.playbooks = await api.getPlaybooks(this.projectId) || [];
    } catch (error) {
      console.error('Error loading playbooks:', error);
      showError('Ошибка при загрузке playbook\'ов');
      this.playbooks = [];
    }
  }

  /**
   * Рендер страницы
   */
  render() {
    const container = typeof this.container === 'string' 
      ? $(this.container) 
      : this.container;

    if (!container) return;

    container.innerHTML = `
      <div class="d-flex align-center mb-4">
        <button class="v-btn v-btn--text v-btn--icon v-btn--dense mr-2" id="menu-btn">
          <i class="v-icon mdi mdi-menu"></i>
        </button>
        <h2 class="text-h4 mb-0">Playbook'и</h2>
        <v-spacer></v-spacer>
        ${this.canManage ? `
          <button class="v-btn v-btn--contained v-btn--primary" id="new-playbook-btn">
            <i class="v-icon mdi mdi-plus"></i>
            Новый Playbook
          </button>
        ` : ''}
      </div>

      <div id="playbooks-content"></div>
    `;

    // Обработчик кнопки меню
    $('#menu-btn')?.addEventListener('click', () => {
      window.dispatchEvent(new CustomEvent('toggle-sidebar'));
    });

    // Обработчик кнопки создания
    $('#new-playbook-btn')?.addEventListener('click', () => {
      this.openNewDialog();
    });

    // Рендер таблицы или пустого состояния
    this.renderContent();
  }

  /**
   * Рендер содержимого
   */
  renderContent() {
    const content = $('#playbooks-content');
    if (!content) return;

    if (!this.playbooks) {
      content.innerHTML = `
        <div class="text-center pa-8">
          <div class="v-progress-circular v-progress-circular--indeterminate" 
               style="width: 48px; height: 48px;"></div>
        </div>
      `;
      return;
    }

    if (this.playbooks.length === 0) {
      content.innerHTML = `
        <div class="text-center pa-8">
          <i class="v-icon mdi mdi-file-document-outline" style="font-size: 80px; color: rgba(0,0,0,0.26);"></i>
          <h3 class="text-h6 pt-4">Нет playbook'ов</h3>
          <p class="text-body-1 grey--text">Создайте первый playbook, чтобы начать работу</p>
          ${this.canManage ? `
            <button class="v-btn v-btn--contained v-btn--primary mt-4" id="empty-new-btn">
              <i class="v-icon mdi mdi-plus"></i>
              Новый Playbook
            </button>
          ` : ''}
        </div>
      `;

      $('#empty-new-btn')?.addEventListener('click', () => {
        this.openNewDialog();
      });
      return;
    }

    // Рендер таблицы
    this.renderTable();
  }

  /**
   * Рендер таблицы
   */
  renderTable() {
    const content = $('#playbooks-content');
    if (!content) return;

    const headers = [
      { text: 'Название', value: 'name', sortable: true },
      { text: 'Тип', value: 'playbook_type', sortable: true },
      { text: 'Описание', value: 'description', sortable: false },
      { text: '', value: 'actions', sortable: false }
    ];

    const rows = this.playbooks.map(pb => ({
      id: pb.id,
      name: pb.name,
      playbook_type: pb.playbook_type,
      description: pb.description || '—',
      _raw: pb
    }));

    this.dataTable = new DataTable(content, {
      headers,
      rows,
      itemsPerPage: 25,
      hideFooter: true,
      customRender: (cell, row) => {
        if (cell.value === 'name') {
          return `
            <div class="d-flex align-center">
              <i class="v-icon mdi mdi-file-document-outline mr-2" style="color: var(--v-primary-base);"></i>
              <span>${this.escapeHtml(row.name)}</span>
            </div>
          `;
        }
        if (cell.value === 'playbook_type') {
          const colors = {
            ansible: 'orange',
            terraform: 'green',
            shell: 'blue'
          };
          const color = colors[row.playbook_type] || 'grey';
          return `
            <span class="v-chip v-chip--small" style="background-color: var(--v-${color}-base); color: white;">
              ${row.playbook_type}
            </span>
          `;
        }
        if (cell.value === 'actions') {
          return this.renderActions(row._raw);
        }
        return this.escapeHtml(row[cell.value] || '—');
      }
    });

    // Делегирование событий для кнопок действий
    delegate(content, 'click', '.playbook-action-btn', (e) => {
      const btn = e.target.closest('.playbook-action-btn');
      const action = btn.dataset.action;
      const id = parseInt(btn.dataset.id);

      if (action === 'edit') this.editPlaybook(id);
      if (action === 'run') this.runPlaybook(id);
      if (action === 'sync') this.syncPlaybook(id);
      if (action === 'delete') this.deletePlaybook(id);
    });
  }

  /**
   * Рендер кнопок действий
   */
  renderActions(playbook) {
    const isAnsible = playbook.playbook_type === 'ansible';
    const hasRepo = !!playbook.repository_id;

    return `
      <div class="v-menu">
        <button class="v-btn v-btn--icon" data-action="menu">
          <i class="v-icon mdi mdi-dots-vertical"></i>
        </button>
        <div class="v-menu__content" style="display: none; position: absolute; right: 0; top: 100%; min-width: 160px;">
          <div class="v-list">
            ${this.canManage ? `
              <div class="v-list-item playbook-action-btn" data-action="edit" data-id="${playbook.id}" style="cursor: pointer;">
                <div class="v-list-item__icon">
                  <i class="v-icon mdi mdi-pencil"></i>
                </div>
                <div class="v-list-item__content">
                  <div class="v-list-item__title">Редактировать</div>
                </div>
              </div>
            ` : ''}
            ${this.canRun && isAnsible ? `
              <div class="v-list-item playbook-action-btn" data-action="run" data-id="${playbook.id}" style="cursor: pointer;">
                <div class="v-list-item__icon">
                  <i class="v-icon mdi mdi-play" style="color: green;"></i>
                </div>
                <div class="v-list-item__content">
                  <div class="v-list-item__title">Запустить</div>
                </div>
              </div>
            ` : ''}
            ${hasRepo && this.canManage ? `
              <div class="v-list-item playbook-action-btn" data-action="sync" data-id="${playbook.id}" style="cursor: pointer;">
                <div class="v-list-item__icon">
                  <i class="v-icon mdi mdi-sync" style="color: blue;"></i>
                </div>
                <div class="v-list-item__content">
                  <div class="v-list-item__title">Sync from Git</div>
                </div>
              </div>
            ` : ''}
            ${this.canManage ? `
              <div class="v-list-item playbook-action-btn" data-action="delete" data-id="${playbook.id}" style="cursor: pointer;">
                <div class="v-list-item__icon">
                  <i class="v-icon mdi mdi-delete" style="color: red;"></i>
                </div>
                <div class="v-list-item__content">
                  <div class="v-list-item__title" style="color: red;">Удалить</div>
                </div>
              </div>
            ` : ''}
          </div>
        </div>
      </div>
    `;
  }

  /**
   * Открыть диалог создания
   */
  openNewDialog() {
    this.currentPlaybookId = null;
    this.showFormDialog();
  }

  /**
   * Редактирование playbook
   * @param {number} id - ID playbook
   */
  editPlaybook(id) {
    this.currentPlaybookId = id;
    this.showFormDialog();
  }

  /**
   * Показ диалога с формой
   */
  showFormDialog() {
    // Создание модального окна
    const modalContainer = createElement('div', { className: 'v-dialog__container' });
    document.body.appendChild(modalContainer);

    const form = new PlaybookForm(modalContainer, {
      projectId: this.projectId,
      playbookId: this.currentPlaybookId,
      onSave: () => {
        this.closeFormDialog();
        this.loadPlaybooks();
        this.render();
        showSuccess(this.currentPlaybookId ? 'Playbook обновлён' : 'Playbook создан');
      },
      onCancel: () => {
        this.closeFormDialog();
      }
    });

    this.editModal = { container: modalContainer, form };
  }

  /**
   * Закрытие диалога формы
   */
  closeFormDialog() {
    if (this.editModal?.container) {
      this.editModal.container.remove();
      this.editModal = null;
    }
  }

  /**
   * Запуск playbook
   * @param {number} id - ID playbook
   */
  async runPlaybook(id) {
    try {
      // Перенаправление на страницу запуска
      window.location.href = `/project/${this.projectId}/playbooks/${id}/run`;
    } catch (error) {
      console.error('Error running playbook:', error);
      showError('Ошибка при запуске playbook');
    }
  }

  /**
   * Синхронизация playbook из Git
   * @param {number} id - ID playbook
   */
  async syncPlaybook(id) {
    try {
      await api.syncPlaybook(this.projectId, id);
      showSuccess('Playbook синхронизирован');
      await this.loadPlaybooks();
      this.render();
    } catch (error) {
      console.error('Error syncing playbook:', error);
      showError(error.response?.data?.error || 'Ошибка при синхронизации playbook');
    }
  }

  /**
   * Удаление playbook
   * @param {number} id - ID playbook
   */
  async deletePlaybook(id) {
    const confirmed = await confirm(
      'Удалить playbook',
      'Вы уверены, что хотите удалить этот playbook? Это действие нельзя отменить.',
      { confirmText: 'Удалить', cancelText: 'Отмена' }
    );

    if (!confirmed) return;

    try {
      await api.deletePlaybook(this.projectId, id);
      showSuccess('Playbook удалён');
      await this.loadPlaybooks();
      this.render();
    } catch (error) {
      console.error('Error deleting playbook:', error);
      showError(error.response?.data?.error || 'Ошибка при удалении playbook');
    }
  }

  /**
   * Экранирование HTML
   * @param {string} text - Текст
   * @returns {string}
   */
  escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  /**
   * Обновление
   */
  async refresh() {
    await this.loadPlaybooks();
    this.render();
  }

  /**
   * Очистка
   */
  destroy() {
    if (this.dataTable) {
      this.dataTable.destroy();
    }
    this.closeFormDialog();
  }
}

export default PlaybookList;
