//! SSH агент для Semaphore UI
//!
//! Предоставляет функциональность для:
//! - Управления SSH ключами
//! - Подключения к SSH серверам
//! - Интеграции с Git через SSH
//! - SSH agent forwarding
//! - Установки ключей доступа (KeyInstaller)

use std::path::{Path, PathBuf};
use ssh2::Session;
use std::net::TcpStream;
use std::io::prelude::*;
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};
use crate::services::task_logger::TaskLogger;

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
#[derive(Clone)]
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

// ============================================================================
// AccessKeyRole - роли ключей доступа (как в Go db/AccessKey.go)
// ============================================================================

/// Роль ключа доступа
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKeyRole {
    /// Ключ используется для Git операций
    Git,
    /// Ключ используется как пароль для Ansible vault
    AnsiblePasswordVault,
    /// Ключ используется для Ansible become user
    AnsibleBecomeUser,
    /// Ключ используется для Ansible user
    AnsibleUser,
}

impl FromStr for AccessKeyRole {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "git" => Ok(AccessKeyRole::Git),
            "ansible_password_vault" => Ok(AccessKeyRole::AnsiblePasswordVault),
            "ansible_become_user" => Ok(AccessKeyRole::AnsibleBecomeUser),
            "ansible_user" => Ok(AccessKeyRole::AnsibleUser),
            _ => Err(format!("Неизвестная роль ключа доступа: {}", s)),
        }
    }
}

impl fmt::Display for AccessKeyRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessKeyRole::Git => write!(f, "git"),
            AccessKeyRole::AnsiblePasswordVault => write!(f, "ansible_password_vault"),
            AccessKeyRole::AnsibleBecomeUser => write!(f, "ansible_become_user"),
            AccessKeyRole::AnsibleUser => write!(f, "ansible_user"),
        }
    }
}

// ============================================================================
// AccessKeyInstallation - результат установки ключа
// ============================================================================

/// Результат установки ключа доступа
pub struct AccessKeyInstallation {
    /// SSH агент (если требуется)
    pub ssh_agent: Option<SshAgent>,
    /// Логин (если требуется)
    pub login: Option<String>,
    /// Пароль (если требуется)
    pub password: Option<String>,
    /// Скрипт (опционально)
    pub script: Option<String>,
}

impl AccessKeyInstallation {
    /// Создаёт новую установку
    pub fn new() -> Self {
        Self {
            ssh_agent: None,
            login: None,
            password: None,
            script: None,
        }
    }

    /// Получает переменные окружения для Git
    pub fn get_git_env(&self) -> Vec<(String, String)> {
        let mut env = Vec::new();

        env.push(("GIT_TERMINAL_PROMPT".to_string(), "0".to_string()));

        if let Some(_agent) = &self.ssh_agent {
            // SSH агент создан, но сокет не доступен напрямую
            // В будущей реализации можно добавить socket_file в SshAgent
            let mut ssh_cmd = "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null".to_string();
            // if let Some(config_path) = crate::config::get_ssh_config_path() {
            //     ssh_cmd.push_str(&format!(" -F {}", config_path));
            // }
            env.push(("GIT_SSH_COMMAND".to_string(), ssh_cmd));
        }

        env
    }

    /// Закрывает ресурсы (SSH агент)
    pub fn destroy(&mut self) -> Result<()> {
        if let Some(agent) = &mut self.ssh_agent {
            agent.disconnect()?;
        }
        Ok(())
    }
}

impl Default for AccessKeyInstallation {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// AccessKey - модель ключа доступа
// ============================================================================

/// Тип ключа доступа
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessKeyType {
    /// SSH ключ
    Ssh,
    /// Логин/пароль
    LoginPassword,
    /// Нет ключа (None)
    None,
}

/// Ключ доступа
#[derive(Debug, Clone)]
pub struct AccessKey {
    /// ID ключа
    pub id: i64,
    /// Тип ключа
    pub key_type: AccessKeyType,
    /// SSH ключ (если тип SSH)
    pub ssh_key: Option<SshKeyData>,
    /// Логин/пароль (если тип LoginPassword)
    pub login_password: Option<LoginPasswordData>,
    /// ID проекта (опционально)
    pub project_id: Option<i64>,
}

/// Данные SSH ключа
#[derive(Debug, Clone)]
pub struct SshKeyData {
    /// Приватный ключ (PEM)
    pub private_key: String,
    /// Passphrase (опционально)
    pub passphrase: String,
    /// Логин
    pub login: String,
}

/// Данные логина/пароля
#[derive(Debug, Clone)]
pub struct LoginPasswordData {
    /// Логин
    pub login: String,
    /// Пароль
    pub password: String,
}

impl AccessKey {
    /// Создаёт SSH ключ
    pub fn new_ssh(id: i64, private_key: String, passphrase: String, login: String, project_id: Option<i64>) -> Self {
        Self {
            id,
            key_type: AccessKeyType::Ssh,
            ssh_key: Some(SshKeyData {
                private_key,
                passphrase,
                login,
            }),
            login_password: None,
            project_id,
        }
    }

