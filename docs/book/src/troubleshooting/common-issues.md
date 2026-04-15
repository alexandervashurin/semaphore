# Устранение проблем

> Частые проблемы и решения
>
> 📖 См. также: [Режим отладки](./debug-mode.md), [Руководство по миграции](./migration.md), [Конфигурация](../getting-started/configuration.md)

---

## Сервер не запускается

**Проверьте логи:**
```bash
RUST_LOG=debug ./target/release/velum server
```

**Частые причины:**
- База данных недоступна — проверьте `VELUM_DB_URL`
- Порт уже занят — измените флаг `--port`
- Отсутствуют веб-файлы — проверьте `VELUM_WEB_PATH`

## Не удаётся войти

**Сброс пароля администратора:**
```bash
VELUM_ADMIN_PASSWORD=newpassword ./target/release/velum server
```

## Задачи завершаются с ошибкой

**Проверьте логи раннера:**
```bash
RUST_LOG=velum=debug ./target/release/velum server
```

**Частые причины:**
- Отсутствует Ansible/Terraform — установите на хост раннера
- Проблемы с SSH-ключом — проверьте ключи доступа
- Репозиторий недоступен — проверьте сеть

---

## Режим отладки

Включите подробное логирование:
```bash
RUST_LOG=velum=debug,tower_http=debug
```

См. также: [Режим отладки](./debug-mode.md)

---

## Проблемы с базой данных

### Ошибки миграции

```bash
# Проверка текущей схемы
psql -U velum -d velum -c "\dt"

# Сброс (ВНИМАНИЕ: удаляет все данные)
psql -U velum -d velum -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
```

---

## Проблемы с Docker

### Контейнер не запускается

```bash
docker logs velum-server
```

### База данных не готова

```bash
docker compose -f docker-compose.yml up db -d
sleep 10  # ждём БД
docker compose -f docker-compose.yml up -d
```

---

## Следующие шаги

- [Режим отладки](./debug-mode.md) — дополнительные варианты отладки
- [Конфигурация](../getting-started/configuration.md) — переменные окружения
- [Настройка окружения](../development/dev-setup.md) — локальное окружение
