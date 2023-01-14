use std::net::{SocketAddr};
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};
use crate::util::Stream;

pub struct Channel {
    id: usize,
    network: ChannelNetwork,
    //TODO: Maybe add a pipeline to this?
    pipeline: Vec<fn(Vec<u8>) -> Vec<u8>>
}

pub struct ChannelNetwork {
    //The raw file descriptor, cached for easy access
    //Without having to enter the critical zone
    raw_fd: RawFd,
    // The socket addr for this socket
    addr: SocketAddr,
    // A stream protected by this lock.
    // We use a dyn object so any implementation that meets our
    // Needs can be used.
    socket: Mutex<Box<dyn Stream>>,
}

impl Channel {

    pub fn new(id: usize, network: ChannelNetwork) -> Arc<Self> {
        Arc::new(Channel {
            id,
            network,
            pipeline: vec![],
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn network(&self) -> &ChannelNetwork {
        &self.network
    }

}

impl ChannelNetwork {

    pub fn new(addr: SocketAddr, socket: Box<dyn Stream>) -> Self {
        ChannelNetwork {
            raw_fd: socket.as_raw_fd(),
            addr,
            socket: Mutex::new(socket)
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn socket(&self) -> &Mutex<Box<dyn Stream>> {
        &self.socket
    }

    pub fn raw_fd(&self) -> RawFd {
        self.raw_fd
    }
}

pub trait ChannelHandler {
    fn handle_connection_established() {}

    fn handle_received_information() {}

    fn handle_connection_lost() {}
}