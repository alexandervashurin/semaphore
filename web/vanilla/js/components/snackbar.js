/**
 * Snackbar Component
 * Компонент уведомлений
 */

import { createElement, $ } from '../utils/dom.js';

export class Snackbar {
  constructor(options = {}) {
    this.position = options.position || 'bottom'; // bottom, top, left, right
    this.duration = options.duration || 3000;
    this.showClose = options.showClose !== false;
  }

  /**
   * Показ уведомления
   * @param {string} text - Текст
   * @param {Object} options - Опции
   */
  show(text, options = {}) {
    const snackbar = createElement('div', {
      className: `v-snackbar v-snackbar--${this.position}`
    });

    const textSpan = createElement('span', {
      id: 'snackbar-text',
      text: text
    });
    snackbar.appendChild(textSpan);

    // Кнопка действия
    if (options.action) {
      const actionBtn = createElement('button', {
        className: 'v-btn v-btn--text',
        text: options.action,
        events: {
          click: () => this.close(snackbar)
        }
      });
      snackbar.appendChild(actionBtn);
    }

    // Кнопка закрытия
    if (this.showClose) {
      const closeBtn = createElement('button', {
        className: 'v-btn v-btn--icon v-btn--text',
        html: '<i class="v-icon mdi mdi-close"></i>',
        events: {
          click: () => this.close(snackbar)
        }
      });
      snackbar.appendChild(closeBtn);
    }

    // Показ
    document.body.appendChild(snackbar);

    // Автозакрытие
    if (this.duration > 0) {
      setTimeout(() => this.close(snackbar), this.duration);
    }

    return snackbar;
  }

  /**
   * Закрытие уведомления
   * @param {HTMLElement} snackbar - Элемент
   */
  close(snackbar) {
    if (snackbar) {
      snackbar.style.opacity = '0';
      snackbar.style.transition = 'opacity 0.3s';
      setTimeout(() => {
        if (snackbar.parentNode) {
          snackbar.remove();
        }
      }, 300);
    }
  }

  /**
   * Успешное уведомление
   * @param {string} text - Текст
   */
  success(text) {
    const el = this.show(text);
    el.style.borderLeft = '4px solid #4caf50';
    return el;
  }

  /**
   * Ошибка
   * @param {string} text - Текст
   */
  error(text) {
    const el = this.show(text, { duration: 5000 });
    el.style.borderLeft = '4px solid #f44336';
    return el;
  }

  /**
   * Предупреждение
   * @param {string} text - Текст
   */
  warning(text) {
    const el = this.show(text);
    el.style.borderLeft = '4px solid #ff9800';
    return el;
  }

  /**
   * Информация
   * @param {string} text - Текст
   */
  info(text) {
    const el = this.show(text);
    el.style.borderLeft = '4px solid #2196f3';
    return el;
  }

  /**
   * Загрузка
   * @param {string} text - Текст
   * @returns {Object} - { close, update }
   */
  loading(text) {
    const el = this.show(text, { duration: 0 });
    el.innerHTML = `
      <div class="v-progress-circular" style="width: 24px; height: 24px; margin-right: 12px;">
        <svg viewBox="0 0 50 50">
          <circle class="v-progress-circular__underlay" cx="25" cy="25" r="20" stroke-width="4"></circle>
          <circle class="v-progress-circular__overlay" cx="25" cy="25" r="20" stroke-width="4" stroke-dasharray="125.6" stroke-dashoffset="31.4"></circle>
        </svg>
      </div>
      <span>${text}</span>
    `;
    el.style.alignItems = 'center';
    
    return {
      close: () => this.close(el),
      update: (newText) => {
        const span = el.querySelector('span');
        if (span) span.textContent = newText;
      }
    };
  }
}

// Глобальный экземпляр
const snackbar = new Snackbar();

// Функции для удобного использования
export function showSnackbar(text, options) {
  return snackbar.show(text, options);
}

export function showSuccess(text) {
  return snackbar.success(text);
}

export function showError(text) {
  return snackbar.error(text);
}

export function showWarning(text) {
  return snackbar.warning(text);
}

export function showInfo(text) {
  return snackbar.info(text);
}

export function showLoading(text) {
  return snackbar.loading(text);
}

export default snackbar;
