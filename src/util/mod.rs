use std::error::Error;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use crate::channel::Channel;

/// Trait object responsible for handling reported I/O events.
/// The stream provided here must be in Non Blocking mode for this
/// Library to work properly
/// As such, when the operation is not ready to be completed an Error with
/// ErrorKind::WouldBlock should be returned
pub trait Stream: Read + Write + AsRawFd + Send {}


pub trait NetworkServer<T>: AsRawFd where T: Stream {

    fn accept(&mut self) -> (T, SocketAddr);

}

/// A handler for the channels
pub trait ChannelHandler: Sync + Send {

    /// Handle a new connection having just been established
    /// This allows the user to alter the channel in any way he likes, as
    /// he must also return a channel that is owned.
    fn handle_connection_established(&self, channel: Channel) -> Channel;

    /// Handle a new message being received
    fn handle_message_received(&self, channel: Arc<Channel>, buf: Vec<u8>);

    /// Handle a connection being removed, either because of errors in the connection
    /// Or because of a request to remove it
    fn handle_connection_removed(&self, channel: Arc<Channel>, err: Option<impl Error>);
}