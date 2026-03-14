/**
 * Dialog Component
 * Компонент диалоговых окон
 */

import { createElement, $, remove } from '../utils/dom.js';

export class Dialog {
  constructor(options = {}) {
    this.title = options.title || '';
    this.content = options.content || '';
    this.subtitle = options.subtitle || '';
    this.persistent = options.persistent || false;
    this.maxWidth = options.maxWidth || 'md'; // sm, md, lg, xl, fullscreen
    this.hideNoButton = options.hideNoButton || false;
    this.yesButtonText = options.yesButtonText || 'OK';
    this.noButtonText = options.noButtonText || 'Отмена';
    this.onConfirm = options.onConfirm || (() => {});
    this.onCancel = options.onCancel || (() => {});
    
    this.dialog = null;
  }

  /**
   * Показ диалога
   */
  show() {
    // Создаём overlay
    this.dialog = createElement('div', {
      className: 'v-dialog__scrim',
      events: {
        click: (e) => {
          if (e.target === this.dialog && !this.persistent) {
            this.close();
            this.onCancel();
          }
        }
      }
    });

    // Создаём карточку
    const card = createElement('div', {
      className: `v-dialog v-dialog--${this.maxWidth}`
    });

    // Заголовок
    const titleDiv = createElement('div', {
      className: 'v-card__title'
    });

    const titleText = createElement('span', {
      className: 'v-card__title-text',
      text: this.title
    });

    titleDiv.appendChild(titleText);

    // Кнопка закрытия (если не persistent)
    if (!this.persistent) {
      const closeBtn = createElement('button', {
        className: 'v-btn v-btn--icon v-btn--text',
        html: '<i class="v-icon mdi mdi-close"></i>',
        events: {
          click: () => {
            this.close();
            this.onCancel();
          }
        }
      });
      titleDiv.appendChild(closeBtn);
    }

    card.appendChild(titleDiv);

    // Подзаголовок (если есть)
    if (this.subtitle) {
      const subtitle = createElement('div', {
        className: 'v-card__subtitle',
        text: this.subtitle
      });
      card.appendChild(subtitle);
    }

    // Текст
    const textDiv = createElement('div', {
      className: 'v-card__text',
      html: this.content
    });
    card.appendChild(textDiv);

    // Действия
    const actions = createElement('div', {
      className: 'v-card__actions'
    });

    // Кнопка Отмена
    if (!this.hideNoButton) {
      const cancelBtn = createElement('button', {
        className: 'v-btn v-btn--text',
        text: this.noButtonText,
        events: {
          click: () => {
            this.close();
            this.onCancel();
          }
        }
      });
      actions.appendChild(cancelBtn);
    }

    // Кнопка OK
    const confirmBtn = createElement('button', {
      className: 'v-btn v-btn--contained v-btn--primary',
      text: this.yesButtonText,
      events: {
        click: () => {
          this.close();
          this.onConfirm();
        }
      }
    });
    actions.appendChild(confirmBtn);

    card.appendChild(actions);
    this.dialog.appendChild(card);
    document.body.appendChild(this.dialog);

    // Фокус на кнопке OK
    setTimeout(() => confirmBtn.focus(), 100);
  }

  /**
   * Закрытие диалога
   */
  close() {
    if (this.dialog) {
      remove(this.dialog);
      this.dialog = null;
    }
  }

  /**
   * Обновление контента
   * @param {Object} options - Опции
   */
  update(options = {}) {
    if (!this.dialog) return;

    if (options.title) {
      $('.v-card__title-text', this.dialog).textContent = options.title;
    }

    if (options.content) {
      $('.v-card__text', this.dialog).innerHTML = options.content;
    }
  }
}

/**
 * Alert Dialog
 * @param {Object} options - Опции
 */
export function alert(options) {
  const dialog = new Dialog({
    ...options,
    hideNoButton: true
  });
  dialog.show();
  return dialog;
}

/**
 * Confirm Dialog
 * @param {Object} options - Опции
 * @returns {Promise}
 */
export function confirm(options = {}) {
  return new Promise((resolve) => {
    const dialog = new Dialog({
      ...options,
      onConfirm: () => resolve(true),
      onCancel: () => resolve(false)
    });
    dialog.show();
  });
}

/**
 * Prompt Dialog
 * @param {Object} options - Опции
 * @returns {Promise}
 */
export function prompt(options = {}) {
  return new Promise((resolve) => {
    const defaultValue = options.defaultValue || '';
    const label = options.label || 'Введите значение';
    
    const dialog = new Dialog({
      ...options,
      content: `
        <div class="v-text-field">
          <input 
            type="text" 
            id="prompt-input" 
            value="${defaultValue}"
            placeholder="${label}"
            style="width: 100%; padding: 8px; border: 1px solid rgba(0,0,0,0.38); border-radius: 4px;"
          />
        </div>
      `,
      onConfirm: () => {
        const input = $('#prompt-input');
        resolve(input ? input.value : null);
      },
      onCancel: () => resolve(null)
    });
    
    dialog.show();
    
    // Фокус на инпуте
    setTimeout(() => {
      const input = $('#prompt-input');
      if (input) {
        input.focus();
        input.select();
      }
    }, 100);
  });
}

export default Dialog;
