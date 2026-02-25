//! SSH агент для Semaphore UI
//!
//! Предоставляет функциональность для:
//! - Управления SSH ключами
//! - Подключения к SSH серверам
//! - Интеграции с Git через SSH
//! - SSH agent forwarding

use std::path::{Path, PathBuf};
use ssh2::Session;
use std::net::TcpStream;
use std::io::prelude::*;

use crate::error::{Error, Result};

/// SSH ключ с опциональным паролем
#[derive(Debug, Clone)]
pub struct SshKey {
    /// Приватный ключ (PEM формат)
    pub private_key: Vec<u8>,
    /// Пароль для ключа (если есть)
    pub passphrase: Option<String>,
    /// Публичный ключ (опционально)
    pub public_key: Option<Vec<u8>>,
}

impl SshKey {
    /// Создаёт новый SSH ключ
    pub fn new(private_key: Vec<u8>, passphrase: Option<String>) -> Self {
        Self {
            private_key,
            passphrase,
            public_key: None,
        }
    }

    /// Создаёт ключ из строки
    pub fn from_string(private_key: String, passphrase: Option<String>) -> Self {
        Self {
            private_key: private_key.into_bytes(),
            passphrase,
            public_key: None,
        }
    }

    /// Устанавливает публичный ключ
    pub fn with_public_key(mut self, public_key: Vec<u8>) -> Self {
        self.public_key = Some(public_key);
        self
    }
}

/// Конфигурация SSH подключения
#[derive(Debug, Clone)]
pub struct SshConfig {
    /// Хост для подключения
    pub host: String,
    /// Порт (по умолчанию 22)
    pub port: u16,
    /// Имя пользователя
    pub username: String,
    /// SSH ключи
    pub keys: Vec<SshKey>,
    /// Таймаут подключения в секундах
    pub timeout_secs: u32,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 22,
            username: String::from("root"),
            keys: Vec::new(),
            timeout_secs: 30,
        }
    }
}

impl SshConfig {
    /// Создаёт новую конфигурацию
    pub fn new(host: String, username: String) -> Self {
        Self {
            host,
            username,
            ..Default::default()
        }
    }

    /// Добавляет SSH ключ
    pub fn add_key(mut self, key: SshKey) -> Self {
        self.keys.push(key);
        self
    }

    /// Устанавливает порт
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Устанавливает таймаут
    pub fn with_timeout(mut self, timeout_secs: u32) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

/// Результат выполнения SSH команды
#[derive(Debug, Clone)]
pub struct SshCommandResult {
    /// Код возврата
    pub exit_code: i32,
    /// Стандартный вывод
    pub stdout: String,
    /// Стандартный вывод ошибок
    pub stderr: String,
}

/// SSH агент
pub struct SshAgent {
    /// Конфигурация
    config: SshConfig,
    /// Активная сессия
    session: Option<Session>,
    /// Путь к сокету агента (для agent forwarding)
    #[allow(dead_code)]
    agent_socket: Option<PathBuf>,
}

impl SshAgent {
    /// Создаёт новый SSH агент
    pub fn new(config: SshConfig) -> Self {
        Self {
            config,
            session: None,
            agent_socket: None,
        }
    }

    /// Создаёт агент с минимальной конфигурацией
    pub fn simple(host: String, username: String, key: SshKey) -> Self {
        let config = SshConfig::new(host, username).add_key(key);
        Self::new(config)
    }

