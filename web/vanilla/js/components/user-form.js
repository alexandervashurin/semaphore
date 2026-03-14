/**
 * User Form Component
 * Форма создания/редактирования пользователя
 */

import { Form } from './forms.js';
import api from '../api.js';
import { $ } from '../utils/dom.js';

export class UserForm {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.userId = options.userId;
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
    if (this.userId) {
      try {
        // Используем глобальный API для пользователей
        const users = await api.get('/users');
        this.data = users.find(u => u.id === this.userId) || {};
      } catch (error) {
        console.error('Error loading user:', error);
      }
    }
    
    this.render();
  }

  /**
   * Рендер формы
   */
  render() {
    const isNew = !this.userId;
    
    const form = new Form(this.container, {
      fields: [
        {
          type: 'text',
          name: 'username',
          label: 'Имя пользователя',
          required: true,
          value: this.data.username || '',
          attributes: {
            autocomplete: 'username'
          }
        },
        {
          type: 'text',
          name: 'name',
          label: 'Полное имя',
          value: this.data.name || ''
        },
        {
          type: 'email',
          name: 'email',
          label: 'Email',
          required: true,
          value: this.data.email || ''
        },
        {
          type: 'password',
          name: 'password',
          label: isNew ? 'Пароль' : 'Новый пароль',
          required: isNew,
          attributes: {
            autocomplete: isNew ? 'new-password' : 'new-password'
          }
        },
        {
          type: 'checkbox',
          name: 'admin',
          label: 'Администратор',
          value: this.data.admin || false
        },
        {
          type: 'checkbox',
          name: 'external',
          label: 'Внешний пользователь (LDAP)',
          value: this.data.external || false,
          attributes: {
            disabled: this.data.external ? 'disabled' : null
          }
        }
      ],
      onSubmit: (data) => this.handleSubmit(data),
      onCancel: () => this.onCancel(),
      submitText: this.userId ? 'Сохранить' : 'Создать'
    });

    if (this.userId) {
      form.setData(this.data);
    }
  }

  /**
   * Обработка сабмита
   */
  async handleSubmit(data) {
    try {
      // Удаляем пустой пароль при редактировании
      if (this.userId && !data.password) {
        delete data.password;
      }

      if (this.userId) {
        await api.put(`/users/${this.userId}`, data);
      } else {
        await api.post('/users', data);
      }
      this.onSave(data);
    } catch (error) {
      console.error('Error saving user:', error);
      throw error;
    }
  }
}

export default UserForm;
