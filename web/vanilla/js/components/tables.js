/**
 * DataTable Component
 * Компонент таблицы данных с сортировкой и пагинацией
 */

import { createElement, $, $$, delegate } from '../utils/dom.js';
import { merge, sortBy, filterBySearch } from '../utils/helpers.js';

export class DataTable {
  constructor(container, options = {}) {
    this.container = typeof container === 'string' ? $(container) : container;
    this.options = {
      headers: options.headers || [],
      data: options.data || [],
      sortable: options.sortable !== false,
      pagination: options.pagination !== false,
      itemsPerPage: options.itemsPerPage || 10,
      itemsPerPageOptions: options.itemsPerPageOptions || [10, 25, 50, 100],
      serverSide: options.serverSide || false,
      loading: options.loading || false,
      dense: options.dense || false,
      selectble: options.selectble || false,
      onRowClick: options.onRowClick || null,
      onRequest: options.onRequest || null, // Для server-side
      ...options
    };
    
    this.state = {
      page: 1,
      sortBy: null,
      sortDesc: false,
      search: '',
      selected: [],
      total: options.data?.length || 0,
      loading: this.options.loading
    };
    
    this.table = null;
    this.init();
  }

  /**
   * Инициализация
   */
  init() {
    this.render();
    this.setupEventListeners();
  }

  /**
   * Рендер таблицы
   */
  render() {
    const wrapper = createElement('div', {
      className: 'v-data-table-wrapper',
      style: { overflow: 'auto' }
    });

    this.table = createElement('table', {
      className: `v-data-table${this.options.dense ? ' v-data-table--dense' : ''}`
    });

    // Заголовок
    const thead = createElement('thead');
    const headerRow = createElement('tr');

    // Checkbox для выбора всех
    if (this.options.selectble) {
      const th = createElement('th', {
        style: { width: '48px' }
      });
      const checkbox = createElement('input', {
        attributes: { type: 'checkbox' },
        events: {
          change: (e) => this.toggleSelectAll(e.target.checked)
        }
      });
      th.appendChild(checkbox);
      headerRow.appendChild(th);
    }

    // Колонки
    this.options.headers.forEach(header => {
      const th = createElement('th', {
        className: header.class || ''
      });

      const content = createElement('span', {
        style: { display: 'flex', alignItems: 'center', gap: '4px' }
      });

      const text = createElement('span', {
        text: header.text
      });
      content.appendChild(text);

      // Иконка сортировки
      if (this.options.sortable && header.sortable !== false) {
        const sortIcon = createElement('span', {
          className: 'v-data-table__sort-icon',
          html: '<i class="v-icon mdi mdi-arrow-up"></i>',
          style: { opacity: '0', cursor: 'pointer' },
          events: {
            click: () => this.handleSort(header.value)
          }
        });
        content.appendChild(sortIcon);
        th.style.cursor = 'pointer';
        th.addEventListener('click', () => this.handleSort(header.value));
      }

      th.appendChild(content);
      headerRow.appendChild(th);
    });

    // Колонка действий
    if (this.options.actions) {
      headerRow.appendChild(createElement('th', {
        text: '',
        style: { width: '1%' }
      }));
    }

    thead.appendChild(headerRow);
    this.table.appendChild(thead);

    // Тело таблицы
    const tbody = createElement('tbody');
    this.table.appendChild(tbody);

    wrapper.appendChild(this.table);
    this.container.innerHTML = '';
    this.container.appendChild(wrapper);

    // Загрузка данных
    this.loadData();

    // Пагинация
    if (this.options.pagination) {
      this.renderPagination();
    }
  }

  /**
   * Загрузка данных
   */
  async loadData() {
    if (this.options.serverSide && this.options.onRequest) {
      this.setLoading(true);
      
      try {
        const response = await this.options.onRequest({
          page: this.state.page,
          itemsPerPage: this.options.itemsPerPage,
          sortBy: this.state.sortBy,
          sortDesc: this.state.sortDesc,
          search: this.state.search
        });
        
        this.state.data = response.data || [];
        this.state.total = response.total || 0;
      } catch (error) {
        console.error('Error loading data:', error);
        this.state.data = [];
      }
      
      this.setLoading(false);
    }
    
    this.renderBody();
  }

