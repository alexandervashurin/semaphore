/**
 * Inventory Form Component
 * Форма создания/редактирования инвентаря
 */

import { Form } from './forms.js';
import api from '../api.js';
import { $ } from '../utils/dom.js';

export class InventoryForm {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.projectId = options.projectId;
    this.inventoryId = options.inventoryId;
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
    if (this.inventoryId) {
      try {
        this.data = await api.getInventory(this.projectId, this.inventoryId);
      } catch (error) {
        console.error('Error loading inventory:', error);
      }
    }
    
    this.render();
  }

  /**
   * Рендер формы
   */
  render() {
    const isFile = this.data.type === 'file' || !this.data.type;
    
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
          type: 'select',
          name: 'type',
          label: 'Тип',
          required: true,
          value: isFile ? 'file' : 'static',
          options: [
            { value: 'file', text: 'Файл' },
            { value: 'static', text: 'Статический' }
          ]
        },
        {
          type: 'text',
          name: 'inventory',
          label: isFile ? 'Путь к файлу' : 'Инвентарь (INI формат)',
          required: true,
          value: this.data.inventory || '',
          placeholder: isFile ? '/path/to/inventory' : '[webservers]\nhost1\nhost2'
        },
        {
          type: 'select',
          name: 'ssh_key_id',
          label: 'SSH ключ',
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
          type: 'select',
          name: 'become_key_id',
          label: 'Becomes ключ (sudo)',
          value: this.data.become_key_id || '',
          options: [
            { value: '', text: 'Не выбрано' },
            ...this.keys.filter(k => k.type === 'ssh').map(k => ({
              value: k.id,
              text: k.name
            }))
          ]
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
      submitText: this.inventoryId ? 'Сохранить' : 'Создать'
    });

    if (this.inventoryId) {
      form.setData(this.data);
    }
  }

  /**
   * Обработка сабмита
   */
  async handleSubmit(data) {
    try {
      if (this.inventoryId) {
        await api.updateInventory(this.projectId, this.inventoryId, data);
      } else {
        await api.createInventory(this.projectId, data);
      }
      this.onSave(data);
    } catch (error) {
      console.error('Error saving inventory:', error);
      throw error;
    }
  }
}

export default InventoryForm;
