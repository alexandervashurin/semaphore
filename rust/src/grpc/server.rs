//! gRPC Server - Сервер gRPC для внутреннего взаимодействия
//!
//! Примечание: Полная реализация требует protoc для генерации кода.

use crate::error::Result;
use crate::grpc::services::GrpcServerConfig;
use std::net::SocketAddr;
use tracing::info;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_server_new() {
        let config = GrpcServerConfig::default();
        let server = GrpcServer::new(config);
        assert_eq!(
            server.config.address,
            "0.0.0.0:50051".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn test_grpc_server_with_defaults() {
        let server = GrpcServer::with_defaults();
        assert_eq!(
            server.address(),
            "0.0.0.0:50051".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn test_grpc_server_with_custom_address() {
        let addr: SocketAddr = "127.0.0.1:9090".parse().unwrap();
        let server = GrpcServer::with_address(addr);
        assert_eq!(server.address(), addr);
    }

    #[test]
    fn test_grpc_server_config_default() {
        let config = GrpcServerConfig::default();
        assert_eq!(
            config.address,
            "0.0.0.0:50051".parse::<SocketAddr>().unwrap()
        );
        assert!(config.enable_reflection);
        assert_eq!(config.max_message_size, 4 * 1024 * 1024);
    }

    #[test]
    fn test_grpc_server_config_custom() {
        let addr: SocketAddr = "10.0.0.1:8080".parse().unwrap();
        let config = GrpcServerConfig {
            address: addr,
            enable_reflection: false,
            max_message_size: 1024,
        };
        assert_eq!(config.address, addr);
        assert!(!config.enable_reflection);
        assert_eq!(config.max_message_size, 1024);
    }

    #[test]
    fn test_grpc_server_config_clone() {
        let config = GrpcServerConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.address, config.address);
        assert_eq!(cloned.max_message_size, config.max_message_size);
    }

    #[test]
    fn test_grpc_server_config_debug() {
        let config = GrpcServerConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("GrpcServerConfig"));
    }

    #[test]
    fn test_grpc_server_config_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<GrpcServerConfig>();
    }

    #[test]
    fn test_grpc_server_address_method() {
        let addr: SocketAddr = "[::1]:5555".parse().unwrap();
        let server = GrpcServer::with_address(addr);
        assert_eq!(server.address(), addr);
    }
}
