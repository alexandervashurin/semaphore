# DEB Package для Velum (Rust)

## 📦 Создание пакета

### Требования

- **Rust** 1.75+ (для сборки)
- **dpkg-deb** (`apt install dpkg-dev`)
- **fakeroot** (`apt install fakeroot`)
- **Git** (для определения версии)

### Быстрая сборка

```bash
# Собрать пакет с версией из git тега
./scripts/build-deb.sh

# Или указать версию явно
./scripts/build-deb.sh 2.1.0
```

### Результат

```
📦 Пакет: build/deb/velum-2.1.0.deb
📊 Размер: ~6.8M
```

---

## 📋 Структура пакета

```
velum-2.1.0.deb
├── DEBIAN/
│   ├── control          # Метаданные пакета
│   ├── postinst         # Скрипт после установки
│   ├── prerm            # Скрипт перед удалением
│   ├── postrm           # Скрипт после удаления
│   ├── conffiles        # Конфигурационные файлы
│   └── md5sums          # Контрольные суммы
├── usr/
│   ├── bin/velum        # Бинарный файл
│   ├── lib/velum/       # Библиотеки
│   └── share/velum/     # Веб-файлы, документация
├── etc/velum/
│   └── config.json      # Конфигурация по умолчанию
├── var/lib/velum/       # Данные приложения
├── var/log/velum/       # Логи
└── lib/systemd/system/
    └── velum.service    # Systemd сервис
```

---

## 🚀 Установка

### 1. Установка пакета

```bash
sudo dpkg -i build/deb/velum-2.1.0.deb
```

Если есть зависимости:

```bash
sudo apt install -f
```

### 2. Проверка установки

```bash
# Версия
velum --version

# Статус сервиса
systemctl status velum

# Журнал
journalctl -u velum -f
```

### 3. Создание admin пользователя

```bash
sudo velum user add \
  --username admin \
  --email admin@example.com \
  --password admin123 \
  --admin
```

### 4. Настройка конфигурации

Отредактируйте `/etc/velum/config.json`:

```json
{
  "dialect": "postgres",
  "host": "localhost",
  "port": 5432,
  "name": "velum",
  "user": "velum",
  "pass": "velum_password",
  "addr": ":3000",
  "web_host": "/usr/share/velum/web"
}
```

### 5. Перезапуск сервиса

```bash
sudo systemctl restart velum
sudo systemctl enable velum
```

---

## 🔧 Управление сервисом

```bash
# Запуск
sudo systemctl start velum

# Остановка
sudo systemctl stop velum

# Перезапуск
sudo systemctl restart velum

# Перезагрузка конфигурации
sudo systemctl reload velum

# Статус
systemctl status velum

# Автозагрузка
sudo systemctl enable velum

# Отключение автозагрузки
sudo systemctl disable velum
```

---

## 📁 Расположение файлов

| Файл/Директория | Описание |
|-----------------|----------|
| `/usr/bin/velum` | Бинарный файл |
| `/etc/velum/config.json` | Конфигурация |
| `/var/lib/velum/` | Данные (БД, backups) |
| `/var/log/velum/` | Логи |
| `/usr/share/velum/web/` | Веб-интерфейс |
| `/lib/systemd/system/velum.service` | Systemd сервис |

---

## 🗑️ Удаление

### Удаление пакета (данные сохраняются)

```bash
sudo apt remove velum
```

### Полное удаление (с данными)

```bash
sudo apt purge velum
sudo rm -rf /var/lib/velum /var/log/velum /etc/velum
```

---

## 🔍 Решение проблем

### Сервис не запускается

```bash
# Проверка логов
journalctl -u velum -f

# Проверка конфигурации
velum --config /etc/velum/config.json

# Проверка прав
ls -la /etc/velum/
ls -la /var/lib/velum/
```

### Конфликт порта

Измените порт в `/etc/velum/config.json`:

```json
{
  "addr": ":3001"
}
```

Перезапустите:

```bash
sudo systemctl restart velum
```

### Ошибки БД

```bash
# Проверка подключения к PostgreSQL
psql -h localhost -U velum -d velum -c "SELECT 1"

# Применение миграций
velum migrate --config /etc/velum/config.json
```

---

## 📊 Зависимости

### Обязательные

- `systemd` - управление сервисом
- `postgresql-client` или `mysql-client` - CLI для БД
- `ansible` (>= 2.9) - для запуска playbook

### Рекомендуемые

- `docker.io` - для запуска в контейнерах
- `docker-compose` - для оркестрации контейнеров

### Опциональные

- `nginx` - reverse proxy

---

## 🔐 Безопасность

### Права доступа

Пакет создаёт:
- Пользователя `velum` (system user)
- Группу `velum` (system group)

Сервис запускается от имени пользователя `velum` с ограниченными правами.

### Security ограничения

Systemd сервис включает:
- `NoNewPrivileges=true`
- `PrivateTmp=true`
- `ProtectSystem=strict`
- `ProtectHome=true`
- `RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX`
- `MemoryDenyWriteExecute=true`

### Firewall

Откройте необходимые порты:

```bash
# HTTP
sudo ufw allow 3000/tcp

# HTTPS (если используете reverse proxy)
sudo ufw allow 443/tcp
```

---

## 📈 Обновление

### Автоматическое обновление

```bash
sudo apt update
sudo apt install --reinstall velum
```

### Ручное обновление

```bash
# Скачать новый пакет
wget https://github.com/tnl-o/semarust/releases/download/v2.1.0/velum-2.1.0.deb

# Установить
sudo dpkg -i velum-2.1.0.deb

# Перезапустить
sudo systemctl restart velum
```

---

## 🏗️ Сборка из исходников

### Требования для сборки

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Зависимости для dpkg
sudo apt install dpkg-dev fakeroot build-essential
```

### Сборка

```bash
# Клонирование репозитория
git clone https://github.com/tnl-o/semarust.git
cd semarust

# Сборка пакета
./scripts/build-deb.sh 2.1.0
```

---

## 📚 Документация

- [README.md](../README.md) - основная документация
- [CONFIG.md](../CONFIG.md) - конфигурация
- [DEPLOYMENT.md](DEPLOYMENT.md) - развёртывание
- [START_SERVER.md](../START_SERVER.md) - запуск сервера

---

## 🐛 Отчёт об ошибках

https://github.com/tnl-o/semarust/issues

---

*Последнее обновление: 20 марта 2026 г.*