    /// Создаёт ключ с логином/паролем
    pub fn new_login_password(id: i64, login: String, password: String, project_id: Option<i64>) -> Self {
        Self {
            id,
            key_type: AccessKeyType::LoginPassword,
            ssh_key: None,
            login_password: Some(LoginPasswordData { login, password }),
            project_id,
        }
    }

    /// Создаёт пустой ключ
    pub fn new_none(id: i64, project_id: Option<i64>) -> Self {
        Self {
            id,
            key_type: AccessKeyType::None,
            ssh_key: None,
            login_password: None,
            project_id,
        }
    }
}

// ============================================================================
// KeyInstaller - установщик ключей доступа
// ============================================================================

/// Установщик ключей доступа
pub struct KeyInstaller;

impl KeyInstaller {
    /// Создаёт новый установщик
    pub fn new() -> Self {
        Self
    }

    /// Устанавливает ключ доступа в соответствии с ролью
    ///
    /// # Аргументы
    /// * `key` - ключ доступа
    /// * `role` - роль ключа
    /// * `logger` - логгер для вывода сообщений
    ///
    /// # Возвращает
    /// * `Result<AccessKeyInstallation>` - установленный ключ или ошибку
    pub fn install(
        &self,
        key: &AccessKey,
        role: AccessKeyRole,
        logger: &dyn TaskLogger,
    ) -> Result<AccessKeyInstallation> {
        let mut installation = AccessKeyInstallation::new();

        match role {
            AccessKeyRole::Git => {
                match &key.key_type {
                    AccessKeyType::Ssh => {
                        if let Some(ssh_key_data) = &key.ssh_key {
                            // Запускаем SSH агент
                            let ssh_key = SshKey::from_string(
                                ssh_key_data.private_key.clone(),
                                if ssh_key_data.passphrase.is_empty() {
                                    None
                                } else {
                                    Some(ssh_key_data.passphrase.clone())
                                },
                            );

                            let mut agent = SshAgent::simple(
                                "localhost".to_string(),
                                ssh_key_data.login.clone(),
                                ssh_key,
                            );

                            // Для Git нам не нужно подключаться, просто добавляем ключ в агент
                            // SSH агент будет создан и готов к использованию
                            installation.ssh_agent = Some(agent);
                            installation.login = Some(ssh_key_data.login.clone());

                            logger.logf("SSH агент запущен для ключа ID={}", format_args!("{}", key.id));
                        } else {
                            return Err(Error::Validation("SSH ключ не найден".to_string()));
                        }
                    }
                    _ => {
                        return Err(Error::Validation(
                            "Неверный тип ключа для Git роли".to_string(),
                        ));
                    }
                }
            }

            AccessKeyRole::AnsiblePasswordVault => {
                match &key.key_type {
                    AccessKeyType::LoginPassword => {
                        if let Some(lp) = &key.login_password {
                            installation.password = Some(lp.password.clone());
                            logger.log("Пароль для Ansible vault установлен");
                        } else {
                            return Err(Error::Validation("Логин/пароль не найдены".to_string()));
                        }
                    }
                    _ => {
                        return Err(Error::Validation(
                            "Неверный тип ключа для Ansible vault роли".to_string(),
                        ));
                    }
                }
            }

            AccessKeyRole::AnsibleBecomeUser => {
                if key.key_type != AccessKeyType::LoginPassword {
                    return Err(Error::Validation(
                        "Неверный тип ключа для Ansible become user роли".to_string(),
                    ));
                }
                if let Some(lp) = &key.login_password {
                    installation.login = Some(lp.login.clone());
                    installation.password = Some(lp.password.clone());
                    logger.logf("Ansible become user: {}", format_args!("{}", lp.login));
                } else {
                    return Err(Error::Validation("Логин/пароль не найдены".to_string()));
                }
            }

            AccessKeyRole::AnsibleUser => {
                match &key.key_type {
                    AccessKeyType::Ssh => {
                        if let Some(ssh_key_data) = &key.ssh_key {
                            let ssh_key = SshKey::from_string(
                                ssh_key_data.private_key.clone(),
                                if ssh_key_data.passphrase.is_empty() {
                                    None
                                } else {
                                    Some(ssh_key_data.passphrase.clone())
                                },
                            );

                            let mut agent = SshAgent::simple(
                                "localhost".to_string(),
                                ssh_key_data.login.clone(),
                                ssh_key,
                            );

                            installation.ssh_agent = Some(agent);
                            installation.login = Some(ssh_key_data.login.clone());

                            logger.logf("SSH агент запущен для Ansible user (ключ ID={})", format_args!("{}", key.id));
                        } else {
                            return Err(Error::Validation("SSH ключ не найден".to_string()));
                        }
                    }
                    AccessKeyType::LoginPassword => {
                        if let Some(lp) = &key.login_password {
                            installation.login = Some(lp.login.clone());
                            installation.password = Some(lp.password.clone());
                            logger.logf("Ansible user: {} (логин/пароль)", format_args!("{}", lp.login));
                        } else {
                            return Err(Error::Validation("Логин/пароль не найдены".to_string()));
                        }
                    }
                    AccessKeyType::None => {
                        // Нет ключа - это допустимо для Ansible user
                        logger.log("Ansible user без ключа доступа");
                    }
                }
            }
        }

        Ok(installation)
    }
}

impl Default for KeyInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod key_installer_tests {
    use super::*;
    use crate::services::task_logger::BasicLogger;

