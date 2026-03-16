-- ============================================================================
-- Миграция 004: Добавление missing колонок для secret storage
-- ============================================================================
-- Дата: 2026-03-16
-- Описание: Добавляет отсутствующие колонки в таблицы environment, inventory,
--           access_key и project для поддержки external secret storages
-- ============================================================================

-- Добавляем колонку secret_storage_id в таблицу environment
ALTER TABLE environment 
ADD COLUMN IF NOT EXISTS secret_storage_id INTEGER;

-- Добавляем колонку secret_storage_key_prefix в таблицу environment
ALTER TABLE environment 
ADD COLUMN IF NOT EXISTS secret_storage_key_prefix VARCHAR(255);

-- Добавляем колонку extra_vars в таблицу inventory (если отсутствует)
ALTER TABLE inventory 
ADD COLUMN IF NOT EXISTS extra_vars TEXT;

-- Добавляем колонку vaults в таблицу inventory (если отсутствует)
ALTER TABLE inventory 
ADD COLUMN IF NOT EXISTS vaults TEXT;

-- Добавляем колонку created в таблицу environment (если отсутствует)
ALTER TABLE environment 
ADD COLUMN IF NOT EXISTS created TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Добавляем колонку created в таблицу inventory (если отсутствует)
ALTER TABLE inventory 
ADD COLUMN IF NOT EXISTS created TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Добавляем колонку alert_chat в таблицу project (если отсутствует)
ALTER TABLE project 
ADD COLUMN IF NOT EXISTS alert_chat VARCHAR(255);

-- Добавляем колонку runner_tag в таблицу inventory (если отсутствует)
ALTER TABLE inventory 
ADD COLUMN IF NOT EXISTS runner_tag VARCHAR(255);

-- Обновляем запись о миграции
INSERT INTO migration (version, name) 
VALUES (4, 'Add secret storage columns')
ON CONFLICT (version) DO NOTHING;

-- Вывод сообщения об успехе
DO $$
BEGIN
    RAISE NOTICE 'Миграция 004 успешно применена: добавлены колонки для secret storage';
END $$;
