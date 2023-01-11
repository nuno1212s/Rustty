use std::error::Error;
use std::io;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::net::ToSocketAddrs;
use polling::{Event, Poller};
use crate::config::ServerConfig;
use crate::event_group::EventGroup;

struct Server {
    config: ServerConfig,
    event_group: EventGroup
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

        //TODO: Apply any custom options to tcp listener

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