    #[test]
    fn test_access_key_role_from_str() {
        assert_eq!(
            AccessKeyRole::from_str("git").unwrap(),
            AccessKeyRole::Git
        );
        assert_eq!(
            AccessKeyRole::from_str("ansible_password_vault").unwrap(),
            AccessKeyRole::AnsiblePasswordVault
        );
        assert!(AccessKeyRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_access_key_installation_new() {
        let installation = AccessKeyInstallation::new();
        assert!(installation.ssh_agent.is_none());
        assert!(installation.login.is_none());
        assert!(installation.password.is_none());
    }

    #[test]
    fn test_access_key_installation_git_env() {
        let installation = AccessKeyInstallation::new();
        let env = installation.get_git_env();
        assert!(env.iter().any(|(k, _)| k == "GIT_TERMINAL_PROMPT"));
    }

    #[test]
    fn test_access_key_new_ssh() {
        let key = AccessKey::new_ssh(
            1,
            "private_key".to_string(),
            "passphrase".to_string(),
            "user".to_string(),
            Some(1),
        );
        assert_eq!(key.key_type, AccessKeyType::Ssh);
        assert!(key.ssh_key.is_some());
    }

    #[test]
    fn test_access_key_new_login_password() {
        let key = AccessKey::new_login_password(
            1,
            "admin".to_string(),
            "secret".to_string(),
            Some(1),
        );
        assert_eq!(key.key_type, AccessKeyType::LoginPassword);
        assert!(key.login_password.is_some());
    }

    #[test]
    fn test_key_installer_install_git_ssh() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_ssh(
            1,
            "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----".to_string(),
            "".to_string(),
            "git".to_string(),
            Some(1),
        );

        let result = installer.install(&key, AccessKeyRole::Git, &logger);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert!(installation.ssh_agent.is_some());
        assert_eq!(installation.login, Some("git".to_string()));
    }

    #[test]
    fn test_key_installer_install_ansible_password_vault() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_login_password(
            1,
            "vault".to_string(),
            "vault_pass".to_string(),
            Some(1),
        );

        let result = installer.install(&key, AccessKeyRole::AnsiblePasswordVault, &logger);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert_eq!(installation.password, Some("vault_pass".to_string()));
    }

    #[test]
    fn test_key_installer_install_ansible_become_user() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_login_password(
            1,
            "become_user".to_string(),
            "become_pass".to_string(),
            Some(1),
        );

        let result = installer.install(&key, AccessKeyRole::AnsibleBecomeUser, &logger);
        assert!(result.is_ok());

        let installation = result.unwrap();
        assert_eq!(installation.login, Some("become_user".to_string()));
        assert_eq!(installation.password, Some("become_pass".to_string()));
    }

    #[test]
    fn test_key_installer_install_ansible_user_none() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_none(1, Some(1));

        let result = installer.install(&key, AccessKeyRole::AnsibleUser, &logger);
        assert!(result.is_ok());
    }

    #[test]
    fn test_key_installer_install_invalid_role() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_login_password(
            1,
            "user".to_string(),
            "pass".to_string(),
            Some(1),
        );

        // Пытаемся использовать LoginPassword ключ для Git роли - должно быть ошибкой
        let result = installer.install(&key, AccessKeyRole::Git, &logger);
        assert!(result.is_err());
    }
}
