# Telegram-бот

> Интеграция Telegram-бота для уведомлений и команд Velum
>
> 📖 См. также: [Конфигурация](../getting-started/configuration.md), [Режим отладки](../troubleshooting/debug-mode.md)

---

## Настройка

### Получение токена

1. Откройте [@BotFather](https://t.me/BotFather) в Telegram
2. Отправьте `/newbot`
3. Следуйте инструкциям
4. Скопируйте токен

### Конфигурация Velum

```bash
VELUM_TELEGRAM_TOKEN=ваш-токен-бота
VELUM_TELEGRAM_CHAT_ID=-1001234567890
```

Где `VELUM_TELEGRAM_CHAT_ID` — ID чата или канала для уведомлений.

## Возможности

- Уведомления о запуске и завершении задач
- Уведомления об ошибках
- Команды управления через бота
- Статус сервера

---

## Следующие шаги

- [Конфигурация](../getting-started/configuration.md) — переменные окружения
- [Режим отладки](../troubleshooting/debug-mode.md) — отладка бота
- [Типовые проблемы](../troubleshooting/common-issues.md) — частые проблемы
