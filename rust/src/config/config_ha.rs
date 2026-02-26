//! Config HA - High Availability конфигурация
//!
//! Аналог util/config.go из Go версии (часть 7: HA)

use serde::{Deserialize, Serialize};

/// HA (High Availability) конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for HAConfigFull {
    fn default() -> Self {
        Self {
            enable: false,
            redis: HARedisConfigFull::default(),
            node_id: String::new(),
        }
    }
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
            format!("redis://{}:{}/{}", 
                self.redis.host, self.redis.port, self.redis.db)
        } else {
            format!("redis://:{}@{}:{}/{}", 
                self.redis.password, self.redis.host, self.redis.port, self.redis.db)
        }
    }
    
    /// Генерирует случайный Node ID
    pub fn generate_node_id(&mut self) {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 16];
        rng.fill_bytes(&mut bytes);
        self.node_id = format!("{:x}", bytes);
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
        
        assert_eq!(
            config.redis_url(),
            "redis://redis.example.com:6380/2"
        );
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
}
