//! FFI модуль для вызова Rust из Go
//!
//! Предоставляет C-совместимый API для интеграции с Go через cgo

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use crate::services::ssh_agent::{
    AccessKeyInstallation, AccessKeyRole, AccessKeyType, AccessKey,
    SshKeyData, LoginPasswordData, KeyInstaller,
};
use crate::services::task_logger::{TaskLogger, BasicLogger, TaskStatus};
use crate::error::Error;

// ============================================================================
// C типы для Go
// ============================================================================

/// C представление AccessKeyRole
#[repr(C)]
pub enum C_AccessKeyRole {
    Git = 0,
    AnsiblePasswordVault = 1,
    AnsibleBecomeUser = 2,
    AnsibleUser = 3,
}

/// C представление AccessKeyType
#[repr(C)]
pub enum C_AccessKeyType {
    Ssh = 0,
    LoginPassword = 1,
    None = 2,
}

/// C представление AccessKey
#[repr(C)]
pub struct C_AccessKey {
    pub id: i64,
    pub key_type: C_AccessKeyType,
    pub private_key: *const c_char,
    pub passphrase: *const c_char,
    pub login: *const c_char,
    pub password: *const c_char,
    pub project_id: i64,
}

/// C представление AccessKeyInstallation
#[repr(C)]
pub struct C_AccessKeyInstallation {
    pub has_ssh_agent: bool,
    pub login: *const c_char,
    pub password: *const c_char,
    pub error: *const c_char,
}

/// C представление TaskStatus
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum C_TaskStatus {
    Waiting = 0,
    Starting = 1,
    WaitingConfirmation = 2,
    Confirmed = 3,
    Rejected = 4,
    Running = 5,
    Stopping = 6,
    Stopped = 7,
    Success = 8,
    Error = 9,
    NotExecuted = 10,
}

/// C представление Logger (opaque pointer)
pub struct C_Logger {
    _private: [u8; 0],
}

// ============================================================================
// Конвертеры
// ============================================================================

impl C_AccessKeyRole {
    fn to_rust(&self) -> AccessKeyRole {
        match self {
            C_AccessKeyRole::Git => AccessKeyRole::Git,
            C_AccessKeyRole::AnsiblePasswordVault => AccessKeyRole::AnsiblePasswordVault,
            C_AccessKeyRole::AnsibleBecomeUser => AccessKeyRole::AnsibleBecomeUser,
            C_AccessKeyRole::AnsibleUser => AccessKeyRole::AnsibleUser,
        }
    }
}

impl C_AccessKey {
    fn to_rust(&self) -> Result<AccessKey, Error> {
        let private_key = unsafe {
            if self.private_key.is_null() {
                String::new()
            } else {
                CStr::from_ptr(self.private_key).to_string_lossy().into_owned()
            }
        };

        let passphrase = unsafe {
            if self.passphrase.is_null() {
                String::new()
            } else {
                CStr::from_ptr(self.passphrase).to_string_lossy().into_owned()
            }
        };

        let login = unsafe {
            if self.login.is_null() {
                String::new()
            } else {
                CStr::from_ptr(self.login).to_string_lossy().into_owned()
            }
        };

        let password = unsafe {
            if self.password.is_null() {
                String::new()
            } else {
                CStr::from_ptr(self.password).to_string_lossy().into_owned()
            }
        };

        match self.key_type {
            C_AccessKeyType::Ssh => Ok(AccessKey::new_ssh(
                self.id,
                private_key,
                passphrase,
                login,
                if self.project_id > 0 { Some(self.project_id) } else { None },
            )),
            C_AccessKeyType::LoginPassword => Ok(AccessKey::new_login_password(
                self.id,
                login,
                password,
                if self.project_id > 0 { Some(self.project_id) } else { None },
            )),
            C_AccessKeyType::None => Ok(AccessKey::new_none(
                self.id,
                if self.project_id > 0 { Some(self.project_id) } else { None },
            )),
        }
    }
}

