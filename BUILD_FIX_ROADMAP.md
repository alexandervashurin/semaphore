# План исправления ошибок компиляции Semaphore Rust

**Дата создания:** 2026-03-02  
**Текущее количество ошибок:** 491  
**Цель:** 0 ошибок

---

## 📊 Статистика ошибок (по категориям)

| Код ошибки | Количество | Описание |
|------------|------------|----------|
| E0282 | 62 | type annotations needed |
| E0308 | 49 | mismatched types |
| E0283 | 36 | type annotations needed |
| E0599 | 30 | no method named `bucket` |
| E0277 | 22 | the size for values of type `[u8]` cannot be known |
| E0599 | 11 | no method named `create_bucket_if_not_exists` |
| E0277 | 8 | can't compare `{integer}` with `Option<usize>` |
| E0061 | 5 | this method takes 4 arguments but 3 were supplied |
| E0609 | 4 | no field `ssh_key` on type `DbRepository` |
| E0609 | 4 | no field `name` on type `&std::string::String` |
| E0599 | 4 | `Option<TemplateType>` doesn't implement `Display` |
| E0599 | 4 | no method named `get_object_refs` |
| E0599 | 4 | no method named `delete_bucket` |
| E0433 | 4 | failed to resolve: use of unresolved crate `libc` |
| E0277 | 4 | trait bound `dyn Any + Send + Sync: Clone` not satisfied |
| E0061 | 4 | this method takes 1 argument but 2 were supplied |

---

## 🎯 Приоритеты исправлений

### 🔴 КРИТИЧЕСКИЙ ПРИОРИТЕТ (блокируют компиляцию)

#### 1. BoltDB API несовместимость (75 ошибок)
**Проблема:** Код использует API BoltDB (bucket, create_bucket_if_not_exists), но реализация на sled.

**Файлы:**
- `src/db/bolt/event.rs` (30 ошибок bucket)
- `src/db/bolt/user.rs` (11 ошибок create_bucket_if_not_exists)
- `src/db/bolt/*.rs` (4 ошибки delete_bucket)

**Решение:**
- [ ] Переписать BoltDB слой для использования API sled
- [ ] Заменить `bucket()` на `open_tree()`
- [ ] Заменить `create_bucket_if_not_exists()` на соответствующие методы sled
- [ ] Заменить `delete_bucket()` на `remove_tree()`

#### 2. Missing Store methods (25 ошибок)
**Проблема:** Отсутствуют методы в Store trait и реализациях.

**Отсутствующие методы:**
- [ ] `get_project_users` - добавить в UserManager (частично выполнено)
- [ ] `get_object_refs` - добавить в Store
- [ ] `get_repository`, `update_repository`, `delete_repository`
- [ ] `get_inventory`, `update_inventory`, `delete_inventory`
- [ ] `get_environment`, `update_environment`, `delete_environment`
- [ ] `get_tasks`, `create_task`, `get_task`, `delete_task`
- [ ] `get_runners`, `create_runner`, `update_runner`, `delete_runner`
- [ ] `get_options`, `set_option`
- [ ] `get_project_schedules`
- [ ] `get_template_users`
- [ ] `get_task_alert_chat`
- [ ] `create_user_without_password`
- [ ] `update_repository`
- [ ] `create_inventory`
- [ ] `create_environment`

#### 3. Type annotations (98 ошибок)
**Проблема:** Компилятор не может вывести типы в цепочках вызовов.

**Решение:**
- [ ] Добавить явные аннотации типов для closure в `.map_err()`
- [ ] Использовать турбофис-нотацию для методов с generic параметрами
- [ ] Добавить типы для промежуточных переменных

#### 4. Missing fields in models (30 ошибок)
**Проблема:** Код обращается к несуществующим полям.

**Отсутствующие поля:**
- [ ] `ssh_key` в `DbRepository` → использовать `key_id` и загружать AccessKey отдельно
- [ ] `name` в различных структурах
- [ ] `secret_type`, `secret` в строках
- [ ] `path` в `DbConfig`
- [ ] `override_secret` в `AccessKey`
- [ ] `ha` в `HAConfig`
- [ ] `ssh_key_id` в `Repository`
- [ ] `project_id` в `IntegrationAlias`
- [ ] `login_password`, `key_type`, `access_key` в `AccessKey`
- [ ] `connection_string` в `DbConfig`
- [ ] `view_id` в `TemplateFilter`
- [ ] `variables` в `Inventory`
- [ ] `hooks`, `params` в `Template`
- [ ] `backend_init_required`, `backend_config`, `workspace` в `TerraformTaskParams`
- [ ] `stage_type`, `project_id` в `TaskOutput`
- [ ] `created` в `Schedule`, `AccessKey`, `APIToken`

#### 5. Missing trait implementations (20 ошибок)
**Проблема:** Типы не реализуют требуемые трейты.

**Требуется реализовать:**
- [ ] `Clone` для `TaskLogger`, `RunningTask`, `AccessKeyInstallerImpl`
- [ ] `DataExporter` для `ExporterChain`
- [ ] `TypeExporter` для `ValueMap<T>`
- [ ] `LocalApp` для `AnsibleApp`, `TerraformApp`
- [ ] `Job` для `LocalJob`
- [ ] `Default` для `HARedisConfig`
- [ ] `FromRow` для `SecretStorage`
- [ ] `Type`, `Decode` для `Task`, `AccessKeyOwner`, `ProjectInvite`, `TemplateType`, `ProjectUserRole`
- [ ] `FromStr` для `TemplateType`, `AccessKeyOwner`
- [ ] `Display` для `Option<TemplateType>`, `Option<usize>`, `Option<String>`

