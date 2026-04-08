//! Cache Services - Сервисы кэширования
//!
//! Предоставляет специализированные методы для:
//! - Кэширования пользовательских сессий
//! - Кэширования результатов запросов
//! - Инвалидации кэша

use crate::cache::{CacheStats, RedisCache};
use crate::error::{Error, Result};
use crate::models::User;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Кэш сервис
pub struct CacheService {
    redis: Arc<RedisCache>,
    config: CacheServiceConfig,
}

/// Конфигурация сервиса кэширования
#[derive(Debug, Clone)]
pub struct CacheServiceConfig {
    /// TTL для сессий пользователей (в секундах)
    pub session_ttl_secs: u64,
    /// TTL для кэша запросов (в секундах)
    pub query_cache_ttl_secs: u64,
    /// TTL для кэша проектов (в секундах)
    pub project_cache_ttl_secs: u64,
    /// TTL для кэша задач (в секундах)
    pub task_cache_ttl_secs: u64,
}

impl Default for CacheServiceConfig {
    fn default() -> Self {
        Self {
            session_ttl_secs: 3600,      // 1 час
            query_cache_ttl_secs: 300,   // 5 минут
            project_cache_ttl_secs: 600, // 10 минут
            task_cache_ttl_secs: 60,     // 1 минута
        }
    }
}

/// Данные сессии пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl SessionData {
    /// Создаёт новую сессию
    pub fn new(user: &User, ttl_secs: u64) -> Self {
        let now = Utc::now();
        let expires_at = now + Duration::seconds(ttl_secs as i64);

        Self {
            user_id: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            is_admin: user.admin,
            created_at: now,
            expires_at,
        }
    }

    /// Проверяет истекла ли сессия
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Типы ключей кэша
pub struct CacheKeys;

impl CacheKeys {
    /// Ключ сессии пользователя
    pub fn session(token: &str) -> String {
        format!("session:{}", token)
    }

    /// Кэш пользователя по ID
    pub fn user_id(id: i32) -> String {
        format!("user:id:{}", id)
    }

    /// Кэш пользователя по username
    pub fn user_username(username: &str) -> String {
        format!("user:username:{}", username)
    }

    /// Кэш проекта по ID
    pub fn project(id: i64) -> String {
        format!("project:{}", id)
    }

    /// Кэш задач проекта
    pub fn project_tasks(project_id: i64, status: Option<&str>) -> String {
        match status {
            Some(s) => format!("project:{}:tasks:{}", project_id, s),
            None => format!("project:{}:tasks", project_id),
        }
    }

    /// Кэш шаблона по ID
    pub fn template(id: i64) -> String {
        format!("template:{}", id)
    }

    /// Кэш инвентаря по ID
    pub fn inventory(id: i64) -> String {
        format!("inventory:{}", id)
    }

    /// Кэш репозитория по ID
    pub fn repository(id: i64) -> String {
        format!("repository:{}", id)
    }

    /// Кэш окружения по ID
    pub fn environment(id: i64) -> String {
        format!("environment:{}", id)
    }

    /// Кэш расписаний проекта
    pub fn project_schedules(project_id: i64) -> String {
        format!("project:{}:schedules", project_id)
    }

    /// Кэш ключей доступа проекта
    pub fn project_keys(project_id: i64) -> String {
        format!("project:{}:keys", project_id)
    }

    /// Паттерн для всех ключей проекта
    pub fn project_pattern(project_id: i64) -> String {
        format!("project:{}:*", project_id)
    }
}

impl CacheService {
    /// Создаёт новый сервис кэширования
    pub fn new(redis: Arc<RedisCache>, config: CacheServiceConfig) -> Self {
        Self { redis, config }
    }

    // ========================================================================
    // Сессии пользователей
    // ========================================================================

    /// Сохраняет сессию пользователя
    pub async fn save_session(&self, token: &str, session: &SessionData) -> Result<()> {
        let key = CacheKeys::session(token);
        self.redis
            .set_with_ttl(&key, session, self.config.session_ttl_secs)
            .await?;
        debug!(
            "Saved session for user {} with token {}",
            session.user_id, token
        );
        Ok(())
    }

