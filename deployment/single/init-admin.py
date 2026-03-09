#!/usr/bin/env python3
"""
Скрипт инициализации Semaphore UI
Создает начального пользователя admin
"""

import sqlite3
import bcrypt
import os
from datetime import datetime

DB_PATH = '/app/data/semaphore.db'
ADMIN_EMAIL = 'admin@example.com'
ADMIN_NAME = 'Admin'
ADMIN_USERNAME = 'admin'
ADMIN_PASSWORD = 'Password123!'

def init_admin_user():
    """Создает пользователя admin если он не существует"""
    
    if not os.path.exists(DB_PATH):
        print(f"База данных {DB_PATH} не найдена")
        return False
    
    try:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()
        
        # Проверяем существование пользователя
        cursor.execute("SELECT id FROM user WHERE email = ?", (ADMIN_EMAIL,))
        if cursor.fetchone():
            print(f"Пользователь {ADMIN_EMAIL} уже существует")
            conn.close()
            return True
        
        # Хэшируем пароль
        password_hash = bcrypt.hashpw(ADMIN_PASSWORD.encode('utf-8'), bcrypt.gensalt()).decode('utf-8')
        
        # Создаем пользователя (с указанием всех NOT NULL полей)
        # external=0, alert=1, pro=0 - значения по умолчанию
        cursor.execute("""
            INSERT INTO user (email, name, username, password, admin, created, external, alert, pro)
            VALUES (?, ?, ?, ?, 1, ?, 0, 1, 0)
        """, (ADMIN_EMAIL, ADMIN_NAME, ADMIN_USERNAME, password_hash, datetime.now().isoformat()))
        
        conn.commit()
        conn.close()
        
        print(f"✓ Пользователь {ADMIN_EMAIL} создан успешно!")
        print(f"  Логин: {ADMIN_EMAIL}")
        print(f"  Пароль: {ADMIN_PASSWORD}")
        
        return True
        
    except Exception as e:
        print(f"Ошибка: {e}")
        return False

if __name__ == '__main__':
    init_admin_user()
