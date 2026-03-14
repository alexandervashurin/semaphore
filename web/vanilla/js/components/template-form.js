/**
 * Template Form Component
 * Форма создания/редактирования шаблона
 */

import { Form } from './forms.js';
import api from '../api.js';
import { $ } from '../utils/dom.js';

export class TemplateForm {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.projectId = options.projectId;
    this.templateId = options.templateId;
    this.onSave = options.onSave || (() => {});
    this.onCancel = options.onCancel || (() => {});
    
    this.data = {};
    this.inventories = [];
    this.repositories = [];
    this.environments = [];
    
    this.init();
  }

  /**
   * Инициализация
   */
  async init() {
    // Загрузка справочников
    await this.loadReferences();
    
    // Загрузка данных если редактирование
    if (this.templateId) {
      await this.loadTemplate();
    }
    
    this.render();
  }

  /**
   * Загрузка справочников
   */
  async loadReferences() {
    try {
      const [inventories, repositories, environments] = await Promise.all([
        api.getInventories(this.projectId),
        api.getRepositories(this.projectId),
        api.getEnvironments(this.projectId)
      ]);
      
      this.inventories = inventories || [];
      this.repositories = repositories || [];
      this.environments = environments || [];
    } catch (error) {
      console.error('Error loading references:', error);
    }
  }

  /**
   * Загрузка шаблона
   */
  async loadTemplate() {
    try {
      this.data = await api.getTemplate(this.projectId, this.templateId);
    } catch (error) {
      console.error('Error loading template:', error);
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
          value: this.data.name || ''
        },
        {
          type: 'text',
          name: 'playbook',
          label: 'Playbook',
          required: true,
          value: this.data.playbook || '',
          placeholder: 'playbook.yml'
        },
        {
          type: 'textarea',
          name: 'description',
          label: 'Описание',
          rows: 3,
          value: this.data.description || ''
        },
        {
          type: 'select',
          name: 'inventory_id',
          label: 'Инвентарь',
          required: true,
          value: this.data.inventory_id || '',
          options: this.inventories.map(inv => ({
            value: inv.id,
            text: inv.name
          }))
        },
        {
          type: 'select',
          name: 'repository_id',
          label: 'Репозиторий',
          required: true,
          value: this.data.repository_id || '',
          options: this.repositories.map(repo => ({
            value: repo.id,
            text: repo.name
          }))
        },
        {
          type: 'select',
          name: 'environment_id',
          label: 'Окружение',
          value: this.data.environment_id || '',
          options: this.environments.map(env => ({
            value: env.id,
            text: env.name
          }))
        },
        {
          type: 'text',
          name: 'arguments',
          label: 'Аргументы',
          placeholder: '--extra-vars="key=value"',
          value: this.data.arguments || ''
        },
        {
          type: 'checkbox',
          name: 'allow_override_args_in_task',
          label: 'Разрешить переопределение аргументов',
          value: this.data.allow_override_args_in_task || false
        },
        {
          type: 'select',
          name: 'type',
          label: 'Тип',
          value: this.data.type || 'ansible',
          options: [
            { value: 'ansible', text: 'Ansible' },
            { value: 'terraform', text: 'Terraform' },
            { value: 'kubectl', text: 'kubectl' }
          ]
        }
      ],
      onSubmit: (data) => this.handleSubmit(data),
      onCancel: () => this.onCancel(),
      submitText: this.templateId ? 'Сохранить' : 'Создать'
    });

    if (this.templateId) {
      form.setData(this.data);
    }
  }

  /**
   * Обработка сабмита
   */
  async handleSubmit(data) {
    try {
      if (this.templateId) {
        await api.updateTemplate(this.projectId, this.templateId, data);
      } else {
        await api.createTemplate(this.projectId, data);
      }
      this.onSave(data);
    } catch (error) {
      console.error('Error saving template:', error);
      throw error;
    }
  }
}

export default TemplateForm;
