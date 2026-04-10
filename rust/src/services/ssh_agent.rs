//! SSH агент для Velum UI
//!
//! Предоставляет функциональность для:
//! - Управления SSH ключами
//! - Подключения к SSH серверам
//! - Интеграции с Git через SSH
//! - SSH agent forwarding
//! - Установки ключей доступа (KeyInstaller)

use ssh2::Session;
use std::fmt;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
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
        let tcp = TcpStream::connect(&addr)
            .map_err(|e| Error::Other(format!("Ошибка TCP подключения: {}", e)))?;

        // Устанавливаем таймаут
        tcp.set_read_timeout(Some(std::time::Duration::from_secs(
            self.config.timeout_secs as u64,
        )))
        .map_err(|e| Error::Other(format!("Ошибка установки таймаута: {}", e)))?;

        // Создаём SSH сессию
        let mut session = Session::new()
            .map_err(|e| Error::Other(format!("Ошибка создания SSH сессии: {}", e)))?;

        session.set_tcp_stream(tcp);

        // Рукопожатие
        session
            .handshake()
            .map_err(|e| Error::Other(format!("Ошибка SSH handshake: {}", e)))?;

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
    fn authenticate_with_key_static(
        session: &mut Session,
        username: &str,
        key: &SshKey,
    ) -> Result<()> {
        // Создаём временный файл для ключа
        let temp_dir = std::env::temp_dir();
        let key_file = temp_dir.join(format!("ssh_key_{}", uuid::Uuid::new_v4()));

        // Записываем ключ в файл
        std::fs::write(&key_file, &key.private_key)
            .map_err(|e| Error::Other(format!("Ошибка записи ключа: {}", e)))?;

        // Устанавливаем права доступа (только чтение для владельца)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_file, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| Error::Other(format!("Ошибка установки прав: {}", e)))?;
        }

        // Пытаемся аутентифицироваться
        let result = if let Some(passphrase) = &key.passphrase {
            session.userauth_pubkey_file(username, None, &key_file, Some(passphrase))
        } else {
            session.userauth_pubkey_file(username, None, &key_file, None)
        };

        // Удаляем временный файл
        let _ = std::fs::remove_file(&key_file);

        result.map_err(|e| Error::Other(format!("Ошибка аутентификации: {}", e)))?;

        // Проверяем успешность аутентификации
        if !session.authenticated() {
            return Err(Error::Other("Аутентификация не удалась".to_string()));
        }

        Ok(())
    }

    /// Выполняет команду на удалённом сервере
    pub fn execute_command(&self, command: &str) -> Result<SshCommandResult> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| Error::Other("SSH сессия не установлена".to_string()))?;

        let mut channel = session
            .channel_session()
            .map_err(|e| Error::Other(format!("Ошибка создания канала: {}", e)))?;

        channel
            .exec(command)
            .map_err(|e| Error::Other(format!("Ошибка выполнения команды: {}", e)))?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        channel
            .read_to_string(&mut stdout)
            .map_err(|e| Error::Other(format!("Ошибка чтения stdout: {}", e)))?;

        let mut stderr_channel = channel.stderr();
        stderr_channel
            .read_to_string(&mut stderr)
            .map_err(|e| Error::Other(format!("Ошибка чтения stderr: {}", e)))?;

        channel
            .wait_close()
            .map_err(|e| Error::Other(format!("Ошибка ожидания завершения: {}", e)))?;

        let exit_code = channel.exit_status().unwrap_or(-1);

        Ok(SshCommandResult {
            exit_code,
            stdout,
            stderr,
        })
    }

    /// Клонирует Git репозиторий через SSH
    pub fn clone_repository(&self, repo_url: &str, target_path: &Path) -> Result<()> {
        use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};

        // Создаём callback для аутентификации
        let mut callbacks = RemoteCallbacks::new();

        // Копируем ключи для closure
        let keys = self.config.keys.clone();
        let username = self.config.username.clone();

        // Настраиваем аутентификацию через SSH ключи
        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            let user = username_from_url.unwrap_or(&username);

            if let Some(key) = keys.first() {
                let private_key_str = String::from_utf8_lossy(&key.private_key);

                if let Some(passphrase) = &key.passphrase {
                    return git2::Cred::ssh_key_from_memory(
                        user,
                        Some(passphrase),
                        &private_key_str,
                        None, // Публичный ключ не обязателен
                    );
                } else {
                    return git2::Cred::ssh_key_from_memory(user, None, &private_key_str, None);
                }
            }

            Err(git2::Error::from_str("Нет доступных SSH ключей"))
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_opts);

        builder
            .clone(repo_url, target_path)
            .map_err(|e| Error::Other(format!("Ошибка клонирования репозитория: {}", e)))?;

        Ok(())
    }

    /// Закрывает подключение
    pub fn disconnect(&mut self) -> Result<()> {
        if let Some(session) = self.session.take() {
            session
                .disconnect(None, "", None)
                .map_err(|e| Error::Other(format!("Ошибка отключения: {}", e)))?;
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
        let private_key =
            fs::read(path).map_err(|e| Error::Other(format!("Ошибка чтения ключа: {}", e)))?;

        Ok(SshKey::new(private_key, passphrase.map(String::from)))
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
        let temp_dir = std::env::temp_dir().join(format!("ssh_agent_{}", uuid::Uuid::new_v4()));

        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| Error::Other(format!("Ошибка создания директории: {}", e)))?;

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
        let config = SshConfig::new("example.com".to_string(), "user".to_string()).with_port(2222);
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

    /// Создаёт новую установку с загрузкой ключа из БД по key_id
    pub fn new_with_key_id(key_id: i32) -> Self {
        // В будущей реализации здесь будет загрузка AccessKey из БД
        // и создание SSH агента или установка логина/пароля
        // Пока создаём пустую установку
        tracing::debug!("AccessKeyInstallation::new_with_key_id({})", key_id);
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
            let mut ssh_cmd =
                "ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null".to_string();
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
    pub fn new_ssh(
        id: i64,
        private_key: String,
        passphrase: String,
        login: String,
        project_id: Option<i64>,
    ) -> Self {
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
    pub fn new_login_password(
        id: i64,
        login: String,
        password: String,
        project_id: Option<i64>,
    ) -> Self {
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

    /// Получает тип ключа
    pub fn get_type(&self) -> &AccessKeyType {
        &self.key_type
    }

    /// Получает SSH ключ данные
    pub fn get_ssh_key_data(&self) -> Option<&SshKeyData> {
        self.ssh_key.as_ref()
    }

    /// Получает логин/пароль данные
    pub fn get_login_password_data(&self) -> Option<&LoginPasswordData> {
        self.login_password.as_ref()
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
                match key.get_type() {
                    AccessKeyType::Ssh => {
                        if let Some(ssh_key_data) = key.get_ssh_key_data() {
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

                            logger.logf(
                                "SSH агент запущен для ключа ID={}",
                                format_args!("{}", key.id),
                            );
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

            AccessKeyRole::AnsiblePasswordVault => match key.get_type() {
                AccessKeyType::LoginPassword => {
                    if let Some(lp) = key.get_login_password_data() {
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
            },

            AccessKeyRole::AnsibleBecomeUser => {
                if key.get_type() != &AccessKeyType::LoginPassword {
                    return Err(Error::Validation(
                        "Неверный тип ключа для Ansible become user роли".to_string(),
                    ));
                }
                if let Some(lp) = key.get_login_password_data() {
                    installation.login = Some(lp.login.clone());
                    installation.password = Some(lp.password.clone());
                    logger.logf("Ansible become user: {}", format_args!("{}", lp.login));
                } else {
                    return Err(Error::Validation("Логин/пароль не найдены".to_string()));
                }
            }

            AccessKeyRole::AnsibleUser => {
                match key.get_type() {
                    AccessKeyType::Ssh => {
                        if let Some(ssh_key_data) = key.get_ssh_key_data() {
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

                            logger.logf(
                                "SSH агент запущен для Ansible user (ключ ID={})",
                                format_args!("{}", key.id),
                            );
                        } else {
                            return Err(Error::Validation("SSH ключ не найден".to_string()));
                        }
                    }
                    AccessKeyType::LoginPassword => {
                        if let Some(lp) = key.get_login_password_data() {
                            installation.login = Some(lp.login.clone());
                            installation.password = Some(lp.password.clone());
                            logger.logf(
                                "Ansible user: {} (логин/пароль)",
                                format_args!("{}", lp.login),
                            );
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
        assert_eq!(AccessKeyRole::from_str("git").unwrap(), AccessKeyRole::Git);
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
        assert_eq!(key.get_type(), &AccessKeyType::Ssh);
        assert!(key.ssh_key.is_some());
    }

    #[test]
    fn test_access_key_new_login_password() {
        let key =
            AccessKey::new_login_password(1, "admin".to_string(), "secret".to_string(), Some(1));
        assert_eq!(key.get_type(), &AccessKeyType::LoginPassword);
        assert!(key.get_login_password_data().is_some());
    }

    #[test]
    fn test_key_installer_install_git_ssh() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_ssh(
            1,
            "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----"
                .to_string(),
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

        let key = AccessKey::new_login_password(1, "user".to_string(), "pass".to_string(), Some(1));

        // Пытаемся использовать LoginPassword ключ для Git роли - должно быть ошибкой
        let result = installer.install(&key, AccessKeyRole::Git, &logger);
        assert!(result.is_err());
    }

    // ── Additional pure function tests ──

    #[test]
    fn test_ssh_key_with_public_key() {
        let key = SshKey::new(b"private".to_vec(), None)
            .with_public_key(b"public".to_vec());
        assert_eq!(key.public_key, Some(b"public".to_vec()));
        assert_eq!(key.private_key, b"private");
    }

    #[test]
    fn test_ssh_key_passphrase_none() {
        let key = SshKey::new(b"key".to_vec(), None);
        assert!(key.passphrase.is_none());
        assert!(key.public_key.is_none());
    }

    #[test]
    fn test_ssh_config_default() {
        let config = SshConfig::default();
        assert_eq!(config.host, "");
        assert_eq!(config.port, 22);
        assert_eq!(config.username, "root");
        assert!(config.keys.is_empty());
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_ssh_config_add_key() {
        let key = SshKey::new(b"key".to_vec(), None);
        let config = SshConfig::new("host".to_string(), "user".to_string()).add_key(key);
        assert_eq!(config.keys.len(), 1);
    }

    #[test]
    fn test_ssh_config_with_timeout() {
        let config = SshConfig::new("host".to_string(), "user".to_string()).with_timeout(60);
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_ssh_config_builder_chain() {
        let key = SshKey::new(b"key".to_vec(), None);
        let config = SshConfig::new("host".to_string(), "user".to_string())
            .with_port(2222)
            .with_timeout(120)
            .add_key(key);
        assert_eq!(config.port, 2222);
        assert_eq!(config.timeout_secs, 120);
        assert_eq!(config.keys.len(), 1);
    }

    #[test]
    fn test_access_key_role_from_str_case_insensitive() {
        assert_eq!(AccessKeyRole::from_str("GIT").unwrap(), AccessKeyRole::Git);
        assert_eq!(AccessKeyRole::from_str("Git").unwrap(), AccessKeyRole::Git);
        assert_eq!(
            AccessKeyRole::from_str("ANSIBLE_PASSWORD_VAULT").unwrap(),
            AccessKeyRole::AnsiblePasswordVault
        );
    }

    #[test]
    fn test_access_key_role_from_str_all_variants() {
        assert_eq!(
            AccessKeyRole::from_str("ansible_become_user").unwrap(),
            AccessKeyRole::AnsibleBecomeUser
        );
        assert_eq!(
            AccessKeyRole::from_str("ansible_user").unwrap(),
            AccessKeyRole::AnsibleUser
        );
    }

    #[test]
    fn test_access_key_role_display() {
        assert_eq!(format!("{}", AccessKeyRole::Git), "git");
        assert_eq!(
            format!("{}", AccessKeyRole::AnsiblePasswordVault),
            "ansible_password_vault"
        );
        assert_eq!(
            format!("{}", AccessKeyRole::AnsibleBecomeUser),
            "ansible_become_user"
        );
        assert_eq!(format!("{}", AccessKeyRole::AnsibleUser), "ansible_user");
    }

    #[test]
    fn test_access_key_new_none() {
        let key = AccessKey::new_none(42, Some(7));
        assert_eq!(key.get_type(), &AccessKeyType::None);
        assert_eq!(key.id, 42);
        assert_eq!(key.project_id, Some(7));
        assert!(key.ssh_key.is_none());
        assert!(key.login_password.is_none());
    }

    #[test]
    fn test_access_key_get_ssh_key_data_none() {
        let key = AccessKey::new_login_password(1, "u".into(), "p".into(), None);
        assert!(key.get_ssh_key_data().is_none());
    }

    #[test]
    fn test_access_key_get_login_password_data_none() {
        let key = AccessKey::new_ssh(1, "pk".into(), "".into(), "u".into(), None);
        assert!(key.get_login_password_data().is_none());
    }

    #[test]
    fn test_access_key_type_variants() {
        let ssh_key = AccessKey::new_ssh(1, "pk".into(), "".into(), "u".into(), None);
        let lp_key = AccessKey::new_login_password(2, "u".into(), "p".into(), None);
        let none_key = AccessKey::new_none(3, None);

        assert_eq!(ssh_key.get_type(), &AccessKeyType::Ssh);
        assert_eq!(lp_key.get_type(), &AccessKeyType::LoginPassword);
        assert_eq!(none_key.get_type(), &AccessKeyType::None);
    }

    #[test]
    fn test_utils_validate_key_valid_rsa() {
        let key = SshKey::new(
            b"-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----".to_vec(),
            None,
        );
        assert!(utils::validate_key(&key).is_ok());
    }

    #[test]
    fn test_utils_validate_key_valid_openssh() {
        let key = SshKey::new(
            b"-----BEGIN OPENSSH PRIVATE KEY-----\ndata\n-----END OPENSSH PRIVATE KEY-----".to_vec(),
            None,
        );
        assert!(utils::validate_key(&key).is_ok());
    }

    #[test]
    fn test_utils_validate_key_garbage() {
        let key = SshKey::new(b"not a key at all".to_vec(), None);
        let err = utils::validate_key(&key).unwrap_err();
        // validate_key returns Error::Other for garbage input
        match err {
            Error::Other(msg) => assert!(msg.contains("формат") || msg.contains("ключ")),
            _ => panic!("Ожидалась ошибка, получили: {:?}", err),
        }
    }

    #[test]
    fn test_access_key_installation_new_with_key_id() {
        let installation = AccessKeyInstallation::new_with_key_id(123);
        assert!(installation.ssh_agent.is_none());
        assert!(installation.login.is_none());
    }

    #[test]
    fn test_access_key_installation_destroy_empty() {
        let mut installation = AccessKeyInstallation::new();
        assert!(installation.destroy().is_ok());
    }

    #[test]
    fn test_key_installer_git_with_login_password_fails() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();
        let key = AccessKey::new_login_password(1, "u".into(), "p".into(), None);
        let result = installer.install(&key, AccessKeyRole::Git, &logger);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_installer_ansible_vault_with_ssh_fails() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();
        let key = AccessKey::new_ssh(1, "pk".into(), "".into(), "u".into(), None);
        let result = installer.install(&key, AccessKeyRole::AnsiblePasswordVault, &logger);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_installer_ansible_become_with_ssh_fails() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();
        let key = AccessKey::new_ssh(1, "pk".into(), "".into(), "u".into(), None);
        let result = installer.install(&key, AccessKeyRole::AnsibleBecomeUser, &logger);
        assert!(result.is_err());
    }

    #[test]
    fn test_ssh_key_from_string() {
        let key = SshKey::from_string("private_key_content".to_string(), Some("pass".to_string()));
        assert_eq!(key.private_key, b"private_key_content".to_vec());
        assert_eq!(key.passphrase, Some("pass".to_string()));
    }

    #[test]
    fn test_ssh_key_with_public_key_chain() {
        let key = SshKey::new(b"priv".to_vec(), None)
            .with_public_key(b"pub".to_vec());
        assert_eq!(key.public_key, Some(b"pub".to_vec()));
    }

    #[test]
    fn test_access_key_role_from_str_error_case() {
        assert!(AccessKeyRole::from_str("invalid_role").is_err());
        assert!(AccessKeyRole::from_str("").is_err());
    }

    #[test]
    fn test_access_key_new_none_no_project() {
        let key = AccessKey::new_none(1, None);
        assert_eq!(key.id, 1);
        assert!(key.project_id.is_none());
        assert_eq!(key.get_type(), &AccessKeyType::None);
    }

    #[test]
    fn test_key_installer_ansible_user_with_login_password() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();
        let key = AccessKey::new_login_password(1, "ansible_user".to_string(), "pass".to_string(), Some(1));
        let result = installer.install(&key, AccessKeyRole::AnsibleUser, &logger);
        assert!(result.is_ok());
        let installation = result.unwrap();
        assert_eq!(installation.login, Some("ansible_user".to_string()));
        assert_eq!(installation.password, Some("pass".to_string()));
    }

    // ========================================================================
    // Расширенные тесты для SshAgent, helper функций, сериализации и enum
    // ========================================================================

    // ── SshAgent struct и методы ──

    #[test]
    fn test_ssh_agent_new() {
        let config = SshConfig::new("localhost".to_string(), "testuser".to_string());
        let agent = SshAgent::new(config);
        assert_eq!(agent.config.host, "localhost");
        assert_eq!(agent.config.username, "testuser");
        assert!(!agent.is_connected());
    }

    #[test]
    fn test_ssh_agent_simple_constructor() {
        let key = SshKey::new(b"key_data".to_vec(), None);
        let agent = SshAgent::simple("10.0.0.1".to_string(), "admin".to_string(), key);
        assert_eq!(agent.config.host, "10.0.0.1");
        assert_eq!(agent.config.port, 22);
        assert_eq!(agent.config.keys.len(), 1);
    }

    #[test]
    fn test_ssh_agent_session_is_none_initially() {
        let config = SshConfig::default();
        let agent = SshAgent::new(config);
        assert!(agent.session().is_none());
    }

    #[test]
    fn test_ssh_agent_is_connected_false() {
        let config = SshConfig::default();
        let agent = SshAgent::new(config);
        assert!(!agent.is_connected());
    }

    #[test]
    fn test_ssh_agent_disconnect_when_not_connected() {
        let mut agent = SshAgent::new(SshConfig::default());
        let result = agent.disconnect();
        assert!(result.is_ok());
    }

    #[test]
    fn test_ssh_agent_clone() {
        let config = SshConfig::new("host".to_string(), "user".to_string());
        let agent = SshAgent::new(config);
        let cloned = agent.clone();
        assert_eq!(agent.config.host, cloned.config.host);
        assert_eq!(agent.config.username, cloned.config.username);
    }

    // ── SshConfig builder pattern ──

    #[test]
    fn test_ssh_config_with_port_chain() {
        let config = SshConfig::default().with_port(8022);
        assert_eq!(config.port, 8022);
    }

    #[test]
    fn test_ssh_config_with_timeout_chain() {
        let config = SshConfig::default().with_timeout(120);
        assert_eq!(config.timeout_secs, 120);
    }

    #[test]
    fn test_ssh_config_add_multiple_keys() {
        let key1 = SshKey::new(b"key1".to_vec(), None);
        let key2 = SshKey::new(b"key2".to_vec(), Some("pass".to_string()));
        let config = SshConfig::new("host".to_string(), "user".to_string())
            .add_key(key1)
            .add_key(key2);
        assert_eq!(config.keys.len(), 2);
        assert!(config.keys[0].passphrase.is_none());
        assert_eq!(config.keys[1].passphrase, Some("pass".to_string()));
    }

    #[test]
    fn test_ssh_config_default_values() {
        let config = SshConfig::default();
        assert!(config.host.is_empty());
        assert_eq!(config.port, 22);
        assert_eq!(config.username, "root");
        assert_eq!(config.timeout_secs, 30);
        assert!(config.keys.is_empty());
    }

    // ── SshKey tests ──

    #[test]
    fn test_ssh_key_new_with_passphrase() {
        let key = SshKey::new(b"private_key".to_vec(), Some("mypass".to_string()));
        assert_eq!(key.private_key, b"private_key");
        assert_eq!(key.passphrase, Some("mypass".to_string()));
        assert!(key.public_key.is_none());
    }

    #[test]
    fn test_ssh_key_new_without_passphrase() {
        let key = SshKey::new(b"private_key".to_vec(), None);
        assert_eq!(key.private_key, b"private_key");
        assert!(key.passphrase.is_none());
        assert!(key.public_key.is_none());
    }

    #[test]
    fn test_ssh_key_from_string_with_passphrase() {
        let key = SshKey::from_string("some_key_data".to_string(), Some("secret".to_string()));
        assert_eq!(key.private_key, b"some_key_data".to_vec());
        assert_eq!(key.passphrase, Some("secret".to_string()));
    }

    #[test]
    fn test_ssh_key_from_string_without_passphrase() {
        let key = SshKey::from_string("key_data".to_string(), None);
        assert_eq!(key.private_key, b"key_data".to_vec());
        assert!(key.passphrase.is_none());
    }

    #[test]
    fn test_ssh_key_with_public_key_returns_self() {
        let key = SshKey::new(b"priv".to_vec(), None);
        let key_with_pub = key.clone().with_public_key(b"pub".to_vec());
        assert_eq!(key_with_pub.public_key, Some(b"pub".to_vec()));
        // Оригинальный key не изменён
        assert!(key.public_key.is_none());
    }

    #[test]
    fn test_ssh_key_clone() {
        let key = SshKey::new(b"key".to_vec(), Some("pass".to_string()))
            .with_public_key(b"pub".to_vec());
        let cloned = key.clone();
        assert_eq!(cloned.private_key, key.private_key);
        assert_eq!(cloned.passphrase, key.passphrase);
        assert_eq!(cloned.public_key, key.public_key);
    }

    #[test]
    fn test_ssh_key_empty_private_key() {
        let key = SshKey::new(vec![], None);
        assert!(key.private_key.is_empty());
    }

    // ── SshCommandResult tests ──

    #[test]
    fn test_ssh_command_result_debug() {
        let result = SshCommandResult {
            exit_code: 0,
            stdout: "output".to_string(),
            stderr: "".to_string(),
        };
        // Проверяем, что Debug реализован
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("SshCommandResult"));
    }

    #[test]
    fn test_ssh_command_result_with_error_code() {
        let result = SshCommandResult {
            exit_code: 127,
            stdout: "".to_string(),
            stderr: "command not found".to_string(),
        };
        assert_eq!(result.exit_code, 127);
        assert_eq!(result.stderr, "command not found");
    }

    // ── AccessKeyRole enum tests ──

    #[test]
    fn test_access_key_role_equality() {
        assert_eq!(AccessKeyRole::Git, AccessKeyRole::Git);
        assert_ne!(AccessKeyRole::Git, AccessKeyRole::AnsibleUser);
    }

    #[test]
    fn test_access_key_role_copy() {
        let role = AccessKeyRole::Git;
        let copied = role;
        assert_eq!(role, copied);
    }

    #[test]
    fn test_access_key_role_from_str_empty() {
        assert!(AccessKeyRole::from_str("").is_err());
    }

    #[test]
    fn test_access_key_role_from_str_whitespace() {
        assert!(AccessKeyRole::from_str(" git ").is_err());
    }

    #[test]
    fn test_access_key_role_display_all_variants() {
        let roles = vec![
            (AccessKeyRole::Git, "git"),
            (AccessKeyRole::AnsiblePasswordVault, "ansible_password_vault"),
            (AccessKeyRole::AnsibleBecomeUser, "ansible_become_user"),
            (AccessKeyRole::AnsibleUser, "ansible_user"),
        ];
        for (role, expected) in roles {
            assert_eq!(format!("{}", role), expected);
        }
    }

    // ── AccessKeyType enum tests ──

    #[test]
    fn test_access_key_type_equality() {
        assert_eq!(AccessKeyType::Ssh, AccessKeyType::Ssh);
        assert_eq!(AccessKeyType::LoginPassword, AccessKeyType::LoginPassword);
        assert_eq!(AccessKeyType::None, AccessKeyType::None);
        assert_ne!(AccessKeyType::Ssh, AccessKeyType::LoginPassword);
    }

    #[test]
    fn test_access_key_type_debug() {
        let ssh_type = AccessKeyType::Ssh;
        let debug_str = format!("{:?}", ssh_type);
        assert!(debug_str.contains("Ssh"));
    }

    // ── AccessKey construction tests ──

    #[test]
    fn test_access_key_ssh_with_empty_passphrase() {
        let key = AccessKey::new_ssh(1, "private".to_string(), "".to_string(), "user".to_string(), None);
        assert!(key.ssh_key.is_some());
        let ssh_data = key.get_ssh_key_data().unwrap();
        assert!(ssh_data.passphrase.is_empty());
        assert_eq!(ssh_data.login, "user");
    }

    #[test]
    fn test_access_key_ssh_with_passphrase() {
        let key = AccessKey::new_ssh(
            42,
            "encrypted_key".to_string(),
            "secret_pass".to_string(),
            "admin".to_string(),
            Some(10),
        );
        let ssh_data = key.get_ssh_key_data().unwrap();
        assert_eq!(ssh_data.passphrase, "secret_pass");
        assert_eq!(key.id, 42);
        assert_eq!(key.project_id, Some(10));
    }

    #[test]
    fn test_access_key_login_password_data() {
        let key = AccessKey::new_login_password(5, "root".to_string(), "toor".to_string(), Some(3));
        let lp_data = key.get_login_password_data().unwrap();
        assert_eq!(lp_data.login, "root");
        assert_eq!(lp_data.password, "toor");
    }

    #[test]
    fn test_access_key_ssh_data_is_none_for_login_password() {
        let key = AccessKey::new_login_password(1, "u".into(), "p".into(), None);
        assert!(key.get_ssh_key_data().is_none());
    }

    #[test]
    fn test_access_key_login_password_is_none_for_ssh() {
        let key = AccessKey::new_ssh(1, "pk".into(), "".into(), "u".into(), None);
        assert!(key.get_login_password_data().is_none());
    }

    #[test]
    fn test_access_key_none_type_has_no_data() {
        let key = AccessKey::new_none(1, None);
        assert!(key.get_ssh_key_data().is_none());
        assert!(key.get_login_password_data().is_none());
    }

    // ── SshKeyData tests ──

    #[test]
    fn test_ssh_key_data_debug() {
        let data = SshKeyData {
            private_key: "key".to_string(),
            passphrase: "pass".to_string(),
            login: "user".to_string(),
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("SshKeyData"));
    }

    #[test]
    fn test_ssh_key_data_clone() {
        let data = SshKeyData {
            private_key: "pk".to_string(),
            passphrase: "".to_string(),
            login: "u".to_string(),
        };
        let cloned = data.clone();
        assert_eq!(cloned.private_key, data.private_key);
        assert_eq!(cloned.login, data.login);
    }

    // ── LoginPasswordData tests ──

    #[test]
    fn test_login_password_data_debug() {
        let data = LoginPasswordData {
            login: "admin".to_string(),
            password: "secret".to_string(),
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("LoginPasswordData"));
    }

    #[test]
    fn test_login_password_data_clone() {
        let data = LoginPasswordData {
            login: "u".to_string(),
            password: "p".to_string(),
        };
        let cloned = data.clone();
        assert_eq!(cloned.login, data.login);
        assert_eq!(cloned.password, data.password);
    }

    // ── AccessKeyInstallation tests ──

    #[test]
    fn test_access_key_installation_default() {
        let installation = AccessKeyInstallation::default();
        assert!(installation.ssh_agent.is_none());
        assert!(installation.login.is_none());
        assert!(installation.password.is_none());
        assert!(installation.script.is_none());
    }

    #[test]
    fn test_access_key_installation_script_none_by_default() {
        let installation = AccessKeyInstallation::new();
        assert!(installation.script.is_none());
    }

    #[test]
    fn test_access_key_installation_git_env_multiple_calls() {
        let installation = AccessKeyInstallation::new();
        let env1 = installation.get_git_env();
        let env2 = installation.get_git_env();
        assert_eq!(env1.len(), env2.len());
    }

    #[test]
    fn test_access_key_installation_get_git_env_contains_terminal_prompt() {
        let installation = AccessKeyInstallation::new();
        let env = installation.get_git_env();
        let has_terminal_prompt = env.iter().any(|(k, v)| k == "GIT_TERMINAL_PROMPT" && v == "0");
        assert!(has_terminal_prompt);
    }

    #[test]
    fn test_access_key_installation_destroy_multiple_calls() {
        let mut installation = AccessKeyInstallation::new();
        assert!(installation.destroy().is_ok());
        assert!(installation.destroy().is_ok());
    }

    // ── KeyInstaller tests ──

    #[test]
    fn test_key_installer_default() {
        let installer = KeyInstaller::default();
        // KeyInstaller - unit struct, проверяем что default создаётся
        let _ = installer;
    }

    #[test]
    fn test_key_installer_new() {
        let installer = KeyInstaller::new();
        let _ = installer;
    }

    #[test]
    fn test_key_installer_install_ansible_user_ssh() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_ssh(
            10,
            "-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----".to_string(),
            "".to_string(),
            "ansible_user".to_string(),
            Some(1),
        );

        let result = installer.install(&key, AccessKeyRole::AnsibleUser, &logger);
        assert!(result.is_ok());
        let installation = result.unwrap();
        assert!(installation.ssh_agent.is_some());
        assert_eq!(installation.login, Some("ansible_user".to_string()));
    }

    #[test]
    fn test_key_installer_install_ansible_vault_wrong_key_type() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_ssh(1, "pk".into(), "".into(), "u".into(), None);
        let result = installer.install(&key, AccessKeyRole::AnsiblePasswordVault, &logger);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_installer_install_ansible_become_wrong_key_type() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_ssh(1, "pk".into(), "".into(), "u".into(), None);
        let result = installer.install(&key, AccessKeyRole::AnsibleBecomeUser, &logger);
        assert!(result.is_err());
    }

    // ── Utils tests ──

    #[test]
    fn test_utils_validate_key_with_begin_and_end_markers() {
        let key = SshKey::new(
            b"-----BEGIN DSA PRIVATE KEY-----\ntest\n-----END DSA PRIVATE KEY-----".to_vec(),
            None,
        );
        assert!(utils::validate_key(&key).is_ok());
    }

    #[test]
    fn test_utils_validate_key_only_begin_no_private_keyword() {
        // "BEGIN RSA PUBLIC KEY" содержит BEGIN, но не содержит PRIVATE KEY
        let key = SshKey::new(b"-----BEGIN RSA PUBLIC KEY-----\ntest\n-----END RSA PUBLIC KEY-----".to_vec(), None);
        let result = utils::validate_key(&key);
        assert!(result.is_err());
    }

    #[test]
    fn test_utils_validate_key_only_end() {
        let key = SshKey::new(b"no begin\n-----END PRIVATE KEY-----".to_vec(), None);
        let result = utils::validate_key(&key);
        assert!(result.is_err());
    }

    #[test]
    fn test_utils_create_temp_ssh_dir() {
        let dir = utils::create_temp_ssh_dir();
        assert!(dir.is_ok());
        let path = dir.unwrap();
        assert!(path.exists());
        assert!(path.is_dir());
        // Cleanup
        let _ = std::fs::remove_dir_all(&path);
    }

    #[test]
    fn test_utils_create_temp_ssh_dir_unique_paths() {
        let dir1 = utils::create_temp_ssh_dir().unwrap();
        let dir2 = utils::create_temp_ssh_dir().unwrap();
        assert_ne!(dir1, dir2);
        // Cleanup
        let _ = std::fs::remove_dir_all(&dir1);
        let _ = std::fs::remove_dir_all(&dir2);
    }

    #[test]
    fn test_utils_load_key_from_string_with_passphrase() {
        let key_data = "-----BEGIN EC PRIVATE KEY-----\ntest\n-----END EC PRIVATE KEY-----";
        let key = utils::load_key_from_string(key_data, Some("ec_pass"));
        assert_eq!(key.passphrase, Some("ec_pass".to_string()));
        assert!(key.private_key.len() > 0);
    }

    #[test]
    fn test_utils_load_key_from_string_without_passphrase() {
        let key_data = "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----";
        let key = utils::load_key_from_string(key_data, None);
        assert!(key.passphrase.is_none());
    }

    // ── Integration: SshAgent + SshConfig ──

    #[test]
    fn test_ssh_agent_with_full_config() {
        let key = SshKey::new(b"key".to_vec(), Some("pass".to_string()));
        let config = SshConfig::new("example.com".to_string(), "deployer".to_string())
            .with_port(2222)
            .with_timeout(60)
            .add_key(key);
        let agent = SshAgent::new(config);
        assert_eq!(agent.config.host, "example.com");
        assert_eq!(agent.config.port, 2222);
        assert_eq!(agent.config.timeout_secs, 60);
        assert_eq!(agent.config.keys.len(), 1);
    }

    #[test]
    fn test_ssh_agent_with_multiple_keys() {
        let key1 = SshKey::new(b"key1".to_vec(), None);
        let key2 = SshKey::new(b"key2".to_vec(), Some("pass2".to_string()));
        let config = SshConfig::new("host".to_string(), "user".to_string())
            .add_key(key1)
            .add_key(key2);
        let agent = SshAgent::new(config);
        assert_eq!(agent.config.keys.len(), 2);
    }

    // ── Enum serialization: Display + FromStr roundtrip ──

    #[test]
    fn test_access_key_role_roundtrip_git() {
        let role = AccessKeyRole::Git;
        let s = format!("{}", role);
        let parsed = AccessKeyRole::from_str(&s).unwrap();
        assert_eq!(role, parsed);
    }

    #[test]
    fn test_access_key_role_roundtrip_ansible_password_vault() {
        let role = AccessKeyRole::AnsiblePasswordVault;
        let s = format!("{}", role);
        let parsed = AccessKeyRole::from_str(&s).unwrap();
        assert_eq!(role, parsed);
    }

    #[test]
    fn test_access_key_role_roundtrip_ansible_become_user() {
        let role = AccessKeyRole::AnsibleBecomeUser;
        let s = format!("{}", role);
        let parsed = AccessKeyRole::from_str(&s).unwrap();
        assert_eq!(role, parsed);
    }

    #[test]
    fn test_access_key_role_roundtrip_ansible_user() {
        let role = AccessKeyRole::AnsibleUser;
        let s = format!("{}", role);
        let parsed = AccessKeyRole::from_str(&s).unwrap();
        assert_eq!(role, parsed);
    }

    // ── SshConfig Debug trait ──

    #[test]
    fn test_ssh_config_debug() {
        let config = SshConfig::new("myhost".to_string(), "myuser".to_string());
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("SshConfig"));
        assert!(debug_str.contains("myhost"));
    }

    // ── AccessKey Debug trait ──

    #[test]
    fn test_access_key_debug_ssh() {
        let key = AccessKey::new_ssh(1, "pk".into(), "".into(), "u".into(), None);
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("AccessKey"));
    }

    #[test]
    fn test_access_key_debug_login_password() {
        let key = AccessKey::new_login_password(1, "u".into(), "p".into(), None);
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("AccessKey"));
    }

    #[test]
    fn test_access_key_debug_none() {
        let key = AccessKey::new_none(1, None);
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("AccessKey"));
    }

    // ── Edge cases ──

    #[test]
    fn test_ssh_key_from_string_empty() {
        let key = SshKey::from_string("".to_string(), None);
        assert!(key.private_key.is_empty());
    }

    #[test]
    fn test_ssh_config_empty_keys() {
        let config = SshConfig::default();
        assert!(config.keys.is_empty());
    }

    #[test]
    fn test_access_key_installation_git_env_idempotent() {
        let installation = AccessKeyInstallation::new();
        let env = installation.get_git_env();
        assert!(env.iter().any(|(k, v)| k == "GIT_TERMINAL_PROMPT" && v == "0"));
    }

    #[test]
    fn test_key_installer_install_with_nonempty_passphrase() {
        let installer = KeyInstaller::new();
        let logger = BasicLogger::new();

        let key = AccessKey::new_ssh(
            1,
            "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----".to_string(),
            "encrypted".to_string(),
            "user".to_string(),
            None,
        );

        let result = installer.install(&key, AccessKeyRole::Git, &logger);
        assert!(result.is_ok());
        let installation = result.unwrap();
        assert!(installation.ssh_agent.is_some());
    }
}
