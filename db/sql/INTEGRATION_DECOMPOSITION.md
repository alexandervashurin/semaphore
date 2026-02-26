# Декомпозиция db/sql/integration.go для миграции на Rust

**Дата**: 2026-02-26  
**Статус**: 📋 План

---

## 📊 Проблема

Файл `db/sql/integration.go` содержит **340 строк кода** и включает:
- CRUD операции для интеграций
- IntegrationMatcher операции
- IntegrationExtractValue операции

---

## ✅ Решение

Разделить на **3 логических модуля**:

| Файл | Строк (Go) | Описание |
|------|------------|----------|
| `integration_crud.rs` | ~200 | CRUD операции для Integration |
| `integration_matcher.rs` | ~80 | IntegrationMatcher CRUD |
| `integration_extract.rs` | ~60 | IntegrationExtractValue CRUD |
| **ИТОГО** | **~340** | **В 3 раза меньше!** |

---

## 🔄 План миграции на Rust

### Этап 1: Integration CRUD (1-2 дня)

**Файл**: `rust/src/db/sql/integration_crud.rs`

**Задачи**:
- [ ] get_integrations() - получение всех интеграций
- [ ] get_integration() - получение по ID
- [ ] create_integration() - создание
- [ ] update_integration() - обновление
- [ ] delete_integration() - удаление

---

### Этап 2: IntegrationMatcher (1 день)

**Файл**: `rust/src/db/sql/integration_matcher.rs`

**Задачи**:
- [ ] get_integration_matchers() - получение matcher'ов
- [ ] create_integration_matcher() - создание
- [ ] update_integration_matcher() - обновление
- [ ] delete_integration_matcher() - удаление

---

### Этап 3: IntegrationExtractValue (0.5 дня)

**Файл**: `rust/src/db/sql/integration_extract.rs`

**Задачи**:
- [ ] get_integration_extract_values() - получение extract values
- [ ] create_integration_extract_value() - создание
- [ ] update_integration_extract_value() - обновление
- [ ] delete_integration_extract_value() - удаление

---

### Этап 4: Тесты (1 день)

**Задачи**:
- [ ] Unit тесты для CRUD
- [ ] Integration тесты

---

**Ответственный**: Alexander Vashurin  
**Дата**: 2026-02-26  
**Статус**: 📋 ПЛАН  
**Оценка времени**: 3.5-4.5 дней
