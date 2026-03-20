#!/bin/bash
# =============================================================================
# Скрипт создания DEB пакета для Velum (Rust)
# =============================================================================
# Использование: ./scripts/build-deb.sh [версия]
# Пример: ./scripts/build-deb.sh 2.1.0
# =============================================================================

set -e

# Версия из аргумента или git тега
VERSION="${1:-$(git describe --tags --always | sed 's/^v//')}"
ARCHITECTURE="amd64"
PACKAGE_NAME="velum"
MAINTAINER="Alexander Vashurin <78410670+alexandervashurin@users.noreply.github.com>"
DESCRIPTION="Velum - Ansible, Terraform, OpenTofu web interface (Rust edition)"
HOMEPAGE="https://github.com/tnl-o/semarust"
LICENSE="MIT"

# Пути
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
RUST_DIR="$ROOT_DIR/rust"
BUILD_DIR="$ROOT_DIR/build/deb"
PACKAGE_DIR="$BUILD_DIR/$PACKAGE_NAME-$VERSION"
DEBIAN_DIR="$PACKAGE_DIR/DEBIAN"

# Цвета
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# =============================================================================
# Подготовка
# =============================================================================

log_info "Создание DEB пакета: $PACKAGE_NAME $VERSION ($ARCHITECTURE)"
echo ""

# Проверка зависимостей
check_command() {
    if ! command -v "$1" &> /dev/null; then
        log_error "$1 не найден. Установите: $2"
        exit 1
    fi
}

check_command cargo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
check_command dpkg-deb "apt install dpkg-dev"
check_command fakeroot "apt install fakeroot"

# Очистка
log_info "Очистка предыдущих сборок..."
rm -rf "$BUILD_DIR"
mkdir -p "$PACKAGE_DIR"
mkdir -p "$DEBIAN_DIR"

# =============================================================================
# Сборка бинарного файла
# =============================================================================

log_info "Сборка бинарного файла..."
cd "$RUST_DIR"
cargo build --release

if [ ! -f "$RUST_DIR/target/release/semaphore" ]; then
    log_error "Бинарный файл не найден после сборки"
    exit 1
fi

log_success "Бинарный файл собран"

# =============================================================================
# Создание структуры пакета
# =============================================================================

log_info "Создание структуры пакета..."

# Директории
mkdir -p "$PACKAGE_DIR/usr/bin"
mkdir -p "$PACKAGE_DIR/usr/lib/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR/usr/share/doc/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR/usr/share/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR/etc/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR/var/lib/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR/var/log/$PACKAGE_NAME"
mkdir -p "$PACKAGE_DIR/lib/systemd/system"

# Копирование бинарного файла
cp "$RUST_DIR/target/release/semaphore" "$PACKAGE_DIR/usr/bin/$PACKAGE_NAME"
chmod 755 "$PACKAGE_DIR/usr/bin/$PACKAGE_NAME"

# Копирование веб-файлов
if [ -d "$ROOT_DIR/web/public" ]; then
    cp -r "$ROOT_DIR/web/public" "$PACKAGE_DIR/usr/share/$PACKAGE_NAME/web"
fi

# Копирование документации
cp "$ROOT_DIR/README.md" "$PACKAGE_DIR/usr/share/doc/$PACKAGE_NAME/" 2>/dev/null || true
cp "$ROOT_DIR/CHANGELOG.md" "$PACKAGE_DIR/usr/share/doc/$PACKAGE_NAME/" 2>/dev/null || true
cp "$ROOT_DIR/LICENSE" "$PACKAGE_DIR/usr/share/doc/$PACKAGE_NAME/" 2>/dev/null || true

# =============================================================================
# Создание control файла
# =============================================================================

log_info "Создание control файла..."

cat > "$DEBIAN_DIR/control" << EOF
Package: $PACKAGE_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCHITECTURE
Depends: systemd, postgresql-client | mysql-client, ansible (>= 2.9)
Recommends: docker.io, docker-compose
Suggests: nginx
Maintainer: $MAINTAINER
Description: $DESCRIPTION
 Velum - это веб-интерфейс для запуска Ansible, Terraform, OpenTofu
 и других DevOps инструментов.
 .
 Особенности:
  - Управление проектами и шаблонами
  - Запуск playbook с веб-интерфейса
  - Интеграция с Git репозиториями
  - Расписания (cron)
  - Webhooks и интеграции
  - Audit log
  - Prometheus метрики
 .
 Этот пакет содержит бинарный файл и systemd сервис.
Homepage: $HOMEPAGE
License: $LICENSE
EOF

# =============================================================================
# Создание postinst скрипта
# =============================================================================

log_info "Создание postinst скрипта..."

cat > "$DEBIAN_DIR/postinst" << 'POSTINST'
#!/bin/bash
set -e

case "$1" in
    configure)
        # Создание пользователя если не существует
        if ! getent group velum > /dev/null; then
            groupadd --system velum
        fi
        
        if ! getent passwd velum > /dev/null; then
            useradd --system \
                --gid velum \
                --home-dir /var/lib/velum \
                --no-create-home \
                --shell /usr/sbin/nologin \
                velum
        fi
        
        # Установка прав
        chown -R velum:velum /var/lib/velum
        chown -R velum:velum /var/log/velum
        chmod 750 /var/lib/velum
        chmod 750 /var/log/velum
        
        # Включение и запуск сервиса
        systemctl daemon-reload
        systemctl enable velum.service 2>/dev/null || true
        
        # Попытка запустить сервис
        if systemctl is-active --quiet velum.service 2>/dev/null; then
            echo "Velum уже запущен"
        else
            systemctl start velum.service 2>/dev/null || true
        fi
        
        echo ""
        echo "✓ Velum установлен успешно"
        echo ""
        echo "Конфигурация: /etc/velum/config.json"
        echo "Логи: journalctl -u velum.service"
        echo ""
        echo "Для создания admin пользователя:"
        echo "  velum user add --username admin --email admin@example.com --password YOUR_PASSWORD --admin"
        echo ""
        ;;
