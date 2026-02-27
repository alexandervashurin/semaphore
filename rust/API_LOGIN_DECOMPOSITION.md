# 📋 Декомпозиция и Миграция api/login.go

**Go файл**: `api/login.go` (829 строк)
**Статус**: 🚧 В ПРОЦЕССЕ

---

## 📊 Анализ Файла

### Основные Компоненты:

1. **LDAP Аутентификация** (~300 строк)
   - `tryFindLDAPUser()`
   - `convertEntryToMap()`
   - LDAP подключения
   - LDAP поиск и фильтрация

2. **OIDC Аутентификация** (~250 строк)
   - `oidcAuth()`
   - `oidcCallback()`
   - OAuth2 flow
   - OIDC provider configuration

3. **Local Аутентификация** (~100 строк)
   - Проверка пароля
   - Создание сессии
   - TOTP верификация

4. **Вспомогательные Функции** (~179 строк)
   - Генерация токенов
   - Cookie management
   - Response formatting

---

## 🎯 План Декомпозиции (Rust)

### 1. Модуль LDAP (`rust/src/api/auth_ldap.rs`)

**Функции**:
- `try_find_ldap_user(username: &str, password: &str) -> Result<User>`
- `convert_entry_to_map(entry: &LdapEntry) -> HashMap<String, Value>`
- `connect_ldap() -> Result<LdapConnection>`
- `search_ldap_user(username: &str) -> Result<Option<LdapEntry>>`

**Зависимости**:
- `ldap3` crate для LDAP подключений
- `tokio-rustls` для TLS

**Строк**: ~350

---

### 2. Модуль OIDC (`rust/src/api/auth_oidc.rs`)

**Функции**:
- `oidc_auth() -> Result<RedirectUrl>`
- `oidc_callback(code: &str) -> Result<User>`
- `create_oidc_provider() -> Result<OidcProvider>`
- `exchange_code_for_token(code: &str) -> Result<TokenResponse>`

**Зависимости**:
- `oidc` crate
- `oauth2` crate
- `reqwest` для HTTP запросов

**Строк**: ~300

---

### 3. Модуль Local Auth (`rust/src/api/auth_local.rs`)

**Функции**:
- `local_login(username: &str, password: &str) -> Result<User>`
- `verify_password(password: &str, hash: &str) -> bool`
- `create_session(user_id: i32) -> Result<Session>`

**Зависимости**:
- `bcrypt` (уже используется)
- `jsonwebtoken` (уже используется)

**Строк**: ~150

---

### 4. Модуль TOTP (`rust/src/api/auth_totp.rs`)

**Функции**:
- `verify_totp(user_id: i32, code: &str) -> Result<bool>`
- `create_totp_secret(user_id: i32) -> Result<TotpSecret>`

**Зависимости**:
- `totp` crate (уже есть в `services/totp.rs`)

**Строк**: ~50

---

### 5. Интеграция в `rust/src/api/auth.rs`

**Обновления**:
- Добавить enum `AuthMethod { Local, Ldap, Oidc }`
- Обновить `AuthService::login()`
- Добавить роуты для LDAP и OIDC

**Строк**: ~100

---

## 📝 Структура Файлов

```
rust/src/api/
├── auth.rs              # Основной auth модуль (обновить)
├── auth_ldap.rs         # LDAP аутентификация (новый)
├── auth_oidc.rs         # OIDC аутентификация (новый)
├── auth_local.rs        # Local аутентификация (новый)
└── auth_totp.rs         # TOTP верификация (новый)
```

---

## 🔧 Зависимости (Cargo.toml)

```toml
[dependencies]
# LDAP
ldap3 = "0.11"
tokio-rustls = "0.26"

# OIDC
oidc = "0.18"
oauth2 = "4.4"

# Уже есть:
# bcrypt = "0.17"
# jsonwebtoken = "9.3"
# reqwest = "0.12"
```

---

## 🎯 Этапы Миграции

### Этап 1: Local Auth (2 часа)

- [ ] Создать `rust/src/api/auth_local.rs`
- [ ] Реализовать `local_login()`
- [ ] Интегрировать с `auth.rs`
- [ ] Тесты: 3 теста

### Этап 2: TOTP (1 час)

- [ ] Создать `rust/src/api/auth_totp.rs`
- [ ] Интегрировать с `services/totp.rs`
- [ ] Тесты: 2 теста

### Этап 3: LDAP (4 часа)

- [ ] Добавить зависимости
- [ ] Создать `rust/src/api/auth_ldap.rs`
- [ ] Реализовать LDAP подключение
- [ ] Реализовать поиск пользователей
- [ ] Тесты: 4 теста

### Этап 4: OIDC (4 часа)

- [ ] Добавить зависимости
- [ ] Создать `rust/src/api/auth_oidc.rs`
- [ ] Реализовать OAuth2 flow
- [ ] Реализовать callback handler
- [ ] Тесты: 4 теста

### Этап 5: Интеграция (2 часа)

- [ ] Обновить `rust/src/api/auth.rs`
- [ ] Обновить `rust/src/api/routes.rs`
- [ ] Добавить роуты
- [ ] Тесты: 5 интеграционных тестов

---

## 📈 Прогресс

| Этап | Статус | Прогресс |
|------|--------|----------|
| **Local Auth** | ⏳ В плане | 0% |
| **TOTP** | ⏳ В плане | 0% |
| **LDAP** | ⏳ В плане | 0% |
| **OIDC** | ⏳ В плане | 0% |
| **Интеграция** | ⏳ В плане | 0% |
| **ВСЕГО** | ⏳ В плане | 0% |

---

## 🚀 Команды

### Создание файлов
```bash
cd rust/src/api
touch auth_ldap.rs auth_oidc.rs auth_local.rs auth_totp.rs
```

### Добавление зависимостей
```bash
cd rust
cargo add ldap3 tokio-rustls
cargo add oidc oauth2
```

### Тесты
```bash
cargo test auth_ldap
cargo test auth_oidc
cargo test auth_local
```

---

**Ответственный**: Alexander Vashurin
**Следующий шаг**: Начало миграции (Local Auth)
