/**
 * Velum - Accessibility & Mobile Support
 * WCAG 2.1 AA, Mobile Responsive, i18n, Keyboard Shortcuts
 */

// ==================== Mobile Menu ====================

function initMobileMenu() {
    const menuToggle = $('.mobile-menu-toggle');
    const sidebar = $('.sidebar');
    const overlay = $('.mobile-sidebar-overlay');

    if (!menuToggle || !sidebar) return;

    // Toggle menu
    menuToggle.addEventListener('click', () => {
        sidebar.classList.toggle('mobile-open');
        if (overlay) {
            overlay.classList.toggle('active');
        }
        menuToggle.setAttribute('aria-expanded', sidebar.classList.contains('mobile-open'));
    });

    // Close on overlay click
    if (overlay) {
        overlay.addEventListener('click', () => {
            sidebar.classList.remove('mobile-open');
            overlay.classList.remove('active');
            menuToggle.setAttribute('aria-expanded', 'false');
        });
    }

    // Close on escape key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && sidebar.classList.contains('mobile-open')) {
            sidebar.classList.remove('mobile-open');
            if (overlay) overlay.classList.remove('active');
            menuToggle.setAttribute('aria-expanded', 'false');
        }
    });
}

// ==================== i18n Support ====================

const translations = {
    ru: {
        // Common
        'loading': 'Загрузка...',
        'error': 'Ошибка',
        'success': 'Успешно',
        'cancel': 'Отмена',
        'save': 'Сохранить',
        'delete': 'Удалить',
        'edit': 'Редактировать',
        'create': 'Создать',
        'search': 'Поиск',
        'filter': 'Фильтр',
        'refresh': 'Обновить',
        'close': 'Закрыть',
        'back': 'Назад',
        'next': 'Далее',
        'previous': 'Назад',
        'yes': 'Да',
        'no': 'Нет',
        'ok': 'OK',
        
        // Navigation
        'nav.dashboard': 'Панель управления',
        'nav.projects': 'Проекты',
        'nav.templates': 'Шаблоны',
        'nav.tasks': 'Задачи',
        'nav.inventory': 'Инвентарь',
        'nav.keys': 'Ключи доступа',
        'nav.users': 'Пользователи',
        'nav.settings': 'Настройки',
        'nav.kubernetes': 'Kubernetes',
        'nav.k8s.pods': 'Поды',
        'nav.k8s.deployments': 'Деплои',
        'nav.k8s.services': 'Сервисы',
        'nav.k8s.configmaps': 'ConfigMaps',
        'nav.k8s.secrets': 'Secrets',
        'nav.k8s.ingress': 'Ingress',
        'nav.k8s.storage': 'Хранилище',
        'nav.k8s.rbac': 'RBAC',
        'nav.k8s.helm': 'Helm',
        'nav.k8s.jobs': 'Jobs',
        'nav.k8s.cronjobs': 'CronJobs',
        'nav.k8s.events': 'События',
        'nav.k8s.metrics': 'Метрики',
        'nav.k8s.topology': 'Топология',
        
        // Status
        'status.running': 'Выполняется',
        'status.pending': 'Ожидание',
        'status.success': 'Успешно',
        'status.failed': 'Ошибка',
        'status.stopped': 'Остановлено',
        
        // Kubernetes
        'k8s.namespace': 'Namespace',
        'k8s.pod': 'Под',
        'k8s.deployment': 'Деплой',
        'k8s.service': 'Сервис',
        'k8s.configmap': 'ConfigMap',
        'k8s.secret': 'Secret',
        'k8s.events': 'События',
        'k8s.logs': 'Логи',
        'k8s.exec': 'Терминал',
        'k8s.restart': 'Перезапустить',
        'k8s.delete': 'Удалить',
        'k8s.apply': 'Применить',
        'k8s.dry-run': 'Dry Run',
        
        // Forms
        'form.name': 'Имя',
        'form.description': 'Описание',
        'form.status': 'Статус',
        'form.created': 'Создано',
        'form.updated': 'Обновлено',
        'form.actions': 'Действия',
        
        // Messages
        'msg.confirm.delete': 'Вы уверены, что хотите удалить этот элемент?',
        'msg.saved': 'Данные успешно сохранены',
        'msg.deleted': 'Элемент удалён',
        'msg.error.save': 'Ошибка при сохранении',
        'msg.error.load': 'Ошибка при загрузке данных',
        'msg.error.network': 'Ошибка сети. Проверьте подключение.',
        
        // Accessibility
        'a11y.skip-to-content': 'Перейти к содержимому',
        'a11y.main-menu': 'Главное меню',
        'a11y.close-modal': 'Закрыть модальное окно',
        'a11y.open-menu': 'Открыть меню',
        'a11y.close-menu': 'Закрыть меню',
        'a11y.loading': 'Загрузка',
        'a11y.search': 'Поиск',
        'a11y.notifications': 'Уведомления',
        'a11y.keyboard-shortcuts': 'Горячие клавиши'
    },
    en: {
        // Common
        'loading': 'Loading...',
        'error': 'Error',
        'success': 'Success',
        'cancel': 'Cancel',
        'save': 'Save',
        'delete': 'Delete',
        'edit': 'Edit',
        'create': 'Create',
        'search': 'Search',
        'filter': 'Filter',
        'refresh': 'Refresh',
        'close': 'Close',
        'back': 'Back',
        'next': 'Next',
        'previous': 'Previous',
        'yes': 'Yes',
        'no': 'No',
        'ok': 'OK',
        
        // Navigation
        'nav.dashboard': 'Dashboard',
        'nav.projects': 'Projects',
        'nav.templates': 'Templates',
        'nav.tasks': 'Tasks',
        'nav.inventory': 'Inventory',
        'nav.keys': 'Access Keys',
        'nav.users': 'Users',
        'nav.settings': 'Settings',
        'nav.kubernetes': 'Kubernetes',
        'nav.k8s.pods': 'Pods',
        'nav.k8s.deployments': 'Deployments',
        'nav.k8s.services': 'Services',
        'nav.k8s.configmaps': 'ConfigMaps',
        'nav.k8s.secrets': 'Secrets',
        'nav.k8s.ingress': 'Ingress',
        'nav.k8s.storage': 'Storage',
        'nav.k8s.rbac': 'RBAC',
        'nav.k8s.helm': 'Helm',
        'nav.k8s.jobs': 'Jobs',
        'nav.k8s.cronjobs': 'CronJobs',
        'nav.k8s.events': 'Events',
        'nav.k8s.metrics': 'Metrics',
        'nav.k8s.topology': 'Topology',
        
        // Status
        'status.running': 'Running',
        'status.pending': 'Pending',
        'status.success': 'Success',
        'status.failed': 'Failed',
        'status.stopped': 'Stopped',
        
        // Kubernetes
        'k8s.namespace': 'Namespace',
        'k8s.pod': 'Pod',
        'k8s.deployment': 'Deployment',
        'k8s.service': 'Service',
        'k8s.configmap': 'ConfigMap',
        'k8s.secret': 'Secret',
        'k8s.events': 'Events',
        'k8s.logs': 'Logs',
        'k8s.exec': 'Terminal',
        'k8s.restart': 'Restart',
        'k8s.delete': 'Delete',
        'k8s.apply': 'Apply',
        'k8s.dry-run': 'Dry Run',
        
        // Forms
        'form.name': 'Name',
        'form.description': 'Description',
        'form.status': 'Status',
        'form.created': 'Created',
        'form.updated': 'Updated',
        'form.actions': 'Actions',
        
        // Messages
        'msg.confirm.delete': 'Are you sure you want to delete this item?',
        'msg.saved': 'Data saved successfully',
        'msg.deleted': 'Item deleted',
        'msg.error.save': 'Error saving data',
        'msg.error.load': 'Error loading data',
        'msg.error.network': 'Network error. Please check your connection.',
        
        // Accessibility
        'a11y.skip-to-content': 'Skip to content',
        'a11y.main-menu': 'Main menu',
        'a11y.close-modal': 'Close modal',
        'a11y.open-menu': 'Open menu',
        'a11y.close-menu': 'Close menu',
        'a11y.loading': 'Loading',
        'a11y.search': 'Search',
        'a11y.notifications': 'Notifications',
        'a11y.keyboard-shortcuts': 'Keyboard shortcuts'
    },
    zh: {
        // Common
        'loading': '加载中...',
        'error': '错误',
        'success': '成功',
        'cancel': '取消',
        'save': '保存',
        'delete': '删除',
        'edit': '编辑',
        'create': '创建',
        'search': '搜索',
        'filter': '筛选',
        'refresh': '刷新',
        'close': '关闭',
        'back': '返回',
        'next': '下一步',
        'previous': '上一步',
        'yes': '是',
        'no': '否',
        'ok': '确定',
        
        // Navigation
        'nav.dashboard': '仪表板',
        'nav.projects': '项目',
        'nav.templates': '模板',
        'nav.tasks': '任务',
        'nav.inventory': '清单',
        'nav.keys': '访问密钥',
        'nav.users': '用户',
        'nav.settings': '设置',
        'nav.kubernetes': 'Kubernetes',
        'nav.k8s.pods': 'Pod',
        'nav.k8s.deployments': '部署',
        'nav.k8s.services': '服务',
        'nav.k8s.configmaps': 'ConfigMap',
        'nav.k8s.secrets': 'Secret',
        'nav.k8s.ingress': 'Ingress',
        'nav.k8s.storage': '存储',
        'nav.k8s.rbac': 'RBAC',
        'nav.k8s.helm': 'Helm',
        'nav.k8s.jobs': 'Jobs',
        'nav.k8s.cronjobs': 'CronJobs',
        'nav.k8s.events': '事件',
        'nav.k8s.metrics': '指标',
        'nav.k8s.topology': '拓扑',
        
        // Status
        'status.running': '运行中',
        'status.pending': '等待中',
        'status.success': '成功',
        'status.failed': '失败',
        'status.stopped': '已停止',
        
        // Kubernetes
        'k8s.namespace': '命名空间',
        'k8s.pod': 'Pod',
        'k8s.deployment': '部署',
        'k8s.service': '服务',
        'k8s.configmap': '配置映射',
        'k8s.secret': '密钥',
        'k8s.events': '事件',
        'k8s.logs': '日志',
        'k8s.exec': '终端',
        'k8s.restart': '重启',
        'k8s.delete': '删除',
        'k8s.apply': '应用',
        'k8s.dry-run': '模拟运行',
        
        // Forms
        'form.name': '名称',
        'form.description': '描述',
        'form.status': '状态',
        'form.created': '创建时间',
        'form.updated': '更新时间',
        'form.actions': '操作',
        
        // Messages
        'msg.confirm.delete': '您确定要删除此项吗？',
        'msg.saved': '数据保存成功',
        'msg.deleted': '项目已删除',
        'msg.error.save': '保存数据时出错',
        'msg.error.load': '加载数据时出错',
        'msg.error.network': '网络错误，请检查连接。',
        
        // Accessibility
        'a11y.skip-to-content': '跳转到内容',
        'a11y.main-menu': '主菜单',
        'a11y.close-modal': '关闭对话框',
        'a11y.open-menu': '打开菜单',
        'a11y.close-menu': '关闭菜单',
        'a11y.loading': '加载中',
        'a11y.search': '搜索',
        'a11y.notifications': '通知',
        'a11y.keyboard-shortcuts': '键盘快捷键'
    }
};

