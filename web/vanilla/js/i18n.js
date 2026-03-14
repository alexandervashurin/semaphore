/**
 * I18n - Интернационализация
 * Простая система локализации
 */

// Загрузка языковых файлов
const translations = {
  en: {
    login: 'Login',
    logout: 'Logout',
    username: 'Username',
    password: 'Password',
    welcome: 'Welcome to Semaphore',
    projects: 'Projects',
    templates: 'Templates',
    inventory: 'Inventory',
    repositories: 'Repositories',
    environment: 'Environment',
    keys: 'Keys',
    team: 'Team',
    schedule: 'Schedule',
    integrations: 'Integrations',
    auditLog: 'Audit Log',
    analytics: 'Analytics',
    settings: 'Settings',
    history: 'History',
    tasks: 'Tasks',
    users: 'Users',
    runners: 'Runners',
    apps: 'Apps',
    tokens: 'Tokens',
    playbooks: 'Playbooks',
    playbook: 'Playbook',
    newPlaybook: 'New Playbook',
    editPlaybook: 'Edit Playbook',
    deletePlaybook: 'Delete Playbook',
    runPlaybook: 'Run Playbook',
    syncPlaybook: 'Sync Playbook',
    playbookType: 'Type',
    playbookContent: 'Content',
    noPlaybooks: 'No playbooks',
    noPlaybooksText: 'Create your first playbook to get started',
    create: 'Create',
    edit: 'Edit',
    delete: 'Delete',
    save: 'Save',
    cancel: 'Cancel',
    confirm: 'Confirm',
    close: 'Close',
    ok: 'OK',
    yes: 'Yes',
    no: 'No',
    loading: 'Loading...',
    noData: 'No data',
    success: 'Success',
    error: 'Error',
    warning: 'Warning',
    info: 'Info',
    actions: 'Actions',
    status: 'Status',
    name: 'Name',
    description: 'Description',
    type: 'Type',
    created: 'Created',
    updated: 'Updated',
    run: 'Run',
    stop: 'Stop',
    restart: 'Restart',
    refresh: 'Refresh',
    search: 'Search',
    filter: 'Filter',
    sort: 'Sort',
    export: 'Export',
    import: 'Import',
    download: 'Download',
    upload: 'Upload',
    add: 'Add',
    remove: 'Remove',
    back: 'Back',
    next: 'Next',
    previous: 'Previous',
    finish: 'Finish',
    retry: 'Retry',
    apply: 'Apply',
    reset: 'Reset',
    clear: 'Clear',
    selectAll: 'Select All',
    deselectAll: 'Deselect All',
    deleteConfirm: 'Are you sure you want to delete?',
    saveSuccess: 'Saved successfully',
    saveError: 'Save error',
    deleteSuccess: 'Deleted successfully',
    deleteError: 'Delete error',
    loadingError: 'Loading error',
    networkError: 'Network error',
    unauthorized: 'Unauthorized',
    forbidden: 'Forbidden',
    notFound: 'Not found',
    serverError: 'Server error'
  },
  ru: {
    login: 'Вход',
    logout: 'Выход',
    username: 'Имя пользователя',
    password: 'Пароль',
    welcome: 'Добро пожаловать в Semaphore',
    projects: 'Проекты',
    templates: 'Шаблоны',
    inventory: 'Инвентари',
    repositories: 'Репозитории',
    environment: 'Окружения',
    keys: 'Ключи',
    team: 'Команда',
    schedule: 'Расписание',
    integrations: 'Интеграции',
    auditLog: 'Audit Log',
    analytics: 'Аналитика',
    settings: 'Настройки',
    history: 'История',
    tasks: 'Задачи',
    users: 'Пользователи',
    runners: 'Раннеры',
    apps: 'Приложения',
    tokens: 'Токены',
    playbooks: 'Playbook\'и',
    playbook: 'Playbook',
    newPlaybook: 'Новый Playbook',
    editPlaybook: 'Редактировать Playbook',
    deletePlaybook: 'Удалить Playbook',
    runPlaybook: 'Запустить Playbook',
    syncPlaybook: 'Синхронизировать Playbook',
    playbookType: 'Тип',
    playbookContent: 'Содержимое',
    noPlaybooks: 'Нет playbook\'ов',
    noPlaybooksText: 'Создайте первый playbook чтобы начать работу',
    create: 'Создать',
    edit: 'Редактировать',
    delete: 'Удалить',
    save: 'Сохранить',
    cancel: 'Отмена',
    confirm: 'Подтвердить',
    close: 'Закрыть',
    ok: 'OK',
    yes: 'Да',
    no: 'Нет',
    loading: 'Загрузка...',
    noData: 'Нет данных',
    success: 'Успех',
    error: 'Ошибка',
    warning: 'Предупреждение',
    info: 'Информация',
    actions: 'Действия',
    status: 'Статус',
    name: 'Название',
    description: 'Описание',
    type: 'Тип',
    created: 'Создано',
    updated: 'Обновлено',
    run: 'Запустить',
    stop: 'Остановить',
    restart: 'Перезапустить',
    refresh: 'Обновить',
    search: 'Поиск',
    filter: 'Фильтр',
    sort: 'Сортировка',
    export: 'Экспорт',
    import: 'Импорт',
    download: 'Скачать',
    upload: 'Загрузить',
    add: 'Добавить',
    remove: 'Удалить',
    back: 'Назад',
    next: 'Далее',
    previous: 'Назад',
    finish: 'Завершить',
    retry: 'Повторить',
    apply: 'Применить',
    reset: 'Сбросить',
    clear: 'Очистить',
    selectAll: 'Выбрать все',
    deselectAll: 'Снять выделение',
    deleteConfirm: 'Вы уверены, что хотите удалить?',
    saveSuccess: 'Успешно сохранено',
    saveError: 'Ошибка сохранения',
    deleteSuccess: 'Успешно удалено',
    deleteError: 'Ошибка удаления',
    loadingError: 'Ошибка загрузки',
    networkError: 'Ошибка сети',
    unauthorized: 'Не авторизован',
    forbidden: 'Доступ запрещён',
    notFound: 'Не найдено',
    serverError: 'Ошибка сервера'
  },
  es: {
    login: 'Acceso',
    logout: 'Cerrar sesión',
    username: 'Nombre de usuario',
    password: 'Contraseña',
    welcome: 'Bienvenido a Semaphore',
    projects: 'Proyectos',
    templates: 'Plantillas',
    inventory: 'Inventario',
    repositories: 'Repositorios',
    environment: 'Entorno',
    keys: 'Claves',
    team: 'Equipo',
    schedule: 'Horario',
    integrations: 'Integraciones',
    auditLog: 'Registro de auditoría',
    analytics: 'Analítica',
    settings: 'Configuración',
    history: 'Historia',
    tasks: 'Tareas',
    users: 'Usuarios',
    runners: 'Ejecutores',
    apps: 'Aplicaciones',
    tokens: 'Tokens',
    playbooks: 'Playbooks',
    playbook: 'Playbook',
    newPlaybook: 'Nuevo Playbook',
    editPlaybook: 'Editar Playbook',
    deletePlaybook: 'Eliminar Playbook',
    runPlaybook: 'Ejecutar Playbook',
    syncPlaybook: 'Sincronizar Playbook',
    playbookType: 'Tipo',
    playbookContent: 'Contenido',
    noPlaybooks: 'Sin playbooks',
    noPlaybooksText: 'Cree su primer playbook para comenzar',
    create: 'Crear',
    edit: 'Editar',
    delete: 'Eliminar',
    save: 'Guardar',
    cancel: 'Cancelar',
    confirm: 'Confirmar',
    close: 'Cerrar',
    ok: 'OK',
    yes: 'Sí',
    no: 'No',
    loading: 'Cargando...',
    noData: 'Sin datos',
    success: 'Éxito',
    error: 'Error',
    warning: 'Advertencia',
    info: 'Información',
    actions: 'Acciones',
    status: 'Estado',
    name: 'Nombre',
    description: 'Descripción',
    type: 'Tipo',
    created: 'Creado',
    updated: 'Actualizado',
    run: 'Ejecutar',
    stop: 'Detener',
    restart: 'Reiniciar',
    refresh: 'Actualizar',
    search: 'Buscar',
    filter: 'Filtrar',
    sort: 'Ordenar',
    export: 'Exportar',
    import: 'Importar',
    download: 'Descargar',
    upload: 'Subir',
    add: 'Añadir',
    remove: 'Eliminar',
    back: 'Atrás',
    next: 'Siguiente',
    previous: 'Anterior',
    finish: 'Finalizar',
    retry: 'Reintentar',
    apply: 'Aplicar',
    reset: 'Restablecer',
    clear: 'Limpiar',
    selectAll: 'Seleccionar todo',
    deselectAll: 'Deseleccionar todo',
    deleteConfirm: '¿Está seguro de que desea eliminar?',
    saveSuccess: 'Guardado con éxito',
    saveError: 'Error al guardar',
    deleteSuccess: 'Eliminado con éxito',
    deleteError: 'Error al eliminar',
    loadingError: 'Error de carga',
    networkError: 'Error de red',
    unauthorized: 'No autorizado',
    forbidden: 'Prohibido',
    notFound: 'No encontrado',
    serverError: 'Error del servidor'
  }
};

