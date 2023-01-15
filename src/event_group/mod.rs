use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossbeam_channel::{Receiver, Sender};
use log::error;
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
                    let duration = Duration::from_millis(5);

                    let poll_result = self.poller.wait(&mut events,
                                                       Some(duration));

                    let collected_events = match poll_result {

                        Ok(collected) => {collected}
                        Err(err) => {
                            error!("Failed to get epoll results because {:?}", err);

                            //Maybe break here? Not sure on how the API works still
                            continue
                        }
                    };

                    if collected_events > 0 {

                        for ev in &events {
                            let channel_id = ev.key;

                            if !ev.writable && !ev.readable {
                                //This means the connection must have suffered some sort of issue.
                                self.remove_connection(channel_id);

                                continue
                            }

                        }

                        self.workers.deliver_io_work(events);
                        //Restate the variable, since we just disposed of the previous value
                        events = Vec::with_capacity(EVENT_LIMIT);
                    }

                    //Listen to any messages intended for the event group, such as new connections
                    //Or connection close attempts
                    while let Ok(channel) = self.event_messages.try_recv() {
                        match channel {
                            EventGroupMessage::AddConnection(channel) => {
                                self.add_connection(channel);
                            }
                            EventGroupMessage::RemoveConnection(channel_id) => {
                                self.remove_connection(channel_id);
                            }
                        }

                    }
                }
            }).expect("Failed to launch event loop thread");
    }


    fn add_connection(&mut self, channel: Arc<Channel>) {
        self.currently_connected.insert(channel.id(), channel.clone());

        self.poller.modify(channel.network().raw_fd(), Event::readable(channel.id())).unwrap();
    }

    fn remove_connection(&mut self, channel_id: usize) {
        if let Some(channel) = self.currently_connected.remove(&channel_id) {
            //Delete the channel from our pool
            self.poller.delete(channel.network().raw_fd()).unwrap();
        }
    }
}

impl EventGroupWorkers {
    fn deliver_io_work(&self, events: Vec<Event>) {

        //TODO: Split up the work according to the amount of threads we have available
        
    }
}

impl EventGroupHandle {

    pub(crate) fn register_new_connection(&self, channel: Arc<Channel>) {
        self.tx.send(EventGroupMessage::AddConnection(channel)).unwrap();
    }

    pub(crate) fn register_write_intention(&self, channel: &Channel) {

    }

    pub(crate) fn close_connection(&self, channel: &Arc<Channel>, err: Option<impl Error>) {

        self.tx.send(EventGroupMessage::RemoveConnection(channel.id())).unwrap();

        //TODO: Call handler method to notify of disconnection

    }

}