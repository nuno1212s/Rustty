use std::net::{SocketAddr, TcpStream};
use crossbeam_channel::Receiver;
use polling::{Event, Poller};
use crate::channel::Channel;

pub type WorkUnit = Work;

pub struct Work {
    event: Event,
    channel: Channel
}

/// The event loop threads. These threads are responsible
pub struct EventLoop {
    work_received: Receiver<Work>
}

impl EventLoop {

    fn begin(self) {
        loop {

            if let Ok(work) = self.work_received.recv() {

                let Work {
                    event,
                    channel
                } = work;

                if event.readable {

                }

                if event.writable {

                }
            }
        }
    }

}