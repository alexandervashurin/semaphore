# Playbook API - План развития

## Текущий статус

✅ **Реализовано (v0.4.1)**
- Полный CRUD API для Playbook
- Поддержка SQLite/PostgreSQL/MySQL
- HTTP handlers и routes
- Интеграция с StoreWrapper
- Документация и тесты

## Ближайшие задачи (v0.4.2)

### 1. Валидация Playbook

#### 1.1 YAML валидация
- [ ] Добавить библиотеку `serde_yaml` для парсинга YAML
- [ ] Валидация синтаксиса YAML при создании/обновлении
- [ ] Проверка структуры Ansible playbook

```rust
// Пример валидации
pub fn validate_ansible_playbook(content: &str) -> Result<(), ValidationError> {
    let playbook: serde_yaml::Value = serde_yaml::from_str(content)?;
    
    // Проверка структуры
    if !playbook.is_sequence() {
        return Err(ValidationError::InvalidStructure);
    }
    
    // Проверка обязательных полей
    for play in playbook.as_sequence().unwrap() {
        if !play.get("hosts").is_some() {
            return Err(ValidationError::MissingHosts);
        }
    }
    
    Ok(())
}
```

#### 1.2 Типы playbook
- [ ] Enum для типов playbook (Ansible, Terraform, Shell)
- [ ] Специфичная валидация для каждого типа
- [ ] Разные шаблоны контента

### 2. Интеграция с Repository

#### 2.1 Загрузка из Git
- [ ] Метод `sync_from_repository()` для PlaybookManager
- [ ] Автоматическая загрузка playbook из Git репозитория
- [ ] Поддержка путей в репозитории

```rust
pub async fn sync_playbook_from_repo(
    &self,
    playbook_id: i32,
    project_id: i32,
) -> Result<Playbook> {
    // 1. Получить playbook
    let playbook = self.get_playbook(playbook_id, project_id).await?;
    
    // 2. Получить repository
    if let Some(repo_id) = playbook.repository_id {
        let repo = self.get_repository(project_id, repo_id).await?;
        
        // 3. Клонируем репозиторий во временную директорию
        let temp_dir = tempfile::tempdir()?;
        clone_repository(&repo, temp_dir.path()).await?;
        
        // 4. Читаем playbook файл
        let content = read_playbook_file(temp_dir.path(), &playbook.name)?;
        
        // 5. Обновляем playbook в БД
        self.update_playbook(playbook_id, project_id, PlaybookUpdate {
            content,
            ..Default::default()
        }).await?;
    }
    
    Ok(playbook)
}
```

#### 2.2 Синхронизация
- [ ] Endpoint `POST /api/project/{id}/playbooks/{id}/sync`
- [ ] Фоновая задача для периодической синхронизации
- [ ] Webhook при изменении в Git

### 3. Запуск Playbook

#### 3.1 Интеграция с Template
- [ ] Создание Template для Playbook
- [ ] Тип Template: `playbook`
- [ ] Параметры запуска:
  - Inventory
  - Environment variables
  - Extra vars
  - Limit hosts

```rust
pub struct PlaybookRunRequest {
    pub playbook_id: i32,
    pub inventory_id: Option<i32>,
    pub environment_id: Option<i32>,
    pub extra_vars: Option<serde_json::Value>,
    pub limit: Option<String>,
    pub tags: Option<Vec<String>>,
    pub skip_tags: Option<Vec<String>>,
}
```

#### 3.2 Endpoint для запуска
```rust
// POST /api/project/{id}/playbooks/{id}/run
pub async fn run_playbook(
    State(state): State<Arc<AppState>>,
    Path((project_id, id)): Path<(i32, i32)>,
    Json(payload): Json<PlaybookRunRequest>,
) -> Result<Json<Task>, (StatusCode, Json<ErrorResponse>)> {
    // 1. Валидация playbook
    // 2. Создание template (если нет)
    // 3. Создание задачи
    // 4. Запуск задачи
}
```

#### 3.3 История запусков
- [ ] Таблица `playbook_run` для истории
- [ ] Статистика запусков (успешно/ошибки/время)
- [ ] Endpoint для получения истории

### 4. Frontend интеграция

#### 4.1 Страница Playbooks
- [ ] Список playbook с поиском и фильтрацией
- [ ] Редактор YAML с подсветкой синтаксиса (Monaco Editor)
- [ ] Предпросмотр playbook
- [ ] Кнопка запуска playbook

#### 4.2 Формы
- [ ] Создание playbook
- [ ] Редактирование playbook
- [ ] Выбор repository (опционально)
- [ ] Настройки запуска

## Среднесрочные задачи (v0.5.0)

### 5. Продвинутые функции

#### 5.1 Шаблоны playbook
- [ ] Библиотека готовых playbook
- [ ] Импорт playbook из Ansible Galaxy
- [ ] Экспорт playbook

#### 5.2 Тестирование playbook
- [ ] Интеграция с ansible-lint
- [ ] Проверка best practices
- [ ] Статический анализ

#### 5.3 Версионирование
- [ ] История изменений playbook
- [ ] Сравнение версий (diff)
- [ ] Откат к предыдущей версии

### 6. Безопасность

#### 6.1 Проверка прав доступа
- [ ] RBAC для playbook (read/write/run)
- [ ] Аудит действий с playbook
- [ ] Ограничение на запуск для определенных пользователей

#### 6.2 Валидация контента
- [ ] Запрет опасных модулей
- [ ] Ограничение на использование sudo
- [ ] Проверка на инъекции

### 7. Производительность

#### 7.1 Кэширование
- [ ] Кэширование содержимого playbook
- [ ] Инвалидация кэша при изменении
- [ ] Redis для кэширования

#### 7.2 Оптимизация БД
- [ ] Индексы для частых запросов
- [ ] Пагинация списков playbook
- [ ] Lazy loading для больших playbook

## Долгосрочные задачи (v0.6.0+)

### 8. Расширенная аналитика

- [ ] Метрики использования playbook
- [ ] Время выполнения playbook
- [ ] Частота запусков
- [ ] Графики и дашборды

### 9. Интеграции

- [ ] Ansible Tower/AWX совместимость
- [ ] Интеграция с CI/CD системами
- [ ] Webhook при завершении запуска
- [ ] Уведомления в Telegram/Slack

### 10. Мульти-регион

- [ ] Репликация playbook между регионами
- [ ] Распределенное выполнение
- [ ] Глобальный каталог playbook

## Приоритеты

### Высокий приоритет (Q1 2026)
1. ✅ CRUD API (выполнено)
2. Валидация YAML
3. Интеграция с Repository
4. Запуск playbook через Template

### Средний приоритет (Q2 2026)
5. Frontend интеграция
6. История запусков
7. RBAC для playbook

### Низкий приоритет (Q3+ 2026)
8. Продвинутые функции
9. Аналитика
10. Мульти-регион

## Метрики успеха

- [ ] 100% тестовое покрытие API
- [ ] Документация всех endpoints
- [ ] Интеграционные тесты
- [ ] Benchmark производительности
- [ ] User feedback от демо-пользователей

## Зависимости

### Библиотеки
- `serde_yaml` - YAML парсинг
- `tempfile` - временные файлы
- `git2` - работа с Git
- `ansible-lint` (опционально) - линтинг

### Инфраструктура
- Git сервер (для repository integration)
- Redis (для кэширования)
- Ansible Controller (опционально)

---

**Последнее обновление:** 2026-03-12
**Версия:** 0.4.1
**Статус:** В разработке
