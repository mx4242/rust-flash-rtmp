pub mod reader;
pub mod writer;

use flash_lso::types::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::net_connection::NetConnection;
use crate::transport::Transport;

#[derive(Clone, Debug)]
pub enum SharedObjectEvent {
    Use,
    Release,
    RequestChange { key: String, value: Value },
    Change { key: String, value: Value },
    Success,
    SendMessage,
    Status { code: String, level: String },
    Clear,
    Remove { key: String },
    RequestRemove { key: String },
    UseSuccess,
}

impl SharedObjectEvent {
    pub fn get_type(&self) -> u8 {
        match self {
            SharedObjectEvent::Use => 1,
            SharedObjectEvent::Release => 2,
            SharedObjectEvent::RequestChange { .. } => 3,
            SharedObjectEvent::Change { .. } => 4,
            SharedObjectEvent::Success => 5,
            SharedObjectEvent::SendMessage => 6,
            SharedObjectEvent::Status { .. } => 7,
            SharedObjectEvent::Clear => 8,
            SharedObjectEvent::Remove { .. } => 9,
            SharedObjectEvent::RequestRemove { .. } => 10,
            SharedObjectEvent::UseSuccess => 11,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SharedObjectFlushState {
    FLUSHED,
    PENDING,
}

#[derive(Clone, Debug)]
pub struct SharedObject {
    pub name: String,
    pub persistant: bool,
    pub version: u32,
    pub data: Arc<Mutex<HashMap<String, Value>>>,

    pub events: Vec<SharedObjectEvent>, // pub on_sync: Option<Box<dyn Fn(&SharedObject, &str, &Value)>>,
    pub flush_state: SharedObjectFlushState,
    pub use_success: bool,
}

impl SharedObject {
    pub fn new(name: String, persistant: bool) -> Self {
        SharedObject {
            name,
            persistant,
            version: 0,
            data: Arc::new(Mutex::new(HashMap::new())),
            events: Vec::new(),
            flush_state: SharedObjectFlushState::FLUSHED,
            use_success: false,
        }
    }

    pub fn new_shared_object(name: String, persistant: bool) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(name, persistant)))
    }

    pub(crate) fn clear_events(&mut self) {
        self.events.clear();
    }

    pub(crate) fn dispatch_event(&mut self, event: SharedObjectEvent) {
        self.events.push(event);
    }

    pub fn connect<T: Transport>(
        shared_object: Arc<Mutex<SharedObject>>,
        connection: &mut NetConnection<T>,
    ) -> std::io::Result<()> {
        let name;

        {
            let mut shared_object = shared_object.lock().unwrap();
            shared_object.dispatch_event(SharedObjectEvent::Use);

            name = shared_object.name.clone();
        }

        connection
            .get_context()
            .add_shared_object(name.clone(), shared_object.clone());
        connection.send_shared_object(name)?;

        Ok(())
    }

    pub fn get_property(&self, key: &str) -> Option<Value> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    pub fn set_property(&mut self, key: String, value: Value) {
        self.flush_state = SharedObjectFlushState::PENDING;
        self.dispatch_event(SharedObjectEvent::RequestChange {
            key: key.clone(),
            value: value.clone(),
        });

        let mut data = self.data.lock().unwrap();
        data.insert(key, value);
    }

    pub fn clear(&self) {
        let mut data = self.data.lock().unwrap();
        data.clear();
    }

    pub fn flush<T: Transport>(
        shared_object: Arc<Mutex<SharedObject>>,
        connection: &mut NetConnection<T>,
    ) -> std::io::Result<()> {
        let name;

        {
            let shared_object = shared_object.lock().unwrap();

            name = shared_object.name.clone();
        }

        let result = connection.send_shared_object(name);

        match result {
            Ok(_) => {
                let mut shared_object = shared_object.lock().unwrap();
                shared_object.flush_state = SharedObjectFlushState::FLUSHED;
            }
            Err(e) => {
                return Err(e);
            }
        }
        
        Ok(())
    }

    pub fn process_events(&mut self) {
        println!("Processing events");
        for event in self.events.iter() {
            match event {
                SharedObjectEvent::Use => {
                    println!("Use event");
                }
                SharedObjectEvent::Release => {
                    println!("Release event");
                }
                SharedObjectEvent::Change { key, value } => {
                    let mut data = self.data.lock().unwrap();
                    data.insert(key.clone(), value.clone());
                }
                SharedObjectEvent::Success => {
                    println!("Success event");
                }
                SharedObjectEvent::SendMessage => {
                    println!("SendMessage event");
                }
                SharedObjectEvent::Status { code, level } => {
                    println!("Status event: {} {}", code, level);
                }
                SharedObjectEvent::Clear => {
                    println!("Clear event");
                }
                SharedObjectEvent::Remove { key } => {
                    println!("Remove event: {}", key);
                }
                SharedObjectEvent::UseSuccess => {
                    self.use_success = true;
                }
                _ => {
                    // encountered an client -> server event, ignore
                }
            }
        }

        self.clear_events();
    }
}
