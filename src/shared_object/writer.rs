use flash_lso::{amf0::write::write_value, types::Value};

use crate::{context::NetConnectionContext, shared_object::SharedObject, transport::Transport};
use std::{rc::Rc, sync::{Arc, Mutex, MutexGuard}};

use super::SharedObjectEvent;

pub struct SharedObjectWriter {
    shared_object: Arc<Mutex<SharedObject>>,
}

impl SharedObjectWriter {
    pub fn new(shared_object: Arc<Mutex<SharedObject>>) -> Self {
        SharedObjectWriter { shared_object }
    }

    fn write_string(
        &self,
        target_string: String,
        payload_vector: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        let string_bytes = target_string.as_bytes();
        let string_length = string_bytes.len() as u16;

        payload_vector.extend_from_slice(string_length.to_be_bytes().as_ref());
        payload_vector.extend_from_slice(string_bytes);

        Ok(())
    }

    fn write_request_change_event(
        &self,
        key: String,
        value: Value,
        event_payload: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        self.write_string(key, event_payload)?;
        write_value(event_payload, &Rc::new(value))?;
        Ok(())
    }

    fn write_shared_object_event(
        &self,
        event: SharedObjectEvent,
        payload_vector: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        let mut event_payload: Vec<u8> = Vec::new();
        let event_type = event.get_type();

        match event {
            SharedObjectEvent::Use => {}
            SharedObjectEvent::RequestChange { key, value } => {
                self.write_request_change_event(key, value, &mut event_payload)?;
            }
            _ => {
                todo!()
            }
        }

        payload_vector.extend_from_slice(&event_type.to_be_bytes());
        payload_vector.extend_from_slice(&(event_payload.len() as u32).to_be_bytes());
        payload_vector.extend_from_slice(&event_payload);

        Ok(())
    }

    fn write_amf0_shared_object<'a>(
        &self,
        mut shared_object: MutexGuard<'a, SharedObject>,
        payload_vector: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        self.write_string(shared_object.name.clone(), payload_vector)?;

        payload_vector.extend_from_slice(shared_object.version.to_be_bytes().as_ref());

        let peristant_value: u32 = if shared_object.persistant { 2 } else { 0 };
        payload_vector.extend_from_slice(&peristant_value.to_be_bytes());

        payload_vector.extend_from_slice(&0u32.to_be_bytes());

        for event in shared_object.events.iter() {
            println!("event: {:?}", event);
            self.write_shared_object_event(event.clone(), payload_vector)?;
        }

        shared_object.clear_events();

        Ok(())
    }

    fn write_amf3_shared_object<'a>(
        &self,
        shared_object: MutexGuard<'a, SharedObject>,
        payload_vector: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        payload_vector.push(0x00); // unknown (not AMF version)

        self.write_amf0_shared_object(shared_object, payload_vector)?;

        Ok(())
    }

    pub fn write<T: Transport>(
        &mut self,
        payload_vector: &mut Vec<u8>,
        _context: &mut NetConnectionContext<T>,
    ) -> std::io::Result<()> {
        // todo!() TODO: implement amf0 shared object some day
        let shared_object = self.shared_object.lock().unwrap();

        self.write_amf3_shared_object(shared_object, payload_vector)?;

        Ok(())
    }
}