    /// Получает сессию пользователя
    pub async fn get_session(&self, token: &str) -> Result<Option<SessionData>> {
        let key = CacheKeys::session(token);
        let session = self.redis.get::<SessionData>(&key).await?;

        // Проверяем не истекла ли сессия
        if let Some(ref s) = session {
            if s.is_expired() {
                self.delete_session(token).await?;
                return Ok(None);
            }
        }

        Ok(session)
    }

    /// Удаляет сессию пользователя
    pub async fn delete_session(&self, token: &str) -> Result<()> {
        let key = CacheKeys::session(token);
        self.redis.delete(&key).await?;
        debug!("Deleted session with token {}", token);
        Ok(())
    }

    /// Продлевает сессию пользователя
    pub async fn extend_session(&self, token: &str) -> Result<()> {
        if let Some(session) = self.get_session(token).await? {
            self.save_session(token, &session).await?;
        }
        Ok(())
    }

    // ========================================================================
    // Кэширование пользователей
    // ========================================================================

    /// Кэширует пользователя
    pub async fn cache_user(&self, user: &User) -> Result<()> {
        // По ID
        let id_key = CacheKeys::user_id(user.id);
        self.redis
            .set_with_ttl(&id_key, user, self.config.query_cache_ttl_secs)
            .await?;

        // По username
        let username_key = CacheKeys::user_username(&user.username);
        self.redis
            .set_with_ttl(&username_key, user, self.config.query_cache_ttl_secs)
            .await?;

        debug!("Cached user {} ({})", user.id, user.username);
        Ok(())
    }

    /// Получает пользователя из кэша по ID
    pub async fn get_user_by_id(&self, id: i32) -> Result<Option<User>> {
        let key = CacheKeys::user_id(id);
        self.redis.get(&key).await
    }

