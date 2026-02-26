//! Config Helpers - вспомогательные функции
//!
//! Аналог util/config.go из Go версии (часть 10: хелперы)

use std::process::Command;
use tracing::{info, warn};

/// Находит исполняемый файл semaphore в PATH
pub fn find_semaphore() -> Option<String> {
    which::which("semaphore")
        .ok()
        .and_then(|p| p.to_str().map(String::from))
}

/// Получает версию Ansible
pub fn get_ansible_version() -> Option<String> {
    let output = Command::new("ansible")
        .arg("--version")
        .output()
        .ok()?;
    
    if output.status.success() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

/// Проверяет наличие обновлений Semaphore
pub fn check_update() -> Option<String> {
    // TODO: Реализовать проверку обновлений через GitHub API
    warn!("Update check not implemented yet");
    None
}

/// Ищет и устанавливает приложения по умолчанию (ansible, terraform, etc.)
pub fn lookup_default_apps() {
    let apps = vec!["ansible", "terraform", "tofu", "terragrunt"];
    
    for app in apps {
        match which::which(app) {
            Ok(path) => info!("Found {}: {}", app, path.display()),
            Err(_) => warn!("{} not found in PATH", app),
        }
    }
}

/// Получает публичный хост из конфигурации
pub fn get_public_host() -> String {
    use std::env;
    
    env::var("SEMAPHORE_WEB_HOST")
        .unwrap_or_else(|_| "http://localhost:3000".to_string())
}

/// Генерирует код восстановления
pub fn generate_recovery_code() -> (String, String) {
    use rand::RngCore;
    
    // Генерируем случайный код
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    
    let code = format!("{:016X}", bytes);
    
    // Хешируем код
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&code);
    let hash = format!("{:x}", hasher.finalize());
    
    (code, hash)
}

/// Проверяет код восстановления
pub fn verify_recovery_code(input_code: &str, stored_hash: &str) -> bool {
    use sha2::{Sha256, Digest};
    
    // Нормализуем ввод (убираем пробелы, приводим к верхнему регистру)
    let normalized_code = input_code.replace(" ", "").to_uppercase();
    
    // Хешируем введённый код
    let mut hasher = Sha256::new();
    hasher.update(&normalized_code);
    let input_hash = format!("{:x}", hasher.finalize());
    
    // Сравниваем с сохранённым хешем
    input_hash == stored_hash
}

/// Получает публичный URL для алиаса
pub fn get_public_alias_url(scope: &str, alias: &str) -> String {
    let base_url = get_public_host();
    format!("{}/api/{}", base_url.trim_end_matches('/'), alias)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_recovery_code() {
        let (code, hash) = generate_recovery_code();
        
        assert_eq!(code.len(), 32); // 16 bytes в hex
        assert_eq!(hash.len(), 64); // SHA256 hash
        
        // Проверяем что код состоит из hex символов
        assert!(code.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_recovery_code_valid() {
        let (code, hash) = generate_recovery_code();
        assert!(verify_recovery_code(&code, &hash));
    }

    #[test]
    fn test_verify_recovery_code_invalid() {
        let (_, hash) = generate_recovery_code();
        assert!(!verify_recovery_code("INVALID_CODE", &hash));
    }

    #[test]
    fn test_verify_recovery_code_normalization() {
        let (code, hash) = generate_recovery_code();
        
        // Проверяем что пробелы игнорируются
        let code_with_spaces = code.chars()
            .enumerate()
            .map(|(i, c)| if i % 4 == 0 && i > 0 { ' ' } else { c })
            .collect::<String>();
        
        assert!(verify_recovery_code(&code_with_spaces, &hash));
    }

    #[test]
    fn test_get_public_host_default() {
        std::env::remove_var("SEMAPHORE_WEB_HOST");
        assert_eq!(get_public_host(), "http://localhost:3000");
    }

    #[test]
    fn test_get_public_host_from_env() {
        std::env::set_var("SEMAPHORE_WEB_HOST", "https://example.com");
        assert_eq!(get_public_host(), "https://example.com");
        std::env::remove_var("SEMAPHORE_WEB_HOST");
    }

    #[test]
    fn test_get_public_alias_url() {
        std::env::remove_var("SEMAPHORE_WEB_HOST");
        let url = get_public_alias_url("test", "alias123");
        assert_eq!(url, "http://localhost:3000/api/alias123");
    }
}
