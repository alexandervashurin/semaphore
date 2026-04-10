//! OIDC Authentication Handlers
//!
//! Обработчики для OIDC аутентификации

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{AppendHeaders, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::api::auth_local::LocalAuthService;
use crate::api::middleware::ErrorResponse;
use crate::api::state::AppState;
use crate::db::store::UserManager;
use crate::error::Error;
use oauth2::TokenResponse;

/// Первое непустое строковое значение claim по списку ключей (для OIDC userinfo).
fn oidc_first_str_claim(info: &serde_json::Value, keys: &[&str]) -> String {
    for k in keys {
        if k.is_empty() {
            continue;
        }
        if let Some(s) = info.get(*k).and_then(|v| v.as_str()) {
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    String::new()
}

// ============================================================================
// API Handlers
// ============================================================================

/// GET /api/auth/oidc/{provider} - Redirect на OIDC провайдер
pub async fn oidc_login(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
) -> std::result::Result<Redirect, (StatusCode, Json<ErrorResponse>)> {
    let provider_config = state
        .config
        .auth
        .oidc_providers
        .iter()
        .find(|p| p.display_name.to_lowercase() == provider.to_lowercase())
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(format!(
                    "OIDC provider '{}' not found",
                    provider
                ))),
            )
        })?;

    if !provider_config.is_configured() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "OIDC provider not configured".to_string(),
            )),
        ));
    }

    let auth_url = if !provider_config.endpoint.auth_url.is_empty() {
        &provider_config.endpoint.auth_url
    } else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "OIDC auth URL not configured".to_string(),
            )),
        ));
    };

    let token_url = if !provider_config.endpoint.token_url.is_empty() {
        &provider_config.endpoint.token_url
    } else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "OIDC token URL not configured".to_string(),
            )),
        ));
    };

    let client =
        oauth2::basic::BasicClient::new(oauth2::ClientId::new(provider_config.client_id.clone()))
            .set_client_secret(oauth2::ClientSecret::new(
                provider_config.client_secret.clone(),
            ))
            .set_auth_uri(oauth2::AuthUrl::new(auth_url.clone()).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Invalid auth URL: {}", e))),
                )
            })?)
            .set_token_uri(oauth2::TokenUrl::new(token_url.clone()).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!("Invalid token URL: {}", e))),
                )
            })?)
            .set_redirect_uri(
                oauth2::RedirectUrl::new(provider_config.redirect_url.clone()).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Invalid redirect URL: {}", e))),
                    )
                })?,
            );

    let (pkce_challenge, pkce_verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
    let state_param = uuid::Uuid::new_v4().to_string();

    {
        let mut cache = state.oidc_state.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to lock OIDC state".to_string())),
            )
        })?;
        cache.insert(
            state_param.clone(),
            crate::api::state::OidcState {
                pkce_verifier: pkce_verifier.secret().to_string(),
                provider: provider.clone(),
            },
        );
    }

    let (auth_url, _) = client
        .authorize_url(|| oauth2::CsrfToken::new(state_param.clone()))
        .add_scopes(
            provider_config
                .scopes
                .iter()
                .map(|s| oauth2::Scope::new(s.clone())),
        )
        .set_pkce_challenge(pkce_challenge)
        .url();

    Ok(Redirect::temporary(auth_url.as_str()))
}

