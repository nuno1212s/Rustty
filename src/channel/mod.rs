use std::io::{ErrorKind, Write};
use std::net::{SocketAddr};
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::event_group::EventGroupHandle;
use crate::util::Stream;

pub struct Channel {
    id: usize,
    network: ChannelNetwork,
    owning_event_group: EventGroupHandle,
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

    //TODO: Flag that allows to see if we have anything to write

    // Do we have any pending information we want to send
    has_pending_tx: AtomicBool,
    //The pending transmission bytes that were not sent
    //as it could not be done in a non blocking way
    pending_tx: Mutex<Vec<u8>>
}

/// Non-blocking
impl Write for Channel {
    
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        return match self.network.socket.lock().unwrap().write(buf) {
            Ok(written) => {
                Ok(written)
            }
            Err(err) => {
                match err.kind() {
                    ErrorKind::WouldBlock =>  {
                        
                        let previous = {
                            let mut lock_guard = self.network.pending_tx.lock().unwrap();

                            let previous = self.network.has_pending_tx.swap(true, Ordering::SeqCst);

                            //Could this be done without performing this copy?
                            //It could if we took ownership of the buffer but that would mean we
                            //No longer implement Write, which isn't ideal
                            lock_guard.extend_from_slice(buf);

                            previous
                        };

                        if !previous {
                            //If we have already registered that we have the intention to write, then
                            //We don't want to do it again
                            self.owning_event_group.register_write_intention(self);
                        }

                        //Do we like returning Ok() here?
                        Ok(buf.len())
                    }
                    _ => {
                        Err(err)
                    }
                }

            }
        };
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl Channel {

    pub(crate) fn new(id: usize, network: ChannelNetwork, owning_event_group: EventGroupHandle) -> Arc<Self> {
        Arc::new(Channel {
            id,
            network,
            owning_event_group,
            pipeline: vec![],
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn network(&self) -> &ChannelNetwork {
        &self.network
    }

    /// Close this channel.
    /// This removes the channel from its event group, meaning it will no longer be able
    /// to receive messages.
    /// The underlying socket is also closed
    pub fn close(self: &Arc<Channel>) {
        self.owning_event_group.close_connection(self)
    }
}

impl ChannelNetwork {

    pub fn new(addr: SocketAddr, socket: Box<dyn Stream>, pending_tx_size: usize) -> Self {
        ChannelNetwork {
            raw_fd: socket.as_raw_fd(),
            addr,
            socket: Mutex::new(socket),
            has_pending_tx: AtomicBool::new(false),
            pending_tx: Mutex::new(Vec::with_capacity(pending_tx_size)),
        }
    }


    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub(crate) fn socket(&self) -> &Mutex<Box<dyn Stream>> {
        &self.socket
    }

    pub(crate) fn raw_fd(&self) -> RawFd {
        self.raw_fd
    }

    ///We want to extends the mutex with the remaining bytes from the previous write
    /// They must be added to the beginning of the vector so we maintain the original
    /// ordering of the application.
    pub(crate) fn begin_extend_from_slice(&self, slice: &[u8]) {
        let mut lock_guard = self.pending_tx.lock().unwrap();

        self.has_pending_tx.store(true, Ordering::SeqCst);

        if lock_guard.is_empty() {
            lock_guard.extend_from_slice(slice);
        } else {

            //This is so the elements that were left from previous send attempts are put at the start of the
            //Vector, instead of at the end, so they are the first to be sent through the Stream (or else
            // We could fracture various packets, etc) This is important for maintaining sequential sending

            //Maybe optimize this?
            //It can actually
            let mut final_vec = Vec::with_capacity(lock_guard.len() + slice.len());

            final_vec.extend_from_slice(slice);
            final_vec.extend_from_slice(&lock_guard[..]);

            let _prev = std::mem::replace(&mut *lock_guard, final_vec);
        }
    }
    
    pub(crate) fn take_pending_tx(&self) -> Option<Vec<u8>> {
        if self.has_pending_tx.load(Ordering::Relaxed) {
            let new_vec = Vec::with_capacity(1024);

            let mut lock_guard = self.pending_tx.lock().unwrap();

            //This must be done after the pending_tx lock so that no one can try to get in between us setting the
            //Atomic variable to false and taking the vector, as that can cause issues
            if let Ok(_) = self.has_pending_tx.compare_exchange(true, false,
                                                    Ordering::SeqCst, Ordering::SeqCst) {
                Some(std::mem::replace(&mut lock_guard, new_vec))
            } else {
                None
            }
        } else {
            None
        }

    }


    pub fn has_pending_tx(&self) -> &AtomicBool {
        &self.has_pending_tx
    }
}
