# DEB-пакет

> Установка Velum из Debian-пакета
>
> 📖 См. также: [Быстрый старт](../getting-started/quick-start.md), [Docker](../deployment/docker-deployment.md), [Релизы](./releases.md)

---

## Установка

Скачайте последнюю версию со [страницы релизов](https://github.com/alexandervashurin/semaphore/releases):

```bash
sudo dpkg -i velum_*.deb
sudo apt-get install -f  # установка зависимостей
```

## Конфигурация

После установки настройте переменные окружения в `/etc/velum/env`:

```bash
VELUM_DB_URL=postgres://user:pass@localhost:5432/velum
VELUM_DB_DIALECT=postgres
VELUM_JWT_SECRET=your-secret-key-32-bytes-long!!
VELUM_WEB_PATH=/usr/share/velum/web/public
```

## Запуск

```bash
sudo systemctl start velum
sudo systemctl enable velum
```

---

## Следующие шаги

- [Быстрый старт](../getting-started/quick-start.md) — запуск Velum
- [Конфигурация](../getting-started/configuration.md) — переменные окружения
- [Релизы](./releases.md) — страница релизов
