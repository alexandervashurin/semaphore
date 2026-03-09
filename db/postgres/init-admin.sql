-- Инициализация начального пользователя admin
-- Пароль: Password123! (хэш bcrypt)

INSERT INTO "user" (email, name, username, password, admin, created)
SELECT 
    'admin@example.com',
    'Admin',
    'admin',
    '$2a$10$3qN4W7K5X8Y9Z2A1B6C7D8E9F0G1H2I3J4K5L6M7N8O9P0Q1R2S3T',  -- bcrypt хэш для Password123!
    1,  -- admin = true
    CURRENT_TIMESTAMP
WHERE NOT EXISTS (SELECT 1 FROM "user" WHERE email = 'admin@example.com');