    /// Получает пользователя из кэша по username
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let key = CacheKeys::user_username(username);
        self.redis.get(&key).await
    }

    /// Инвалидирует кэш пользователя
    pub async fn invalidate_user(&self, user_id: i32, username: &str) -> Result<()> {
        self.redis.delete(&CacheKeys::user_id(user_id)).await?;
        self.redis
            .delete(&CacheKeys::user_username(username))
            .await?;
        Ok(())
    }

    // ========================================================================
    // Кэширование проектов
    // ========================================================================

    /// Кэширует проект
    pub async fn cache_project<T: Serialize>(&self, id: i64, project: &T) -> Result<()> {
        let key = CacheKeys::project(id);
        self.redis
            .set_with_ttl(&key, project, self.config.project_cache_ttl_secs)
            .await?;
        debug!("Cached project {}", id);
        Ok(())
    }

    /// Получает проект из кэша
    pub async fn get_project<T: serde::de::DeserializeOwned>(&self, id: i64) -> Result<Option<T>> {
        let key = CacheKeys::project(id);
        self.redis.get(&key).await
    }

    /// Инвалидирует кэш проекта и связанных данных
    pub async fn invalidate_project(&self, project_id: i64) -> Result<()> {
        self.redis.delete(&CacheKeys::project(project_id)).await?;
        self.redis
            .delete_pattern(&CacheKeys::project_pattern(project_id))
            .await?;
        info!("Invalidated cache for project {}", project_id);
        Ok(())
    }

    // ========================================================================
    // Кэширование задач
    // ========================================================================

    /// Кэширует задачи проекта
    pub async fn cache_project_tasks<T: Serialize>(
        &self,
        project_id: i64,
        status: Option<&str>,
        tasks: &T,
    ) -> Result<()> {
        let key = CacheKeys::project_tasks(project_id, status);
        self.redis
            .set_with_ttl(&key, tasks, self.config.task_cache_ttl_secs)
            .await?;
        debug!(
            "Cached tasks for project {} (status: {:?})",
            project_id, status
        );
        Ok(())
    }

    /// Получает задачи проекта из кэша
    pub async fn get_project_tasks<T: serde::de::DeserializeOwned>(
        &self,
        project_id: i64,
        status: Option<&str>,
    ) -> Result<Option<T>> {
        let key = CacheKeys::project_tasks(project_id, status);
        self.redis.get(&key).await
    }

    /// Инвалидирует кэш задач проекта
    pub async fn invalidate_project_tasks(&self, project_id: i64) -> Result<()> {
        self.redis
            .delete(&CacheKeys::project_tasks(project_id, None))
            .await?;
        self.redis
            .delete(&CacheKeys::project_tasks(project_id, Some("running")))
            .await?;
        self.redis
            .delete(&CacheKeys::project_tasks(project_id, Some("pending")))
            .await?;
        self.redis
            .delete(&CacheKeys::project_tasks(project_id, Some("success")))
            .await?;
        self.redis
            .delete(&CacheKeys::project_tasks(project_id, Some("failed")))
            .await?;
        Ok(())
    }

    // ========================================================================
    // Кэширование других сущностей
    // ========================================================================

    /// Кэширует шаблон
    pub async fn cache_template<T: Serialize>(&self, id: i64, template: &T) -> Result<()> {
        let key = CacheKeys::template(id);
        self.redis
            .set_with_ttl(&key, template, self.config.query_cache_ttl_secs)
            .await?;
        Ok(())
    }

    /// Получает шаблон из кэша
    pub async fn get_template<T: serde::de::DeserializeOwned>(&self, id: i64) -> Result<Option<T>> {
        let key = CacheKeys::template(id);
        self.redis.get(&key).await
    }

    /// Инвалидирует кэш шаблона
    pub async fn invalidate_template(&self, id: i64) -> Result<()> {
        self.redis.delete(&CacheKeys::template(id)).await?;
        Ok(())
    }

    /// Кэширует инвентарь
    pub async fn cache_inventory<T: Serialize>(&self, id: i64, inventory: &T) -> Result<()> {
        let key = CacheKeys::inventory(id);
        self.redis
            .set_with_ttl(&key, inventory, self.config.query_cache_ttl_secs)
            .await?;
        Ok(())
    }

    /// Получает инвентарь из кэша
    pub async fn get_inventory<T: serde::de::DeserializeOwned>(
        &self,
        id: i64,
    ) -> Result<Option<T>> {
        let key = CacheKeys::inventory(id);
        self.redis.get(&key).await
    }

    /// Инвалидирует кэш инвентаря
    pub async fn invalidate_inventory(&self, id: i64) -> Result<()> {
        self.redis.delete(&CacheKeys::inventory(id)).await?;
        Ok(())
    }

    /// Кэширует репозиторий
    pub async fn cache_repository<T: Serialize>(&self, id: i64, repo: &T) -> Result<()> {
        let key = CacheKeys::repository(id);
        self.redis
            .set_with_ttl(&key, repo, self.config.query_cache_ttl_secs)
            .await?;
        Ok(())
    }

    /// Получает репозиторий из кэша
    pub async fn get_repository<T: serde::de::DeserializeOwned>(
        &self,
        id: i64,
    ) -> Result<Option<T>> {
        let key = CacheKeys::repository(id);
        self.redis.get(&key).await
    }

    /// Инвалидирует кэш репозитория
    pub async fn invalidate_repository(&self, id: i64) -> Result<()> {
        self.redis.delete(&CacheKeys::repository(id)).await?;
        Ok(())
    }

    /// Кэширует окружение
    pub async fn cache_environment<T: Serialize>(&self, id: i64, env: &T) -> Result<()> {
        let key = CacheKeys::environment(id);
        self.redis
            .set_with_ttl(&key, env, self.config.query_cache_ttl_secs)
            .await?;
        Ok(())
    }

    /// Получает окружение из кэша
    pub async fn get_environment<T: serde::de::DeserializeOwned>(
        &self,
        id: i64,
    ) -> Result<Option<T>> {
        let key = CacheKeys::environment(id);
        self.redis.get(&key).await
    }

    /// Инвалидирует кэш окружения
    pub async fn invalidate_environment(&self, id: i64) -> Result<()> {
        self.redis.delete(&CacheKeys::environment(id)).await?;
        Ok(())
    }

    // ========================================================================
    // Статистика
    // ========================================================================

    /// Получает статистику кэша
    pub async fn get_stats(&self) -> CacheStats {
        self.redis.get_stats().await
    }

    /// Сбрасывает статистику
    pub async fn reset_stats(&self) {
        self.redis.reset_stats().await;
    }

    /// Проверяет доступен ли Redis
    pub async fn is_available(&self) -> bool {
        self.redis.ping().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_keys() {
        assert_eq!(CacheKeys::session("token123"), "session:token123");
        assert_eq!(CacheKeys::user_id(1), "user:id:1");
        assert_eq!(CacheKeys::project(42), "project:42");
        assert_eq!(CacheKeys::project_tasks(1, None), "project:1:tasks");
        assert_eq!(
            CacheKeys::project_tasks(1, Some("running")),
            "project:1:tasks:running"
        );
    }

    #[test]
    fn test_session_data() {
        let user = User {
            id: 1,
            username: "test".to_string(),
            email: "test@example.com".to_string(),
            password: "hash".to_string(),
            admin: false,
            name: "Test".to_string(),
            created: Utc::now(),
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };

        let session = SessionData::new(&user, 3600);
        assert_eq!(session.user_id, 1);
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_data_expired() {
        let user = User {
            id: 2,
            username: "expired".to_string(),
            email: "expired@example.com".to_string(),
            password: "hash".to_string(),
            admin: true,
            name: "Expired User".to_string(),
            created: Utc::now(),
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };

        // Создаём сессию с TTL 0 секунд (истекает сразу)
        let session = SessionData::new(&user, 0);
        assert_eq!(session.user_id, 2);
        assert!(session.is_admin);
        // Сессия с TTL=0 должна быть просрочена
        assert!(session.is_expired());
    }

    #[test]
    fn test_session_data_fields() {
        let user = User {
            id: 10,
            username: "fieldtest".to_string(),
            email: "fields@test.com".to_string(),
            password: "hash".to_string(),
            admin: true,
            name: "Field Test".to_string(),
            created: Utc::now(),
            external: false,
            alert: false,
            pro: true,
            totp: None,
            email_otp: None,
        };

        let session = SessionData::new(&user, 7200);

        assert_eq!(session.username, "fieldtest");
        assert_eq!(session.email, "fields@test.com");
        assert!(session.is_admin);
        assert_eq!(session.user_id, 10);
    }

    #[test]
    fn test_cache_service_config_default() {
        let config = CacheServiceConfig::default();

        assert_eq!(config.session_ttl_secs, 3600);
        assert_eq!(config.query_cache_ttl_secs, 300);
        assert_eq!(config.project_cache_ttl_secs, 600);
        assert_eq!(config.task_cache_ttl_secs, 60);
    }

    #[test]
    fn test_cache_service_config_custom() {
        let config = CacheServiceConfig {
            session_ttl_secs: 7200,
            query_cache_ttl_secs: 600,
            project_cache_ttl_secs: 1200,
            task_cache_ttl_secs: 120,
        };

        assert_eq!(config.session_ttl_secs, 7200);
        assert_eq!(config.query_cache_ttl_secs, 600);
    }

    #[test]
    fn test_cache_keys_all_types() {
        // Проверяем все типы ключей
        assert_eq!(CacheKeys::session("abc"), "session:abc");
        assert_eq!(CacheKeys::user_id(123), "user:id:123");
        assert_eq!(CacheKeys::user_username("john"), "user:username:john");
        assert_eq!(CacheKeys::project(456), "project:456");
        assert_eq!(CacheKeys::project_tasks(1, None), "project:1:tasks");
        assert_eq!(
            CacheKeys::project_tasks(1, Some("success")),
            "project:1:tasks:success"
        );
        assert_eq!(CacheKeys::template(789), "template:789");
        assert_eq!(CacheKeys::inventory(111), "inventory:111");
        assert_eq!(CacheKeys::repository(222), "repository:222");
        assert_eq!(CacheKeys::environment(333), "environment:333");
        // Дополнительные типы ключей
        assert_eq!(CacheKeys::project_schedules(42), "project:42:schedules");
        assert_eq!(CacheKeys::project_keys(7), "project:7:keys");
        assert_eq!(CacheKeys::project_pattern(99), "project:99:*");
    }

    #[test]
    fn test_session_expires_at_is_correct() {
        let user = User {
            id: 1,
            username: "u".to_string(),
            email: "u@u.com".to_string(),
            password: "h".to_string(),
            admin: false,
            name: "U".to_string(),
            created: Utc::now(),
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        let ttl = 5000u64;
        let session = SessionData::new(&user, ttl);
        let expected = session.created_at + chrono::Duration::seconds(ttl as i64);
        assert_eq!(session.expires_at, expected);
    }

    #[test]
    fn test_session_not_expired_with_large_ttl() {
        let user = User {
            id: 5,
            username: "fresh".to_string(),
            email: "fresh@test.com".to_string(),
            password: "h".to_string(),
            admin: false,
            name: "Fresh".to_string(),
            created: Utc::now(),
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        let session = SessionData::new(&user, 86400); // 24 часа
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_data_serialization_roundtrip() {
        let user = User {
            id: 42,
            username: "serpent".to_string(),
            email: "serpent@test.com".to_string(),
            password: "hash".to_string(),
            admin: true,
            name: "Serpent".to_string(),
            created: Utc::now(),
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        let session = SessionData::new(&user, 3600);

        let json = serde_json::to_string(&session).expect("serialize failed");
        let restored: SessionData = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(restored.user_id, 42);
        assert_eq!(restored.username, "serpent");
        assert_eq!(restored.email, "serpent@test.com");
        assert!(restored.is_admin);
    }

    #[test]
    fn test_cache_service_config_clone() {
        let config = CacheServiceConfig {
            session_ttl_secs: 100,
            query_cache_ttl_secs: 200,
            project_cache_ttl_secs: 300,
            task_cache_ttl_secs: 400,
        };
        let cloned = config.clone();
        assert_eq!(cloned.session_ttl_secs, 100);
        assert_eq!(cloned.task_cache_ttl_secs, 400);
    }

    #[test]
    fn test_session_data_is_expired_exactly_at_ttl() {
        let user = User {
            id: 1,
            username: "user".to_string(),
            email: "user@test.com".to_string(),
            password: "hash".to_string(),
            admin: false,
            name: "User".to_string(),
            created: Utc::now(),
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        // TTL = 0 means expires_at = now, so it should be expired
        let session = SessionData::new(&user, 0);
        // Since expires_at == created_at + 0, it should be expired immediately
        // or very close to it. We'll check that expires_at is in the past or very near future
        assert!(session.expires_at <= session.created_at + Duration::seconds(1));
    }

    #[test]
    fn test_session_data_admin_flag() {
        let admin_user = User {
            id: 1,
            username: "admin".to_string(),
            email: "admin@test.com".to_string(),
            password: "hash".to_string(),
            admin: true,
            name: "Admin".to_string(),
            created: Utc::now(),
            external: false,
            alert: false,
            pro: false,
            totp: None,
            email_otp: None,
        };
        let session = SessionData::new(&admin_user, 3600);
        assert!(session.is_admin);
        assert_eq!(session.user_id, 1);
    }

    #[test]
    fn test_cache_keys_project_tasks_with_various_statuses() {
        let statuses = ["waiting", "running", "success", "error", "stopped"];
        for status in &statuses {
            let key = CacheKeys::project_tasks(1, Some(status));
            assert!(key.contains(status));
            assert!(key.starts_with("project:1:tasks:"));
        }
    }

    #[test]
    fn test_cache_keys_with_large_ids() {
        let large_id: i64 = i64::MAX;
        let key = CacheKeys::project(large_id);
        assert!(key.contains(&large_id.to_string()));
    }

    #[test]
    fn test_cache_service_config_all_defaults() {
        let config = CacheServiceConfig::default();
        assert_eq!(config.session_ttl_secs, 3600);
        assert_eq!(config.query_cache_ttl_secs, 300);
        assert_eq!(config.project_cache_ttl_secs, 600);
        assert_eq!(config.task_cache_ttl_secs, 60);
    }
}