impl C_AccessKeyInstallation {
    fn from_rust(installation: &AccessKeyInstallation, error: Option<&str>) -> Self {
        let login = installation.login.as_ref().map(|s| CString::new(s.as_str()).unwrap());
        let password = installation.password.as_ref().map(|s| CString::new(s.as_str()).unwrap());
        let error_str = error.map(|s| CString::new(s).unwrap());

        Self {
            has_ssh_agent: installation.ssh_agent.is_some(),
            login: login.map_or(ptr::null(), |s| s.as_ptr()),
            password: password.map_or(ptr::null(), |s| s.as_ptr()),
            error: error_str.map_or(ptr::null(), |s| s.as_ptr()),
        }
    }
}

// ============================================================================
// FFI функции
// ============================================================================

/// Установить ключ доступа
///
/// # Safety
/// Эта функция небезопасна, так как работает с raw pointers
#[no_mangle]
pub unsafe extern "C" fn rust_install_access_key(
    key: *const C_AccessKey,
    role: C_AccessKeyRole,
    _logger: *mut C_Logger,
) -> C_AccessKeyInstallation {
    if key.is_null() {
        return C_AccessKeyInstallation {
            has_ssh_agent: false,
            login: ptr::null(),
            password: ptr::null(),
            error: CString::new("Null key pointer").unwrap().into_raw(),
        };
    }

    let key_ref = &*key;
    let role = role.to_rust();
    let logger = BasicLogger::new();

    let access_key = match key_ref.to_rust() {
        Ok(k) => k,
        Err(e) => {
            return C_AccessKeyInstallation {
                has_ssh_agent: false,
                login: ptr::null(),
                password: ptr::null(),
                error: CString::new(e.to_string()).unwrap().into_raw(),
            };
        }
    };

    let installer = KeyInstaller::new();

    match installer.install(&access_key, role, &logger) {
        Ok(installation) => C_AccessKeyInstallation::from_rust(&installation, None),
        Err(e) => C_AccessKeyInstallation::from_rust(
            &AccessKeyInstallation::new(),
            Some(&e.to_string()),
        ),
    }
}

/// Освободить память C_AccessKeyInstallation
///
/// # Safety
/// Эта функция должна вызываться для освобождения памяти
#[no_mangle]
pub unsafe extern "C" fn rust_free_access_key_installation(
    installation: *mut C_AccessKeyInstallation,
) {
    if installation.is_null() {
        return;
    }

    let inst = &mut *installation;

    if !inst.login.is_null() {
        let _ = CString::from_raw(inst.login as *mut c_char);
    }

    if !inst.password.is_null() {
        let _ = CString::from_raw(inst.password as *mut c_char);
    }

    if !inst.error.is_null() {
        let _ = CString::from_raw(inst.error as *mut c_char);
    }
}

/// Создать новый логгер
///
/// # Safety
/// Возвращает raw pointer, который должен быть освобождён через rust_free_logger
#[no_mangle]
pub unsafe extern "C" fn rust_create_logger() -> *mut C_Logger {
    let logger = Box::new(BasicLogger::new());
    Box::into_raw(logger) as *mut C_Logger
}

/// Освободить логгер
///
/// # Safety
/// Должен вызываться для логгеров, созданных через rust_create_logger
#[no_mangle]
pub unsafe extern "C" fn rust_free_logger(logger: *mut C_Logger) {
    if logger.is_null() {
        return;
    }
    let _ = Box::from_raw(logger as *mut BasicLogger);
}

/// Записать лог
///
/// # Safety
/// Работает с raw pointers
#[no_mangle]
pub unsafe extern "C" fn rust_logger_log(
    logger: *mut C_Logger,
    message: *const c_char,
) {
    if logger.is_null() || message.is_null() {
        return;
    }

    let logger_ref = &*(logger as *mut BasicLogger);
    let msg = CStr::from_ptr(message).to_string_lossy();
    logger_ref.log(&msg);
}

/// Установить статус логгера
///
/// # Safety
/// Работает с raw pointers
#[no_mangle]
pub unsafe extern "C" fn rust_logger_set_status(
    logger: *mut C_Logger,
    status: C_TaskStatus,
) {
    if logger.is_null() {
        return;
    }

    let logger_ref = &*(logger as *mut BasicLogger);
    let rust_status = match status {
        C_TaskStatus::Waiting => TaskStatus::Waiting,
        C_TaskStatus::Starting => TaskStatus::Starting,
        C_TaskStatus::WaitingConfirmation => TaskStatus::WaitingConfirmation,
        C_TaskStatus::Confirmed => TaskStatus::Confirmed,
        C_TaskStatus::Rejected => TaskStatus::Rejected,
        C_TaskStatus::Running => TaskStatus::Running,
        C_TaskStatus::Stopping => TaskStatus::Stopping,
        C_TaskStatus::Stopped => TaskStatus::Stopped,
        C_TaskStatus::Success => TaskStatus::Success,
        C_TaskStatus::Error => TaskStatus::Error,
        C_TaskStatus::NotExecuted => TaskStatus::NotExecuted,
    };
    logger_ref.set_status(rust_status);
}

