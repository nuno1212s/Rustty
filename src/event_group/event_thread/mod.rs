use std::io::{ErrorKind, Read, Write};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use crossbeam_channel::Receiver;
use polling::Event;

use crate::channel::Channel;
use crate::event_group::{EventGroupCommon, EventGroupHandle};

type Work = IOWork;

const READ_BUFFER_SIZE: usize = 1024;

struct IOWork {
    channel: Arc<Channel>,
    event: Event,
}

pub struct EventGroupWorker {
    ev_group_worker_id: usize,
    ev_group_info: EventGroupHandle,

    work_receiver: Receiver<Work>,
}

impl EventGroupWorker {
    pub fn begin(self) {
        loop {
            let (channel, event) = match self.work_receiver.recv() {
                Ok(Work { channel, event }) => {
                    (channel, event)
                }
                Err(err) => {
                    break;
                }
            };

            self.handle_event(event, &channel);
        }
    }

    /// Handle an event, received from the epoll layer
    fn handle_event(&self, ev: Event, channel: &Arc<Channel>) {
        if ev.readable {
            self.handle_ev_readable(channel);
        }

        if ev.writable {
            self.handle_ev_writable(channel);
        }
    }

    /// handle a readable event
    fn handle_ev_readable(&self, channel: &Arc<Channel>) {
        let read = {
            let mut final_buffer = Vec::with_capacity(READ_BUFFER_SIZE);

            let mut read_buffer = Vec::with_capacity(READ_BUFFER_SIZE);

            let mut socket = channel.network().socket().lock().unwrap();

            loop {
                let result = socket.read(&mut read_buffer[..]);

                match result {
                    Ok(read_bytes) => {
                        if read_bytes > 0 {
                            final_buffer.append(&mut read_buffer);
                        } else {
                            //We did not read anything, so there must be nothing to read
                            break;
                        }
                    }
                    Err(err) => {
                        match err.kind() {
                            ErrorKind::WouldBlock => {
                                //We are done reading until the next IO event
                                break;
                            }
                            _ => {
                                //TODO: This means the socket has failed and as such must be disconnected
                            }
                        }
                    }
                }
            }

            final_buffer
        };

        //TODO: deliver the read message to the client
    }

    /// Handle a writable event
    fn handle_ev_writable(&self, channel: &Arc<Channel>) {
        //If we are receiving this event, this means that we have attempt to perform a send
        //That would block, and as such we had to wait for the epoll event
        let vec = channel.network().take_pending_tx();

        if let Some(pending_tx) = vec {
            //Acquire the lock to the socket, so that we write this in the correct order

            let mut pending_tx_slice = &pending_tx[..];
            let mut bytes_to_write = pending_tx_slice.len();

            let mut error = None;

            let remaining_bytes = {
                let mut socket = channel.network().socket().lock().unwrap();

                loop {
                    //Attempt to send all of the bytes in our pending_tx buffer.
                    //We always keep the lock since this write method does not block
                    //And as such this should be quite quick.
                    match socket.write(pending_tx_slice) {
                        Ok(bytes_written) => {
                            if bytes_written > 0 && bytes_written == bytes_to_write {
                                break None;
                            } else if bytes_written > 0 {
                                //Take the bytes that were not yet written
                                pending_tx_slice = &pending_tx_slice[bytes_written..];

                                bytes_to_write -= bytes_written;
                            } else {
                                //What?
                                break Some(pending_tx_slice);
                            }
                        }
                        Err(err) => {
                            match err.kind() {
                                ErrorKind::WouldBlock => {
                                    break Some(pending_tx_slice);
                                }
                                _ => {
                                    error = Some(err);

                                    break Some(pending_tx_slice);
                                }
                            }
                        }
                    }
                }
                //Return the remaining bytes that were not sent so they can be added to the
                //pending_tx buffer again. They must be added at the beginning so that
                //We maintain request ordering of the applications
            };

            if let Some(err) = error {
                //TODO: Disconnect connection
                self.ev_group_info.close_connection(channel, Some(err))
            } else {
                if let Some(remaining_bytes) = remaining_bytes {
                    channel.network().begin_extend_from_slice(remaining_bytes);
                    //Register that we still have some more things to write to the socket
                    self.ev_group_info.register_write_intention(channel);
                }
            }
        }
    }
}