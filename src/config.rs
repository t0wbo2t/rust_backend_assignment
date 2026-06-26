use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub address: SocketAddr,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3000),
        }
    }
}