/// Получить статус логгера
///
/// # Safety
/// Работает с raw pointers
#[no_mangle]
pub unsafe extern "C" fn rust_logger_get_status(
    logger: *mut C_Logger,
) -> C_TaskStatus {
    if logger.is_null() {
        return C_TaskStatus::Waiting;
    }

    let logger_ref = &*(logger as *mut BasicLogger);
    match logger_ref.get_status() {
        TaskStatus::Waiting => C_TaskStatus::Waiting,
        TaskStatus::Starting => C_TaskStatus::Starting,
        TaskStatus::WaitingConfirmation => C_TaskStatus::WaitingConfirmation,
        TaskStatus::Confirmed => C_TaskStatus::Confirmed,
        TaskStatus::Rejected => C_TaskStatus::Rejected,
        TaskStatus::Running => C_TaskStatus::Running,
        TaskStatus::Stopping => C_TaskStatus::Stopping,
        TaskStatus::Stopped => C_TaskStatus::Stopped,
        TaskStatus::Success => C_TaskStatus::Success,
        TaskStatus::Error => C_TaskStatus::Error,
        TaskStatus::NotExecuted => C_TaskStatus::NotExecuted,
    }
}

// ============================================================================
// Тесты
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_c_access_key_role_conversion() {
        assert_eq!(C_AccessKeyRole::Git.to_rust(), AccessKeyRole::Git);
        assert_eq!(
            C_AccessKeyRole::AnsiblePasswordVault.to_rust(),
            AccessKeyRole::AnsiblePasswordVault
        );
    }

    #[test]
    fn test_c_access_key_to_rust_ssh() {
        let private_key = CString::new("test_key").unwrap();
        let login = CString::new("test_user").unwrap();

        let c_key = C_AccessKey {
            id: 1,
            key_type: C_AccessKeyType::Ssh,
            private_key: private_key.as_ptr(),
            passphrase: ptr::null(),
            login: login.as_ptr(),
            password: ptr::null(),
            project_id: 1,
        };

        let rust_key = c_key.to_rust().unwrap();
        assert_eq!(rust_key.id, 1);
        assert_eq!(rust_key.key_type, AccessKeyType::Ssh);
    }

    #[test]
    fn test_rust_install_access_key_ssh() {
        let private_key = CString::new("-----BEGIN OPENSSH PRIVATE KEY-----\ntest\n-----END OPENSSH PRIVATE KEY-----").unwrap();
        let login = CString::new("git").unwrap();

        let c_key = C_AccessKey {
            id: 1,
            key_type: C_AccessKeyType::Ssh,
            private_key: private_key.as_ptr(),
            passphrase: ptr::null(),
            login: login.as_ptr(),
            password: ptr::null(),
            project_id: 1,
        };

        unsafe {
            let logger = rust_create_logger();
            let result = rust_install_access_key(&c_key, C_AccessKeyRole::Git, logger);
            
            // Проверяем, что SSH агент создан
            assert!(result.has_ssh_agent);
            
            // Освобождаем память
            let mut result_mut = result;
            rust_free_access_key_installation(&mut result_mut);
            rust_free_logger(logger);
        }
    }

    #[test]
    fn test_rust_logger_functions() {
        unsafe {
            let logger = rust_create_logger();
            
            // Проверяем установку и получение статуса
            rust_logger_set_status(logger, C_TaskStatus::Running);
            let status = rust_logger_get_status(logger);
            assert_eq!(status, C_TaskStatus::Running);
            
            rust_logger_set_status(logger, C_TaskStatus::Success);
            let status = rust_logger_get_status(logger);
            assert_eq!(status, C_TaskStatus::Success);
            
            rust_free_logger(logger);
        }
    }
}
