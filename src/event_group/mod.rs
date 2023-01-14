use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Receiver, Sender};
use polling::{Event, Poller};
use crate::channel::Channel;

mod event_thread;

pub struct EventGroupHandle {
    tx: Sender<EventGroupMessage>
}

/// Messages to communicate with the event group
pub enum EventGroupMessage {
    AddConnection(Arc<Channel>),
    RemoveConnection(usize)
}

struct EventGroupCommon {
    currently_connected: Mutex<BTreeMap<usize, Arc<Channel>>>,
    poller: Arc<Poller>
}

/// The event group for a given server
/// The Event Group is responsible for handling the I/O events and
pub struct EventGroup {
    ev_loop_id: usize,
    event_messages: Receiver<EventGroupMessage>,
    currently_connected: BTreeMap<usize, Arc<Channel>>,
    workers: EventGroupWorkers,
    poller: Poller
}

/// The workers for an event group
/// To load balance, we use a simple round robin approach
struct EventGroupWorkers;

const EVENT_LIMIT: usize = 1024;

impl EventGroup {

    pub fn initialize_event_group(event_loop_id: usize, thread_count: usize) -> EventGroupHandle {
        let (comm_tx, comm_rx) = crossbeam_channel::bounded(1024);

        let poller = Poller::new().unwrap();

        let ev_group = EventGroup {
            ev_loop_id: event_loop_id,
            event_messages: comm_rx,
            currently_connected: Default::default(),
            workers: EventGroupWorkers {},
            poller
        };

        ev_group.begin();

        EventGroupHandle {
            tx: comm_tx
        }
    }

    fn begin(mut self) {
        std::thread::Builder::new()
            .name(format!("Event loop thread #{}", self.ev_loop_id))
            .spawn(move || {
                let mut events = Vec::with_capacity(EVENT_LIMIT);

                loop {
                    //Event loop

                    //Clear any previous events that were not cleared for some reason
                    events.clear();

                    //Receive the events. Add a 5 ms max time to register new connections
                    //And delete previous connections
                    if self.poller.wait(&mut events, Some(Duration::from_millis(5))).unwrap() > 0 {
                        for ev in &events {
                            let channel_id = ev.key;

                            if let Some(channel) = self.currently_connected.get(&channel_id) {
                                self.poller.modify(channel.network().raw_fd(), Event::readable(channel_id))
                                    .unwrap();
                            }
                        }

                        let events_to_deliver: Vec<Event> = events.drain(..).collect();

                        self.workers.deliver_io_work(events_to_deliver);
                    }

                    //Listen to any messages intended for the event group, such as new connections
                    //Or connection close attempts
                    while let Ok(channel) = self.event_messages.try_recv() {
                        match channel {
                            EventGroupMessage::AddConnection(channel) => {
                                self.currently_connected.insert(channel.id(), channel.clone());

                                self.poller.modify(channel.network().raw_fd(), Event::readable(channel.id())).unwrap();
                            }
                            EventGroupMessage::RemoveConnection(channel_id) => {
                                if let Some(channel) = self.currently_connected.remove(&channel_id) {
                                    //Delete the channel from our pool
                                    self.poller.delete(channel.network().raw_fd()).unwrap();
                                }
                            }
                        }

                    }
                }
            }).expect("Failed to launch event loop thread");
    }
}

impl EventGroupWorkers {
    fn deliver_io_work(&self, events: Vec<Event>) {

        //TODO: Split up the work according to the amount of threads we have available
        
    }
}

impl EventGroupHandle {

    pub fn register_new_connection(&self, channel: Arc<Channel>) {
        self.tx.send(EventGroupMessage::AddConnection(channel)).unwrap();
    }

}