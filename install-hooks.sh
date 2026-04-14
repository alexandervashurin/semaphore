#!/usr/bin/env bash
# install-hooks.sh
# Устанавливает хуки из директории hooks/ в .git/hooks/

set -euo pipefail

HOOKS_DIR=".git/hooks"
SOURCE_DIR="hooks" # Положи все скрипты выше в папку hooks/

[ -d "$SOURCE_DIR" ] || { echo "❌ Директория $SOURCE_DIR не найдена"; exit 1; }

echo "🔧 Установка git-хуков..."

for hook in "$SOURCE_DIR"/*; do
  hook_name=$(basename "$hook")
  target="$HOOKS_DIR/$hook_name"
  
  # Делаем исполняемым
  chmod +x "$hook"
  
  # Создаём симлинк (или копию, если симлинки не поддерживаются)
  if [ -L "$target" ] || [ -f "$target" ]; then
    rm -f "$target"
  fi
  ln -sf "$(cd "$(dirname "$hook")" && pwd)/$hook_name" "$target"
  echo "✅ Установлен: $hook_name"
done

echo "🎉 Все хуки установлены. Можешь коммитить!"