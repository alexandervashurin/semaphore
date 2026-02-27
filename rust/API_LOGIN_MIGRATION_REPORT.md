# 🦀 Миграция API Login на Rust

**Дата**: 2026-02-28
**Статус**: ✅ Local Auth ЗАВЕРШЕН

---

## 📊 Обзор

**Go файл**: `api/login.go` (829 строк)

**Rust файлы**:
- ✅ `rust/src/api/auth_local.rs` (160 строк)
- ⏳ `rust/src/api/auth_ldap.rs` (в плане)
- ⏳ `rust/src/api/auth_oidc.rs` (в плане)
- ⏳ `rust/src/api/auth_totp.rs` (в плане)

---

## ✅ Выполнено: Local Authentication

### Созданный Функционал:

**Сервис**: `LocalAuthService`

**Методы**:
- ✅ `login(username, password) -> User` - аутентификация
- ✅ `register(username, email, name, password) -> User` - регистрация
- ✅ `verify_password(password, hash) -> bool` - проверка пароля
- ✅ `hash_password(password) -> String` - хеширование
- ✅ `change_password(store, user_id, old_pwd, new_pwd)` - смена пароля

**Тесты**: 4 теста
- `test_hash_password()`
- `test_verify_password_correct()`
- `test_verify_password_incorrect()`
- `test_verify_password_empty()`

---

## 📋 Оставшиеся Компоненты

### 1. LDAP Authentication

**Go функции**:
- `tryFindLDAPUser(username, password)`
- `convertEntryToMap(entry)`

**План**:
- Создать `rust/src/api/auth_ldap.rs`
- Использовать `ldap3` crate
- Реализовать TLS поддержку
- 4 теста

**Строк**: ~350

---

### 2. OIDC Authentication

**Go функции**:
- `oidcAuth()`
- `oidcCallback()`
- OAuth2 flow

**План**:
- Создать `rust/src/api/auth_oidc.rs`
- Использовать `oidc` и `oauth2` crates
- Реализовать callback handler
- 4 теста

**Строк**: ~300

---

### 3. TOTP Verification

**Go функции**:
- TOTP проверка в login

**План**:
- Создать `rust/src/api/auth_totp.rs`
- Интегрировать с `services/totp.rs`
- 2 теста

**Строк**: ~50

---

## 📈 Прогресс

| Компонент | Статус | Строк Go | Строк Rust | Прогресс |
|-----------|--------|----------|------------|----------|
| **Local Auth** | ✅ Готово | ~100 | 160 | 100% |
| **LDAP** | ⏳ В плане | ~300 | - | 0% |
| **OIDC** | ⏳ В плане | ~250 | - | 0% |
| **TOTP** | ⏳ В плане | ~50 | - | 0% |
| **Вспомогательные** | ⏳ В плане | ~129 | - | 0% |
| **ВСЕГО** | ⏳ В плане | **829** | **160** | **~20%** |

---

## 🔧 Технические Детали

### Безопасность:

**Хеширование паролей**:
- Алгоритм: bcrypt
- Cost factor: 12
- Длина хэша: 60 символов

**Проверка паролей**:
- Constant-time comparison (через bcrypt)
- Защита от timing attacks

**Обработка ошибок**:
- Не разглашать, что именно неверно (логин или пароль)
- Возвращать общее сообщение об ошибке

---

## 🚀 Следующие Шаги

### 1. LDAP Integration (4 часа)

```bash
# Добавить зависимости
cargo add ldap3 tokio-rustls

# Создать файл
touch rust/src/api/auth_ldap.rs

# Реализовать
edit rust/src/api/auth_ldap.rs
```

### 2. OIDC Integration (4 часа)

```bash
# Добавить зависимости
cargo add oidc oauth2

# Создать файл
touch rust/src/api/auth_oidc.rs

# Реализовать
edit rust/src/api/auth_oidc.rs
```

### 3. TOTP Integration (1 час)

```bash
# Создать файл
touch rust/src/api/auth_totp.rs

# Интегрировать с services/totp.rs
edit rust/src/api/auth_totp.rs
```

### 4. Интеграция (2 часа)

```bash
# Обновить handlers.rs
edit rust/src/api/handlers.rs

# Обновить routes.rs
edit rust/src/api/routes.rs

# Тесты
cargo test auth
```

---

## 📚 API Endpoints

### Local Authentication

**POST** `/api/auth/login`
```json
{
  "username": "admin",
  "password": "password123"
}
```

**Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

**POST** `/api/auth/register`
```json
{
  "username": "newuser",
  "email": "user@example.com",
  "name": "New User",
  "password": "password123"
}
```

**PUT** `/api/auth/password`
```json
{
  "old_password": "oldpass123",
  "new_password": "newpass456"
}
```

---

## 🧪 Тесты

### Запуск тестов:

```bash
cd rust
cargo test auth_local
```

### Покрытие:

- ✅ Хеширование паролей
- ✅ Проверка паролей
- ✅ Обработка неверных паролей
- ✅ Обработка пустых паролей

---

**Ответственный**: Alexander Vashurin
**Следующий шаг**: LDAP Authentication