/// GET /api/auth/oidc/{provider}/callback - Callback от OIDC провайдера
pub async fn oidc_callback(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> std::result::Result<
    (
        AppendHeaders<[(axum::http::HeaderName, String); 1]>,
        Redirect,
    ),
    (StatusCode, Json<ErrorResponse>),
> {
    let code = params.get("code").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Missing code parameter".to_string())),
        )
    })?;

    let state_param = params.get("state").ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("Missing state parameter".to_string())),
        )
    })?;

    let oidc_state = {
        let mut cache = state.oidc_state.lock().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new("Failed to lock OIDC state".to_string())),
            )
        })?;
        cache.remove(state_param).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new("Invalid or expired state".to_string())),
            )
        })?
    };

    if oidc_state.provider != provider {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("State mismatch".to_string())),
        ));
    }

    let provider_config = state
        .config
        .auth
        .oidc_providers
        .iter()
        .find(|p| p.display_name.to_lowercase() == provider.to_lowercase())
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse::new(format!(
                    "OIDC provider '{}' not found",
                    provider
                ))),
            )
        })?;

    let client =
        oauth2::basic::BasicClient::new(oauth2::ClientId::new(provider_config.client_id.clone()))
            .set_client_secret(oauth2::ClientSecret::new(
                provider_config.client_secret.clone(),
            ))
            .set_auth_uri(oauth2::AuthUrl::new(provider_config.endpoint.auth_url.clone()).map_err(
                |e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Invalid auth URL: {}", e))),
                    )
                },
            )?)
            .set_token_uri(oauth2::TokenUrl::new(provider_config.endpoint.token_url.clone()).map_err(
                |e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Invalid token URL: {}", e))),
                    )
                },
            )?)
            .set_redirect_uri(
                oauth2::RedirectUrl::new(provider_config.redirect_url.clone()).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Invalid redirect URL: {}", e))),
                    )
                })?,
            );

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(format!(
                    "HTTP client build failed: {}",
                    e
                ))),
            )
        })?;

    let token_result = client
        .exchange_code(oauth2::AuthorizationCode::new(code.clone()))
        .set_pkce_verifier(oauth2::PkceCodeVerifier::new(oidc_state.pkce_verifier))
        .request_async(&http_client)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(format!("Token exchange failed: {}", e))),
            )
        })?;

    let access_token = token_result.access_token().secret();
    let userinfo_url = if !provider_config.endpoint.userinfo_url.is_empty() {
        provider_config.endpoint.userinfo_url.clone()
    } else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                "OIDC userinfo URL not configured".to_string(),
            )),
        ));
    };

    let client = reqwest::Client::new();
    let userinfo: serde_json::Value = client
        .get(&userinfo_url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse::new(format!(
                    "Userinfo request failed: {}",
                    e
                ))),
            )
        })?
        .json()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse::new(format!("Userinfo parse failed: {}", e))),
            )
        })?;

    let ec = provider_config.email_claim.as_str();
    let uc = provider_config.username_claim.as_str();
    let nc = provider_config.name_claim.as_str();

    let email = oidc_first_str_claim(&userinfo, &[ec, "email", "mail", "upn"]);
    let mut username = oidc_first_str_claim(&userinfo, &[uc, "preferred_username", "name", "sub"]);
    if username.is_empty() {
        username = email.clone();
    }
    let mut name = oidc_first_str_claim(&userinfo, &[nc, "name", "preferred_username"]);
    if name.is_empty() {
        name = username.clone();
    }

    if email.is_empty() && username.is_empty() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(ErrorResponse::new(
                "OIDC userinfo missing email/username".to_string(),
            )),
        ));
    }

    let user = match state
        .store
        .get_user_by_login_or_email(&username, &email)
        .await
    {
        Ok(u) => u,
        Err(Error::NotFound(_)) => {
            let password_hash = crate::api::auth_local::hash_password(
                &uuid::Uuid::new_v4().to_string(),
            )
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse::new(format!(
                        "Failed to hash password: {}",
                        e
                    ))),
                )
            })?;
            let new_user = crate::models::User {
                id: 0,
                created: chrono::Utc::now(),
                username: username.clone(),
                name: name.clone(),
                email: email.clone(),
                password: password_hash.clone(),
                admin: false,
                external: true,
                alert: false,
                pro: false,
                totp: None,
                email_otp: None,
            };
            state
                .store
                .create_user(new_user, &password_hash)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse::new(format!("Failed to create user: {}", e))),
                    )
                })?
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse::new(e.to_string())),
            ))
        }
    };

    let auth_service = LocalAuthService::new(state.store.clone());
    let token_info = auth_service.generate_token(&user).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new(format!(
                "Token generation failed: {}",
                e
            ))),
        )
    })?;

    let base_url = std::env::var("SEMAPHORE_WEB_ROOT").unwrap_or_else(|_| "/".to_string());
    let redirect_url = format!(
        "{}?token={}",
        base_url.trim_end_matches('/'),
        token_info.token
    );

    let cookie_value = format!(
        "semaphore={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        token_info.token, token_info.expires_in
    );

    let headers = AppendHeaders([(header::SET_COOKIE, cookie_value)]);

    Ok((headers, Redirect::temporary(&redirect_url)))
}

