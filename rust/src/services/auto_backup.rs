//! Auto Backup Service - автоматическое резервное копирование
//!
//! Планировщик регулярных бэкапов проектов

use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
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
    use flate2::write::GzEncoder;
    use flate2::Compression;
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
    backups.sort_by(|a, b| b.1.cmp(&a.1));

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
}
