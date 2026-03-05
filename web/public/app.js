// Минимальный JS для тестирования backend API
// Заменяет полноценную сборку Vue.js

const API_BASE = '/api';

// Состояние
let token = localStorage.getItem('semaphore_token');
let currentUser = null;

// DOM элементы
const loginView = document.getElementById('login-view');
const dashboardView = document.getElementById('dashboard-view');
const loginForm = document.getElementById('login-form');
const loginError = document.getElementById('login-error');
const userInfo = document.getElementById('user-info');
const logoutBtn = document.getElementById('logout-btn');

// Проверка авторизации при загрузке
async function checkAuth() {
    if (!token) {
        showLogin();
        return;
    }
    
    try {
        const res = await fetch(`${API_BASE}/user`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (res.ok) {
            currentUser = await res.json();
            showDashboard();
        } else {
            localStorage.removeItem('token');
            token = null;
            showLogin();
        }
    } catch (e) {
        console.error('Auth check failed:', e);
        showLogin();
    }
}

// Показать login форму
function showLogin() {
    loginView.classList.remove('hidden');
    dashboardView.classList.add('hidden');
}

// Показать dashboard
function showDashboard() {
    loginView.classList.add('hidden');
    dashboardView.classList.remove('hidden');
    if (currentUser) {
        userInfo.textContent = currentUser.username || currentUser.login || 'User';
    }
}

// Login
loginForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    
    try {
        const res = await fetch(`${API_BASE}/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ login: username, password })
        });
        
        const data = await res.json();
        
        if (res.ok && data.token) {
            token = data.token;
            localStorage.setItem('semaphore_token', token);
            loginError.textContent = '';
            
            // Получить информацию о пользователе
            const userRes = await fetch(`${API_BASE}/user`, {
                headers: { 'Authorization': `Bearer ${token}` }
            });
            currentUser = await userRes.json();
            
            showDashboard();
            loadProjects();
        } else {
            loginError.textContent = data.message || 'Ошибка авторизации';
        }
    } catch (e) {
        console.error('Login failed:', e);
        loginError.textContent = 'Ошибка соединения с сервером';
    }
});

// Logout
logoutBtn.addEventListener('click', () => {
    token = null;
    currentUser = null;
    localStorage.removeItem('semaphore_token');
    showLogin();
});

// Загрузка проектов
async function loadProjects() {
    try {
        const res = await fetch(`${API_BASE}/projects`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (res.ok) {
            const projects = await res.json();
            renderProjects(projects);
        }
    } catch (e) {
        console.error('Failed to load projects:', e);
    }
}

// Рендер проектов
function renderProjects(projects) {
    const container = document.getElementById('projects-list');
    if (!container) return;
    
    if (projects.length === 0) {
        container.innerHTML = '<p class="empty-message">Проектов нет</p>';
        return;
    }
    
    container.innerHTML = projects.map(p => `
        <div class="list-item">
            <h3>${p.name || 'Без названия'}</h3>
            <p>${p.description || ''}</p>
        </div>
    `).join('');
}

// Навигация
document.querySelectorAll('.nav-link').forEach(link => {
    link.addEventListener('click', (e) => {
        e.preventDefault();
        const page = link.dataset.page;
        
        document.querySelectorAll('.nav-link').forEach(l => l.classList.remove('active'));
        document.querySelectorAll('.page').forEach(p => p.classList.add('hidden'));
        
        link.classList.add('active');
        document.getElementById(`${page}-page`).classList.remove('hidden');
        
        if (page === 'projects') loadProjects();
    });
});

// Инициализация
checkAuth();

console.log('Semaphore UI initialized');
