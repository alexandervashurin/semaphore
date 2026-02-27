//! System Process Attributes (Unix/Linux)
//!
//! Системные атрибуты процесса для Unix/Linux

use std::os::unix::process::CommandExt;
use std::process::Command;

/// Конфигурация процесса
#[derive(Debug, Clone, Default)]
pub struct ProcessConfig {
    pub user: Option<String>,
    pub group: Option<String>,
    pub chroot: Option<String>,
    pub gid: Option<u32>,
}

/// Применяет системные атрибуты к процессу
///
/// # Безопасность
///
/// Эта функция должна вызываться только перед exec()
pub fn configure_process_command(cmd: &mut Command, config: &ProcessConfig) -> std::io::Result<()> {
    // Устанавливаем chroot если указан
    if let Some(ref chroot) = config.chroot {
        unsafe {
            cmd.pre_exec(move || {
                use std::os::unix::ffi::OsStrExt;
                let path = std::ffi::CStr::from_bytes_with_nul_unchecked(
                    format!("{}\0", chroot).as_bytes()
                );
                libc::chroot(path.as_ptr());
                Ok(())
            });
        }
    }

    // Устанавливаем GID если указан
    if let Some(gid) = config.gid {
        unsafe {
            cmd.pre_exec(move || {
                libc::setgid(gid);
                Ok(())
            });
        }
    }

    // Устанавливаем UID если указан пользователь
    if let Some(ref username) = config.user {
        let username = username.clone();
        unsafe {
            cmd.pre_exec(move || {
                use std::ffi::CString;
                
                // Получаем UID пользователя
                let c_username = CString::new(username.as_str()).unwrap();
                let pwd = libc::getpwnam(c_username.as_ptr());
                
                if !pwd.is_null() {
                    let uid = (*pwd).pw_uid;
                    libc::setuid(uid);
                }
                
                Ok(())
            });
        }
    }

    Ok(())
}

// ============================================================================
// Windows версия (заглушка)
// ============================================================================

#[cfg(target_os = "windows")]
pub mod windows {
    use std::process::Command;

    #[derive(Debug, Clone, Default)]
    pub struct ProcessConfig {
        pub user: Option<String>,
        pub group: Option<String>,
        pub chroot: Option<String>,
        pub gid: Option<u32>,
    }

    pub fn configure_process_command(cmd: &mut Command, _config: &ProcessConfig) -> std::io::Result<()> {
        // Windows не поддерживает chroot и setuid/setgid
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_config_default() {
        let config = ProcessConfig::default();
        assert!(config.user.is_none());
        assert!(config.group.is_none());
        assert!(config.chroot.is_none());
        assert!(config.gid.is_none());
    }

    #[test]
    fn test_process_config_with_values() {
        let config = ProcessConfig {
            user: Some("testuser".to_string()),
            gid: Some(1000),
            ..Default::default()
        };
        assert_eq!(config.user, Some("testuser".to_string()));
        assert_eq!(config.gid, Some(1000));
    }
}