    /// Подключается к SSH серверу
    pub fn connect(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        
        // Устанавливаем TCP подключение
        let tcp = TcpStream::connect(&addr).map_err(|e| {
            Error::Other(format!("Ошибка TCP подключения: {}", e))
        })?;

        // Устанавливаем таймаут
        tcp.set_read_timeout(Some(std::time::Duration::from_secs(self.config.timeout_secs as u64)))
            .map_err(|e| Error::Other(format!("Ошибка установки таймаута: {}", e)))?;

        // Создаём SSH сессию
        let mut session = Session::new().map_err(|e| {
            Error::Other(format!("Ошибка создания SSH сессии: {}", e))
        })?;

        session.set_tcp_stream(tcp);

        // Рукопожатие
        session.handshake().map_err(|e| {
            Error::Other(format!("Ошибка SSH handshake: {}", e))
        })?;

        // Пробуем аутентификацию с каждым ключом
        // Копируем ключи для избежания проблем с borrow checker
        let keys = self.config.keys.clone();
        let mut auth_error = None;
        let username = self.config.username.clone();
        
        for key in &keys {
            match Self::authenticate_with_key_static(&mut session, &username, key) {
                Ok(_) => {
                    self.session = Some(session);
                    return Ok(());
                }
                Err(e) => {
                    auth_error = Some(e);
                    continue;
                }
            }
        }

        Err(auth_error.unwrap_or_else(|| {
            Error::Other("Аутентификация не удалась: нет доступных ключей".to_string())
        }))
    }

    /// Аутентификация с использованием ключа (статический метод)
    fn authenticate_with_key_static(session: &mut Session, username: &str, key: &SshKey) -> Result<()> {
        // Создаём временный файл для ключа
        let temp_dir = std::env::temp_dir();
        let key_file = temp_dir.join(format!("ssh_key_{}", uuid::Uuid::new_v4()));

        // Записываем ключ в файл
        std::fs::write(&key_file, &key.private_key).map_err(|e| {
            Error::Other(format!("Ошибка записи ключа: {}", e))
        })?;

        // Устанавливаем права доступа (только чтение для владельца)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| Error::Other(format!("Ошибка установки прав: {}", e)))?;
        }

        // Пытаемся аутентифицироваться
        let result = if let Some(passphrase) = &key.passphrase {
            session.userauth_pubkey_file(
                username,
                None,
                &key_file,
                Some(passphrase),
            )
        } else {
            session.userauth_pubkey_file(
                username,
                None,
                &key_file,
                None,
            )
        };

        // Удаляем временный файл
        let _ = std::fs::remove_file(&key_file);

        result.map_err(|e| {
            Error::Other(format!("Ошибка аутентификации: {}", e))
        })?;

        // Проверяем успешность аутентификации
        if !session.authenticated() {
            return Err(Error::Other("Аутентификация не удалась".to_string()));
        }

        Ok(())
    }

    /// Выполняет команду на удалённом сервере
    pub fn execute_command(&self, command: &str) -> Result<SshCommandResult> {
        let session = self.session.as_ref().ok_or_else(|| {
            Error::Other("SSH сессия не установлена".to_string())
        })?;

        let mut channel = session.channel_session().map_err(|e| {
            Error::Other(format!("Ошибка создания канала: {}", e))
        })?;

        channel.exec(command).map_err(|e| {
            Error::Other(format!("Ошибка выполнения команды: {}", e))
        })?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        channel.read_to_string(&mut stdout).map_err(|e| {
            Error::Other(format!("Ошибка чтения stdout: {}", e))
        })?;

        let mut stderr_channel = channel.stderr();
        stderr_channel.read_to_string(&mut stderr).map_err(|e| {
            Error::Other(format!("Ошибка чтения stderr: {}", e))
        })?;

        channel.wait_close().map_err(|e| {
            Error::Other(format!("Ошибка ожидания завершения: {}", e))
        })?;

        let exit_code = channel.exit_status().unwrap_or(-1);

        Ok(SshCommandResult {
            exit_code,
            stdout,
            stderr,
        })
    }

    /// Клонирует Git репозиторий через SSH
    pub fn clone_repository(
        &self,
        repo_url: &str,
        target_path: &Path,
    ) -> Result<()> {
        use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};

        // Создаём callback для аутентификации
        let mut callbacks = RemoteCallbacks::new();

        // Копируем ключи для closure
        let keys = self.config.keys.clone();
        let username = self.config.username.clone();

        // Настраиваем аутентификацию через SSH ключи
        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            let user = username_from_url.unwrap_or(&username);

