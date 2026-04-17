//! Auto Backup Service - автоматическое резервное копирование
//!
//! Планировщик регулярных бэкапов проектов

use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};
use tracing::{error, info, instrument, warn};

use crate::db::store::Store;
use crate::error::Result;
use crate::services::backup::BackupFormat;

/// Конфигурация автобэкапа
#[derive(Debug, Clone)]
pub struct AutoBackupConfig {
    /// Включить автобэкап
    pub enabled: bool,
    /// Интервал между бэкапами (в часах)
    pub interval_hours: u64,
    /// Путь для хранения бэкапов
    pub backup_path: String,
    /// Максимальное количество хранимых бэкапов
    pub max_backups: usize,
    /// Сжимать бэкапы (gzip)
    pub compress: bool,
}

impl Default for AutoBackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_hours: 24,
            backup_path: "./backups".to_string(),
            max_backups: 7,
            compress: true,
        }
    }
}

/// Статистика бэкапов
#[derive(Debug, Clone, Default)]
pub struct BackupStats {
    pub total_backups: u64,
    pub successful_backups: u64,
    pub failed_backups: u64,
    pub last_backup_time: Option<DateTime<Utc>>,
    pub last_backup_size_bytes: u64,
    pub next_backup_time: Option<DateTime<Utc>>,
}

/// AutoBackupService - сервис автоматического резервного копирования
pub struct AutoBackupService {
    config: AutoBackupConfig,
    store: Arc<dyn Store + Send + Sync>,
    stats: Arc<RwLock<BackupStats>>,
    running: Arc<RwLock<bool>>,
}

impl AutoBackupService {
    /// Создаёт новый сервис автобэкапа
    pub fn new(config: AutoBackupConfig, store: Arc<dyn Store + Send + Sync>) -> Self {
        Self {
            config,
            store,
            stats: Arc::new(RwLock::new(BackupStats::default())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Запускает сервис автобэкапа
    pub async fn start(&self) {
        if !self.config.enabled {
            info!("Auto backup service is disabled");
            return;
        }

        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        info!(
            "Starting auto backup service (interval: {} hours, path: {})",
            self.config.interval_hours, self.config.backup_path
        );

        let config = self.config.clone();
        let store = Arc::clone(&self.store);
        let stats = Arc::clone(&self.stats);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                // Проверка флага остановки
                {
                    let is_running = running.read().await;
                    if !*is_running {
                        break;
                    }
                }

                // Выполнение бэкапа
                match Self::run_backup(&config, &store, &stats).await {
                    Ok(_) => {
                        info!("Auto backup completed successfully");
                    }
                    Err(e) => {
                        error!("Auto backup failed: {}", e);
                        let mut s = stats.write().await;
                        s.failed_backups += 1;
                    }
                }

                // Ожидание следующего интервала
                sleep(Duration::from_secs(config.interval_hours * 3600)).await;
            }

            info!("Auto backup service stopped");
        });
    }

