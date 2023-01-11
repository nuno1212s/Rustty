use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use crossbeam_channel::Receiver;
use polling::{Event, Poller};
use crate::channel::Channel;

mod event_thread;

/// The event group for a given server
/// The Event Group is responsible for handling the I/O events and
pub struct EventGroup {
    ev_loop_id: usize,
    add_connections: Receiver<Arc<Channel>>,
    currently_connected: BTreeMap<usize, Arc<Channel>>,
    workers: EventGroupWorkers,
    remove_connections: Receiver<usize>,
}

/// The workers for an event group
/// To load balance, we use a simple round robin approach
struct EventGroupWorkers;

impl EventGroup {
    pub fn register_new_connection(&self, channel: Arc<Channel>) {}

    fn begin(mut self) {
        std::thread::Builder::new()
            .name(format!("Event loop thread #{}", self.ev_loop_id))
            .spawn(move || {
                let poller = Poller::new().unwrap();

                let mut events = Vec::new();

                loop {
                    //Event loop
                    events.clear();

                    //Receive the events. Add a 50 ms max time to register new connections
                    //And delete previous connections
                    if poller.wait(&mut events, Some(Duration::from_millis(50))).unwrap() > 0 {
                        for ev in &events {
                            let channel_id = ev.key;

                            if let Some(channel) = self.currently_connected.get(&channel_id) {
                                poller.modify(channel.socket(), Event::all(channel_id))
                                    .unwrap();
                            }
                        }

                        let events_to_deliver: Vec<Event> = events.drain(..).collect();

                        self.workers.deliver_io_work(events_to_deliver);
                    }

                    //Add the newly established connection
                    while let Ok(channel) = self.add_connections.try_recv() {
                        self.currently_connected.insert(channel.id(), channel.clone());

                        poller.modify(channel.socket(), Event::all(channel.id())).unwrap();
                    }

                    // Check the connections that we want to disconnect
                    while let Ok(channel_id) = self.remove_connections.try_recv() {
                        if let Some(channel) = self.currently_connected.remove(&channel_id) {
                            //Delete the channel from our pool
                            poller.delete(channel.socket()).unwrap();
                        }
                    }
                }
            }).expect("Failed to launch event loop thread");
    }
}

impl EventGroupWorkers {
    pub fn deliver_io_work(&self, events: Vec<Event>) {}
}