// Текущий язык
let currentLang = 'ru';

/**
 * Установка языка
 * @param {string} lang - Код языка
 */
export function setLanguage(lang) {
  if (translations[lang]) {
    currentLang = lang;
    localStorage.setItem('semaphore_lang', lang);
    document.documentElement.lang = lang;
  }
}

/**
 * Получение перевода
 * @param {string} key - Ключ
 * @param {Object} params - Параметры
 * @returns {string}
 */
export function t(key, params = {}) {
  const lang = translations[currentLang] || translations.en;
  let text = lang[key] || translations.en[key] || key;
  
  // Подстановка параметров
  Object.entries(params).forEach(([k, v]) => {
    text = text.replace(new RegExp(`{${k}}`, 'g'), v);
  });
  
  return text;
}

/**
 * Получение всех переводов текущего языка
 * @returns {Object}
 */
export function getTranslations() {
  return translations[currentLang] || translations.en;
}

/**
 * Загрузка сохранённого языка
 */
export function loadLanguage() {
  const saved = localStorage.getItem('semaphore_lang') || 'ru';
  setLanguage(saved);
}

/**
 * Директива для перевода текста
 * @param {HTMLElement} el - Элемент
 * @param {string} key - Ключ
 */
export function translateElement(el, key) {
  if (el) {
    el.textContent = t(key);
  }
}

/**
 * Массовый перевод элементов
 * @param {string} selector - Селектор
 */
export function translateAll(selector) {
  document.querySelectorAll(selector).forEach(el => {
    const key = el.dataset.i18n;
    if (key) {
      translateElement(el, key);
    }
  });
}

// Автозагрузка языка
loadLanguage();

export default {
  t,
  setLanguage,
  getTranslations,
  loadLanguage,
  translateElement,
  translateAll
};
