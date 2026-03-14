/**
 * Key Form Component
 * Форма создания/редактирования ключа доступа
 */

import { Form } from './forms.js';
import api from '../api.js';
import { $ } from '../utils/dom.js';

export class KeyForm {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.projectId = options.projectId;
    this.keyId = options.keyId;
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
    if (this.keyId) {
      try {
        this.data = await api.getKey(this.projectId, this.keyId);
      } catch (error) {
        console.error('Error loading key:', error);
      }
    }
    
    this.render();
  }

  /**
   * Рендер формы
   */
  render() {
    const isLoginPassword = this.data.type === 'login_password' || !this.data.type;
    
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
          value: isLoginPassword ? 'login_password' : 'ssh',
          options: [
            { value: 'login_password', text: 'Логин/пароль' },
            { value: 'ssh', text: 'SSH ключ' }
          ]
        },
        {
          type: 'text',
          name: 'login_password_username',
          label: 'Имя пользователя',
          value: this.data.login_password?.username || this.data.username || '',
          attributes: {
            'data-show-when': 'type=login_password'
          }
        },
        {
          type: 'password',
          name: 'login_password_password',
          label: 'Пароль',
          value: this.data.login_password?.password || '',
          attributes: {
            'data-show-when': 'type=login_password',
            autocomplete: 'new-password'
          }
        },
        {
          type: 'textarea',
          name: 'ssh_key_private_key',
          label: 'Приватный ключ',
          rows: 8,
          value: this.data.ssh_key?.private_key || this.data.private_key || '',
          placeholder: '-----BEGIN RSA PRIVATE KEY-----\n...',
          attributes: {
            'data-show-when': 'type=ssh'
          }
        },
        {
          type: 'password',
          name: 'ssh_key_passphrase',
          label: 'Парольная фраза (опционально)',
          value: this.data.ssh_key?.passphrase || '',
          attributes: {
            'data-show-when': 'type=ssh',
            autocomplete: 'new-password'
          }
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
      submitText: this.keyId ? 'Сохранить' : 'Создать'
    });

    if (this.keyId) {
      form.setData(this.data);
    }

    // Обработка переключения типа
    const typeSelect = form.form?.querySelector('[name="type"]');
    if (typeSelect) {
      typeSelect.addEventListener('change', (e) => {
        this.render();
      });
    }
  }

  /**
   * Обработка сабмита
   */
  async handleSubmit(data) {
    try {
      // Преобразование данных в зависимости от типа
      const payload = {
        name: data.name,
        description: data.description,
        type: data.type
      };

      if (data.type === 'login_password') {
        payload.login_password = {
          username: data.login_password_username,
          password: data.login_password_password
        };
      } else if (data.type === 'ssh') {
        payload.ssh_key = {
          private_key: data.ssh_key_private_key,
          passphrase: data.ssh_key_passphrase || undefined
        };
      }

      if (this.keyId) {
        await api.updateKey(this.projectId, this.keyId, payload);
      } else {
        await api.createKey(this.projectId, payload);
      }
      this.onSave(payload);
    } catch (error) {
      console.error('Error saving key:', error);
      throw error;
    }
  }
}

export default KeyForm;