            for key in &keys {
                let private_key_str = String::from_utf8_lossy(&key.private_key);
                
                if let Some(passphrase) = &key.passphrase {
                    return git2::Cred::ssh_key_from_memory(
                        user,
                        Some(passphrase),
                        &private_key_str,
                        None, // Публичный ключ не обязателен
                    );
                } else {
                    return git2::Cred::ssh_key_from_memory(
                        user,
                        None,
                        &private_key_str,
                        None,
                    );
                }
            }

            Err(git2::Error::from_str("Нет доступных SSH ключей"))
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_opts);

        builder.clone(repo_url, target_path).map_err(|e| {
            Error::Other(format!("Ошибка клонирования репозитория: {}", e))
        })?;

        Ok(())
    }

    /// Закрывает подключение
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = self.session.take() {
            session.disconnect(None, "", None).map_err(|e| {
                Error::Other(format!("Ошибка отключения: {}", e))
            })?;
        }
        Ok(())
    }

    /// Проверяет, подключены ли мы
    pub fn is_connected(&self) -> bool {
        self.session.is_some()
    }

    /// Получает сессию
    pub fn session(&self) -> Option<&Session> {
        self.session.as_ref()
    }
}

impl Drop for SshAgent {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

/// Утилиты для работы с SSH
pub mod utils {
    use super::*;
    use std::fs;

    /// Загружает SSH ключ из файла
    pub fn load_key_from_file(path: &Path, passphrase: Option<&str>) -> Result<SshKey> {
        let private_key = fs::read(path).map_err(|e| {
            Error::Other(format!("Ошибка чтения ключа: {}", e))
        })?;

        Ok(SshKey::new(
            private_key,
            passphrase.map(String::from),
        ))
    }

    /// Загружает SSH ключ из строки
    pub fn load_key_from_string(private_key: &str, passphrase: Option<&str>) -> SshKey {
        SshKey::from_string(private_key.to_string(), passphrase.map(String::from))
    }

    /// Проверяет валидность SSH ключа
    pub fn validate_key(key: &SshKey) -> Result<()> {
        // Простая проверка формата PEM
        let key_str = String::from_utf8_lossy(&key.private_key);
        
        if !key_str.contains("BEGIN") || !key_str.contains("PRIVATE KEY") {
            return Err(Error::Other("Неверный формат SSH ключа".to_string()));
        }

        Ok(())
    }

    /// Создаёт временную директорию для SSH сокетов
    pub fn create_temp_ssh_dir() -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir()
            .join(format!("ssh_agent_{}", uuid::Uuid::new_v4()));
        
        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            Error::Other(format!("Ошибка создания директории: {}", e))
        })?;

        Ok(temp_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_key_creation() {
        let key = SshKey::new(b"private key".to_vec(), Some("passphrase".to_string()));
        assert_eq!(key.private_key, b"private key");
        assert_eq!(key.passphrase, Some("passphrase".to_string()));
    }

    #[test]
    fn test_ssh_config_creation() {
        let config = SshConfig::new("example.com".to_string(), "user".to_string());
        assert_eq!(config.host, "example.com");
        assert_eq!(config.username, "user");
        assert_eq!(config.port, 22);
    }

    #[test]
    fn test_ssh_config_with_port() {
        let config = SshConfig::new("example.com".to_string(), "user".to_string())
            .with_port(2222);
        assert_eq!(config.port, 2222);
    }

    #[test]
    fn test_ssh_key_from_string() {
        let key_data = "-----BEGIN OPENSSH PRIVATE KEY-----
test
-----END OPENSSH PRIVATE KEY-----";
        let key = SshKey::from_string(key_data.to_string(), None);
        assert!(key.private_key.len() > 0);
    }

    #[test]
    fn test_utils_load_key_from_string() {
        let key_data = "-----BEGIN RSA PRIVATE KEY-----
test
-----END RSA PRIVATE KEY-----";
        let key = utils::load_key_from_string(key_data, None);
        assert!(key.private_key.len() > 0);
    }

    #[test]
    fn test_utils_validate_key_invalid() {
        let key = SshKey::new(b"invalid key".to_vec(), None);
        assert!(utils::validate_key(&key).is_err());
    }
}
