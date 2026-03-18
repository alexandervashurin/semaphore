//! Plugin System - Модуль плагинов
//!
//! Система плагинов позволяет расширять функциональность Velum UI
//! без изменения основного кода приложения.
//!
//! Поддерживаемые типы плагинов:
//! - Task Executors - кастомные исполнители задач
//! - Notification Providers - провайдеры уведомлений
//! - Storage Providers - провайдеры хранилищ
//! - Auth Providers - провайдеры аутентификации
//! - API Extensions - расширения API
//! - Hook Plugins - хуки для событий
//! - WASM Plugins - динамические WASM плагины

pub mod base;
pub mod hooks;
pub mod wasm_loader;
pub mod wasm_runtime;

pub use base::*;
pub use hooks::*;
pub use wasm_loader::*;
pub use wasm_runtime::*;
