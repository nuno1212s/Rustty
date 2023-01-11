use std::net::IpAddr;

pub struct ServerConfig {
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

}