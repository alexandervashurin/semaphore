# 🐛 Исправление: Обновление проекта (UPDATE)

## Проблема

При попытке обновить проект через API или UI возникала ошибка:

```
Failed to deserialize the JSON body into the target type: missing field `created`
```

## Причина

Обработчик `update_project` в файле `rust/src/api/handlers/projects/project.rs` принимал полную модель `Project`:

```rust
pub async fn update_project(
    ...
    Json(payload): Json<Project>,  // ❌ Требует все поля включая `created`
) -> ...
```

Модель `Project` требует все поля:
- `id`
- `created` (дата создания)
- `name`
- `alert`
- `max_parallel_tasks`
- `type`
- etc.

Это означало, что для обновления нужно было отправлять ВСЕ поля проекта, включая `created`, что нелогично.

## Решение

### 1. Создан отдельный Payload для обновления

```rust
/// Payload для обновления проекта
#[derive(Debug, Deserialize)]
pub struct UpdateProjectPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_chat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_parallel_tasks: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_secret_storage_id: Option<i32>,
}
```

**Преимущества:**
- Все поля опциональны (`Option<T>`)
- Можно обновлять только нужные поля
- Не нужно отправлять `id` и `created`

### 2. Обновлена логика обработчика

```rust
pub async fn update_project(
    State(state): State<Arc<AppState>>,
    Path(project_id): Path<i32>,
    Json(payload): Json<UpdateProjectPayload>,  // ✅ Partial update
) -> std::result::Result<StatusCode, ...> {
    // Получаем текущий проект
    let mut project = state.store.get_project(project_id).await?;

    // Обновляем только указанные поля
    if let Some(name) = payload.name {
        project.name = name;
    }
    if let Some(alert) = payload.alert {
        project.alert = alert;
    }
    // ... и так далее для всех полей

    state.store.update_project(project).await?;
    Ok(StatusCode::OK)
}
```

### 3. Обновлён JavaScript frontend

Файл `web/public/demo-crud.js`:

```javascript
async function updateProject(projectId, projectData) {
    try {
        // Отправляем только изменённые поля (API теперь поддерживает partial update)
        await apiRequest(`/projects/${projectId}`, {
            method: 'PUT',
            body: JSON.stringify(projectData),
        });
        // ...
    }
}
```

## Примеры использования

### ✅ Обновление одного поля

```bash
curl -X PUT http://localhost:3000/api/projects/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "New Project Name"}'
```

### ✅ Обновление нескольких полей

```bash
curl -X PUT http://localhost:3000/api/projects/1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "Updated Project",
    "alert": true,
    "max_parallel_tasks": 10
  }'
```

### ✅ Через UI

Теперь в CRUD демо можно:
1. Нажать "✏️" на проекте
2. Изменить название
3. Нажать "Сохранить"
4. ✅ Проект обновится без ошибок

## Тестирование

```bash
# Запуск полного теста CRUD
./test-full-crud.sh

# Результат:
✅ CREATE - Создание проекта
✅ READ - Получение проекта (по ID и списка)
✅ UPDATE - Обновление проекта (partial update)
✅ DELETE - Удаление проекта
```

## Изменённые файлы

| Файл | Изменения |
|------|-----------|
| `rust/src/api/handlers/projects/project.rs` | ✅ Добавлен `UpdateProjectPayload`<br>✅ Обновлена функция `update_project` |
| `web/public/demo-crud.js` | ✅ Обновлена функция `updateProject` |

## Статус

✅ Исправление применено  
✅ Все CRUD операции работают  
✅ Тесты пройдены  
✅ UI редактирование работает

## Примечание

Это исправление следует тому же принципу, что и `CreateProjectPayload` - разделение моделей для создания и обновления. Это распространённая практика в API дизайне.
