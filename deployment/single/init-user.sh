#!/bin/bash
# Скрипт инициализации Semaphore UI
# Создание初始льного пользователя admin

set -e

echo "=== Инициализация Semaphore UI ==="

# Ждем запуска сервера
echo "Ожидание запуска сервера..."
sleep 5

# Регистрация初始льного пользователя
echo "Создание пользователя admin..."
curl -s -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "name": "Admin",
    "password": "Password123!",
    "username": "admin"
  }' || echo "Пользователь уже существует или ошибка регистрации"

echo "=== Готово ==="
echo "Веб-интерфейс: http://localhost:80"
echo "Логин: admin@example.com"
echo "Пароль: Password123!"