    /// Останавливает сервис автобэкапа
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stopping auto backup service...");
    }

    /// Выполняет один бэкап
    #[instrument(skip(config, store, stats), name = "auto_backup")]
    async fn run_backup(
        config: &AutoBackupConfig,
        store: &Arc<dyn Store + Send + Sync>,
        stats: &Arc<RwLock<BackupStats>>,
    ) -> Result<()> {
        info!("Starting automatic backup...");

        // Получаем все проекты
        let projects = store.get_projects(None).await?;

        let mut total_size = 0u64;
        let mut backup_count = 0u64;

        for project in projects {
            // Формируем бэкап проекта
            let mut backup = BackupFormat {
                version: "1.0".to_string(),
                project: crate::services::backup::BackupProject {
                    name: project.name.clone(),
                    alert: Some(project.alert),
                    alert_chat: project.alert_chat.clone(),
                    max_parallel_tasks: Some(project.max_parallel_tasks),
                },
                templates: vec![],
                repositories: vec![],
                inventories: vec![],
                environments: vec![],
                access_keys: vec![],
                schedules: vec![],
                integrations: vec![],
                views: vec![],
            };

            // Получаем связанные сущности
            if let Ok(templates) = store.get_templates(project.id).await {
                backup.templates = templates
                    .into_iter()
                    .map(|t| crate::services::backup::BackupTemplate {
                        name: t.name,
                        playbook: t.playbook,
                        arguments: t.arguments,
                        template_type: "ansible".to_string(),
                        inventory: None,
                        repository: None,
                        environment: None,
                        cron: None,
                    })
                    .collect();
            }

            // Сохраняем бэкап
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
            let filename = format!(
                "{}_project_{}_backup.json{}",
                timestamp,
                project.id,
                if config.compress { ".gz" } else { "" }
            );

            let filepath = format!("{}/{}", config.backup_path, filename);

            // Создаём директорию если не существует
            if let Err(e) = std::fs::create_dir_all(&config.backup_path) {
                error!("Failed to create backup directory: {}", e);
                continue;
            }

            // Сериализуем и сохраняем
            let json = serde_json::to_string_pretty(&backup)?;
            let bytes = if config.compress {
                gzip_encode(json.as_bytes())?
            } else {
                json.into_bytes()
            };

            total_size += bytes.len() as u64;
            backup_count += 1;

            if let Err(e) = std::fs::write(&filepath, &bytes) {
                error!("Failed to write backup file {}: {}", filepath, e);
            } else {
                info!("Backup saved: {} ({} bytes)", filepath, bytes.len());
            }
        }

        // Обновление статистики
        {
            let mut s = stats.write().await;
            s.total_backups += 1;
            s.successful_backups += 1;
            s.last_backup_time = Some(Utc::now());
            s.last_backup_size_bytes = total_size;
            s.next_backup_time =
                Some(Utc::now() + chrono::Duration::hours(config.interval_hours as i64));
        }

        // Очистка старых бэкапов
        cleanup_old_backups(&config.backup_path, config.max_backups)?;

        info!(
            "Backup completed: {} projects, {} bytes",
            backup_count, total_size
        );

        Ok(())
    }

    /// Возвращает текущую статистику
    pub async fn get_stats(&self) -> BackupStats {
        self.stats.read().await.clone()
    }

    /// Возвращает статус сервиса
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Возвращает конфигурацию
    pub fn get_config(&self) -> &AutoBackupConfig {
        &self.config
    }
}

