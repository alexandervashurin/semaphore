# 🔔 Webhook API - Документация

> **Универсальная система отправки webhook уведомлений**

---

## 📋 Содержание

1. [Обзор](#обзор)
2. [Поддерживаемые типы](#поддерживаемые-типы)
3. [Модель данных](#модель-данных)
4. [Примеры использования](#примеры-использования)
5. [Интеграция с сервисами](#интеграция-с-сервисами)

---

## 📖 Обзор

Webhook сервис позволяет отправлять уведомления о событиях системы во внешние сервисы:

- **Уведомления о задачах**: запуск, завершение, ошибка
- **Уведомления о пользователях**: создание, удаление, изменения
- **Уведомления о проектах**: CRUD операции
- **Кастомные события**: любые другие события системы

### Особенности

- ✅ 5 встроенных типов webhook
- ✅ Автоматические повторные попытки
- ✅ Экспоненциальная задержка
- ✅ Кастомные заголовки
- ✅ Секретная аутентификация
- ✅ Детальное логирование

---

## 🎯 Поддерживаемые типы

### Generic

Стандартный JSON webhook:

```json
{
  "event": "task_success",
  "timestamp": "2026-03-09T10:00:00Z",
  "data": {
    "task_id": 1,
    "task_name": "Deploy",
    "status": "success"
  },
  "metadata": {
    "source": "semaphore-ui",
    "version": "0.1.0"
  }
}
```

### Slack

Форматированное сообщение с attachments:

```json
{
  "attachments": [{
    "color": "good",
    "author_name": "✅ Velum",
    "title": "Задача: Deploy",
    "text": "Задача успешно завершена",
    "fields": [
      {"title": "Событие", "value": "task_success", "short": true},
      {"title": "Время", "value": "2026-03-09 10:00:00", "short": true}
    ]
  }]
}
```

### Microsoft Teams

MessageCard формат:

```json
{
  "@type": "MessageCard",
  "@context": "http://schema.org/extensions",
  "themeColor": "8BC34A",
  "summary": "Задача: Deploy",
  "sections": [{
    "activityTitle": "Задача: Deploy",
    "activitySubtitle": "Velum",
    "activityText": "Задача успешно завершена"
  }]
}
```

### Discord

Embed сообщение:

```json
{
  "embeds": [{
    "title": "Задача: Deploy",
    "description": "Задача успешно завершена",
    "color": 65280,
    "fields": [
      {"name": "Событие", "value": "task_success", "inline": true},
      {"name": "Время", "value": "2026-03-09 10:00:00", "inline": true}
    ]
  }]
}
```

### Telegram

HTML форматирование:

```json
{
  "text": "<b>✅ Задача: Deploy</b>\n\nЗадача успешно завершена\n\n<i>Время: 2026-03-09 10:00:00</i>",
  "parse_mode": "HTML"
}
```

---

## 🗃️ Модель данных

### WebhookConfig

| Поле | Тип | Описание |
|------|-----|----------|
| `id` | i64 | Уникальный ID |
| `project_id` | Option<i64> | ID проекта |
| `name` | String | Название |
| `type` | WebhookType | Тип webhook |
| `url` | String | URL endpoint |
| `secret` | Option<String> | Секрет для аутентификации |
| `headers` | Option<Json> | Кастомные заголовки |
| `active` | bool | Активен ли |
| `events` | Vec<String> | Список событий |
| `retry_count` | i32 | Количество попыток |
| `timeout_secs` | i64 | Таймаут в секундах |

### WebhookEvent

| Поле | Тип | Описание |
|------|-----|----------|
| `event_type` | String | Тип события |
| `timestamp` | DateTime | Время события |
| `data` | Json | Данные события |
| `metadata` | WebhookMetadata | Метаданные |

### WebhookResult

| Поле | Тип | Описание |
|------|-----|----------|
| `success` | bool | Успешно ли |
| `status_code` | Option<u16> | HTTP статус |
| `response_body` | Option<String> | Тело ответа |
| `error` | Option<String> | Ошибка |
| `attempts` | u32 | Количество попыток |

---

## 💡 Примеры использования

### Создание webhook

```rust
use crate::services::webhook::{WebhookConfig, WebhookType};

let config = WebhookConfig {
    id: 1,
    project_id: Some(1),
    name: "Slack Notifications".to_string(),
    r#type: WebhookType::Slack,
    url: "https://hooks.slack.com/services/XXX/YYY/ZZZ".to_string(),
    secret: None,
    headers: None,
    active: true,
    events: vec![
        "task_success".to_string(),
        "task_failed".to_string(),
    ],
    retry_count: 3,
    timeout_secs: 30,
};
```

### Отправка события

```rust
use crate::services::webhook::{
    WebhookService, WebhookEvent, WebhookMetadata,
    create_task_event,
};
use serde_json::json;

// Создание сервиса
let webhook_service = WebhookService::new();

// Создание события
let event = create_task_event(
    "task_success",
    1,      // task_id
    "Deploy", // task_name
    Some(1), // project_id
    Some(1), // user_id
    Some("success"),
);

// Отправка
let result = webhook_service.send_webhook(&config, &event).await?;

if result.success {
    println!("Webhook успешно отправлен!");
} else {
    println!("Ошибка: {:?}", result.error);
}
```

### С кастомными заголовками

```rust
use serde_json::json;

let config = WebhookConfig {
    // ...
    headers: Some(json!({
        "X-Custom-Header": "value",
        "Authorization": "Bearer token"
    })),
    // ...
};
```

### С секретом

```rust
let config = WebhookConfig {
    // ...
    secret: Some("my-secret-key".to_string()),
    // ...
};
```

Секрет автоматически добавляется в заголовок `Authorization: Bearer <secret>`.

---

## 🔧 Интеграция с сервисами

### Slack

1. Создайте Incoming Webhook: https://my.slack.com/services/new/incoming-webhook/
2. Получите URL webhook
3. Настройте в Semaphore:

```json
{
  "name": "Slack",
  "type": "slack",
  "url": "https://hooks.slack.com/services/XXX/YYY/ZZZ",
  "events": ["task_success", "task_failed"]
}
```

### Microsoft Teams

1. Откройте канал Teams
2. Нажмите "..." → "Connectors"
3. Добавьте "Incoming Webhook"
4. Скопируйте URL

```json
{
  "name": "Teams",
  "type": "teams",
  "url": "https://outlook.office.com/webhook/XXX",
  "events": ["task_success", "task_failed"]
}
```

### Discord

1. Откройте настройки канала
2. Перейдите в "Integrations" → "Webhooks"
3. Создайте новый webhook
4. Скопируйте URL

```json
{
  "name": "Discord",
  "type": "discord",
  "url": "https://discord.com/api/webhooks/XXX/YYY",
  "events": ["task_success", "task_failed"]
}
```

### Telegram

1. Создайте бота через @BotFather
2. Получите токен
3. Узнайте chat_id через @getmyid_bot
4. Используйте URL:

```
https://api.telegram.org/bot<token>/sendMessage?chat_id=<chat_id>
```

```json
{
  "name": "Telegram",
  "type": "telegram",
  "url": "https://api.telegram.org/botXXX/sendMessage?chat_id=YYY",
  "events": ["task_success", "task_failed"]
}
```

---

## 🔄 Механизм повторных попыток

При неудачной отправке webhook сервис автоматически повторяет попытку:

- **retry_count**: количество повторных попыток (по умолчанию 3)
- **Задержка**: экспоненциальная (100ms, 200ms, 400ms, ...)
- **Максимум**: 8 секунд между попытками

```rust
let config = WebhookConfig {
    // ...
    retry_count: 5,  // 5 попыток
    timeout_secs: 60, // 60 секунд таймаут
    // ...
};
```

---

## 📝 События

### Task Events

- `task_created` - Задача создана
- `task_started` - Задача запущена
- `task_success` - Задача успешна
- `task_failed` - Задача провалена
- `task_stopped` - Задача остановлена

### User Events

- `user_created` - Пользователь создан
- `user_updated` - Пользователь обновлён
- `user_deleted` - Пользователь удалён
- `user_login` - Пользователь вошёл
- `user_logout` - Пользователь вышел

### Project Events

- `project_created` - Проект создан
- `project_updated` - Проект обновлён
- `project_deleted` - Проект удалён

---

## 🛡️ Безопасность

### Secret Authentication

```rust
let config = WebhookConfig {
    // ...
    secret: Some("my-secret-key".to_string()),
    // ...
};
```

Сервис автоматически добавляет заголовок:
```
Authorization: Bearer my-secret-key
```

### Custom Headers

```json
{
  "headers": {
    "X-Signature": "sha256=abc123...",
    "X-Request-ID": "unique-id"
  }
}
```

---

*Последнее обновление: 9 марта 2026 г.*
