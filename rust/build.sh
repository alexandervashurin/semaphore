#!/bin/bash

# Скрипт сборки Velum (Rust)
# Использование: ./build.sh [release|debug]

set -e

BUILD_TYPE="${1:-release}"

echo "🦀 Сборка Velum (Rust)"
echo "Тип сборки: $BUILD_TYPE"

# Проверка наличия Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust не установлен. Установите Rust: https://rustup.rs/"
    exit 1
fi

# Версия Rust
RUST_VERSION=$(rustc --version)
echo "✓ Версия Rust: $RUST_VERSION"

# Сборка
if [ "$BUILD_TYPE" = "release" ]; then
    echo "🔨 Оптимизированная сборка..."
    cargo build --release
    echo "✓ Сборка завершена!"
    echo "📦 Бинарный файл: target/release/semaphore"
    ls -lh target/release/semaphore
else
    echo "🔨 Отладочная сборка..."
    cargo build
    echo "✓ Сборка завершена!"
    echo "📦 Бинарный файл: target/debug/semaphore"
    ls -lh target/debug/semaphore
fi

echo ""
echo "📚 Документация:"
echo "  - README.md - основная документация"
echo "  - CONFIG.md - конфигурация"
echo "  - API.md - API документация"
echo "  - MIGRATION.md - миграция с Go"
