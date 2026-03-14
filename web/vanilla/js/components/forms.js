/**
 * Form Component
 * Компонент для работы с формами
 */

import { createElement, $, $$ } from './dom.js';

export class Form {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.options = {
      fields: options.fields || [],
      onSubmit: options.onSubmit || null,
      onCancel: options.onCancel || null,
      submitText: options.submitText || 'Сохранить',
      cancelText: options.cancelText || 'Отмена',
      showCancel: options.showCancel !== false,
      ...options
    };
    
    this.form = null;
    this.data = {};
    this.errors = {};
    this.init();
  }

  /**
   * Инициализация
   */
  init() {
    this.render();
    this.setupValidation();
  }

  /**
   * Рендер формы
   */
  render() {
    this.form = createElement('form', {
      events: {
        submit: (e) => this.handleSubmit(e)
      }
    });

    this.options.fields.forEach(field => {
      const fieldEl = this.renderField(field);
      if (fieldEl) {
        this.form.appendChild(fieldEl);
      }
    });

    // Кнопки
    const actions = createElement('div', {
      className: 'v-card__actions',
      style: { display: 'flex', gap: '8px', justifyContent: 'flex-end', marginTop: '16px' }
    });

    if (this.options.showCancel) {
      const cancelBtn = createElement('button', {
        type: 'button',
        className: 'v-btn v-btn--text',
        text: this.options.cancelText,
        events: {
          click: (e) => {
            e.preventDefault();
            if (this.options.onCancel) {
              this.options.onCancel();
            }
          }
        }
      });
      actions.appendChild(cancelBtn);
    }

    const submitBtn = createElement('button', {
      type: 'submit',
      className: 'v-btn v-btn--contained v-btn--primary',
      text: this.options.submitText
    });
    actions.appendChild(submitBtn);

    this.form.appendChild(actions);
    this.container.innerHTML = '';
    this.container.appendChild(this.form);
  }

  /**
   * Рендер поля
   */
  renderField(field) {
    const wrapper = createElement('div', {
      className: `v-text-field${this.errors[field.name] ? ' v-text-field--error' : ''}`
    });

    let input;

    switch (field.type) {
      case 'textarea':
        input = createElement('textarea', {
          id: field.name,
          name: field.name,
          attributes: {
            placeholder: ' ',
            required: field.required ? 'required' : null
          },
          events: {
            input: (e) => this.updateField(field.name, e.target.value)
          }
        });
        if (field.rows) {
          input.rows = field.rows;
        }
        break;

      case 'select':
        wrapper.classList.remove('v-text-field');
        wrapper.classList.add('v-select');
        
        const selectWrapper = createElement('div', {
          className: 'v-select__selection'
        });
        
        input = createElement('select', {
          id: field.name,
          name: field.name,
          attributes: {
            required: field.required ? 'required' : null
          },
          events: {
            change: (e) => this.updateField(field.name, e.target.value)
          }
        });

        // Default option
        if (field.placeholder) {
          const defaultOption = createElement('option', {
            value: '',
            text: field.placeholder
          });
          input.appendChild(defaultOption);
        }

        // Options
        if (field.options) {
          field.options.forEach(opt => {
            const option = createElement('option', {
              value: opt.value,
              text: opt.text
            });
            input.appendChild(option);
          });
        }

        selectWrapper.appendChild(input);
        wrapper.appendChild(selectWrapper);

        if (field.label) {
          const label = createElement('label', {
            attributes: { for: field.name },
            text: field.label
          });
          wrapper.insertBefore(label, selectWrapper);
        }

        return wrapper;

      case 'checkbox':
        wrapper.classList.remove('v-text-field');
        wrapper.classList.add('v-checkbox');
        
        const checkbox = createElement('input', {
          type: 'checkbox',
          id: field.name,
          name: field.name,
          events: {
            change: (e) => this.updateField(field.name, e.target.checked)
          }
        });

        const ripple = createElement('span', {
          className: 'v-checkbox__ripple'
        });

        const label = createElement('span', {
          className: 'v-label',
          text: field.label
        });

        wrapper.appendChild(checkbox);
        wrapper.appendChild(ripple);
        wrapper.appendChild(label);

        return wrapper;

      case 'password':
      default:
        input = createElement('input', {
          type: field.type || 'text',
          id: field.name,
          name: field.name,
          attributes: {
            placeholder: ' ',
            required: field.required ? 'required' : null,
            autocomplete: field.autocomplete || null
          },
          events: {
            input: (e) => this.updateField(field.name, e.target.value)
          }
        });

        if (field.value !== undefined) {
          input.value = field.value;
          this.data[field.name] = field.value;
        }
    }

    if (field.label) {
      const label = createElement('label', {
        attributes: { for: field.name },
        text: field.label
      });
      wrapper.appendChild(label);
    }

    wrapper.appendChild(input);

    // Сообщение об ошибке
    if (this.errors[field.name]) {
      const error = createElement('div', {
        className: 'v-text-field__error-messages',
        style: { display: 'block' },
        text: this.errors[field.name]
      });
      wrapper.appendChild(error);
    }

    return wrapper;
  }

  /**
   * Обновление поля
   */
  updateField(name, value) {
    this.data[name] = value;
    this.validateField(name, value);
  }

  /**
   * Настройка валидации
   */
  setupValidation() {
    // Валидация при сабмите
  }

  /**
   * Валидация поля
   */
  validateField(name, value) {
    const field = this.options.fields.find(f => f.name === name);
    if (!field) return;

    let error = null;

    // Required
    if (field.required && (value === '' || value === null || value === undefined)) {
      error = 'Это поле обязательно для заполнения';
    }

    // Email
    if (field.type === 'email' && value) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(value)) {
        error = 'Неверный формат email';
      }
    }

    // Min length
    if (field.minLength && value && value.length < field.minLength) {
      error = `Минимум ${field.minLength} символов`;
    }

    // Max length
    if (field.maxLength && value && value.length > field.maxLength) {
      error = `Максимум ${field.maxLength} символов`;
    }

    // Pattern
    if (field.pattern && value && !new RegExp(field.pattern).test(value)) {
      error = field.patternMessage || 'Неверный формат';
    }

    this.errors[name] = error;
    return error;
  }

  /**
   * Валидация всей формы
   */
  validate() {
    let valid = true;
    
    this.options.fields.forEach(field => {
      const value = this.data[field.name];
      const error = this.validateField(field.name, value);
      if (error) {
        valid = false;
      }
    });

    // Перерисовка с ошибками
    this.render();
    
    return valid;
  }

  /**
   * Обработка сабмита
   */
  handleSubmit(e) {
    e.preventDefault();
    
    if (!this.validate()) {
      return;
    }

    if (this.options.onSubmit) {
      this.options.onSubmit(this.data);
    }
  }

  /**
   * Получение данных
   */
  getData() {
    return { ...this.data };
  }

  /**
   * Установка данных
   */
  setData(data) {
    this.data = { ...data };
    
    // Обновление полей
    Object.entries(data).forEach(([name, value]) => {
      const input = this.form?.querySelector(`[name="${name}"]`);
      if (input) {
        if (input.type === 'checkbox') {
          input.checked = value;
        } else {
          input.value = value;
        }
      }
    });
  }

  /**
   * Сброс формы
   */
  reset() {
    this.data = {};
    this.errors = {};
    this.render();
  }

  /**
   * Установка ошибки
   */
  setError(field, message) {
    this.errors[field] = message;
    this.render();
  }

  /**
   * Очистка ошибок
   */
  clearErrors() {
    this.errors = {};
    this.render();
  }
}

export default Form;
