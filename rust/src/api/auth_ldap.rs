//! LDAP Authentication Service
//!
//! Аутентификация через LDAP/Active Directory.
//! Включается через переменную окружения SEMAPHORE_LDAP_ENABLE=true

use crate::config::config_ldap::LdapConfigFull;
use crate::error::{Error, Result};

/// Результат LDAP аутентификации
#[derive(Debug)]
pub struct LdapUserInfo {
    pub username: String,
    pub email: String,
    pub name: String,
    /// Список DN групп, в которых состоит пользователь (из атрибута memberOf)
    pub groups: Vec<String>,
}

/// Аутентифицирует пользователя через LDAP.
///
/// Выполняет:
/// 1. Bind с сервисными учётными данными (bind_dn / bind_password)
/// 2. Поиск пользователя по фильтру (search_filter содержит {username})
/// 3. Bind с найденным DN и паролем пользователя — подтверждение пароля
pub async fn ldap_authenticate(
    config: &LdapConfigFull,
    username: &str,
    password: &str,
) -> Result<LdapUserInfo> {
    use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};

    if !config.is_enabled() {
        return Err(Error::Other("LDAP is not enabled".to_string()));
    }

    if config.server.is_empty() {
        return Err(Error::Other("LDAP server is not configured".to_string()));
    }

    let settings = LdapConnSettings::new()
        .set_no_tls_verify(!config.need_tls);

    let url = config.ldap_url();

    let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &url)
        .await
        .map_err(|e| Error::Other(format!("LDAP connection failed: {}", e)))?;

    // Запускаем фоновую обработку соединения
    ldap3::drive!(conn);

    // Bind с сервисным аккаунтом для поиска пользователя
    if !config.bind_dn.is_empty() {
        ldap.simple_bind(&config.bind_dn, &config.bind_password)
            .await
            .map_err(|e| Error::Other(format!("LDAP service bind failed: {}", e)))?
            .success()
            .map_err(|e| Error::Other(format!("LDAP service bind error: {}", e)))?;
    }

    // Строим фильтр поиска: заменяем {username} на реальное имя
    let filter = if config.search_filter.is_empty() {
        format!("(uid={})", ldap_escape(username))
    } else {
        config.search_filter.replace("{username}", &ldap_escape(username))
    };

    // Атрибуты для получения
    let dn_attr = if config.mappings.dn.is_empty() { "dn" } else { &config.mappings.dn };
    let mail_attr = if config.mappings.mail.is_empty() { "mail" } else { &config.mappings.mail };
    let uid_attr = if config.mappings.uid.is_empty() { "uid" } else { &config.mappings.uid };
    let cn_attr = if config.mappings.cn.is_empty() { "cn" } else { &config.mappings.cn };

    let attrs = vec![dn_attr, mail_attr, uid_attr, cn_attr, "memberOf"];

    let search_base = config.full_search_dn();
    let (results, _res) = ldap
        .search(&search_base, Scope::Subtree, &filter, attrs)
        .await
        .map_err(|e| Error::Other(format!("LDAP search failed: {}", e)))?
        .success()
        .map_err(|e| Error::Unauthorized(format!("LDAP search error: {}", e)))?;

    let entry = results
        .into_iter()
        .next()
        .ok_or_else(|| Error::Unauthorized(format!("User '{}' not found in LDAP", username)))?;

    let entry = SearchEntry::construct(entry);
    let user_dn = entry.dn.clone();

    // Bind с DN пользователя для проверки пароля
    ldap.simple_bind(&user_dn, password)
        .await
        .map_err(|_| Error::Unauthorized("Invalid LDAP credentials".to_string()))?
        .success()
        .map_err(|_| Error::Unauthorized("Invalid LDAP credentials".to_string()))?;

    // Извлекаем атрибуты пользователя
    let email = entry
        .attrs
        .get(mail_attr)
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_else(|| format!("{}@ldap", username));

    let name = entry
        .attrs
        .get(cn_attr)
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_else(|| username.to_string());

    let resolved_username = entry
        .attrs
        .get(uid_attr)
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_else(|| username.to_string());

    // Извлекаем группы из атрибута memberOf (Active Directory / openLDAP с memberOf overlay)
    let groups = entry
        .attrs
        .get("memberOf")
        .cloned()
        .unwrap_or_default();

    ldap.unbind()
        .await
        .map_err(|e| Error::Other(format!("LDAP unbind error: {}", e)))?;

    Ok(LdapUserInfo {
        username: resolved_username,
        email,
        name,
        groups,
    })
}

/// Экранирует специальные символы в LDAP-фильтрах (RFC 4515)
fn ldap_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '*' => out.push_str("\\2a"),
            '(' => out.push_str("\\28"),
            ')' => out.push_str("\\29"),
            '\\' => out.push_str("\\5c"),
            '\0' => out.push_str("\\00"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ldap_escape_normal() {
        assert_eq!(ldap_escape("john.doe"), "john.doe");
    }

    #[test]
    fn test_ldap_escape_special_chars() {
        assert_eq!(ldap_escape("user*(name)"), "user\\2a\\28name\\29");
    }

    #[test]
    fn test_ldap_escape_backslash() {
        assert_eq!(ldap_escape("back\\slash"), "back\\5cslash");
    }

    #[tokio::test]
    async fn test_ldap_auth_disabled() {
        let config = LdapConfigFull::default(); // enable = false
        let result = ldap_authenticate(&config, "user", "pass").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not enabled"));
    }

    #[tokio::test]
    async fn test_ldap_auth_no_server() {
        let mut config = LdapConfigFull::default();
        config.enable = true;
        // server is empty
        let result = ldap_authenticate(&config, "user", "pass").await;
        assert!(result.is_err());
    }
}