/// Gzip сжатие
fn gzip_encode(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

/// Очистка старых бэкапов
fn cleanup_old_backups(backup_path: &str, max_backups: usize) -> Result<()> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(backup_path);
    if !path.exists() {
        return Ok(());
    }

    let mut backups: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json" || ext == "gz")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            e.metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|time| (e.path(), time))
        })
        .collect();

    // Сортируем по времени (новые первые)
    backups.sort_by_key(|b| std::cmp::Reverse(b.1));

    // Удаляем старые если превышен лимит
    if backups.len() > max_backups {
        for (path, _) in backups.iter().skip(max_backups) {
            if let Err(e) = fs::remove_file(path) {
                warn!("Failed to remove old backup {:?}: {}", path, e);
            } else {
                info!("Removed old backup: {:?}", path);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::mock::MockStore;

    #[test]
    fn test_auto_backup_config_default() {
        let config = AutoBackupConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.interval_hours, 24);
        assert_eq!(config.max_backups, 7);
        assert!(config.compress);
    }

    #[test]
    fn test_auto_backup_service_creation() {
        let store: Arc<dyn Store + Send + Sync> = Arc::new(MockStore::new());
        let config = AutoBackupConfig::default();
        let service = AutoBackupService::new(config, store);

        assert!(!service.get_config().enabled);
    }

    #[tokio::test]
    async fn test_backup_stats_default() {
        let store: Arc<dyn Store + Send + Sync> = Arc::new(MockStore::new());
        let config = AutoBackupConfig::default();
        let service = AutoBackupService::new(config, store);

        let stats = service.get_stats().await;
        assert_eq!(stats.total_backups, 0);
        assert_eq!(stats.successful_backups, 0);
        assert_eq!(stats.failed_backups, 0);
        assert!(stats.last_backup_time.is_none());
    }

    #[test]
    fn test_gzip_encode_and_decode() {
        let data = b"Hello, world!";
        let compressed = gzip_encode(data).unwrap();
        // Compressed data should be different from original (gzip header)
        assert_ne!(compressed, data);
        // Can verify by decompressing
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[tokio::test]
    async fn test_auto_backup_service_not_running_initially() {
        let store: Arc<dyn Store + Send + Sync> = Arc::new(MockStore::new());
        let config = AutoBackupConfig::default();
        let service = AutoBackupService::new(config, store);

        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_auto_backup_disabled_does_not_start() {
        let store: Arc<dyn Store + Send + Sync> = Arc::new(MockStore::new());
        let config = AutoBackupConfig::default(); // enabled = false
        let service = AutoBackupService::new(config, store);

        // start() should return immediately when disabled
        service.start().await;
        assert!(!service.is_running().await);
    }

    #[test]
    fn test_cleanup_old_backups_empty_dir() {
        // Создаём временную директорию
        let temp_dir = std::env::temp_dir().join("semaphore_test_cleanup");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Очистка пустой директории должна succeed
        let result = cleanup_old_backups(temp_dir.to_str().unwrap(), 5);
        assert!(result.is_ok());

        // Убираем за собой
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_auto_backup_config_custom() {
        let config = AutoBackupConfig {
            enabled: true,
            interval_hours: 12,
            backup_path: "/var/backups/velum".to_string(),
            max_backups: 14,
            compress: false,
        };
        assert!(config.enabled);
        assert_eq!(config.interval_hours, 12);
        assert_eq!(config.backup_path, "/var/backups/velum");
        assert_eq!(config.max_backups, 14);
        assert!(!config.compress);
    }

    #[test]
    fn test_backup_stats_clone() {
        let stats = BackupStats {
            total_backups: 10,
            successful_backups: 8,
            failed_backups: 2,
            last_backup_time: Some(Utc::now()),
            last_backup_size_bytes: 1024,
            next_backup_time: Some(Utc::now() + chrono::Duration::hours(24)),
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_backups, stats.total_backups);
        assert_eq!(cloned.failed_backups, stats.failed_backups);
    }

    #[test]
    fn test_backup_format_serialization() {
        let backup = BackupFormat {
            version: "1.0".to_string(),
            project: crate::services::backup::BackupProject {
                name: "Test".to_string(),
                alert: Some(false),
                alert_chat: None,
                max_parallel_tasks: Some(5),
            },
            templates: Vec::new(),
            repositories: Vec::new(),
            inventories: Vec::new(),
            environments: Vec::new(),
            access_keys: Vec::new(),
            schedules: Vec::new(),
            integrations: Vec::new(),
            views: Vec::new(),
        };
        let json = serde_json::to_string(&backup).unwrap();
        assert!(json.contains("\"version\":\"1.0\""));
        assert!(json.contains("\"name\":\"Test\""));
    }

    #[test]
    fn test_gzip_encode_empty_data() {
        let data = b"";
        let compressed = gzip_encode(data).unwrap();
        assert_ne!(compressed, data);
        // Can verify by decompressing
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_auto_backup_service_get_config() {
        let store: Arc<dyn Store + Send + Sync> = Arc::new(MockStore::new());
        let config = AutoBackupConfig {
            enabled: true,
            interval_hours: 6,
            backup_path: "/tmp".to_string(),
            max_backups: 3,
            compress: true,
        };
        let service = AutoBackupService::new(config.clone(), store);
        let retrieved = service.get_config();
        assert_eq!(retrieved.interval_hours, config.interval_hours);
        assert_eq!(retrieved.max_backups, config.max_backups);
    }

    #[test]
    fn test_auto_backup_config_clone_test() {
        let config = AutoBackupConfig {
            enabled: true,
            interval_hours: 12,
            backup_path: "/var/backups".to_string(),
            max_backups: 14,
            compress: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.enabled, config.enabled);
        assert_eq!(cloned.interval_hours, config.interval_hours);
    }

    #[test]
    fn test_backup_stats_clone_test() {
        let stats = BackupStats {
            total_backups: 100,
            successful_backups: 95,
            failed_backups: 5,
            last_backup_time: Some(Utc::now()),
            last_backup_size_bytes: 1024 * 1024,
            next_backup_time: Some(Utc::now() + chrono::Duration::hours(24)),
        };
        let cloned = stats.clone();
        assert_eq!(cloned.total_backups, stats.total_backups);
        assert_eq!(cloned.last_backup_size_bytes, stats.last_backup_size_bytes);
    }

    #[test]
    fn test_backup_stats_default_test() {
        let stats = BackupStats::default();
        assert_eq!(stats.total_backups, 0);
        assert_eq!(stats.successful_backups, 0);
        assert_eq!(stats.failed_backups, 0);
        assert!(stats.last_backup_time.is_none());
        assert_eq!(stats.last_backup_size_bytes, 0);
    }

    #[test]
    fn test_auto_backup_config_default_values_test() {
        let config = AutoBackupConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.interval_hours, 24);
        assert_eq!(config.max_backups, 7);
        assert!(config.compress);
    }

    #[test]
    fn test_auto_backup_config_clone_debug() {
        let config = AutoBackupConfig {
            enabled: true,
            interval_hours: 48,
            backup_path: "/opt/backups".to_string(),
            max_backups: 30,
            compress: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.enabled, true);
        assert_eq!(cloned.interval_hours, 48);
        assert_eq!(cloned.backup_path, "/opt/backups");
        assert_eq!(cloned.max_backups, 30);
        assert!(!cloned.compress);
    }

    #[test]
    fn test_backup_stats_debug_trait() {
        let stats = BackupStats {
            total_backups: 50,
            successful_backups: 48,
            failed_backups: 2,
            last_backup_time: Some(Utc::now()),
            last_backup_size_bytes: 2048,
            next_backup_time: Some(Utc::now() + chrono::Duration::hours(12)),
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("BackupStats"));
        assert!(debug_str.contains("total_backups"));
    }

    #[test]
    fn test_backup_stats_default_all_fields() {
        let stats = BackupStats::default();
        assert_eq!(stats.total_backups, 0);
        assert_eq!(stats.successful_backups, 0);
        assert_eq!(stats.failed_backups, 0);
        assert!(stats.last_backup_time.is_none());
        assert_eq!(stats.last_backup_size_bytes, 0);
        assert!(stats.next_backup_time.is_none());
    }

    #[test]
    fn test_backup_stats_next_backup_time_calculation() {
        let now = Utc::now();
        let interval = 6u64;
        let expected_next = now + chrono::Duration::hours(interval as i64);
        let diff = expected_next.signed_duration_since(now);
        assert_eq!(diff.num_hours(), interval as i64);
    }

    #[test]
    fn test_backup_stats_update_counters() {
        let mut stats = BackupStats::default();
        stats.total_backups += 1;
        stats.successful_backups += 1;
        stats.last_backup_time = Some(Utc::now());
        stats.last_backup_size_bytes = 4096;

        assert_eq!(stats.total_backups, 1);
        assert_eq!(stats.successful_backups, 1);
        assert_eq!(stats.failed_backups, 0);
        assert!(stats.last_backup_time.is_some());
    }

    #[test]
    fn test_backup_stats_failed_counter() {
        let mut stats = BackupStats::default();
        stats.total_backups += 1;
        stats.failed_backups += 1;

        assert_eq!(stats.total_backups, 1);
        assert_eq!(stats.failed_backups, 1);
        assert_eq!(stats.successful_backups, 0);
    }

    #[test]
    fn test_auto_backup_config_all_values_varied() {
        let configs: Vec<AutoBackupConfig> = vec![
            AutoBackupConfig {
                enabled: true,
                interval_hours: 1,
                ..AutoBackupConfig::default()
            },
            AutoBackupConfig {
                enabled: false,
                interval_hours: 168,
                ..AutoBackupConfig::default()
            },
            AutoBackupConfig {
                compress: false,
                ..AutoBackupConfig::default()
            },
            AutoBackupConfig {
                max_backups: 100,
                ..AutoBackupConfig::default()
            },
        ];
        assert_eq!(configs[0].interval_hours, 1);
        assert_eq!(configs[1].interval_hours, 168);
        assert!(!configs[2].compress);
        assert_eq!(configs[3].max_backups, 100);
    }

    #[test]
    fn test_backup_format_serialization_with_templates() {
        let backup = BackupFormat {
            version: "2.0".to_string(),
            project: crate::services::backup::BackupProject {
                name: "Full Project".to_string(),
                alert: Some(true),
                alert_chat: Some("#alerts".to_string()),
                max_parallel_tasks: Some(3),
            },
            templates: vec![crate::services::backup::BackupTemplate {
                name: "Deploy".to_string(),
                playbook: "deploy.yml".to_string(),
                arguments: None,
                template_type: "ansible".to_string(),
                inventory: None,
                repository: None,
                environment: None,
                cron: None,
            }],
            repositories: vec![],
            inventories: vec![],
            environments: vec![],
            access_keys: vec![],
            schedules: vec![],
            integrations: vec![],
            views: vec![],
        };
        let json = serde_json::to_string(&backup).unwrap();
        assert!(json.contains("\"version\":\"2.0\""));
        assert!(json.contains("\"name\":\"Full Project\""));
        assert!(json.contains("\"alert\":true"));
        assert!(json.contains("\"playbook\":\"deploy.yml\""));
    }

    #[test]
    fn test_backup_format_deserialization() {
        let json = r#"{
            "version":"1.0",
            "project":{"name":"Test","alert":null,"alert_chat":null,"max_parallel_tasks":null},
            "templates":[],
            "repositories":[],
            "inventories":[],
            "environments":[],
            "access_keys":[],
            "schedules":[],
            "integrations":[],
            "views":[]
        }"#;
        let backup: BackupFormat = serde_json::from_str(json).unwrap();
        assert_eq!(backup.version, "1.0");
        assert_eq!(backup.project.name, "Test");
        assert_eq!(backup.templates.len(), 0);
    }

    #[test]
    fn test_auto_backup_config_boundary_values() {
        let config = AutoBackupConfig {
            enabled: true,
            interval_hours: 0,
            backup_path: "".to_string(),
            max_backups: 0,
            compress: true,
        };
        assert_eq!(config.interval_hours, 0);
        assert_eq!(config.max_backups, 0);
        assert!(config.enabled);
    }

    #[test]
    fn test_backup_stats_equality_after_clone() {
        let stats = BackupStats {
            total_backups: 777,
            successful_backups: 770,
            failed_backups: 7,
            last_backup_time: None,
            last_backup_size_bytes: 999999,
            next_backup_time: None,
        };
        let cloned = stats.clone();
        assert_eq!(stats.total_backups, cloned.total_backups);
        assert_eq!(stats.successful_backups, cloned.successful_backups);
        assert_eq!(stats.failed_backups, cloned.failed_backups);
        assert_eq!(stats.last_backup_size_bytes, cloned.last_backup_size_bytes);
    }

    #[test]
    fn test_gzip_encode_large_data() {
        let data = vec![b'a'; 10000];
        let compressed = gzip_encode(&data).unwrap();
        // Compressed data should be smaller for repetitive data
        assert!(compressed.len() < data.len());
        // Verify round-trip
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&compressed[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_cleanup_old_backups_nonexistent_dir() {
        let result = cleanup_old_backups("/nonexistent/path/that/does/not/exist", 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_backup_service_config_reference() {
        let store: Arc<dyn Store + Send + Sync> = Arc::new(MockStore::new());
        let config = AutoBackupConfig {
            enabled: true,
            interval_hours: 2,
            backup_path: "/tmp/test".to_string(),
            max_backups: 1,
            compress: false,
        };
        let service = AutoBackupService::new(config, store);
        let retrieved = service.get_config();
        assert_eq!(retrieved.interval_hours, 2);
        assert_eq!(retrieved.backup_path, "/tmp/test");
        assert_eq!(retrieved.max_backups, 1);
        assert!(!retrieved.compress);
    }
}
