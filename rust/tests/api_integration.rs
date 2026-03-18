//! Integration tests for the Velum REST API
//!
//! Tests run against a real SQLite in-memory-like (temp-file) database and
//! the actual Axum router.  No mocks, no stubs — every test hits the full
//! HTTP stack, exactly like a real client would.
//!
//! Pattern:
//!   1. Spin up `create_app(SqlStore)` backed by a fresh temp-file DB.
//!   2. Send HTTP requests via `tower::ServiceExt::oneshot`.
//!   3. Assert status codes and JSON payloads.
//!
//! Run with:
//!   cargo test --test api_integration

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt; // .collect() on response body
use serde_json::{json, Value};
use semaphore_ffi::{api::create_app, db::SqlStore};
use tower::ServiceExt; // .oneshot()

// ── helpers ───────────────────────────────────────────────────────────────

/// Convert a Windows path (possibly with \\?\ prefix) to a sqlite:// URL.
fn sqlite_url_from_path(path: &std::path::Path) -> String {
    // canonicalize() resolves the \\?\ extended-length prefix on Windows
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    // Strip \\?\ or \\?\UNC\ prefixes that Windows sometimes adds
    let path_str = canonical.to_string_lossy();
    let stripped = path_str
        .strip_prefix(r"\\?\UNC\")
        .map(|s| format!("//{}",  s))
        .or_else(|| path_str.strip_prefix(r"\\?\").map(|s| s.to_string()))
        .unwrap_or_else(|| path_str.into_owned());
    // Forward slashes
    let forward = stripped.replace('\\', "/");
    format!("sqlite:///{}", forward)
}

/// Create a fresh Axum app backed by a brand-new temp-file SQLite database.
/// The `NamedTempFile` is returned so it is kept alive for the duration of
/// the test.  When it drops, the temp file is deleted automatically.
async fn test_app() -> (
    axum::Router,
    tempfile::NamedTempFile,
) {
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let url = sqlite_url_from_path(temp.path());

    let store = SqlStore::new(&url).await.expect("SqlStore::new");
    let app = create_app(std::sync::Arc::new(store));

    (app, temp)
}

/// POST JSON body, return (status, parsed JSON value).
async fn post_json(
    app: axum::Router,
    uri: &str,
    body: Value,
) -> (StatusCode, Value) {
    post_json_with_token(app, uri, body, None).await
}

/// POST JSON body with optional Bearer token.
async fn post_json_with_token(
    app: axum::Router,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let body_str = serde_json::to_string(&body).unwrap();
    let mut req = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(tok) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {}", tok));
    }

    let request = req.body(Body::from(body_str)).unwrap();
    let response = app.oneshot(request).await.expect("oneshot");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// DELETE with optional Bearer token, returns status code only.
async fn delete_req(
    app: axum::Router,
    uri: &str,
    token: Option<&str>,
) -> StatusCode {
    let mut req = Request::builder().method("DELETE").uri(uri);
    if let Some(tok) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {}", tok));
    }
    let request = req.body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.expect("oneshot");
    response.status()
}

/// PUT JSON body with optional Bearer token.
async fn put_json(
    app: axum::Router,
    uri: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let body_str = serde_json::to_string(&body).unwrap();
    let mut req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(tok) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {}", tok));
    }
    let request = req.body(Body::from(body_str)).unwrap();
    let response = app.oneshot(request).await.expect("oneshot");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// GET with optional Bearer token.
async fn get_json(
    app: axum::Router,
    uri: &str,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method("GET").uri(uri);

    if let Some(tok) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {}", tok));
    }

    let request = req.body(Body::empty()).unwrap();
    let response = app.oneshot(request).await.expect("oneshot");
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// Register an admin user and return the access token.
async fn register_and_login(app: &axum::Router) -> String {
    // Register via CLI would require spawning a process; instead we create a
    // user directly through the store.  Because `create_app` takes ownership
    // of the store, we log in as the user that was pre-seeded in the DB.
    //
    // For integration tests we rely on the admin user being creatable through
    // the POST /api/users endpoint (which some semaphore builds expose) or we
    // use a dedicated test-only setup helper.
    //
    // Here we use the login endpoint — but first we need a user in the DB.
    // We call an internal helper instead of going through HTTP to avoid
    // circular dependency.
    //
    // Strategy: create a user through the DB store directly before calling
    // create_app.  Since we can't do that here (app already built), we use
    // the same temp DB and build a second, ephemeral store just for seeding.
    //
    // Simpler approach for integration tests:
    // We just test the happy-path of login assuming the seeding has been done.
    // The `seed_test_db` helper below creates the user before building the app.
    //
    // This function is only valid when the app was created via `seeded_app()`.
    let (status, body) = post_json(
        app.clone(),
        "/api/auth/login",
        json!({ "username": "testadmin", "password": "Test1234!" }),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "login should succeed; body={:?}",
        body
    );

    body["token"]
        .as_str()
        .expect("token in login response")
        .to_string()
}

/// Create an app with one pre-seeded admin user `testadmin / Test1234!`.
async fn seeded_app() -> (axum::Router, tempfile::NamedTempFile) {
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let url = sqlite_url_from_path(temp.path());

    // Seed the user into the DB before building the app
    {
        use semaphore_ffi::db::store::UserManager;
        use semaphore_ffi::models::User;
        use chrono::Utc;

        let store = SqlStore::new(&url).await.expect("SqlStore::new for seeding");

        let user = User {
            id: 0,
            username: "testadmin".into(),
            name: "Test Admin".into(),
            email: "testadmin@test.local".into(),
            password: String::new(), // password set via create_user second arg
            admin: true,
            external: false,
            alert: false,
            pro: false,
            created: Utc::now(),
            totp: None,
            email_otp: None,
        };

        store.create_user(user, "Test1234!").await.expect("create test user");
    }

    let store = SqlStore::new(&url).await.expect("SqlStore::new for app");
    let app = create_app(std::sync::Arc::new(store));

    (app, temp)
}

// ── tests ─────────────────────────────────────────────────────────────────

// ── Health check ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_check() {
    let (app, _temp) = test_app().await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// ── Auth: login / logout / refresh ────────────────────────────────────────

#[tokio::test]
async fn test_login_wrong_password_returns_401() {
    let (app, _temp) = seeded_app().await;

    let (status, body) = post_json(
        app,
        "/api/auth/login",
        json!({ "username": "testadmin", "password": "wrongpassword" }),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "body={:?}", body);
}

#[tokio::test]
async fn test_login_unknown_user_returns_401() {
    let (app, _temp) = seeded_app().await;

    let (status, body) = post_json(
        app,
        "/api/auth/login",
        json!({ "username": "nobody", "password": "whatever" }),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED, "body={:?}", body);
}

#[tokio::test]
async fn test_login_success_returns_token() {
    let (app, _temp) = seeded_app().await;

    let (status, body) = post_json(
        app,
        "/api/auth/login",
        json!({ "username": "testadmin", "password": "Test1234!" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body={:?}", body);
    assert!(
        body["token"].is_string(),
        "response must contain 'token'; body={:?}",
        body
    );
}

#[tokio::test]
async fn test_refresh_token() {
    let (app, _temp) = seeded_app().await;

    // Login first
    let (_, login_body) = post_json(
        app.clone(),
        "/api/auth/login",
        json!({ "username": "testadmin", "password": "Test1234!" }),
    )
    .await;

    let refresh_token = login_body["refresh_token"]
        .as_str()
        .expect("refresh_token in login response");

    // Use refresh token
    let (status, body) = post_json(
        app,
        "/api/auth/refresh",
        json!({ "refresh_token": refresh_token }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "refresh should succeed; body={:?}", body);
    assert!(body["token"].is_string(), "new token in refresh response; body={:?}", body);
}

// ── Auth: protected endpoints require token ────────────────────────────────

#[tokio::test]
async fn test_get_projects_requires_auth() {
    let (app, _temp) = test_app().await;

    let (status, _body) = get_json(app, "/api/projects", None).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "unauthenticated request must return 401"
    );
}

#[tokio::test]
async fn test_get_current_user_requires_auth() {
    let (app, _temp) = test_app().await;

    let (status, _body) = get_json(app, "/api/user", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

// ── Projects CRUD ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_projects() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    // Create project
    let (status, body) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({
            "name": "Integration Test Project",
            "max_parallel_tasks": 1,
            "alert": false
        }),
        Some(&token),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED, "create project; body={:?}", body);
    let project_id = body["id"].as_i64().expect("project id in response");

    // List projects
    let (status, list_body) = get_json(app.clone(), "/api/projects", Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "list projects; body={:?}", list_body);

    let projects = list_body.as_array().expect("projects array");
    assert!(
        projects.iter().any(|p| p["id"].as_i64() == Some(project_id)),
        "created project must appear in list"
    );
}

#[tokio::test]
async fn test_delete_project() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    // Create project
    let (status, body) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "To Delete", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "body={:?}", body);
    let project_id = body["id"].as_i64().expect("project id");

    // Delete project
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/api/projects/{}", project_id))
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::NO_CONTENT
            || response.status() == StatusCode::OK,
        "delete project returned {:?}",
        response.status()
    );
}

// ── Users API ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_current_user_with_token() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (status, body) = get_json(app, "/api/user", Some(&token)).await;

    assert_eq!(status, StatusCode::OK, "body={:?}", body);
    assert_eq!(
        body["username"].as_str(),
        Some("testadmin"),
        "current user username; body={:?}",
        body
    );
}

// ── Access Keys CRUD ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_access_keys() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    // Create a project first
    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Keys Project", "max_parallel_tasks": 1, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    // Create a 'none' key (simplest type, no secrets needed)
    let (status, body) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/keys", project_id),
        json!({ "name": "no-op key", "type": "none" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create key; body={:?}", body);
    let key_id = body["id"].as_i64().expect("key id");

    // List keys
    let (status, list) = get_json(
        app.clone(),
        &format!("/api/projects/{}/keys", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list keys; body={:?}", list);
    let keys = list.as_array().expect("keys array");
    assert!(
        keys.iter().any(|k| k["id"].as_i64() == Some(key_id)),
        "created key must appear in list"
    );
}

// ── Inventories CRUD ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_inventories() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    // Create project
    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Inv Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    // Create inventory
    let (status, body) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/inventories", project_id),
        json!({ "name": "localhost", "inventory": "static" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create inventory; body={:?}", body);
    let inv_id = body["id"].as_i64().expect("inventory id");

    // List inventories
    let (status, list) = get_json(
        app.clone(),
        &format!("/api/projects/{}/inventories", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list inventories; body={:?}", list);
    let invs = list.as_array().expect("inventories array");
    assert!(
        invs.iter().any(|i| i["id"].as_i64() == Some(inv_id)),
        "created inventory must appear in list"
    );
}

#[tokio::test]
async fn test_get_inventory_not_found() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "NF Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (status, _) = get_json(
        app,
        &format!("/api/projects/{}/inventories/99999", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Repositories CRUD ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_repositories() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Repo Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (status, body) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/repositories", project_id),
        json!({
            "name": "test-repo",
            "git_url": "https://github.com/example/repo.git"
        }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create repo; body={:?}", body);
    let repo_id = body["id"].as_i64().expect("repo id");

    let (status, list) = get_json(
        app,
        &format!("/api/projects/{}/repositories", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list repos; body={:?}", list);
    let repos = list.as_array().expect("repos array");
    assert!(repos.iter().any(|r| r["id"].as_i64() == Some(repo_id)));
}

// ── Tasks CRUD ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_tasks_empty() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Tasks Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (status, body) = get_json(
        app,
        &format!("/api/projects/{}/tasks", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list tasks; body={:?}", body);
    // Empty project has no tasks
    let tasks = body.as_array().expect("tasks array");
    assert!(tasks.is_empty(), "new project should have no tasks");
}

// ── Environments CRUD ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_environments() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Env Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (status, body) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/environments", project_id),
        json!({
            "project_id": project_id,
            "name": "production",
            "json": "{\"DEPLOY_ENV\": \"prod\"}"
        }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create env; body={:?}", body);
    let env_id = body["id"].as_i64().expect("env id");

    let (status, list) = get_json(
        app,
        &format!("/api/projects/{}/environments", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list envs; body={:?}", list);
    let envs = list.as_array().expect("environments array");
    assert!(envs.iter().any(|e| e["id"].as_i64() == Some(env_id)));
}

// ── Templates CRUD ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_templates() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Tpl Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (status, body) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({
            "name": "deploy",
            "playbook": "deploy.yml"
        }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create template; body={:?}", body);
    let tpl_id = body["id"].as_i64().expect("template id");

    let (status, list) = get_json(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list templates; body={:?}", list);
    let tpls = list.as_array().expect("templates array");
    assert!(tpls.iter().any(|t| t["id"].as_i64() == Some(tpl_id)), "created template must appear in list");
}

#[tokio::test]
async fn test_get_template_by_id() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "TplGet Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (_, created) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({ "name": "my-tpl", "playbook": "site.yml" }),
        Some(&token),
    )
    .await;
    let tpl_id = created["id"].as_i64().expect("template id");

    let (status, got) = get_json(
        app.clone(),
        &format!("/api/projects/{}/templates/{}", project_id, tpl_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get template; body={:?}", got);
    assert_eq!(got["id"].as_i64(), Some(tpl_id));
    assert_eq!(got["name"].as_str(), Some("my-tpl"));
}

#[tokio::test]
async fn test_get_template_not_found() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Tpl404", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (status, _) = get_json(
        app.clone(),
        &format!("/api/projects/{}/templates/99999", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── GET project by ID ────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_project_by_id() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, body) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Get By ID Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = body["id"].as_i64().expect("project id");

    let (status, got) = get_json(
        app.clone(),
        &format!("/api/project/{}", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get project by id; body={:?}", got);
    assert_eq!(got["id"].as_i64(), Some(project_id));
    assert_eq!(got["name"].as_str(), Some("Get By ID Project"));
}

// ── GET project not found ─────────────────────────────────────────────────

#[tokio::test]
async fn test_get_project_not_found() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (status, _) = get_json(app.clone(), "/api/project/99999", Some(&token)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── DELETE access key ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_access_key() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Key Delete Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (_, key_body) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/keys", project_id),
        json!({ "name": "temp-key", "type": "none" }),
        Some(&token),
    )
    .await;
    let key_id = key_body["id"].as_i64().expect("key id");

    let status = delete_req(
        app.clone(),
        &format!("/api/projects/{}/keys/{}", project_id, key_id),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::NO_CONTENT || status == StatusCode::OK,
        "delete key returned {:?}",
        status
    );

    // Key should no longer appear in list
    let (_, list) = get_json(
        app.clone(),
        &format!("/api/projects/{}/keys", project_id),
        Some(&token),
    )
    .await;
    let empty = vec![];
    let keys = list.as_array().unwrap_or(&empty);
    assert!(
        !keys.iter().any(|k| k["id"].as_i64() == Some(key_id)),
        "deleted key must not appear in list"
    );
}

// ── Schedules CRUD ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_schedules() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Sched Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    // Need a template for the schedule
    let (_, tpl) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({ "name": "sched-tpl", "playbook": "cron.yml" }),
        Some(&token),
    )
    .await;
    let tpl_id = tpl["id"].as_i64().expect("template id");

    let (status, body) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/schedules", project_id),
        json!({
            "id": 0,
            "template_id": tpl_id,
            "project_id": project_id,
            "name": "hourly",
            "cron": "0 * * * *",
            "active": true
        }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create schedule; body={:?}", body);
    let sched_id = body["id"].as_i64().expect("schedule id");

    let (status, list) = get_json(
        app.clone(),
        &format!("/api/projects/{}/schedules", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list schedules; body={:?}", list);
    let scheds = list.as_array().expect("schedules array");
    assert!(
        scheds.iter().any(|s| s["id"].as_i64() == Some(sched_id)),
        "created schedule must appear in list"
    );
}

// ── Update project name (PUT) ─────────────────────────────────────────────

#[tokio::test]
async fn test_update_project_name() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, body) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Before Update", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = body["id"].as_i64().expect("project id");

    let (status, updated) = put_json(
        app.clone(),
        &format!("/api/projects/{}", project_id),
        json!({ "name": "After Update", "max_parallel_tasks": 2, "alert": false }),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::NO_CONTENT,
        "update project returned {:?}; body={:?}",
        status,
        updated
    );

    // Verify the change persisted
    let (status, got) = get_json(
        app.clone(),
        &format!("/api/project/{}", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get after update; body={:?}", got);
    assert_eq!(
        got["name"].as_str(),
        Some("After Update"),
        "project name should be updated; body={:?}",
        got
    );
}

// ── Task run (create task from template) ─────────────────────────────────

#[tokio::test]
async fn test_create_task_from_template() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Task Run Project", "max_parallel_tasks": 1, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    // Create a template to run
    let (_, tpl) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({ "name": "run-me", "playbook": "run.yml" }),
        Some(&token),
    )
    .await;
    let tpl_id = tpl["id"].as_i64().expect("template id");

    // Start a task — route is POST /api/project/{id}/tasks
    let (status, task) = post_json_with_token(
        app.clone(),
        &format!("/api/project/{}/tasks", project_id),
        json!({ "template_id": tpl_id }),
        Some(&token),
    )
    .await;
    // 201 = task queued successfully
    assert_eq!(status, StatusCode::CREATED, "create task; body={:?}", task);
    let task_id = task["id"].as_i64().expect("task id");

    // Verify task appears in history
    let (status, list) = get_json(
        app.clone(),
        &format!("/api/project/{}/tasks", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list tasks; body={:?}", list);
    let tasks = list.as_array().expect("tasks array");
    assert!(tasks.iter().any(|t| t["id"].as_i64() == Some(task_id)), "created task must appear in history");
}

// ── Task output endpoint ──────────────────────────────────────────────────

#[tokio::test]
async fn test_get_task_output() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Output Project", "max_parallel_tasks": 1, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (_, tpl) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({ "name": "out-tpl", "playbook": "out.yml" }),
        Some(&token),
    )
    .await;
    let tpl_id = tpl["id"].as_i64().expect("template id");

    let (_, task) = post_json_with_token(
        app.clone(),
        &format!("/api/project/{}/tasks", project_id),
        json!({ "template_id": tpl_id }),
        Some(&token),
    )
    .await;
    let task_id = task["id"].as_i64().expect("task id");

    // Output endpoint should return 200 with an array (may be empty if task not yet run)
    let (status, output) = get_json(
        app.clone(),
        &format!("/api/project/{}/tasks/{}/output", project_id, task_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get task output; body={:?}", output);
    assert!(output.is_array(), "task output should be an array");
}

// ── Delete inventory ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_inventory() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Del Inv Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (_, inv) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/inventories", project_id),
        json!({ "name": "to-delete", "inventory": "static" }),
        Some(&token),
    )
    .await;
    let inv_id = inv["id"].as_i64().expect("inventory id");

    let status = delete_req(
        app.clone(),
        &format!("/api/projects/{}/inventories/{}", project_id, inv_id),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::NO_CONTENT || status == StatusCode::OK,
        "delete inventory returned {:?}",
        status
    );

    // Verify gone
    let (not_found_status, _) = get_json(
        app.clone(),
        &format!("/api/projects/{}/inventories/{}", project_id, inv_id),
        Some(&token),
    )
    .await;
    assert_eq!(not_found_status, StatusCode::NOT_FOUND, "deleted inventory should return 404");
}

// ── Delete template ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_template() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Del Tpl Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (_, tpl) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({ "name": "to-delete", "playbook": "del.yml" }),
        Some(&token),
    )
    .await;
    let tpl_id = tpl["id"].as_i64().expect("template id");

    let status = delete_req(
        app.clone(),
        &format!("/api/projects/{}/templates/{}", project_id, tpl_id),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::NO_CONTENT || status == StatusCode::OK,
        "delete template returned {:?}",
        status
    );
}

// ── Views CRUD ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_and_list_views() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Views Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (status, body) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/views", project_id),
        json!({ "id": 0, "title": "Production", "position": 0, "project_id": project_id }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create view; body={:?}", body);
    let view_id = body["id"].as_i64().expect("view id");

    let (status, list) = get_json(
        app.clone(),
        &format!("/api/projects/{}/views", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list views; body={:?}", list);
    let views = list.as_array().expect("views array");
    assert!(views.iter().any(|v| v["id"].as_i64() == Some(view_id)), "created view must appear in list");
}

// ── Users list (admin) ────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_users_as_admin() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (status, body) = get_json(app.clone(), "/api/users", Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "list users; body={:?}", body);
    let users = body.as_array().expect("users array");
    assert!(!users.is_empty(), "should have at least the seeded admin user");
    assert!(
        users.iter().any(|u| u["username"].as_str() == Some("testadmin")),
        "testadmin should be in user list"
    );
}

// ── E2E: Full resource cycle ───────────────────────────────────────────────

/// Full E2E cycle: project → key → inventory → environment → template → task
/// Verifies that all entities can be created and linked together end-to-end,
/// mimicking the real workflow a user would follow in the UI.
#[tokio::test]
async fn test_e2e_full_resource_cycle() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    // 1. Create project
    let (status, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "E2E Test Project", "max_parallel_tasks": 1, "alert": false }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create project; body={:?}", proj);
    let project_id = proj["id"].as_i64().expect("project id");

    // 2. Create access key (none type — no secrets required)
    let (status, key) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/keys", project_id),
        json!({ "name": "e2e-key", "type": "none" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create key; body={:?}", key);
    let _key_id = key["id"].as_i64().expect("key id");

    // 3. Create inventory (type "static" = INI-based static inventory)
    let (status, inv) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/inventories", project_id),
        json!({ "name": "e2e-inventory", "inventory": "static" }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create inventory; body={:?}", inv);
    let inv_id = inv["id"].as_i64().expect("inventory id");

    // 4. Create environment with variables
    let (status, env) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/environments", project_id),
        json!({
            "project_id": project_id,
            "name": "e2e-env",
            "json": "{\"E2E_VAR\": \"hello\"}"
        }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create environment; body={:?}", env);
    let env_id = env["id"].as_i64().expect("environment id");

    // 5. Create template linking inventory + environment
    let (status, tpl) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({
            "name": "e2e-template",
            "playbook": "echo.sh",
            "inventory_id": inv_id,
            "environment_id": env_id
        }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create template; body={:?}", tpl);
    let tpl_id = tpl["id"].as_i64().expect("template id");

    // 6. Start a task from the template
    let (status, task) = post_json_with_token(
        app.clone(),
        &format!("/api/project/{}/tasks", project_id),
        json!({ "template_id": tpl_id }),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "create task; body={:?}", task);
    let task_id = task["id"].as_i64().expect("task id");

    // 7. Task appears in history
    let (status, tasks_list) = get_json(
        app.clone(),
        &format!("/api/project/{}/tasks", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list tasks; body={:?}", tasks_list);
    assert!(
        tasks_list.as_array().expect("array").iter().any(|t| t["id"].as_i64() == Some(task_id)),
        "task must appear in history"
    );

    // 8. Task output endpoint is accessible
    let (status, output) = get_json(
        app.clone(),
        &format!("/api/project/{}/tasks/{}/output", project_id, task_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "task output; body={:?}", output);
    assert!(output.is_array(), "output should be an array");

    // 9. Get task by ID
    let (status, got_task) = get_json(
        app.clone(),
        &format!("/api/project/{}/tasks/{}", project_id, task_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "get task by id; body={:?}", got_task);
    assert_eq!(got_task["id"].as_i64(), Some(task_id));
    assert_eq!(got_task["template_id"].as_i64(), Some(tpl_id));
}

// ── E2E: Project team management ──────────────────────────────────────────

/// Tests the team management API used by team.html (B-FE-20):
/// list members, add a second user, update role, remove.
#[tokio::test]
async fn test_project_team_management() {
    // Seed two users before building the app
    let temp = tempfile::NamedTempFile::new().expect("temp file");
    let url = sqlite_url_from_path(temp.path());
    let teammate_id;
    {
        use semaphore_ffi::db::store::UserManager;
        use semaphore_ffi::models::User;
        use chrono::Utc;
        let store = SqlStore::new(&url).await.expect("seed store");
        store.create_user(
            User { id: 0, username: "testadmin".into(), name: "Test Admin".into(),
                   email: "testadmin@test.local".into(), password: String::new(),
                   admin: true, external: false, alert: false, pro: false,
                   created: Utc::now(), totp: None, email_otp: None },
            "Test1234!",
        ).await.expect("create admin");
        let teammate = store.create_user(
            User { id: 0, username: "teammate".into(), name: "Team Mate".into(),
                   email: "teammate@test.local".into(), password: String::new(),
                   admin: false, external: false, alert: false, pro: false,
                   created: Utc::now(), totp: None, email_otp: None },
            "Teammate1!",
        ).await.expect("create teammate");
        teammate_id = teammate.id as i64;
    }
    let store2 = SqlStore::new(&url).await.expect("app store");
    let app = semaphore_ffi::api::create_app(std::sync::Arc::new(store2));
    let token = register_and_login(&app).await;

    // Create project
    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Team E2E Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    // List project members
    let (status, members) = get_json(
        app.clone(),
        &format!("/api/project/{}/users", project_id),
        Some(&token),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "list project users; body={:?}", members);

    // Add teammate as task_runner
    let (status, _) = post_json_with_token(
        app.clone(),
        &format!("/api/project/{}/users", project_id),
        json!({ "user_id": teammate_id, "role": "task_runner" }),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::CREATED || status == StatusCode::OK,
        "add team member returned {:?}",
        status
    );

    // Update role to manager
    let (status, _) = put_json(
        app.clone(),
        &format!("/api/project/{}/users/{}", project_id, teammate_id),
        json!({ "role": "manager" }),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::NO_CONTENT,
        "update role returned {:?}",
        status
    );

    // Remove from project
    let status = delete_req(
        app.clone(),
        &format!("/api/project/{}/users/{}", project_id, teammate_id),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::NO_CONTENT || status == StatusCode::OK,
        "remove team member returned {:?}",
        status
    );
    drop(temp); // keep temp file alive until end
}

// ── E2E: Update resources ─────────────────────────────────────────────────

/// Tests PUT (update) endpoints for keys, inventories, environments, repositories.
#[tokio::test]
async fn test_update_resources() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Update Resources Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    // -- Update key --
    let (_, key) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/keys", project_id),
        json!({ "name": "original-key", "type": "none" }),
        Some(&token),
    )
    .await;
    let key_id = key["id"].as_i64().expect("key id");

    let (status, _) = put_json(
        app.clone(),
        &format!("/api/projects/{}/keys/{}", project_id, key_id),
        json!({ "name": "updated-key", "type": "none" }),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::NO_CONTENT,
        "update key returned {:?}",
        status
    );

    // -- Update inventory --
    let (_, inv) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/inventories", project_id),
        json!({ "name": "original-inv", "inventory": "static" }),
        Some(&token),
    )
    .await;
    let inv_id = inv["id"].as_i64().expect("inventory id");

    let (status, _) = put_json(
        app.clone(),
        &format!("/api/projects/{}/inventories/{}", project_id, inv_id),
        json!({ "name": "updated-inv", "inventory": "localhost\n192.168.1.1" }),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::NO_CONTENT,
        "update inventory returned {:?}",
        status
    );

    // -- Update environment --
    let (_, env) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/environments", project_id),
        json!({ "project_id": project_id, "name": "original-env", "json": "{}" }),
        Some(&token),
    )
    .await;
    let env_id = env["id"].as_i64().expect("env id");

    let (status, _) = put_json(
        app.clone(),
        &format!("/api/projects/{}/environments/{}", project_id, env_id),
        json!({ "project_id": project_id, "name": "updated-env", "json": "{\"FOO\":\"bar\"}" }),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::NO_CONTENT,
        "update environment returned {:?}",
        status
    );

    // -- Update template --
    let (_, tpl) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({ "name": "original-tpl", "playbook": "old.yml" }),
        Some(&token),
    )
    .await;
    let tpl_id = tpl["id"].as_i64().expect("template id");

    let (status, _) = put_json(
        app.clone(),
        &format!("/api/projects/{}/templates/{}", project_id, tpl_id),
        json!({ "name": "updated-tpl", "playbook": "new.yml" }),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::OK || status == StatusCode::NO_CONTENT,
        "update template returned {:?}",
        status
    );
}

// ── E2E: WebSocket endpoint reachable ─────────────────────────────────────

/// Verifies the WebSocket endpoint at /api/ws accepts an upgrade handshake.
/// Full message-level testing requires a live server with I/O; here we confirm
/// the route is wired and returns 101 Switching Protocols (not 404/405).
#[tokio::test]
async fn test_websocket_endpoint_accepts_upgrade() {
    let (app, _temp) = seeded_app().await;

    // Send a WebSocket upgrade request to the global ws endpoint.
    // The server should respond with 101 Switching Protocols if the route exists.
    let request = Request::builder()
        .method("GET")
        .uri("/api/ws")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", "dGhlIHNhbXBsZSBub25jZQ==")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.expect("oneshot");
    // 101 = successful upgrade; 400 = bad request (auth missing but route found).
    // Both are acceptable — what matters is the route is not 404/405.
    assert_ne!(
        response.status(),
        StatusCode::NOT_FOUND,
        "WebSocket route /api/ws must exist"
    );
    assert_ne!(
        response.status(),
        StatusCode::METHOD_NOT_ALLOWED,
        "WebSocket route /api/ws must accept GET"
    );
}

// ── Delete schedule ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_schedule() {
    let (app, _temp) = seeded_app().await;
    let token = register_and_login(&app).await;

    let (_, proj) = post_json_with_token(
        app.clone(),
        "/api/projects",
        json!({ "name": "Del Sched Project", "max_parallel_tasks": 0, "alert": false }),
        Some(&token),
    )
    .await;
    let project_id = proj["id"].as_i64().expect("project id");

    let (_, tpl) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/templates", project_id),
        json!({ "name": "del-sched-tpl", "playbook": "del.yml" }),
        Some(&token),
    )
    .await;
    let tpl_id = tpl["id"].as_i64().expect("template id");

    let (_, sched) = post_json_with_token(
        app.clone(),
        &format!("/api/projects/{}/schedules", project_id),
        json!({
            "id": 0,
            "template_id": tpl_id,
            "project_id": project_id,
            "name": "daily",
            "cron": "0 0 * * *",
            "active": false
        }),
        Some(&token),
    )
    .await;
    let sched_id = sched["id"].as_i64().expect("schedule id");

    let status = delete_req(
        app.clone(),
        &format!("/api/projects/{}/schedules/{}", project_id, sched_id),
        Some(&token),
    )
    .await;
    assert!(
        status == StatusCode::NO_CONTENT || status == StatusCode::OK,
        "delete schedule returned {:?}",
        status
    );
}
