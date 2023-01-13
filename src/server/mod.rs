mod tcp_server;

use std::io;
use std::net::{TcpListener};
use polling::{Event, Poller};
use crate::channel::{Channel, ChannelNetwork};
use crate::config::ServerConfig;
use crate::event_group::{EventGroup, EventLoopHandle};

struct Server {
    config: ServerConfig,
    event_group: EventLoopHandle
}

impl Server {

    fn begin(self) -> io::Result<()> {
        let bind_result = TcpListener::bind((self.config.bind_addr(), self.config.port()));

        let tcp_listener = match bind_result {
            Ok(tcp_listener) => {

                tcp_listener

            }
            Err(err) => {
                return Err(err);
            }
        };

        tcp_listener.set_nonblocking(true).unwrap();

        std::thread::Builder::new().name(format!("Server {:?}", self.config.bind_addr()))
            .spawn(move || {

                let key = 6;

                let poller = Poller::new().unwrap();

                poller.add(&tcp_listener, Event::readable(key)).unwrap();

                let mut events = Vec::new();
                loop {

                    events.clear();
                    poller.wait(&mut events, None).unwrap();

                    for event in &events {
                        if event.key == key {
                            let tcp_conn = tcp_listener.accept();

                            match tcp_conn {
                                Ok((conn, addr)) => {

                                    conn.set_nonblocking(true).expect("Failed to make connection non blocking");

                                    let channel = Channel::new(1,
                                    ChannelNetwork::new(addr, Box::new(conn)));

                                    self.event_group.register_new_connection(channel)
                                }
                                Err(err) => {

                                }
                            }
                        }
                    }


                    poller.modify(&tcp_listener, Event::readable(key)).unwrap();
                }

            }).expect("Failed to launch server monitor thread thread");

        Ok(())
    }

}

pub trait ServerHandler {

}