let currentLang = localStorage.getItem(LANG_KEY) || 'ru';

/**
 * Get translation
 * @param {string} key - Translation key
 * @param {string?} lang - Optional language override
 * @returns {string} Translated string or key if not found
 */
function t(key, lang = null) {
    const language = lang || currentLang;
    const langData = translations[language] || translations.ru;
    return langData[key] || translations.ru[key] || key;
}

/**
 * Set current language
 * @param {string} lang - Language code (ru, en, zh)
 */
function setLanguage(lang) {
    if (!translations[lang]) return;
    currentLang = lang;
    localStorage.setItem(LANG_KEY, lang);
    document.documentElement.lang = lang;
    
    // Update all elements with data-i18n attribute
    $$('[data-i18n]').forEach(el => {
        const key = el.getAttribute('data-i18n');
        el.textContent = t(key);
    });
    
    // Update placeholder attributes
    $$('[data-i18n-placeholder]').forEach(el => {
        const key = el.getAttribute('data-i18n-placeholder');
        el.placeholder = t(key);
    });
    
    // Update title attributes
    $$('[data-i18n-title]').forEach(el => {
        const key = el.getAttribute('data-i18n-title');
        el.title = t(key);
    });
    
    // Dispatch event for custom handlers
    window.dispatchEvent(new CustomEvent('languageChanged', { detail: { lang } }));
}

