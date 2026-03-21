# 🔌 Plugin System - Документация

> **Система плагинов для расширения функциональности Velum**

---

## 📋 Содержание

1. [Обзор](#обзор)
2. [Типы плагинов](#типы-плагинов)
3. [Архитектура](#архитектура)
4. [Создание плагина](#создание-плагина)
5. [API плагинов](#api-плагинов)
6. [Хуки](#хуки)
7. [Менеджер плагинов](#менеджер-плагинов)

---

## 📖 Обзор

Система плагинов позволяет расширять функциональность Velum без изменения основного кода приложения.

### Возможности

- ✅ Динамическая загрузка/выгрузка плагинов
- ✅ Зависимости между плагинами
- ✅ Конфигурация на уровне плагинов
- ✅ Система хуков для событий
- ✅ Несколько типов плагинов
- ✅ Безопасное выполнение

---

## 🎯 Типы плагинов

### TaskExecutor

Плагины-исполнители задач для поддержки кастомных типов задач.

```rust
#[async_trait]
pub trait TaskExecutorPlugin: Plugin {
    async fn can_execute(&self, task: &Task) -> bool;
    async fn execute(&self, context: PluginContext, task: &Task) -> Result<TaskResult>;
    async fn stop(&self, context: PluginContext, task_id: i64) -> Result<()>;
}
```

**Пример использования:**
- Выполнение Docker контейнеров
- Запуск Kubernetes jobs
- Интеграция с CI/CD системами

### NotificationProvider

Провайдеры уведомлений для отправки в различные каналы.

```rust
#[async_trait]
pub trait NotificationPlugin: Plugin {
    async fn send(&self, context: PluginContext, notification: Notification) -> Result<()>;
    fn get_channels(&self) -> Vec<NotificationChannel>;
}
```

**Пример использования:**
- Email уведомления
- Push уведомления
- SMS уведомления

### StorageProvider

Провайдеры хранилищ для сохранения данных.

```rust
#[async_trait]
pub trait StoragePlugin: Plugin {
    async fn save(&self, key: &str, data: JsonValue) -> Result<()>;
    async fn load(&self, key: &str) -> Result<Option<JsonValue>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>>;
}
```

**Пример использования:**
- S3 хранилище
- Redis кэш
- Файловое хранилище

### AuthProvider

Провайдеры аутентификации для поддержки внешних систем.

```rust
#[async_trait]
pub trait AuthPlugin: Plugin {
    async fn authenticate(&self, credentials: AuthCredentials) -> Result<AuthResult>;
    async fn validate_token(&self, token: &str) -> Result<AuthResult>;
    async fn create_token(&self, user_id: i64) -> Result<String>;
}
```

**Пример использования:**
- LDAP/AD аутентификация
- OAuth2 провайдеры
- SAML интеграция

### ApiExtension

Расширения API для добавления новых endpoints.

```rust
#[async_trait]
pub trait ApiExtensionPlugin: Plugin {
    fn get_routes(&self) -> Vec<ApiRoute>;
}
```

**Пример использования:**
- Кастомные API endpoints
- GraphQL API
- WebSocket handlers

### Hook

Хуки для реагирования на события системы.

```rust
#[async_trait]
pub trait HookPlugin: Plugin {
    fn get_hooks(&self) -> Vec<String>;
    async fn execute_hook(&self, event: HookEvent) -> Result<HookResult>;
}
```

**Пример использования:**
- Аудит действий
- Валидация данных
- Кастомная логика

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                   Plugin Manager                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │   Plugin    │  │   Plugin    │  │   Plugin    │    │
│  │  Executor   │  │ Notification│  │    Hook     │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
│         │                │                │            │
│         ▼                ▼                ▼            │
│  ┌─────────────────────────────────────────────────┐  │
│  │              Hook Registry                       │  │
│  └─────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
         │                │                │
         ▼                ▼                ▼
  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   Task      │  │   Email     │  │   Audit     │
│   Runner    │  │   Service   │  │   Log       │
└─────────────┘  └─────────────┘  └─────────────┘
```

---

## 🛠️ Создание плагина

### Базовая структура

```rust
use semaphore::plugins::*;

declare_plugin!(
    MyPlugin,
    PluginType::Custom,
    "1.0.0",
    "Мой кастомный плагин",
    "Author Name"
);

#[async_trait]
impl HookPlugin for MyPlugin {
    fn get_hooks(&self) -> Vec<String> {
        vec!["task.after_complete".to_string()]
    }

    async fn execute_hook(&self, event: HookEvent) -> Result<HookResult> {
        info!("Hook triggered: {:?}", event.name);
        
        Ok(HookResult {
            success: true,
            data: Some(json!({"processed": true})),
            error: None,
        })
    }
}
```

### Регистрация плагина

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = PluginManager::new(PluginManagerConfig {
        plugins_dir: "/path/to/plugins".to_string(),
        enabled_plugins: vec!["my_plugin".to_string()],
        ..Default::default()
    });

    let plugin = Arc::new(RwLock::new(MyPlugin::new()));
    manager.register(plugin).await?;
    manager.load_all().await?;

    Ok(())
}
```

---

## 🌐 API плагинов

### GET /api/plugins

Получить список всех плагинов.

**Требуемые права**: Администратор

**Ответ**:

```json
[
  {
    "id": "my_plugin",
    "name": "MyPlugin",
    "version": "1.0.0",
    "description": "Мой кастомный плагин",
    "author": "Author Name",
    "type": "custom",
    "enabled": true,
    "status": "loaded"
  }
]
```

### GET /api/plugins/:id

Получить информацию о плагине.

### POST /api/plugins/:id/enable

Включить плагин.

### POST /api/plugins/:id/disable

Отключить плагин.

### PUT /api/plugins/:id/config

Обновить конфигурацию плагина.

---

## 🔔 Хуки

### Доступные хуки

#### Задачи (Task)

| Хук | Описание |
|-----|----------|
| `task.before_create` | Перед созданием задачи |
| `task.after_create` | После создания задачи |
| `task.before_start` | Перед запуском задачи |
| `task.after_start` | После запуска задачи |
| `task.before_complete` | Перед завершением задачи |
| `task.after_complete` | После завершения задачи |
| `task.before_fail` | Перед провалом задачи |
| `task.after_fail` | После провала задачи |
| `task.before_stop` | Перед остановкой задачи |
| `task.after_stop` | После остановки задачи |
| `task.before_delete` | Перед удалением задачи |
| `task.after_delete` | После удаления задачи |

#### Проекты (Project)

| Хук | Описание |
|-----|----------|
| `project.before_create` | Перед созданием проекта |
| `project.after_create` | После создания проекта |
| `project.before_update` | Перед обновлением проекта |
| `project.after_update` | После обновления проекта |
| `project.before_delete` | Перед удалением проекта |
| `project.after_delete` | После удаления проекта |

#### Пользователи (User)

| Хук | Описание |
|-----|----------|
| `user.before_login` | Перед входом пользователя |
| `user.after_login` | После входа пользователя |
| `user.before_logout` | Перед выходом пользователя |
| `user.after_logout` | После выхода пользователя |
| `user.before_create` | Перед созданием пользователя |
| `user.after_create` | После создания пользователя |

### Пример обработчика хука

```rust
use semaphore::plugins::*;

pub struct AuditHook {
    name: String,
}

impl AuditHook {
    pub fn new() -> Self {
        Self {
            name: "audit_hook".to_string(),
        }
    }
}

#[async_trait]
impl HookHandler for AuditHook {
    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        10 // Средний приоритет
    }

    async fn handle(&self, event: HookEvent) -> Result<HookResult> {
        // Логирование события
        tracing::info!("Audit event: {} - {:?}", event.name, event.data);
        
        Ok(HookResult {
            success: true,
            data: None,
            error: None,
        })
    }
}
```

---

## 🎛️ Менеджер плагинов

### Конфигурация

```rust
pub struct PluginManagerConfig {
    pub plugins_dir: String,           // Директория с плагинами
    pub enabled_plugins: Vec<String>,  // Включённые плагины
    pub disabled_plugins: Vec<String>, // Отключённые плагины
    pub auto_load: bool,               // Автозагрузка
}
```

### Методы

```rust
// Создание менеджера
let manager = PluginManager::new(config);

// Регистрация плагина
manager.register(plugin).await?;

// Загрузка всех плагинов
manager.load_all().await?;

// Выгрузка всех плагинов
manager.unload_all().await?;

// Список плагинов
let plugins = manager.list_plugins().await;

// Включение плагина
manager.enable_plugin("my_plugin")?;

// Отключение плагина
manager.disable_plugin("my_plugin")?;
```

---

## 📦 Структура плагина

```
my_plugin/
├── manifest.json       # Метаданные плагина
├── config.schema.json  # Схема конфигурации
├── plugin.wasm         # Бинарник плагина (опционально)
└── README.md           # Документация
```

### manifest.json

```json
{
  "id": "my_plugin",
  "name": "MyPlugin",
  "version": "1.0.0",
  "description": "Мой кастомный плагин",
  "author": "Author Name",
  "homepage": "https://example.com",
  "repository": "https://github.com/example/my_plugin",
  "license": "MIT",
  "type": "hook",
  "min_semaphore_version": "0.1.0",
  "dependencies": [],
  "config": {
    "fields": [
      {
        "name": "api_key",
        "type": "string",
        "label": "API Key",
        "required": true
      }
    ]
  }
}
```

---

## 🔒 Безопасность

### Изоляция плагинов

- Плагины выполняются в изолированном контексте
- Ограниченный доступ к системным ресурсам
- Таймауты для операций плагинов

### Валидация

- Проверка манифеста плагина
- Валидация конфигурации
- Проверка зависимостей

---

*Последнее обновление: 9 марта 2026 г.*
