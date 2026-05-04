#[cfg(not(feature = "verus"))]
extern crate std;

use alloc::string::String;
use alloc::vec::Vec;

#[cfg(not(feature = "verus"))]
use shadowsocks::{
    config::{ServerAddr, ServerConfig},
    relay::socks5::Address,
};
#[cfg(not(feature = "verus"))]
use shadowsocks_service::{
    config::{Config, ConfigType, LocalConfig, ServerInstanceConfig},
    local::context::ServiceContext,
    local::loadbalancing::{PingBalancerBuilder, ServerIdent},
    local::socks::server::SocksBuilder,
    server::ServerBuilder,
};
#[cfg(not(feature = "verus"))]
use std::sync::Arc;
#[cfg(not(feature = "verus"))]
use tokio::runtime::Runtime;

pub struct VpnConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub password: String,
}

impl VpnConfig {
    pub fn new(server_addr: String, server_port: u16, password: String) -> Self {
        Self {
            server_addr,
            server_port,
            password,
        }
    }

    #[cfg(not(feature = "verus"))]
    pub fn to_server_config(&self) -> ServerConfig {
        ServerConfig::new(
            ServerAddr::DomainName(self.server_addr.clone(), self.server_port),
            self.password.clone(),
            shadowsocks::crypto::CipherKind::AES_256_GCM,
        )
        .unwrap()
    }
}

pub struct ShadowsocksClient {
    pub config: VpnConfig,
}

impl ShadowsocksClient {
    pub fn new(config: VpnConfig) -> Self {
        Self { config }
    }

    #[cfg(not(feature = "verus"))]
    pub async fn run(&self) {
        let server_config = self.config.to_server_config();

        let mut context = ServiceContext::new();
        let context = Arc::new(context);

        let mut instance_config = ServerInstanceConfig::with_server_config(server_config);

        let mut balancer_builder =
            PingBalancerBuilder::new(context.clone(), shadowsocks::config::Mode::TcpOnly);
        balancer_builder.add_server(instance_config);
        let balancer = balancer_builder.build().await.unwrap();

        let mut builder = SocksBuilder::new(
            shadowsocks::config::ServerAddr::SocketAddr("127.0.0.1:1080".parse().unwrap()),
            balancer,
        );
        let server = builder.build().await.unwrap();
        server.run().await.unwrap();
    }
}

pub struct ShadowsocksServer {
    pub config: VpnConfig,
}

impl ShadowsocksServer {
    pub fn new(config: VpnConfig) -> Self {
        Self { config }
    }

    #[cfg(not(feature = "verus"))]
    pub async fn run(&self) {
        let server_config = self.config.to_server_config();

        let mut builder = ServerBuilder::new(server_config);
        let server = builder.build().await.unwrap();
        server.run().await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vpn_config_creation() {
        let config = VpnConfig::new("127.0.0.1".into(), 8388, "password".into());
        assert_eq!(config.server_addr, "127.0.0.1");
        assert_eq!(config.server_port, 8388);
    }

    #[tokio::test]
    async fn test_shadowsocks_client() {
        let config = VpnConfig::new("127.0.0.1".into(), 8388, "password".into());
        let client = ShadowsocksClient::new(config);
        assert_eq!(client.config.server_port, 8388);

        // Spawn the client and then abort it
        let handle = tokio::spawn(async move {
            client.run().await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        handle.abort();
    }

    #[tokio::test]
    async fn test_shadowsocks_server() {
        let config = VpnConfig::new("127.0.0.1".into(), 8389, "password".into());
        let server = ShadowsocksServer::new(config);
        assert_eq!(server.config.server_port, 8389);

        // Spawn the server and then abort it
        let handle = tokio::spawn(async move {
            server.run().await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        handle.abort();
    }
}