esac

exit 0
POSTINST

chmod 755 "$DEBIAN_DIR/postinst"

# =============================================================================
# Создание prerm скрипта
# =============================================================================

log_info "Создание prerm скрипта..."

cat > "$DEBIAN_DIR/prerm" << 'PRERM'
#!/bin/bash
set -e

case "$1" in
    remove|deconfigure)
        # Остановка сервиса
        systemctl stop velum.service 2>/dev/null || true
        systemctl disable velum.service 2>/dev/null || true
        systemctl daemon-reload
        ;;
esac

exit 0
PRERM

chmod 755 "$DEBIAN_DIR/prerm"

# =============================================================================
# Создание postrm скрипта
# =============================================================================

log_info "Создание postrm скрипта..."

cat > "$DEBIAN_DIR/postrm" << 'POSTRM'
#!/bin/bash
set -e

case "$1" in
    remove)
        echo "Velum удалён. Данные сохранены в /var/lib/velum"
        echo "Для полного удаления данных:"
        echo "  rm -rf /var/lib/velum /var/log/velum /etc/velum"
        ;;
    purge)
        # Полное удаление
        rm -rf /var/lib/velum
        rm -rf /var/log/velum
        rm -rf /etc/velum
        
        # Удаление пользователя
        userdel velum 2>/dev/null || true
        groupdel velum 2>/dev/null || true
        ;;
esac

exit 0
POSTRM

chmod 755 "$DEBIAN_DIR/postrm"

# =============================================================================
# Создание systemd сервиса
# =============================================================================

log_info "Создание systemd сервиса..."

cat > "$PACKAGE_DIR/lib/systemd/system/velum.service" << 'SYSTEMD'
[Unit]
Description=Velum (Rust) - Ansible, Terraform web interface
Documentation=https://github.com/tnl-o/semarust
After=network.target postgresql.service mysql.service
Wants=network-online.target

[Service]
Type=simple
User=velum
Group=velum
ExecStart=/usr/bin/velum server --config /etc/velum/config.json
Restart=on-failure
RestartSec=5
Environment="HOME=/var/lib/velum"
Environment="SEMAPHORE_TMP_PATH=/tmp/velum"

# Директории
ReadWritePaths=/var/lib/velum /var/log/velum
StateDirectory=velum
LogsDirectory=velum

# Security
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
CapabilityBoundingSet=CAP_NET_BIND_SERVICE
AmbientCapabilities=
RestrictAddressFamilies=AF_INET AF_INET6 AF_UNIX
RestrictNamespaces=true
RestrictRealtime=true
RestrictSUIDSGID=true
MemoryDenyWriteExecute=true
LockPersonality=true

# Limits
LimitNOFILE=65536
LimitNPROC=1024

[Install]
WantedBy=multi-user.target
SYSTEMD

chmod 644 "$PACKAGE_DIR/lib/systemd/system/velum.service"

# =============================================================================
# Создание конфигурации по умолчанию
# =============================================================================

log_info "Создание конфигурации по умолчанию..."

cat > "$PACKAGE_DIR/etc/velum/config.json" << 'CONFIG'
{
  "dialect": "sqlite",
  "path": "/var/lib/velum/database.db",
  "addr": ":3000",
  "web_host": "/usr/share/velum/web",
  "tmp_path": "/tmp/velum",
  "admin": {
    "name": "admin",
    "email": "admin@localhost"
  },
  "auto_backup": {
    "enabled": true,
    "interval_hours": 24,
    "backup_path": "/var/lib/velum/backups",
    "max_backups": 7,
    "compress": true
  }
}
CONFIG

chmod 640 "$PACKAGE_DIR/etc/velum/config.json"

# =============================================================================
# Создание conffiles
# =============================================================================

log_info "Создание conffiles..."

echo "/etc/velum/config.json" > "$DEBIAN_DIR/conffiles"

# =============================================================================
# Создание md5sums
# =============================================================================

log_info "Создание md5sums..."

cd "$PACKAGE_DIR"
find . -type f ! -path "./DEBIAN/*" -exec md5sum {} \; > "$DEBIAN_DIR/md5sums"

# =============================================================================
# Сборка пакета
# =============================================================================

log_info "Сборка DEB пакета..."
cd "$BUILD_DIR"

fakeroot dpkg-deb --build "$PACKAGE_NAME-$VERSION"

# Проверка пакета
log_info "Проверка пакета..."
dpkg-deb --info "$PACKAGE_NAME-$VERSION.deb"
dpkg-deb --field "$PACKAGE_NAME-$VERSION.deb"

# =============================================================================
# Результат
# =============================================================================

echo ""
log_success "DEB пакет создан успешно!"
echo ""
echo "📦 Пакет: $BUILD_DIR/$PACKAGE_NAME-$VERSION.deb"
echo "📊 Размер: $(du -h "$BUILD_DIR/$PACKAGE_NAME-$VERSION.deb" | cut -f1)"
echo ""
echo "Установка:"
echo "  sudo dpkg -i $BUILD_DIR/$PACKAGE_NAME-$VERSION.deb"
echo ""
echo "Проверка:"
echo "  dpkg -l | grep $PACKAGE_NAME"
echo "  systemctl status velum"
echo ""
echo "Удаление:"
echo "  sudo apt remove $PACKAGE_NAME"
echo "  sudo apt purge $PACKAGE_NAME"
echo ""