#### 6. Missing crate dependencies (6 ошибок)
**Проблема:** Используются крейты, не добавленные в Cargo.toml.

**Решение:**
- [ ] Добавить `libc = "0.2"` в зависимости
- [ ] Добавить `which = "4"` в зависимости

---

### 🟡 ВЫСОКИЙ ПРИОРИТЕТ

#### 7. Git Client issues (10 ошибок)
**Проблема:** Неправильное использование GitRepository.

**Файлы:**
- `src/db_lib/go_git_client.rs`
- `src/db_lib/cmd_git_client.rs`

**Решение:**
- [ ] Исправить обращение к `repo.repository.ssh_key` → загрузка через Store
- [ ] Добавить метод `get_full_path()` для `Repository`
- [ ] Исправить lifetime параметры

#### 8. Method signature mismatches (15 ошибок)
**Проблема:** Несоответствие сигнатур методов.

**Решение:**
- [ ] Исправить `Job::run` - 4 параметра вместо 3
- [ ] Исправить методы Store - правильное количество параметров
- [ ] Исправить `extract_params()`, `validate()` для Template и AccessKey

#### 9. SQLx type compatibility (15 ошибок)
**Проблема:** Типы не совместимы с SQLx Encode/Decode.

**Решение:**
- [ ] Реализовать `Type`, `Encode`, `Decode` для кастомных типов
- [ ] Исправить `HashMap` → `serde_json::Value` где нужно
- [ ] Исправить `Option` типы в запросах

---

### 🟢 СРЕДНИЙ ПРИОРИТЕТ

#### 10. Async/await issues (5 ошибок)
**Проблема:** Синхронные методы вызывают асинхронные.

**Решение:**
- [ ] Сделать `restore()` async
- [ ] Исправить вызовы async методов в синхронном контексте

#### 11. Backup/Restore field mismatches (10 ошибок)
**Проблема:** Поля Backup* структур не соответствуют моделям.

**Решение:**
- [ ] Обновить `BackupTemplate` поля
- [ ] Обновить `BackupRepository` поля
- [ ] Обновить `BackupProject` поля
- [ ] Исправить инициализацию в restore.rs

#### 12. Config issues (5 ошибок)
**Проблема:** Обращение к методам как к полям.

**Решение:**
- [ ] Исправить `config.db_path` → `config.db_path()`
- [ ] Исправить `config.db_dialect` → `config.db_dialect()`
- [ ] Исправить `config.non_admin_can_create_project` → метод

---

## 📋 План работ по этапам

### Этап 1: BoltDB API (8-10 часов)
- [ ] 1.1 Переписать event.rs на sled API
- [ ] 1.2 Переписать user.rs на sled API
- [ ] 1.3 Переписать остальные файлы BoltDB

### Этап 2: Store methods (6-8 часов)
- [ ] 2.1 Добавить missing методы в Store trait
- [ ] 2.2 Реализовать методы в SqlStore
- [ ] 2.3 Реализовать методы в BoltStore

### Этап 3: Type annotations (4-6 часов)
- [ ] 3.1 Исправить аннотации в API handlers
- [ ] 3.2 Исправить аннотации в сервисах
- [ ] 3.3 Исправить аннотации в DB слое

### Этап 4: Model fields (4-6 часов)
- [ ] 4.1 Добавить missing поля в модели
- [ ] 4.2 Исправить использование полей в коде

### Этап 5: Trait implementations (4-6 часов)
- [ ] 5.1 Реализовать Clone где нужно
- [ ] 5.2 Реализовать SQLx трейты
- [ ] 5.3 Реализовать остальные трейты

### Этап 6: Dependencies и прочее (2-3 часа)
- [ ] 6.1 Добавить libc и which в Cargo.toml
- [ ] 6.2 Исправить Git client
- [ ] 6.3 Исправить Config обращения

### Этап 7: Финальная полировка (2-3 часа)
- [ ] 7.1 Исправить оставшиеся ошибки
- [ ] 7.2 Удалить предупреждения
- [ ] 7.3 Запустить тесты

---

## 📈 Прогресс

| Этап | Статус | Исправлено ошибок | Осталось |
|------|--------|-------------------|----------|
| Начало | - | 0 | 491 |
| После коммита 1 | ✅ Завершено | 50 | 441 |
| После коммита 2 | ✅ Завершено | 22 | 419 |
| После коммита 3 | ✅ Завершено | 2 | 417 |
| **Текущий прогресс** | 🟡 В процессе | **74** | **417** |

**Исправлено ошибок:** 74 из 491 (15%)

---

## 🔧 Полезные команды

```bash
# Проверка количества ошибок
cargo build 2>&1 | grep -E "^error\[E" | wc -l

# Группировка ошибок по типам
cargo build 2>&1 | grep -E "^error\[E" | sort | uniq -c | sort -rn

# Просмотр конкретных ошибок
cargo build 2>&1 | grep -A 3 "error\[E0282\]"

# Запуск clippy для поиска дополнительных проблем
cargo clippy --all-targets

# Форматирование кода
cargo fmt
```

---

## 📝 Заметки

1. **BoltDB vs sled:** Основная проблема - код написан под API BoltDB, но используется sled. Требуется значительная переработка.

2. **Store trait:** Многие методы отсутствуют в trait и реализациях. Нужно добавить систематически.

3. **Type inference:** Rust компилятор иногда требует явных аннотаций в сложных цепочках вызовов.

4. **Архитектурные изменения:** Некоторые исправления требуют изменения архитектуры (например, загрузка AccessKey через Store вместо прямого доступа).

---

**Последнее обновление:** 2026-03-02  
**Следующий шаг:** Исправление BoltDB API (этап 1)
