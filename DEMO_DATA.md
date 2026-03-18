# 📦 Демо-данные Velum

## 🎯 Обзор

Демонстрационное окружение полностью наполнено тестовыми данными для всех сущностей.

## 👥 Пользователи

| Логин | Пароль | Имя | Роль | Email |
|-------|--------|-----|------|-------|
| `admin` | `demo123` | Administrator | Администратор | admin@semaphore.local |
| `john.doe` | `demo123` | John Doe | Менеджер | john.doe@semaphore.local |
| `jane.smith` | `demo123` | Jane Smith | Менеджер | jane.smith@semaphore.local |
| `devops` | `demo123` | DevOps Engineer | Исполнитель | devops@semaphore.local |

## 📁 Проекты (4)

### 1. Demo Infrastructure (ID: 1)
- **Описание:** Основная демонстрационная инфраструктура
- **Alert:** Включён
- **Max параллельных задач:** 5
- **Тип:** default

### 2. Web Application Deployment (ID: 2)
- **Описание:** Деплой веб-приложений
- **Alert:** Отключён
- **Max параллельных задач:** 3
- **Тип:** default

### 3. Database Management (ID: 3)
- **Описание:** Управление базами данных
- **Alert:** Включён
- **Max параллельных задач:** 2
- **Тип:** default

### 4. Security & Compliance (ID: 4)
- **Описание:** Безопасность и соответствие требованиям
- **Alert:** Отключён
- **Max параллельных задач:** 1
- **Тип:** default

## 📋 Шаблоны (12)

| ID | Название | Playbook | Проект | Тип | App | Ветка |
|----|----------|----------|--------|-----|-----|-------|
| 1 | Deploy Infrastructure | site.yml | 1 | ansible | ansible | main |
| 2 | Update Servers | update.yml | 1 | ansible | ansible | main |
| 3 | Staging Deploy | deploy.yml | 1 | ansible | ansible | develop |
| 4 | Deploy Web App | deploy-webapp.yml | 2 | ansible | ansible | master |
| 5 | Rollback Web App | rollback.yml | 2 | ansible | ansible | hotfix |
| 6 | Deploy API Service | deploy-api.yml | 2 | ansible | ansible | main |
| 7 | Backup Databases | backup.yml | 3 | ansible | ansible | main |
| 8 | Restore Database | restore.yml | 3 | ansible | ansible | main |
| 9 | Database Maintenance | maintenance.yml | 3 | ansible | ansible | main |
| 10 | Security Scan | security-scan.yml | 4 | ansible | ansible | master |
| 11 | Compliance Check | compliance-check.yml | 4 | ansible | ansible | main |
| 12 | Vulnerability Scan | vulnerability-scan.yml | 4 | ansible | ansible | develop |

## 🖥️ Инвентари (5)

### 1. Production Servers (ID: 1)
**Тип:** static
**SSH:** root:22
**Данные:**
```yaml
all:
  children:
    webservers:
      hosts:
        web1.example.com:
          ansible_user: ansible
          ansible_port: 22
        web2.example.com:
          ansible_user: ansible
          ansible_port: 22
    databases:
      hosts:
        db1.example.com:
          ansible_user: ansible
          ansible_port: 22
        db2.example.com:
          ansible_user: ansible
          ansible_port: 22
    monitoring:
      hosts:
        monitor1.example.com:
          ansible_user: ansible
          ansible_port: 22
```

### 2. Staging Environment (ID: 2)
**Тип:** static
**SSH:** ubuntu:22
**Данные:**
```ini
[staging]
staging-web1 ansible_host=192.168.1.100 ansible_user=ubuntu
staging-app1 ansible_host=192.168.1.101 ansible_user=ubuntu

[staging:vars]
ansible_port=22
ansible_ssh_private_key_file=~/.ssh/staging_key
```

### 3. Web App Cluster (ID: 3)
**Тип:** static
**SSH:** root:22

### 4. Database Cluster (ID: 4)
**Тип:** static
**SSH:** postgres:22

### 5. Security Scan Targets (ID: 5)
**Тип:** static
**SSH:** root:22

## 📦 Репозитории (5)

