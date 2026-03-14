//! Semaphore Desktop - Tauri Application
//! 
//! Desktop приложение для Semaphore UI с поддержкой:
//! - Системного трея
//! - Уведомлений
//! - Локального сервера
//! - Быстрого доступа к задачам

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tokio::sync::Mutex;
use std::sync::Arc;

// ============================================================================
// Application State
// ============================================================================

/// Состояние приложения
#[derive(Debug, Clone)]
pub struct AppState {
    pub server_url: String,
    pub api_token: Option<String>,
    pub connected: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:3000".to_string(),
            api_token: None,
            connected: false,
        }
    }
}

// ============================================================================
// API Types
// ============================================================================

/// Задача Semaphore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    pub project_id: i64,
    pub template_id: i64,
    pub status: String,
    pub created_at: String,
    pub output: Option<String>,
}

/// Проект Semaphore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
}

/// Уведомление
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopNotification {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Получить состояние подключения
#[tauri::command]
async fn get_connection_state(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<ConnectionStateDto, String> {
    let state = state.lock().await;
    Ok(ConnectionStateDto {
        server_url: state.server_url.clone(),
        connected: state.connected,
        has_token: state.api_token.is_some(),
    })
}

/// Подключиться к серверу Semaphore
#[tauri::command]
async fn connect_to_server(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    server_url: String,
    api_token: String,
) -> Result<bool, String> {
    let mut state = state.lock().await;
    
    // Проверяем подключение
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/user", server_url))
        .header("Authorization", format!("Bearer {}", api_token))
        .send()
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;
    
    if response.status().is_success() {
        state.server_url = server_url;
        state.api_token = Some(api_token);
        state.connected = true;
        Ok(true)
    } else {
        Err("Invalid credentials or server URL".to_string())
    }
}

/// Отключиться от сервера
#[tauri::command]
async fn disconnect_from_server(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.connected = false;
    state.api_token = None;
    Ok(())
}

/// Получить список проектов
#[tauri::command]
async fn get_projects(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<Project>, String> {
    let state = state.lock().await;
    
    if !state.connected {
        return Err("Not connected to server".to_string());
    }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/projects", state.server_url))
        .header("Authorization", format!("Bearer {}", state.api_token.as_ref().unwrap()))
        .send()
        .await
        .map_err(|e| format!("Failed to get projects: {}", e))?;
    
    let projects: Vec<Project> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse projects: {}", e))?;
    
    Ok(projects)
}

/// Получить последние задачи
#[tauri::command]
async fn get_recent_tasks(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    limit: i32,
) -> Result<Vec<Task>, String> {
    let state = state.lock().await;
    
    if !state.connected {
        return Err("Not connected to server".to_string());
    }
    
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/tasks?limit={}", state.server_url, limit))
        .header("Authorization", format!("Bearer {}", state.api_token.as_ref().unwrap()))
        .send()
        .await
        .map_err(|e| format!("Failed to get tasks: {}", e))?;
    
    let tasks: Vec<Task> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse tasks: {}", e))?;
    
    Ok(tasks)
}

/// Запустить задачу
#[tauri::command]
async fn run_task(
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
    template_id: i64,
    project_id: i64,
) -> Result<i64, String> {
    let state = state.lock().await;
    
    if !state.connected {
        return Err("Not connected to server".to_string());
    }
    
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/tasks", state.server_url))
        .header("Authorization", format!("Bearer {}", state.api_token.as_ref().unwrap()))
        .json(&serde_json::json!({
            "template_id": template_id,
            "project_id": project_id
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to run task: {}", e))?;
    
    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let task_id = result["id"].as_i64().ok_or("Invalid response")?;
    
    Ok(task_id)
}

/// Отправить уведомление
#[tauri::command]
async fn send_notification(
    app: tauri::AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    
    app.notification()
        .builder()
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| format!("Failed to send notification: {}", e))?;
    
    Ok(())
}

/// Открыть внешнюю ссылку
#[tauri::command]
async fn open_external_link(
    app: tauri::AppHandle,
    url: String,
) -> Result<(), String> {
    use tauri_plugin_shell::ShellExt;
    
    app.shell()
        .open(&url, None)
        .map_err(|e| format!("Failed to open link: {}", e))?;
    
    Ok(())
}

// ============================================================================
// DTO Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStateDto {
    pub server_url: String,
    pub connected: bool,
    pub has_token: bool,
}

// ============================================================================
// Main Application
// ============================================================================

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .manage(Arc::new(Mutex::new(AppState::default())))
        .setup(|app| {
            // Создаём системный трей
            let show_item = MenuItem::with_id(app, "show", "Показать", true, None::<&str>)?;
            let check_updates_item = MenuItem::with_id(app, "check_updates", "Проверить обновления", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            
            let menu = Menu::with_items(app, &[&show_item, &check_updates_item, &quit_item])?;
            
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "check_updates" => {
                        // Проверка обновлений
                        println!("Checking for updates...");
                    }
                    "quit" => {
                        std::process::exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: tauri::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_connection_state,
            connect_to_server,
            disconnect_from_server,
            get_projects,
            get_recent_tasks,
            run_task,
            send_notification,
            open_external_link
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
