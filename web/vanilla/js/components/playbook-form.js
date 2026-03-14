/**
 * Playbook Form Component
 * Форма создания/редактирования Playbook
 */

import { Form } from './forms.js';
import api from '../api.js';
import { $ } from '../utils/dom.js';

export class PlaybookForm {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.projectId = options.projectId;
    this.playbookId = options.playbookId;
    this.onSave = options.onSave || (() => {});
    this.onCancel = options.onCancel || (() => {});

    this.data = {};
    this.repositories = [];

    this.init();
  }

  /**
   * Инициализация
   */
  async init() {
    // Загрузка справочников
    await this.loadReferences();

    // Загрузка данных если редактирование
    if (this.playbookId) {
      await this.loadPlaybook();
    }

    this.render();
  }

  /**
   * Загрузка справочников
   */
  async loadReferences() {
    try {
      this.repositories = await api.getRepositories(this.projectId) || [];
    } catch (error) {
      console.error('Error loading repositories:', error);
    }
  }

  /**
   * Загрузка playbook
   */
  async loadPlaybook() {
    try {
      this.data = await api.getPlaybook(this.projectId, this.playbookId);
    } catch (error) {
      console.error('Error loading playbook:', error);
    }
  }

  /**
   * Рендер формы
   */
  render() {
    const form = new Form(this.container, {
      fields: [
        {
          type: 'text',
          name: 'name',
          label: 'Название',
          required: true,
          value: this.data.name || '',
          placeholder: 'My Playbook'
        },
        {
          type: 'select',
          name: 'playbook_type',
          label: 'Тип',
          required: true,
          value: this.data.playbook_type || 'ansible',
          options: [
            { value: 'ansible', text: 'Ansible' },
            { value: 'terraform', text: 'Terraform' },
            { value: 'shell', text: 'Shell' }
          ]
        },
        {
          type: 'textarea',
          name: 'content',
          label: 'Содержимое (YAML)',
          required: true,
          value: this.data.content || '',
          rows: 15,
          placeholder: '---\n- name: My Playbook\n  hosts: all\n  tasks:\n    - name: Task 1\n      debug:\n        msg: "Hello"\n',
          className: 'code-editor'
        },
        {
          type: 'textarea',
          name: 'description',
          label: 'Описание',
          required: false,
          value: this.data.description || '',
          rows: 3,
          placeholder: 'Краткое описание playbook'
        },
        {
          type: 'select',
          name: 'repository_id',
          label: 'Репозиторий (опционально)',
          required: false,
          value: this.data.repository_id || '',
          options: [
            { value: '', text: 'Не выбрано' },
            ...this.repositories.map(r => ({
              value: r.id,
              text: r.name
            }))
          ],
          help: 'Если выбран репозиторий, playbook можно синхронизировать из Git'
        }
      ],
      buttons: [
        {
          type: 'submit',
          text: 'Сохранить',
          color: 'primary',
          onClick: () => this.submit()
        },
        {
          type: 'button',
          text: 'Отмена',
          color: 'secondary',
          onClick: () => this.onCancel()
        }
      ]
    });

    this.form = form;
  }

  /**
   * Отправка формы
   */
  async submit() {
    const formData = this.form.getData();

    // Валидация
    if (!formData.name || !formData.name.trim()) {
      this.form.setError('name', 'Название обязательно');
      return;
    }

    if (!formData.content || !formData.content.trim()) {
      this.form.setError('content', 'Содержимое обязательно');
      return;
    }

    // Валидация YAML (простая проверка)
    if (!formData.content.trim().startsWith('---') && !formData.content.trim().startsWith('-')) {
      this.form.setError('content', 'Неверный формат YAML. Должен начинаться с "---" или "-"');
      return;
    }

    try {
      const data = {
        name: formData.name.trim(),
        content: formData.content.trim(),
        description: formData.description?.trim() || null,
        playbook_type: formData.playbook_type,
        repository_id: formData.repository_id ? parseInt(formData.repository_id) : null
      };

      if (this.playbookId) {
        // Обновление
        await api.updatePlaybook(this.projectId, this.playbookId, data);
      } else {
        // Создание
        await api.createPlaybook(this.projectId, data);
      }

      this.onSave(data);
    } catch (error) {
      console.error('Error saving playbook:', error);
      this.form.setGlobalError(error.response?.data?.error || 'Ошибка при сохранении playbook');
    }
  }

  /**
   * Установка данных
   * @param {Object} data - Данные
   */
  setData(data) {
    this.data = { ...this.data, ...data };
    this.render();
  }

  /**
   * Получение данных
   * @returns {Object}
   */
  getData() {
    return this.form ? this.form.getData() : this.data;
  }

  /**
   * Валидация
   * @returns {boolean}
   */
  validate() {
    if (!this.form) return false;
    return this.form.validate();
  }

  /**
   * Сброс формы
   */
  reset() {
    if (this.form) {
      this.form.reset();
    }
    this.data = {};
    this.render();
  }
}

export default PlaybookForm;