/**
 * Initialize language switcher
 * @param {string} containerSelector - Container element selector
 */
function initLanguageSwitcher(containerSelector) {
    const container = $(containerSelector);
    if (!container) return;
    
    container.innerHTML = `
        <div class="lang-switcher" role="group" aria-label="Language selection">
            <button class="lang-btn ${currentLang === 'ru' ? 'active' : ''}" 
                    data-lang="ru" aria-pressed="${currentLang === 'ru'}">RU</button>
            <button class="lang-btn ${currentLang === 'en' ? 'active' : ''}" 
                    data-lang="en" aria-pressed="${currentLang === 'en'}">EN</button>
            <button class="lang-btn ${currentLang === 'zh' ? 'active' : ''}" 
                    data-lang="zh" aria-pressed="${currentLang === 'zh'}">中文</button>
        </div>
    `;
    
    $$('.lang-btn', container).forEach(btn => {
        btn.addEventListener('click', () => {
            const lang = btn.getAttribute('data-lang');
            setLanguage(lang);
            
            // Update active state
            $$('.lang-btn', container).forEach(b => {
                b.classList.remove('active');
                b.setAttribute('aria-pressed', 'false');
            });
            btn.classList.add('active');
            btn.setAttribute('aria-pressed', 'true');
        });
    });
}

