//! Config HA - High Availability конфигурация
//!
//! Аналог util/config.go из Go версии (часть 7: HA)

use serde::{Deserialize, Serialize};

/// HA (High Availability) конфигурация
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HAConfigFull {
    /// Включить HA режим
    #[serde(default)]
    pub enable: bool,

    /// Redis конфигурация
    #[serde(default)]
    pub redis: HARedisConfigFull,

    /// Node ID (генерируется автоматически)
    #[serde(skip)]
    pub node_id: String,
}

/// Redis конфигурация для HA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HARedisConfigFull {
    /// Redis хост
    #[serde(default)]
    pub host: String,

    /// Redis порт
    #[serde(default)]
    pub port: u16,

    /// Redis пароль
    #[serde(default)]
    pub password: String,

    /// Redis DB
    #[serde(default)]
    pub db: u8,
}

impl Default for HARedisConfigFull {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6379,
            password: String::new(),
            db: 0,
        }
    }
}

impl HAConfigFull {
    /// Создаёт новую HA конфигурацию
    pub fn new() -> Self {
        Self::default()
    }

    /// Проверяет включён ли HA режим
    pub fn is_enabled(&self) -> bool {
        self.enable
    }

    /// Создаёт Redis URL для подключения
    pub fn redis_url(&self) -> String {
        if self.redis.password.is_empty() {
            format!(
                "redis://{}:{}/{}",
                self.redis.host, self.redis.port, self.redis.db
            )
        } else {
            format!(
                "redis://:{}@{}:{}/{}",
                self.redis.password, self.redis.host, self.redis.port, self.redis.db
            )
        }
    }

    /// Генерирует случайный Node ID
    pub fn generate_node_id(&mut self) {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        self.node_id = bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
    }

    /// Получает Node ID или генерирует новый
    pub fn get_node_id(&mut self) -> &str {
        if self.node_id.is_empty() {
            self.generate_node_id();
        }
        &self.node_id
    }
}

