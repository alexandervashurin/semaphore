/**
 * Environment Form Component
 * Форма создания/редактирования окружения
 */

import { Form } from './forms.js';
import api from '../api.js';
import { $ } from '../utils/dom.js';

export class EnvironmentForm {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.projectId = options.projectId;
    this.environmentId = options.environmentId;
    this.onSave = options.onSave || (() => {});
    this.onCancel = options.onCancel || (() => {});
    
    this.data = {};
    
    this.init();
  }

  /**
   * Инициализация
   */
  async init() {
    // Загрузка данных если редактирование
    if (this.environmentId) {
      try {
        this.data = await api.getEnvironment(this.projectId, this.environmentId);
      } catch (error) {
        console.error('Error loading environment:', error);
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
          type: 'textarea',
          name: 'json',
          label: 'JSON переменные',
          rows: 10,
          value: this.data.json || '{\n  "key": "value"\n}',
          placeholder: '{\n  "variable1": "value1",\n  "variable2": "value2"\n}'
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
      submitText: this.environmentId ? 'Сохранить' : 'Создать'
    });

    if (this.environmentId) {
      form.setData(this.data);
    }
  }

  /**
   * Обработка сабмита
   */
  async handleSubmit(data) {
    try {
      // Валидация JSON
      if (data.json) {
        try {
          JSON.parse(data.json);
        } catch (e) {
          alert('Неверный формат JSON: ' + e.message);
          return;
        }
      }
      
      if (this.environmentId) {
        await api.updateEnvironment(this.projectId, this.environmentId, data);
      } else {
        await api.createEnvironment(this.projectId, data);
      }
      this.onSave(data);
    } catch (error) {
      console.error('Error saving environment:', error);
      throw error;
    }
  }
}

export default EnvironmentForm;