// ==================== Keyboard Shortcuts ====================

const keyboardShortcuts = {
    // Global shortcuts
    '?': () => showShortcutHelp(),
    'Escape': () => closeModals(),
    
    // Navigation
    'g d': () => navigateTo('/dashboard'),
    'g p': () => navigateTo('/projects'),
    'g t': () => navigateTo('/templates'),
    'g k': () => navigateTo('/kubernetes'),
    
    // Actions
    'n': () => triggerNewAction(),
    'r': () => refreshCurrentPage(),
    '/': () => focusSearch(),
    
    // Kubernetes specific
    'k p': () => navigateTo('/kubernetes/pods'),
    'k d': () => navigateTo('/kubernetes/deployments'),
    'k e': () => navigateTo('/kubernetes/events'),
};

let shortcutBuffer = '';
let shortcutTimer = null;

/**
 * Initialize keyboard shortcuts
 */
function initKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        // Ignore shortcuts in input fields
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA' || e.target.isContentEditable) {
            return;
        }
        
        // Ignore if modifier keys are pressed (for browser shortcuts)
        if (e.ctrlKey || e.altKey || e.metaKey) {
            return;
        }
        
        const key = e.key.toLowerCase();
        
        // Handle single key shortcuts
        if (keyboardShortcuts[key]) {
            e.preventDefault();
            keyboardShortcuts[key]();
            return;
        }
        
        // Handle multi-key sequences (e.g., "g d")
        if (key === 'g') {
            shortcutBuffer = 'g';
            clearTimeout(shortcutTimer);
            shortcutTimer = setTimeout(() => {
                shortcutBuffer = '';
            }, 800);
            return;
        }
        
        if (shortcutBuffer === 'g' && keyboardShortcuts[`g ${key}`]) {
            e.preventDefault();
            keyboardShortcuts[`g ${key}`]();
            shortcutBuffer = '';
            clearTimeout(shortcutTimer);
        }
    });
}

/**
 * Show keyboard shortcuts help modal
 */
function showShortcutHelp() {
    const existingOverlay = $('.shortcut-help-overlay');
    if (existingOverlay) return; // Already open
    
    const overlay = document.createElement('div');
    overlay.className = 'shortcut-help-overlay';
    overlay.setAttribute('role', 'dialog');
    overlay.setAttribute('aria-modal', 'true');
    overlay.setAttribute('aria-labelledby', 'shortcut-help-title');
    
    const modal = document.createElement('div');
    modal.className = 'shortcut-help-modal';
    modal.innerHTML = `
        <h2 id="shortcut-help-title" class="shortcut-help-title">${t('a11y.keyboard-shortcuts')}</h2>
        <div class="shortcut-help-content">
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">${t('a11y.search')}</span>
                <span class="kbd-shortcut"><kbd>/</kbd></span>
            </div>
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">Dashboard</span>
                <span class="kbd-shortcut"><kbd>g</kbd> <kbd>d</kbd></span>
            </div>
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">Projects</span>
                <span class="kbd-shortcut"><kbd>g</kbd> <kbd>p</kbd></span>
            </div>
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">Templates</span>
                <span class="kbd-shortcut"><kbd>g</kbd> <kbd>t</kbd></span>
            </div>
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">Kubernetes</span>
                <span class="kbd-shortcut"><kbd>g</kbd> <kbd>k</kbd></span>
            </div>
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">New action</span>
                <span class="kbd-shortcut"><kbd>n</kbd></span>
            </div>
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">${t('refresh')}</span>
                <span class="kbd-shortcut"><kbd>r</kbd></span>
            </div>
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">${t('close')}</span>
                <span class="kbd-shortcut"><kbd>Esc</kbd></span>
            </div>
            <div class="shortcut-help-item">
                <span class="shortcut-help-desc">${t('a11y.keyboard-shortcuts')}</span>
                <span class="kbd-shortcut"><kbd>?</kbd></span>
            </div>
        </div>
    `;
    
    overlay.appendChild(modal);
    document.body.appendChild(overlay);
    
    // Close on click outside or escape
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) {
            overlay.remove();
        }
    });
    
    // Focus trap
    modal.focus();
}