  /**
   * Рендер тела таблицы
   */
  renderBody() {
    const tbody = $('tbody', this.table);
    if (!tbody) return;

    tbody.innerHTML = '';

    // Loading state
    if (this.state.loading) {
      const tr = createElement('tr');
      const td = createElement('td', {
        attributes: { colspan: this.options.headers.length + (this.options.selectble ? 1 : 0) },
        style: { textAlign: 'center', padding: '48px' }
      });
      td.innerHTML = '<div class="v-progress-circular" style="width: 40px; height: 40px;"><svg viewBox="0 0 50 50"><circle class="v-progress-circular__underlay" cx="25" cy="25" r="20" stroke-width="4"></circle><circle class="v-progress-circular__overlay" cx="25" cy="25" r="20" stroke-width="4" stroke-dasharray="125.6" stroke-dashoffset="94.2"></circle></svg></div>';
      tr.appendChild(td);
      tbody.appendChild(tr);
      return;
    }

    // Нет данных
    if (this.state.data.length === 0) {
      const tr = createElement('tr');
      const td = createElement('td', {
        attributes: { colspan: this.options.headers.length + (this.options.selectble ? 1 : 0) },
        style: { textAlign: 'center', padding: '48px', color: 'rgba(0,0,0,0.6)' }
      });
      td.textContent = this.options.noDataText || 'Нет данных';
      tr.appendChild(td);
      tbody.appendChild(tr);
      return;
    }

    // Данные
    let data = this.state.data;

    // Локальная сортировка
    if (!this.options.serverSide && this.state.sortBy) {
      data = sortBy(data, this.state.sortBy, this.state.sortDesc ? 'desc' : 'asc');
    }

    // Локальная пагинация
    if (!this.options.serverSide) {
      const start = (this.state.page - 1) * this.options.itemsPerPage;
      const end = start + this.options.itemsPerPage;
      data = data.slice(start, end);
    }

    data.forEach((item, index) => {
      const tr = createElement('tr', {
        className: this.state.selected.includes(item.id) ? 'v-data-table__selected' : '',
        events: {
          click: (e) => {
            if (this.options.onRowClick && !e.target.closest('.v-btn')) {
              this.options.onRowClick(item, index);
            }
          }
        }
      });

      // Checkbox
      if (this.options.selectble) {
        const td = createElement('td');
        const checkbox = createElement('input', {
          attributes: { type: 'checkbox' },
          events: {
            change: (e) => this.toggleSelect(item.id, e.target.checked)
          }
        });
        checkbox.checked = this.state.selected.includes(item.id);
        td.appendChild(checkbox);
        tr.appendChild(td);
      }

      // Ячейки
      this.options.headers.forEach(header => {
        const td = createElement('td', {
          className: header.class || ''
        });

        const value = this.getCellValue(item, header.value);
        td.innerHTML = header.format ? header.format(value, item) : value;

        tr.appendChild(td);
      });

      // Действия
      if (this.options.actions) {
        const td = createElement('td');
        const actionsDiv = createElement('div', {
          style: { display: 'flex', gap: '4px', justifyContent: 'flex-end' }
        });

        this.options.actions.forEach(action => {
          const btn = createElement('button', {
            className: `v-btn v-btn--icon v-btn--text ${action.class || ''}`,
            html: action.icon ? `<i class="v-icon ${action.icon}"></i>` : '',
            attributes: { title: action.tooltip || action.text },
            events: {
              click: (e) => {
                e.stopPropagation();
                action.handler(item, index);
              }
            }
          });
          actionsDiv.appendChild(btn);
        });

        td.appendChild(actionsDiv);
        tr.appendChild(td);
      }

      tbody.appendChild(tr);
    });
  }

  /**
   * Получение значения ячейки
   */
  getCellValue(item, valuePath) {
    const keys = valuePath.split('.');
    let value = item;
    for (const key of keys) {
      value = value?.[key];
    }
    return value ?? '—';
  }

