//! System Process Attributes (Unix/Linux)
//!
//! Системные атрибуты процесса для Unix/Linux

#[cfg(unix)]
use nix;
#[cfg(unix)]
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
#[cfg(unix)]
pub fn configure_process_command(cmd: &mut Command, config: &ProcessConfig) -> std::io::Result<()> {
    // Устанавливаем chroot если указан
    if let Some(ref chroot) = config.chroot {
        let chroot = chroot.clone();
        unsafe {
            cmd.pre_exec(move || {
                nix::unistd::chroot(chroot.as_str()).map_err(std::io::Error::from)?;
                Ok(())
            });
        }
    }

    // Устанавливаем GID если указан
    if let Some(gid) = config.gid {
        let gid = nix::unistd::Gid::from_raw(gid);
        unsafe {
            cmd.pre_exec(move || {
                nix::unistd::setgid(gid).map_err(std::io::Error::from)?;
                Ok(())
            });
        }
    }

    // Устанавливаем UID если указан пользователь
    if let Some(ref username) = config.user {
        let username = username.clone();
        unsafe {
            cmd.pre_exec(move || {
                // Получаем UID пользователя
                let user = nix::unistd::User::from_name(&username)
                    .map_err(std::io::Error::from)?
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::NotFound, "User not found")
                    })?;
                nix::unistd::setuid(user.uid).map_err(std::io::Error::from)?;
                Ok(())
            });
        }
    }

    Ok(())
}

// ============================================================================
// Windows версия (заглушка)
// ============================================================================

#[cfg(not(unix))]
pub fn configure_process_command(
    _cmd: &mut Command,
    _config: &ProcessConfig,
) -> std::io::Result<()> {
    // Windows не поддерживает chroot и setuid/setgid
    Ok(())
}

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

    pub fn configure_process_command(
        cmd: &mut Command,
        _config: &ProcessConfig,
    ) -> std::io::Result<()> {
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