/// GET /api/auth/login - Metadata для login страницы
pub async fn get_login_metadata(
    State(state): State<Arc<AppState>>,
) -> std::result::Result<Json<LoginMetadataResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Получаем OIDC провайдеры из конфига
    let oidc_providers: Vec<OidcProviderMetadata> = state
        .config
        .auth
        .oidc_providers
        .iter()
        .map(|p| OidcProviderMetadata {
            name: p.display_name.clone(),
            color: p.color.clone(),
            icon: p.icon.clone(),
            login_url: format!("/api/auth/oidc/{}", p.display_name.to_lowercase()),
        })
        .collect();

    Ok(Json(LoginMetadataResponse {
        oidc_providers,
        totp_enabled: state.config.auth.totp.enable,
        email_enabled: state.config.auth.email_login_enabled,
        login_with_password: true, // Включаем форму username+password для локальной аутентификации
    }))
}

// ============================================================================
// Types
// ============================================================================

/// Metadata для OIDC провайдера
#[derive(Debug, Serialize, Deserialize)]
pub struct OidcProviderMetadata {
    pub name: String,
    pub color: String,
    pub icon: String,
    pub login_url: String,
}

/// Response для login metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginMetadataResponse {
    pub oidc_providers: Vec<OidcProviderMetadata>,
    pub totp_enabled: bool,
    pub email_enabled: bool,
    /// Когда true, Vue показывает форму username+password вместо magic link
    #[serde(rename = "login_with_password")]
    pub login_with_password: bool,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oidc_first_str_claim_respects_key_order() {
        let v = serde_json::json!({"upn": "u@corp.example", "email": "e@corp.example"});
        assert_eq!(
            oidc_first_str_claim(&v, &["upn", "email"]),
            "u@corp.example"
        );
        assert_eq!(
            oidc_first_str_claim(&v, &["email", "upn"]),
            "e@corp.example"
        );
    }

    #[test]
    fn test_oidc_provider_metadata_serialization() {
        let metadata = OidcProviderMetadata {
            name: "Google".to_string(),
            color: "#4285F4".to_string(),
            icon: "google".to_string(),
            login_url: "/api/auth/oidc/google".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("Google"));
        assert!(json.contains("#4285F4"));
    }

    #[test]
    fn test_login_metadata_response_serialization() {
        let response = LoginMetadataResponse {
            oidc_providers: vec![],
            totp_enabled: false,
            email_enabled: true,
            login_with_password: true,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("false"));
        assert!(json.contains("true"));
    }

    // ========================================================================
    // Tests for oidc_first_str_claim helper
    // ========================================================================

    #[test]
    fn test_oidc_first_str_claim_single_existing_key() {
        let v = serde_json::json!({"email": "user@example.com"});
        assert_eq!(
            oidc_first_str_claim(&v, &["email"]),
            "user@example.com"
        );
    }

    #[test]
    fn test_oidc_first_str_claim_empty_keys_list() {
        let v = serde_json::json!({"email": "user@example.com"});
        assert_eq!(oidc_first_str_claim(&v, &[]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_skips_empty_keys() {
        let v = serde_json::json!({"email": "user@example.com"});
        assert_eq!(
            oidc_first_str_claim(&v, &["", "email"]),
            "user@example.com"
        );
    }

    #[test]
    fn test_oidc_first_str_claim_nonexistent_key() {
        let v = serde_json::json!({"email": "user@example.com"});
        assert_eq!(oidc_first_str_claim(&v, &["missing"]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_non_string_value() {
        let v = serde_json::json!({"count": 42, "email": "user@example.com"});
        assert_eq!(oidc_first_str_claim(&v, &["count"]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_empty_string_value() {
        let v = serde_json::json!({"email": "", "name": "John"});
        assert_eq!(
            oidc_first_str_claim(&v, &["email", "name"]),
            "John"
        );
    }

    #[test]
    fn test_oidc_first_str_claim_all_empty_strings() {
        let v = serde_json::json!({"a": "", "b": ""});
        assert_eq!(oidc_first_str_claim(&v, &["a", "b"]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_null_value() {
        let v = serde_json::json!({"email": null});
        assert_eq!(oidc_first_str_claim(&v, &["email"]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_boolean_value() {
        let v = serde_json::json!({"active": true});
        assert_eq!(oidc_first_str_claim(&v, &["active"]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_array_value() {
        let v = serde_json::json!({"roles": ["admin", "user"]});
        assert_eq!(oidc_first_str_claim(&v, &["roles"]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_nested_object_value() {
        let v = serde_json::json!({"info": {"email": "a@b.com"}});
        assert_eq!(oidc_first_str_claim(&v, &["info"]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_fallback_chain() {
        let v = serde_json::json!({
            "preferred_username": "johnd",
            "name": "John Doe",
            "sub": "12345"
        });
        assert_eq!(
            oidc_first_str_claim(&v, &["preferred_username", "name", "sub"]),
            "johnd"
        );
        assert_eq!(
            oidc_first_str_claim(&v, &["missing", "name", "sub"]),
            "John Doe"
        );
        assert_eq!(
            oidc_first_str_claim(&v, &["missing1", "missing2", "sub"]),
            "12345"
        );
    }

    #[test]
    fn test_oidc_first_str_claim_unicode_value() {
        let v = serde_json::json!({"name": "Иван"});
        assert_eq!(oidc_first_str_claim(&v, &["name"]), "Иван");
    }

    // ========================================================================
    // Tests for OidcProviderMetadata
    // ========================================================================

    #[test]
    fn test_oidc_provider_metadata_default_fields() {
        let metadata = OidcProviderMetadata {
            name: String::new(),
            color: String::new(),
            icon: String::new(),
            login_url: String::new(),
        };
        assert!(metadata.name.is_empty());
        assert!(metadata.color.is_empty());
        assert!(metadata.icon.is_empty());
        assert!(metadata.login_url.is_empty());
    }

    #[test]
    fn test_oidc_provider_metadata_deserialization() {
        let json = r##"{"name":"Azure","color":"#0078D4","icon":"azure","login_url":"/api/auth/oidc/azure"}"##;
        let metadata: OidcProviderMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.name, "Azure");
        assert_eq!(metadata.color, "#0078D4");
        assert_eq!(metadata.icon, "azure");
        assert_eq!(metadata.login_url, "/api/auth/oidc/azure");
    }

    #[test]
    fn test_oidc_provider_metadata_roundtrip() {
        let original = OidcProviderMetadata {
            name: "Keycloak".to_string(),
            color: "#4C4C4C".to_string(),
            icon: "keycloak".to_string(),
            login_url: "/api/auth/oidc/keycloak".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: OidcProviderMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.color, deserialized.color);
        assert_eq!(original.icon, deserialized.icon);
        assert_eq!(original.login_url, deserialized.login_url);
    }

    // ========================================================================
    // Tests for LoginMetadataResponse
    // ========================================================================

    #[test]
    fn test_login_metadata_response_with_providers() {
        let response = LoginMetadataResponse {
            oidc_providers: vec![
                OidcProviderMetadata {
                    name: "Google".to_string(),
                    color: "#4285F4".to_string(),
                    icon: "google".to_string(),
                    login_url: "/api/auth/oidc/google".to_string(),
                },
                OidcProviderMetadata {
                    name: "GitHub".to_string(),
                    color: "#24292E".to_string(),
                    icon: "github".to_string(),
                    login_url: "/api/auth/oidc/github".to_string(),
                },
            ],
            totp_enabled: true,
            email_enabled: false,
            login_with_password: true,
        };
        assert_eq!(response.oidc_providers.len(), 2);
        assert!(response.totp_enabled);
        assert!(!response.email_enabled);
        assert!(response.login_with_password);
    }

    #[test]
    fn test_login_metadata_response_all_disabled() {
        let response = LoginMetadataResponse {
            oidc_providers: vec![],
            totp_enabled: false,
            email_enabled: false,
            login_with_password: false,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("totp_enabled"));
        assert!(json.contains("email_enabled"));
        assert!(json.contains("login_with_password"));
    }

    #[test]
    fn test_login_metadata_response_deserialization() {
        let json = r#"{"oidc_providers":[],"totp_enabled":true,"email_enabled":true,"login_with_password":false}"#;
        let response: LoginMetadataResponse = serde_json::from_str(json).unwrap();
        assert!(response.totp_enabled);
        assert!(response.email_enabled);
        assert!(!response.login_with_password);
    }

    #[test]
    fn test_login_metadata_response_roundtrip() {
        let original = LoginMetadataResponse {
            oidc_providers: vec![OidcProviderMetadata {
                name: "Okta".to_string(),
                color: "#007DC1".to_string(),
                icon: "okta".to_string(),
                login_url: "/api/auth/oidc/okta".to_string(),
            }],
            totp_enabled: true,
            email_enabled: true,
            login_with_password: true,
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: LoginMetadataResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(original.oidc_providers.len(), deserialized.oidc_providers.len());
        assert_eq!(original.totp_enabled, deserialized.totp_enabled);
        assert_eq!(original.email_enabled, deserialized.email_enabled);
        assert_eq!(
            original.login_with_password,
            deserialized.login_with_password
        );
    }

    // ========================================================================
    // Edge-case tests
    // ========================================================================

    #[test]
    fn test_oidc_first_str_claim_special_characters_in_value() {
        let v = serde_json::json!({"email": "user+tag@example.com"});
        assert_eq!(
            oidc_first_str_claim(&v, &["email"]),
            "user+tag@example.com"
        );
    }

    #[test]
    fn test_oidc_first_str_claim_very_long_value() {
        let long_value = "a".repeat(1000);
        let v = serde_json::json!({"claim": long_value.clone()});
        assert_eq!(oidc_first_str_claim(&v, &["claim"]), long_value);
    }

    #[test]
    fn test_oidc_first_str_claim_whitespace_only_value() {
        let v = serde_json::json!({"name": "   "});
        assert_eq!(oidc_first_str_claim(&v, &["name"]), "   ");
    }

    #[test]
    fn test_oidc_provider_metadata_empty_serialization() {
        let metadata = OidcProviderMetadata {
            name: String::new(),
            color: String::new(),
            icon: String::new(),
            login_url: String::new(),
        };
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"name\":\"\""));
    }

    // ========================================================================
    // Tests for serde_json::Value edge cases in claim extraction
    // ========================================================================

    #[test]
    fn test_oidc_first_str_claim_empty_json_object() {
        let v = serde_json::json!({});
        assert_eq!(oidc_first_str_claim(&v, &["any"]), "");
    }

    #[test]
    fn test_oidc_first_str_claim_mixed_types() {
        let v = serde_json::json!({
            "a": null,
            "b": 123,
            "c": "valid",
            "d": true
        });
        assert_eq!(oidc_first_str_claim(&v, &["a", "b", "c", "d"]), "valid");
    }
}