  /**
   * Рендер пагинации
   */
  renderPagination() {
    const existing = $('.v-pagination', this.container.parentNode);
    if (existing) existing.remove();

    const totalPages = Math.ceil(this.state.total / this.options.itemsPerPage);
    
    if (totalPages <= 1) return;

    const pagination = createElement('div', {
      className: 'v-pagination',
      style: { marginTop: '16px' }
    });

    // Кнопка назад
    const prevBtn = createElement('button', {
      className: 'v-btn v-btn--icon v-btn--text',
      html: '<i class="v-icon mdi mdi-chevron-left"></i>',
      events: {
        click: () => this.goToPage(this.state.page - 1)
      }
    });
    if (this.state.page === 1) prevBtn.style.opacity = '0.5';
    pagination.appendChild(prevBtn);

    // Страницы
    const startPage = Math.max(1, this.state.page - 2);
    const endPage = Math.min(totalPages, this.state.page + 2);

    for (let i = startPage; i <= endPage; i++) {
      const btn = createElement('button', {
        className: `v-pagination__item${i === this.state.page ? ' v-pagination__item--active' : ''}`,
        text: String(i),
        events: {
          click: () => this.goToPage(i)
        }
      });
      pagination.appendChild(btn);
    }

    // Кнопка вперёд
    const nextBtn = createElement('button', {
      className: 'v-btn v-btn--icon v-btn--text',
      html: '<i class="v-icon mdi mdi-chevron-right"></i>',
      events: {
        click: () => this.goToPage(this.state.page + 1)
      }
    });
    if (this.state.page === totalPages) nextBtn.style.opacity = '0.5';
    pagination.appendChild(nextBtn);

    this.container.parentNode.appendChild(pagination);
  }

  /**
   * Переход на страницу
   */
  goToPage(page) {
    const totalPages = Math.ceil(this.state.total / this.options.itemsPerPage);
    if (page < 1 || page > totalPages) return;
    
    this.state.page = page;
    this.loadData();
    this.renderPagination();
  }

  /**
   * Обработка сортировки
   */
  handleSort(column) {
    if (this.state.sortBy === column) {
      this.state.sortDesc = !this.state.sortDesc;
    } else {
      this.state.sortBy = column;
      this.state.sortDesc = false;
    }
    
    this.loadData();
  }

  /**
   * Выбор всех
   */
  toggleSelectAll(checked) {
    if (checked) {
      this.state.selected = this.state.data.map(item => item.id);
    } else {
      this.state.selected = [];
    }
    this.renderBody();
  }

  /**
   * Выбор элемента
   */
  toggleSelect(id, checked) {
    const index = this.state.selected.indexOf(id);
    if (checked && index === -1) {
      this.state.selected.push(id);
    } else if (!checked && index !== -1) {
      this.state.selected.splice(index, 1);
    }
    this.renderBody();
  }

  /**
   * Установка loading состояния
   */
  setLoading(loading) {
    this.state.loading = loading;
    if (loading) {
      this.table?.classList.add('v-data-table--loading');
    } else {
      this.table?.classList.remove('v-data-table--loading');
    }
  }

  /**
   * Обновление данных
   */
  setData(data) {
    this.state.data = data;
    this.state.total = data.length;
    this.state.page = 1;
    this.loadData();
    this.renderPagination();
  }

  /**
   * Получение выбранных
   */
  getSelected() {
    return this.state.selected;
  }

  /**
   * Настройка обработчиков
   */
  setupEventListeners() {
    // Поиск
    if (this.options.searchable) {
      const searchInput = createElement('input', {
        className: 'v-text-field',
        attributes: { 
          type: 'text', 
          placeholder: 'Поиск...' 
        },
        style: { marginBottom: '16px', width: '100%' },
        events: {
          input: (e) => {
            this.state.search = e.target.value;
            this.state.page = 1;
            this.loadData();
          }
        }
      });
      this.container.parentNode.insertBefore(searchInput, this.container);
    }
  }

  /**
   * Уничтожение
   */
  destroy() {
    this.container.innerHTML = '';
  }
}

export default DataTable;
