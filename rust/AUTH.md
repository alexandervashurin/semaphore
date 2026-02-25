# Аутентификация и Авторизация

## Обзор

Система аутентификации Semaphore UI основана на JWT (JSON Web Tokens).

## Компоненты

### 1. JWT Токены

**Структура токена:**
```json
{
  "sub": 1,           // ID пользователя
  "username": "admin", // Имя пользователя
  "email": "admin@example.com",
  "admin": true,       // Флаг администратора
  "exp": 1234567890,   // Время истечения
  "iat": 1234567000    // Время выпуска
}
```

**Параметры:**
- Время жизни: 24 часа
- Алгоритм подписи: HS256
- Секретный ключ: хранится в конфигурации

### 2. AuthService

Сервис для управления токенами:

```rust
use crate::api::auth::AuthService;

let auth_service = AuthService::new();

// Генерация токена
let token_info = auth_service.generate_token(&user)?;

// Проверка токена
let claims = auth_service.verify_token(&token)?;

// Обновление токена
let new_token = auth_service.refresh_token(&old_token)?;
```

### 3. Extractors

Извлекатели для обработчиков:

#### AuthUser - Аутентифицированный пользователь

```rust
use crate::api::extractors::AuthUser;

pub async fn handler(
    auth_user: AuthUser,
) -> Result<Json<Response>, AppError> {
    println!("User ID: {}", auth_user.user_id);
    println!("Username: {}", auth_user.username);
    println!("Email: {}", auth_user.email);
    println!("Admin: {}", auth_user.admin);
}
```

#### AdminUser - Администратор

```rust
use crate::api::extractors::AdminUser;

pub async fn admin_handler(
    admin: AdminUser,
) -> Result<Json<Response>, AppError> {
    // Доступно только администраторам
    let user = admin.into_inner();
}
```

#### OptionalAuthUser - Опциональная аутентификация

```rust
use crate::api::extractors::OptionalAuthUser;

pub async fn optional_auth(
    auth: OptionalAuthUser,
) -> Result<Json<Response>, AppError> {
    match auth.0 {
        Some(user) => println!("Аутентифицирован: {}", user.username),
        None => println!("Анонимный пользователь"),
    }
}
```

#### AuthToken - Сырой токен

```rust
use crate::api::extractors::AuthToken;

pub async fn handler(
    AuthToken(token): AuthToken,
) -> Result<Json<Response>, AppError> {
    // Используем токен напрямую
}
```

## API Endpoints

### POST /api/auth/login

Вход в систему.

**Запрос:**
```json
{
  "username": "admin",
  "password": "password123"
}
```

**Ответ:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

**Коды ошибок:**
- `401 UNAUTHORIZED` - неверный логин или пароль

### POST /api/auth/logout

Выход из системы.

**Ответ:**
- `200 OK` - успешный выход

**Примечание:** Токен добавляется в чёрный список (TODO).

## Использование в обработчиках

### Пример 1: Защищённый endpoint

```rust
use axum::{Json, extract::State};
use std::sync::Arc;
use crate::api::extractors::AuthUser;
use crate::api::state::AppState;

pub async fn get_profile(
    auth_user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<UserProfile>, AppError> {
    let user = state.store.get_user(auth_user.user_id).await?;
    
    Ok(Json(UserProfile {
        id: user.id,
        username: user.username,
        email: user.email,
    }))
}
```

### Пример 2: Админский endpoint

```rust
use crate::api::extractors::AdminUser;

pub async fn delete_user(
    admin: AdminUser,
    Path(user_id): Path<i32>,
    State(state): State<Arc<AppState>>,
) -> Result<StatusCode, AppError> {
    // Только администраторы могут удалять пользователей
    state.store.delete_user(user_id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}
```

### Пример 3: Endpoint с опциональной аутентификацией

```rust
use crate::api::extractors::OptionalAuthUser;

pub async fn get_public_data(
    auth: OptionalAuthUser,
) -> Result<Json<PublicData>, AppError> {
    let data = fetch_public_data().await?;
    
    // Если пользователь аутентифицирован, добавляем персонализацию
    if let Some(user) = auth.0 {
        data.personalized = true;
        data.user_id = Some(user.user_id);
    }
    
    Ok(Json(data))
}
```

## Безопасность

### Хранение секретного ключа

**Production:**
```bash
export SEMAPHORE_JWT_SECRET="your-secure-random-string-at-least-32-chars"
```

**Development:**
```rust
let auth_service = AuthService::with_secret("dev-secret".to_string());
```

### Best Practices

1. **Используйте HTTPS** - токены передаются в заголовках
2. **Регулярно меняйте секрет** - особенно в production
3. **Используйте чёрный список** - для отзыва токенов
4. **Ограничьте время жизни** - 24 часа по умолчанию
5. **Логируйте неудачные попытки** - для обнаружения атак

## Обработка ошибок

### Типы ошибок

```rust
use crate::api::auth::AuthError;

match auth_service.verify_token(&token) {
    Ok(claims) => { /* Успех */ }
    Err(AuthError::TokenExpired) => { /* Токен истёк */ }
    Err(AuthError::InvalidToken(msg)) => { /* Неверный токен */ }
    Err(AuthError::TokenNotYetValid) => { /* Токен ещё не действителен */ }
    Err(AuthError::UserNotFound) => { /* Пользователь не найден */ }
    Err(AuthError::InvalidCredentials) => { /* Неверный логин/пароль */ }
}
```

### Формат ответа об ошибке

```json
{
  "error": "Токен истёк",
  "code": "TOKEN_EXPIRED",
  "details": null
}
```

## Тестирование

### Unit-тесты

```bash
cargo test api::auth::tests
cargo test api::extractors::tests
```

### Интеграционные тесты

```rust
#[tokio::test]
async fn test_login_success() {
    let app = create_app(Box::new(test_store()));
    
    let response = request(&app)
        .method("POST")
        .uri("/api/auth/login")
        .json(&json!({
            "username": "admin",
            "password": "password123"
        }))
        .await;
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body: LoginResponse = response.into_json();
    assert!(!body.token.is_empty());
    assert_eq!(body.token_type, "Bearer");
}
```

## Миграция с Go

### Отличия от Go-версии

| Характеристика | Go | Rust |
|----------------|----|----|
| Формат токена | JWT | JWT |
| Время жизни | 24 часа | 24 часа |
| Хранение сессий | БД | В памяти (TODO) |
| Bcrypt cost | 10 | 12 |

### Совместимость

- ✅ Формат JWT токенов совместим
- ✅ Bcrypt хеши совместимы
- ✅ API endpoints совместимы

## TODO

- [ ] Чёрный список токенов
- [ ] Refresh токены
- [ ] 2FA (TOTP)
- [ ] OAuth2 провайдеры
- [ ] Rate limiting для login
- [ ] Аудит успешных/неуспешных входов
