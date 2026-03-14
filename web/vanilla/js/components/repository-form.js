/**
 * Repository Form Component
 * Форма создания/редактирования репозитория
 */

import { Form } from './forms.js';
import api from '../api.js';
import { $ } from '../utils/dom.js';

export class RepositoryForm {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.projectId = options.projectId;
    this.repositoryId = options.repositoryId;
    this.onSave = options.onSave || (() => {});
    this.onCancel = options.onCancel || (() => {});
    
    this.data = {};
    this.keys = [];
    
    this.init();
  }

  /**
   * Инициализация
   */
  async init() {
    // Загрузка ключей
    try {
      this.keys = await api.getKeys(this.projectId) || [];
    } catch (error) {
      console.error('Error loading keys:', error);
    }
    
    // Загрузка данных если редактирование
    if (this.repositoryId) {
      try {
        this.data = await api.getRepository(this.projectId, this.repositoryId);
      } catch (error) {
        console.error('Error loading repository:', error);
      }
    }
    
    this.render();
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
          value: this.data.name || ''
        },
        {
          type: 'text',
          name: 'url',
          label: 'URL репозитория',
          required: true,
          value: this.data.url || '',
          placeholder: 'https://github.com/user/repo.git'
        },
        {
          type: 'text',
          name: 'branch',
          label: 'Ветка',
          value: this.data.branch || 'main',
          placeholder: 'main'
        },
        {
          type: 'select',
          name: 'ssh_key_id',
          label: 'SSH ключ доступа',
          value: this.data.ssh_key_id || '',
          options: [
            { value: '', text: 'Не выбрано' },
            ...this.keys.filter(k => k.type === 'ssh').map(k => ({
              value: k.id,
              text: k.name
            }))
          ]
        },
        {
          type: 'checkbox',
          name: 'git_verify_ssl',
          label: 'Проверять SSL',
          value: this.data.git_verify_ssl !== false
        },
        {
          type: 'textarea',
          name: 'description',
          label: 'Описание',
          rows: 2,
          value: this.data.description || ''
        }
      ],
      onSubmit: (data) => this.handleSubmit(data),
      onCancel: () => this.onCancel(),
      submitText: this.repositoryId ? 'Сохранить' : 'Создать'
    });

    if (this.repositoryId) {
      form.setData(this.data);
    }
  }

  /**
   * Обработка сабмита
   */
  async handleSubmit(data) {
    try {
      if (this.repositoryId) {
        await api.updateRepository(this.projectId, this.repositoryId, data);
      } else {
        await api.createRepository(this.projectId, data);
      }
      this.onSave(data);
    } catch (error) {
      console.error('Error saving repository:', error);
      throw error;
    }
  }
}

export default RepositoryForm;
