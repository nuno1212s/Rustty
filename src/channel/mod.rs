use std::net::{SocketAddr, TcpStream};

pub struct Channel {
    id: usize,
    network: ChannelNetwork
    //TODO: Maybe add a pipeline to this?
}

pub struct ChannelNetwork {
    addr: SocketAddr,
    socket: TcpStream,
}

impl Channel {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn network(&self) -> &ChannelNetwork {
        &self.network
    }
}

impl ChannelNetwork {

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn socket(&self) -> &TcpStream {
        &self.socket
    }

}

pub trait ChannelHandler {

    fn handle_connection_established() {}

    fn handle_received_information() {}

    fn handle_connection_lost() {}

}