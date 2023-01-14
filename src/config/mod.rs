use std::net::IpAddr;

/// The base configuration, common to both servers and clients
pub struct BaseConfig {
    event_loop_thread_count: usize
}

/// Communication that is related to the server, in conjunction with the base configurations
pub struct ServerConfig {
    base_config: BaseConfig,
    bind_addr: IpAddr,
    port: u16,
}

impl ServerConfig {

    pub fn bind_addr(&self) -> IpAddr {
        self.bind_addr
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn base_config(&self) -> &BaseConfig {
        &self.base_config
    }
}

impl BaseConfig {
    pub fn event_loop_thread_count(&self) -> usize {
        self.event_loop_thread_count
    }
}