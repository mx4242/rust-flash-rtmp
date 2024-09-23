use crate::net_connection::transaction_manager::TransactionManager;
use crate::shared_object::SharedObject;
use crate::transport::Transport;
use flash_lso::types::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum ObjectEncoding {
    AMF0 = 0,
    AMF3 = 3,
}

#[derive(Debug)]
pub struct ConnectionArgs {
    pub app: String,
    pub flash_ver: String,
    pub swf_url: String,
    pub tc_url: String,
    pub fpad: bool,
    pub audio_codecs: u32,
    pub video_codecs: u32,
    pub video_function: u32,
    pub page_url: String,
    pub object_encoding: ObjectEncoding,
    pub additional_args: Vec<Value>,
}

#[derive(Debug)]
pub struct NetConnectionContext<T: Transport> {
    pub transport: T,

    pub transaction_manager: TransactionManager,
    pub connection_args: Option<ConnectionArgs>,

    pub shared_objects: HashMap<String, Arc<Mutex<SharedObject>>>,

    pub last_ping_sent: Option<u32>,
    pub chunk_size: u32,
    pub window_ack_size: Option<u32>,
    pub relative_timestamp: u32,
}

pub fn allocate_net_connection_context<T: Transport>(transport: T) -> NetConnectionContext<T> {
    NetConnectionContext {
        transport,
        transaction_manager: TransactionManager::new(),
        connection_args: None,

        shared_objects: HashMap::new(),
        last_ping_sent: None,
        chunk_size: 128,
        window_ack_size: None,
        relative_timestamp: 0,
    }
}

impl<T: Transport> NetConnectionContext<T> {
    pub fn get_shared_object(&self, name: &str) -> std::io::Result<Arc<Mutex<SharedObject>>> {
        self.shared_objects.get(name).cloned().ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Shared Object {} not found", name),
        ))
    }

    pub fn get_shared_object_mut(&mut self, name: &str) -> Option<&mut Arc<Mutex<SharedObject>>> {
        self.shared_objects.get_mut(name)
    }

    pub fn add_shared_object(&mut self, name: String, shared_object: Arc<Mutex<SharedObject>>) {
        self.shared_objects.insert(name, shared_object);
    }

    pub fn remove_shared_object(&mut self, name: &str) {
        self.shared_objects.remove(name);
    }

    pub fn has_shared_object(&self, name: &str) -> bool {
        self.shared_objects.contains_key(name)
    }
}