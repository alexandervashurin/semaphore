# 🔒 Security Configuration

> Руководство по настройке безопасности Velum
> **Последнее обновление:** 10 марта 2026 г.

---

## 📋 Содержание

1. [Security Headers](#security-headers)
2. [CORS Configuration](#cors-configuration)
3. [Rate Limiting](#rate-limiting)
4. [Production Checklist](#production-checklist)

---

## 🛡️ Security Headers

Semaphore автоматически добавляет security headers ко всем HTTP ответам.

### Добавляемые заголовки

| Заголовок | Значение | Описание |
|-----------|----------|----------|
| **X-Frame-Options** | `DENY` | Защита от clickjacking атак |
| **X-Content-Type-Options** | `nosniff` | Запрет MIME type sniffing |
| **X-XSS-Protection** | `1; mode=block` | Включение XSS фильтра браузера |
| **Strict-Transport-Security** | `max-age=31536000; includeSubDomains` | HSTS (принудительный HTTPS) |
| **Content-Security-Policy** | См. ниже | Ограничение источников контента |
| **Referrer-Policy** | `strict-origin-when-cross-origin` | Контроль передачи Referer |
| **Permissions-Policy** | `geolocation=(), microphone=(), camera=()` | Отключение опасных функций |

### Content Security Policy

```
default-src 'self'
script-src 'self'
style-src 'self' 'unsafe-inline'
img-src 'self' data:
font-src 'self'
connect-src 'self'
frame-ancestors 'none'
base-uri 'self'
form-action 'self'
```

---

## 🌐 CORS Configuration

### Development режим

По умолчанию разрешены запросы с любых доменов.

### Production режим

Для production рекомендуется настроить конкретные домены через `strict_cors_headers`.

---

## ⏱️ Rate Limiting

### Конфигурация по умолчанию

| Endpoint | Лимит | Период |
|----------|-------|--------|
| **API** | 100 запросов | 60 секунд |
| **Auth** | 5 запросов | 60 секунд |

---

## 📊 Production Checklist

### Перед развёртыванием

- [ ] Настроить CORS для конкретных доменов
- [ ] Включить HTTPS (TLS/SSL)
- [ ] Настроить rate limiting под нагрузку
- [ ] Проверить security headers
- [ ] Включить audit logging
- [ ] Настроить backup стратегии

---

*Документ будет обновляться по мере добавления новых функций безопасности*
