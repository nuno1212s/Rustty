use std::sync::Arc;
use crossbeam_channel::Receiver;
use polling::Event;

use crate::channel::Channel;
use crate::event_group::{EventGroupCommon, EventGroupHandle};

type Work = IOWork;

struct IOWork {
    channel: Arc<Channel>,
    event: Event
}

pub struct EventGroupWorker {
    
    ev_group_worker_id: usize,
    ev_group_info: EventGroupHandle,

    work_receiver: Receiver<Work>,
}

impl EventGroupWorker {
    
    pub fn begin(self) {
        loop {
            match self.work_receiver.recv() {
                Ok(_) => {}
                Err(err) => {
                    
                    break
                }
            }
            
        }
    }
    
}