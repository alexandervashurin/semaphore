//! gRPC Server - Сервер gRPC для внутреннего взаимодействия
//!
//! Примечание: Полная реализация требует protoc для генерации кода.

use std::net::SocketAddr;
use tracing::info;
use crate::error::Result;
use crate::grpc::services::GrpcServerConfig;

/// gRPC сервер Velum
pub struct GrpcServer {
    config: GrpcServerConfig,
}

impl GrpcServer {
    pub fn new(config: GrpcServerConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(GrpcServerConfig::default())
    }

    pub fn with_address(address: SocketAddr) -> Self {
        Self::new(GrpcServerConfig {
            address,
            ..Default::default()
        })
    }

    /// Запускает gRPC сервер
    pub async fn serve(self) -> Result<()> {
        info!("gRPC server stub running on {}", self.config.address);
        info!("Full implementation requires protoc");
        
        // Заглушка - просто ждём
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    }

    pub fn address(&self) -> SocketAddr {
        self.config.address
    }
}