/**
 * Close all open modals
 */
function closeModals() {
    $$('.modal-overlay, .log-modal-overlay, .exec-modal-overlay, .shortcut-help-overlay').forEach(overlay => {
        overlay.remove();
    });
}

/**
 * Navigate to a page
 * @param {string} path - URL path
 */
function navigateTo(path) {
    window.location.href = path;
}

/**
 * Trigger new action (context-dependent)
 */
function triggerNewAction() {
    const newBtn = $('.btn-new, .create-btn, [data-action="new"]');
    if (newBtn) {
        newBtn.click();
    }
}

/**
 * Refresh current page
 */
function refreshCurrentPage() {
    window.location.reload();
}

/**
 * Focus search input
 */
function focusSearch() {
    const searchInput = $('.search-box input, input[type="search"], input[placeholder*="search" i]');
    if (searchInput) {
        searchInput.focus();
    }
}

// ==================== Accessibility Enhancements ====================

/**
 * Add skip link to page
 */
function addSkipLink() {
    const existingSkipLink = $('.skip-link');
    if (existingSkipLink) return;
    
    const skipLink = document.createElement('a');
    skipLink.href = '#main-content';
    skipLink.className = 'skip-link';
    skipLink.textContent = t('a11y.skip-to-content');
    skipLink.setAttribute('data-i18n', 'a11y.skip-to-content');
    document.body.insertBefore(skipLink, document.body.firstChild);
}

/**
 * Add ARIA landmarks to page
 */
function addAriaLandmarks() {
    // Add main content ID if not present
    const mainContent = $('main, [role="main"], .content-area');
    if (mainContent && !mainContent.id) {
        mainContent.id = 'main-content';
    }
    
    // Add role to sidebar if not present
    const sidebar = $('.sidebar');
    if (sidebar && !sidebar.getAttribute('role')) {
        sidebar.setAttribute('role', 'navigation');
        sidebar.setAttribute('aria-label', t('a11y.main-menu'));
    }
}

/**
 * Announce message to screen readers
 * @param {string} message - Message to announce
 * @param {string} priority - 'polite' or 'assertive'
 */
function announce(message, priority = 'polite') {
    let announcer = $('#aria-announcer');
    
    if (!announcer) {
        announcer = document.createElement('div');
        announcer.id = 'aria-announcer';
        announcer.setAttribute('aria-live', priority);
        announcer.setAttribute('aria-atomic', 'true');
        announcer.className = 'sr-only';
        document.body.appendChild(announcer);
    }
    
    announcer.textContent = message;
    
    // Clear after announcement
    setTimeout(() => {
        announcer.textContent = '';
    }, 1000);
}

/**
 * Initialize accessibility features
 */
function initAccessibility() {
    addSkipLink();
    addAriaLandmarks();
    initMobileMenu();
    initKeyboardShortcuts();
    
    // Set initial language
    setLanguage(currentLang);
    
    // Announce page load
    announce(`${document.title} ${t('a11y.loading')}`);
}

// ==================== Initialize on DOM Ready ====================

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initAccessibility);
} else {
    initAccessibility();
}

// ==================== Export ====================

window.t = t;
window.setLanguage = setLanguage;
window.initLanguageSwitcher = initLanguageSwitcher;
window.initMobileMenu = initMobileMenu;
window.initKeyboardShortcuts = initKeyboardShortcuts;
window.showShortcutHelp = showShortcutHelp;
window.announce = announce;
window.initAccessibility = initAccessibility;