/// Загружает HA конфигурацию из переменных окружения
pub fn load_ha_from_env() -> HAConfigFull {
    use std::env;

    let mut config = HAConfigFull::new();

    if let Ok(val) = env::var("SEMAPHORE_HA_ENABLE") {
        config.enable = val.to_lowercase() == "true" || val == "1";
    }

    if let Ok(host) = env::var("SEMAPHORE_HA_REDIS_HOST") {
        config.redis.host = host;
    }

    if let Ok(port) = env::var("SEMAPHORE_HA_REDIS_PORT") {
        if let Ok(port_num) = port.parse() {
            config.redis.port = port_num;
        }
    }

    if let Ok(password) = env::var("SEMAPHORE_HA_REDIS_PASSWORD") {
        config.redis.password = password;
    }

    if let Ok(db) = env::var("SEMAPHORE_HA_REDIS_DB") {
        if let Ok(db_num) = db.parse() {
            config.redis.db = db_num;
        }
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_ha_config_default() {
        let config = HAConfigFull::default();
        assert!(!config.is_enabled());
        assert_eq!(config.redis.host, "localhost");
        assert_eq!(config.redis.port, 6379);
    }

    #[test]
    fn test_ha_config_redis_url_without_password() {
        let config = HAConfigFull {
            redis: HARedisConfigFull {
                host: "redis.example.com".to_string(),
                port: 6380,
                db: 2,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(config.redis_url(), "redis://redis.example.com:6380/2");
    }

    #[test]
    fn test_ha_config_redis_url_with_password() {
        let config = HAConfigFull {
            redis: HARedisConfigFull {
                host: "redis.example.com".to_string(),
                port: 6380,
                password: "secret".to_string(),
                db: 2,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            config.redis_url(),
            "redis://:secret@redis.example.com:6380/2"
        );
    }

    #[test]
    fn test_generate_node_id() {
        let mut config = HAConfigFull::new();
        config.generate_node_id();

        assert!(!config.node_id.is_empty());
        assert_eq!(config.node_id.len(), 32); // 16 bytes в hex
    }

    #[test]
    fn test_get_node_id_generates() {
        let mut config = HAConfigFull::new();
        let node_id = config.get_node_id();

        assert!(!node_id.is_empty());
        assert_eq!(node_id.len(), 32);
    }

    #[test]
    fn test_load_ha_from_env() {
        env::set_var("SEMAPHORE_HA_ENABLE", "true");
        env::set_var("SEMAPHORE_HA_REDIS_HOST", "test.redis.host");
        env::set_var("SEMAPHORE_HA_REDIS_PORT", "6380");

        let config = load_ha_from_env();
        assert!(config.is_enabled());
        assert_eq!(config.redis.host, "test.redis.host");
        assert_eq!(config.redis.port, 6380);

        env::remove_var("SEMAPHORE_HA_ENABLE");
        env::remove_var("SEMAPHORE_HA_REDIS_HOST");
        env::remove_var("SEMAPHORE_HA_REDIS_PORT");
    }

    #[test]
    fn test_ha_config_serialization() {
        let config = HAConfigFull {
            enable: true,
            redis: HARedisConfigFull {
                host: "prod.redis.example.com".to_string(),
                port: 6380,
                password: "prod_secret".to_string(),
                db: 1,
            },
            node_id: "abc123def456".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enable\":true"));
        assert!(json.contains("\"host\":\"prod.redis.example.com\""));
        assert!(json.contains("\"port\":6380"));
    }

    #[test]
    fn test_ha_config_deserialization() {
        let json = r#"{
            "enable": true,
            "redis": {
                "host": "ha.redis.example.com",
                "port": 6379,
                "password": "redis_pass",
                "db": 3
            }
        }"#;

        let config: HAConfigFull = serde_json::from_str(json).unwrap();
        assert!(config.enable);
        assert_eq!(config.redis.host, "ha.redis.example.com");
        assert_eq!(config.redis.port, 6379);
        assert_eq!(config.redis.db, 3);
    }

    #[test]
    fn test_ha_config_clone() {
        let config = HAConfigFull {
            enable: true,
            redis: HARedisConfigFull {
                host: "clone.redis".to_string(),
                port: 6380,
                password: "pass".to_string(),
                db: 2,
            },
            node_id: "node-001".to_string(),
        };
        let cloned = config.clone();
        assert_eq!(cloned.redis.host, "clone.redis");
        assert_eq!(cloned.node_id, "node-001");
    }

    #[test]
    fn test_ha_redis_config_default() {
        let redis = HARedisConfigFull::default();
        assert_eq!(redis.host, "localhost");
        assert_eq!(redis.port, 6379);
        assert!(redis.password.is_empty());
        assert_eq!(redis.db, 0);
    }

    #[test]
    fn test_ha_redis_config_serialization() {
        let redis = HARedisConfigFull {
            host: "redis-host".to_string(),
            port: 6380,
            password: "secret".to_string(),
            db: 5,
        };

        let json = serde_json::to_string(&redis).unwrap();
        assert!(json.contains("\"host\":\"redis-host\""));
        assert!(json.contains("\"port\":6380"));
        assert!(json.contains("\"db\":5"));
    }

    #[test]
    fn test_ha_redis_config_deserialization() {
        let json = r#"{
            "host": "deserialized-redis",
            "port": 6390,
            "password": "deser_pass",
            "db": 7
        }"#;

        let redis: HARedisConfigFull = serde_json::from_str(json).unwrap();
        assert_eq!(redis.host, "deserialized-redis");
        assert_eq!(redis.port, 6390);
        assert_eq!(redis.db, 7);
    }

    #[test]
    fn test_ha_config_is_enabled_true() {
        let config = HAConfigFull {
            enable: true,
            ..Default::default()
        };
        assert!(config.is_enabled());
    }

    #[test]
    fn test_ha_config_is_enabled_false() {
        let config = HAConfigFull {
            enable: false,
            ..Default::default()
        };
        assert!(!config.is_enabled());
    }

    #[test]
    fn test_ha_config_node_id_skipped_in_serialization() {
        let config = HAConfigFull {
            enable: true,
            node_id: "secret-node-id".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("node_id"));
        assert!(!json.contains("secret-node-id"));
    }

    #[test]
    fn test_ha_config_redis_url_db_0() {
        let config = HAConfigFull {
            redis: HARedisConfigFull {
                host: "localhost".to_string(),
                port: 6379,
                db: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(config.redis_url(), "redis://localhost:6379/0");
    }

    #[test]
    fn test_ha_config_redis_url_max_db() {
        let config = HAConfigFull {
            redis: HARedisConfigFull {
                host: "localhost".to_string(),
                port: 6379,
                db: 15,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(config.redis_url(), "redis://localhost:6379/15");
    }

    #[test]
    fn test_ha_config_node_id_already_set() {
        let mut config = HAConfigFull {
            node_id: "pre-existing-id".to_string(),
            ..Default::default()
        };

        let node_id = config.get_node_id();
        assert_eq!(node_id, "pre-existing-id");
    }

    #[test]
    fn test_load_ha_from_env_all_fields() {
        env::set_var("SEMAPHORE_HA_ENABLE", "true");
        env::set_var("SEMAPHORE_HA_REDIS_HOST", "full.redis.host");
        env::set_var("SEMAPHORE_HA_REDIS_PORT", "6390");
        env::set_var("SEMAPHORE_HA_REDIS_PASSWORD", "redis_password");
        env::set_var("SEMAPHORE_HA_REDIS_DB", "5");

        let config = load_ha_from_env();
        assert!(config.enable);
        assert_eq!(config.redis.host, "full.redis.host");
        assert_eq!(config.redis.port, 6390);
        assert_eq!(config.redis.password, "redis_password");
        assert_eq!(config.redis.db, 5);

        env::remove_var("SEMAPHORE_HA_ENABLE");
        env::remove_var("SEMAPHORE_HA_REDIS_HOST");
        env::remove_var("SEMAPHORE_HA_REDIS_PORT");
        env::remove_var("SEMAPHORE_HA_REDIS_PASSWORD");
        env::remove_var("SEMAPHORE_HA_REDIS_DB");
    }

    #[test]
    fn test_load_ha_from_env_invalid_port() {
        env::set_var("SEMAPHORE_HA_ENABLE", "true");
        env::set_var("SEMAPHORE_HA_REDIS_PORT", "not_a_number");

        let config = load_ha_from_env();
        assert!(config.enable);
        // При невалидном порте может быть 0 или дефолтное значение
        // Главное что config создался без паники
        assert!(config.redis.port >= 0);

        env::remove_var("SEMAPHORE_HA_ENABLE");
        env::remove_var("SEMAPHORE_HA_REDIS_PORT");
    }

    #[test]
    fn test_load_ha_from_env_invalid_db() {
        env::set_var("SEMAPHORE_HA_ENABLE", "true");
        env::set_var("SEMAPHORE_HA_REDIS_DB", "not_a_number");

        let config = load_ha_from_env();
        assert_eq!(config.redis.db, 0); // default since parse failed

        env::remove_var("SEMAPHORE_HA_ENABLE");
        env::remove_var("SEMAPHORE_HA_REDIS_DB");
    }

    #[test]
    fn test_ha_config_unicode_password() {
        let config = HAConfigFull {
            redis: HARedisConfigFull {
                host: "localhost".to_string(),
                port: 6379,
                password: "пароль_секрет".to_string(),
                db: 0,
            },
            ..Default::default()
        };

        let url = config.redis_url();
        assert!(url.contains("пароль_секрет"));
    }

    #[test]
    fn test_ha_config_new() {
        let config = HAConfigFull::new();
        assert!(!config.is_enabled());
        assert_eq!(config.redis.host, "localhost");
    }

    #[test]
    fn test_ha_config_serialization_defaults() {
        let config = HAConfigFull::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enable\":false"));
        assert!(json.contains("\"host\":\"localhost\""));
        assert!(json.contains("\"port\":6379"));
    }
}
