//! gRPC Module - Модуль gRPC сервисов для внутреннего взаимодействия
//!
//! Этот модуль предоставляет gRPC API для:
//! - Управления задачами (TaskService)
//! - Управления проектами (ProjectService)
//! - Управления раннерами (RunnerService)
//!
//! Примечание: Для полной компиляции требуется protoc.
//! В этой версии предоставлены только базовые структуры.

pub mod server;
pub mod services;

// Типы и сервисы будут определены в services.rs
pub use server::GrpcServer;
pub use services::{TaskServiceImpl, ProjectServiceImpl, RunnerServiceImpl};
