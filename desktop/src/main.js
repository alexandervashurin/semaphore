//! Semaphore Desktop - Frontend JavaScript
//! 
//! Взаимодействие с Tauri backend через invoke API

const { invoke } = window.__TAURI__.core;
const { notification } = window.__TAURI__;

// Элементы DOM
const loginForm = document.getElementById('loginForm');
const dashboard = document.getElementById('dashboard');
const connectForm = document.getElementById('connectForm');
const connectBtn = document.getElementById('connectBtn');
const logoutBtn = document.getElementById('logoutBtn');
const statusDiv = document.getElementById('status');
const serverUrlInput = document.getElementById('serverUrl');
const apiTokenInput = document.getElementById('apiToken');
const projectCountEl = document.getElementById('projectCount');
const taskCountEl = document.getElementById('taskCount');
const taskListEl = document.getElementById('taskList');

// Показать статус
function showStatus(message, type = 'info') {
    statusDiv.textContent = message;
    statusDiv.className = `status ${type}`;
    statusDiv.style.display = 'block';
    
    if (type === 'error') {
        // Отправляем уведомление об ошибке
        invoke('send_notification', { 
            title: 'Ошибка подключения', 
            body: message 
        }).catch(console.error);
    }
}

// Скрыть статус
function hideStatus() {
    statusDiv.style.display = 'none';
}

// Подключение к серверу
connectForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const serverUrl = serverUrlInput.value.trim();
    const apiToken = apiTokenInput.value.trim();
    
    if (!serverUrl || !apiToken) {
        showStatus('Введите URL сервера и API токен', 'error');
        return;
    }
    
    connectBtn.disabled = true;
    connectBtn.textContent = 'Подключение...';
    hideStatus();
    
    try {
        const success = await invoke('connect_to_server', {
            serverUrl,
            apiToken
        });
        
        if (success) {
            showStatus('Успешное подключение!', 'success');
            setTimeout(() => {
                loginForm.classList.add('hidden');
                dashboard.classList.add('active');
                loadDashboard();
            }, 1000);
        } else {
            showStatus('Не удалось подключиться к серверу', 'error');
        }
    } catch (error) {
        showStatus(`Ошибка: ${error}`, 'error');
    } finally {
        connectBtn.disabled = false;
        connectBtn.textContent = 'Подключиться';
    }
});

// Отключение от сервера
logoutBtn.addEventListener('click', async () => {
    try {
        await invoke('disconnect_from_server');
        dashboard.classList.remove('active');
        loginForm.classList.remove('hidden');
        apiTokenInput.value = '';
        showStatus('Отключено от сервера', 'info');
    } catch (error) {
        showStatus(`Ошибка: ${error}`, 'error');
    }
});

// Загрузка dashboard
async function loadDashboard() {
    try {
        // Загружаем проекты
        const projects = await invoke('get_projects');
        projectCountEl.textContent = projects.length;
        
        // Загружаем последние задачи
        const tasks = await invoke('get_recent_tasks', { limit: 5 });
        taskCountEl.textContent = tasks.length;
        
        // Отображаем задачи
        renderTaskList(tasks);
        
        // Отправляем уведомление о загрузке
        invoke('send_notification', {
            title: 'Velum',
            body: `Загружено ${projects.length} проектов и ${tasks.length} задач`
        }).catch(console.error);
        
    } catch (error) {
        showStatus(`Ошибка загрузки: ${error}`, 'error');
    }
}

// Рендеринг списка задач
function renderTaskList(tasks) {
    if (!tasks || tasks.length === 0) {
        taskListEl.innerHTML = '<p style="color: #666; text-align: center;">Нет задач</p>';
        return;
    }
    
    taskListEl.innerHTML = tasks.map(task => `
        <div class="task-item">
            <div>
                <div style="font-weight: 600;">Task #${task.id}</div>
                <div style="font-size: 12px; color: #666;">
                    Проект: ${task.project_id} | Шаблон: ${task.template_id}
                </div>
            </div>
            <span class="task-status ${getTaskStatusClass(task.status)}">
                ${task.status}
            </span>
        </div>
    `).join('');
}

// Получение класса статуса
function getTaskStatusClass(status) {
    switch (status.toLowerCase()) {
        case 'success':
            return 'success';
        case 'running':
            return 'running';
        case 'failed':
            return 'failed';
        default:
            return '';
    }
}

// Проверка состояния при загрузке
async function checkConnectionState() {
    try {
        const state = await invoke('get_connection_state');
        if (state.connected) {
            loginForm.classList.add('hidden');
            dashboard.classList.add('active');
            loadDashboard();
        }
    } catch (error) {
        console.error('Failed to check connection state:', error);
    }
}

// Автообновление данных каждые 30 секунд
setInterval(() => {
    if (dashboard.classList.contains('active')) {
        loadDashboard();
    }
}, 30000);

// Инициализация при загрузке
checkConnectionState();