| ID | Название | Git URL | Ветка | Тип |
|----|----------|---------|-------|-----|
| 1 | Infrastructure Playbooks | https://github.com/semaphore-demo/infrastructure-playbooks.git | main | git |
| 2 | Web App Deployment | https://github.com/semaphore-demo/webapp-deploy.git | master | git |
| 3 | Database Playbooks | https://github.com/semaphore-demo/db-playbooks.git | main | git |
| 4 | Security Scripts | https://github.com/semaphore-demo/security-scripts.git | master | git |
| 5 | Common Roles | https://github.com/semaphore-demo/common-roles.git | develop | git |

## ⚙️ Окружения (5)

### 1. Production Variables (ID: 1)
```json
{
  "env": "production",
  "domain": "example.com",
  "ssl_enabled": true,
  "monitoring_enabled": true,
  "backup_enabled": true,
  "log_level": "warn"
}
```

### 2. Staging Variables (ID: 2)
```json
{
  "env": "staging",
  "domain": "staging.example.com",
  "ssl_enabled": true,
  "monitoring_enabled": true,
  "backup_enabled": false,
  "log_level": "debug"
}
```

### 3. Web App Config (ID: 3)
```json
{
  "app_name": "MyWebApp",
  "app_port": 8080,
  "workers": 4,
  "cache_enabled": true,
  "session_timeout": 3600
}
```

### 4. Database Config (ID: 4)
```json
{
  "postgres_version": "15",
  "mysql_version": "8.0",
  "max_connections": 200,
  "shared_buffers": "256MB",
  "backup_retention_days": 7
}
```

### 5. Security Scan Config (ID: 5)
```json
{
  "scan_type": "full",
  "severity_threshold": "medium",
  "report_format": "html",
  "notify_on_failure": true
}
```

## 🔑 Ключи доступа (5)

| ID | Название | Тип | Логин/Ключ |
|----|----------|-----|------------|
| 1 | Demo SSH Key | ssh | SSH Private Key |
| 2 | Demo Login/Password | login_password | ansible / demo123 |
| 3 | Web App SSH Key | ssh | SSH Private Key |
| 4 | DB Admin Key | login_password | dbadmin / dbpass123 |
| 5 | Security Audit Key | ssh | SSH Private Key |

## ⚡ Задачи (6)

| ID | Шаблон | Проект | Статус | Playbook | Сообщение |
|----|--------|--------|--------|----------|-----------|
| 1 | 1 | 1 | success | site.yml | Infrastructure deployed successfully |
| 2 | 4 | 2 | success | deploy-webapp.yml | Web App v1.2.0 deployed |
| 3 | 7 | 3 | success | backup.yml | Database backup completed |
| 4 | 10 | 4 | success | security-scan.yml | Security scan completed |
| 5 | 2 | 1 | running | update.yml | Server update in progress |
| 6 | 1 | 1 | waiting | site.yml | Waiting for execution |

## 🕐 Расписания (4)

| ID | Название | Cron | Активно | Шаблон |
|----|----------|------|---------|--------|
| 1 | Weekly Server Update | 0 3 * * 0 | ✅ | Update Servers |
| 2 | Daily Database Backup | 0 2 * * * | ✅ | Backup Databases |
| 3 | Weekly Security Scan | 0 4 * * 1 | ✅ | Security Scan |
| 4 | Daily Compliance Check | 0 6 * * * | ✅ | Compliance Check |

## 📊 События

Система содержит демонстрационные события:
- Выполненные задачи
- Созданные проекты
- Изменения в конфигурации

## 🚀 Как использовать

### 1. Запуск окружения
```bash
./start.sh
```

### 2. Открыть UI
http://localhost:80/

### 3. Войти
- Логин: `admin`
- Пароль: `demo123`

### 4. CRUD операции
Все сущности доступны для:
- ✅ Просмотра
- ✅ Создания
- ✅ Редактирования
- ✅ Удаления

## 📝 Примечания

1. **Пароли пользователей:** Все пользователи используют пароль `demo123`
2. **SSH ключи:** Демо-ключи недействительны, замените на реальные
3. **Git репозитории:** URL демонстрационные, замените на свои
4. **Инвентари:** Хосты примерные, обновите под свою инфраструктуру

## 🔄 Сброс данных

Для сброса к демонстрационным данным:
```bash
./start.sh --clean
```

Или через cleanup:
```bash
./cleanup.sh --volumes
./start.sh
```

## 📚 Дополнительная документация

- [CRUD_DEMO.md](CRUD_DEMO.md) - Руководство по CRUD демо
- [API.md](API.md) - Документация API
- [CRUD_TESTS.md](CRUD_TESTS.md) - Тестирование CRUD
