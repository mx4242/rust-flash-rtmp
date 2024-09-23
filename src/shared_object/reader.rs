use std::sync::{Arc, Mutex, MutexGuard};

use crate::{context::NetConnectionContext, shared_object::SharedObject, transport::Transport};
use nom::bytes::complete::take;
use nom::number::complete::{be_u16, be_u32, be_u8};

use flash_lso::amf0::read::AMF0Decoder;
use std::rc::Rc;

use super::SharedObjectEvent;

pub struct SharedObjectReader<'a> {
    shared_object: Option<MutexGuard<'a, SharedObject>>,
}

impl<'a> SharedObjectReader<'a> {
    pub fn new() -> Self {
        SharedObjectReader {
            shared_object: None,
        }
    }

    fn read_string<'b>(&self, payload: &'a [u8]) -> std::io::Result<(&'a [u8], String)> {
        let (i, string_length) =
            be_u16(payload).map_err(|e: nom::Err<nom::error::Error<&[u8]>>| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse string length: {:?}", e),
                )
            })?;

        let (i, string_bytes) =
            take(string_length)(i).map_err(|e: nom::Err<nom::error::Error<&[u8]>>| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse string bytes: {:?}", e),
                )
            })?;

        let string = String::from_utf8(string_bytes.to_vec()).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse string: {:?}", e),
            )
        })?;

        Ok((i, string))
    }

    fn read_change_event<'b>(&self, payload: &'a [u8]) -> std::io::Result<SharedObjectEvent> {
        // handle case where the event payload size is 0
        let (j, key) = self.read_string(payload)?;
        let (j, value) = AMF0Decoder::default().parse_single_element(j).unwrap();

        if !j.is_empty() {
            println!(
                "for some reason, there are still bytes left in the payload: {:?}",
                j
            );
        }

        Ok(SharedObjectEvent::Change {
            key,
            value: Rc::try_unwrap(value).unwrap(),
        })
    }

    fn read_event<'b>(&self, payload: &'a [u8]) -> std::io::Result<(&'a [u8], SharedObjectEvent)> {
		println!("payload: {:?}", payload);
        let (i, event_type) = be_u8(payload).map_err(|e: nom::Err<nom::error::Error<&[u8]>>| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse event type: {:?}", e),
            )
        })?;

        let (i, event_length) = be_u32(i).map_err(|e: nom::Err<nom::error::Error<&[u8]>>| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse event length: {:?}", e),
            )
        })?;

        let (i, event_payload) =
            take(event_length as usize)(i).map_err(|e: nom::Err<nom::error::Error<&[u8]>>| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse event payload: {:?}", e),
                )
            })?;

		println!("event type: {:?}, event length: {:?}, event payload: {:?}", event_type, event_length, event_payload);

        let event: SharedObjectEvent = match event_type {
            // 0x01 => self.read_use_event(event_payload)?,
            // 0x02 => self.read_release_event(event_payload)?,
            0x04 => self.read_change_event(event_payload)?,
            // 0x06 => self.send_message_event(event_payload)?,
            // 0x08 => self.read_clear_event(event_payload)?,
            // 0x09 => self.read_remove_event(event_payload)?,
            0x0b => SharedObjectEvent::UseSuccess,
            // _ => Err(std::io::Error::new(
            //     std::io::ErrorKind::InvalidData,
            //     format!("Unknown event type: {:?}", event_type),
            // ))?
            _ => {
                println!("Unknown event type: {:?}", event_type);
                SharedObjectEvent::UseSuccess
            }
        };

        Ok((i, event))
    }

    pub fn read<'b, T: Transport>(
        &self,
        context: &mut NetConnectionContext<T>,
        payload: Vec<u8>,
    ) -> std::io::Result<Arc<Mutex<SharedObject>>> {
        let (i, name) = self.read_string(payload.as_slice())?;

        let (i, version) = be_u32(i).map_err(|e: nom::Err<nom::error::Error<&[u8]>>| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse shared object version: {:?}", e),
            )
        })?;

        let (i, _flags) = be_u32(i).map_err(|e: nom::Err<nom::error::Error<&[u8]>>| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse shared object flags: {:?}", e),
            )
        })?;

        let (mut i, _flags1) = be_u32(i).map_err(|e: nom::Err<nom::error::Error<&[u8]>>| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse shared object flags1: {:?}", e),
            )
        })?;

        let shared_object_arc = context.get_shared_object(&name)?;

        {
            let mut shared_object = shared_object_arc.lock().unwrap();

            while !i.is_empty() {
                let (j, event) = self.read_event(i)?;
                shared_object.dispatch_event(event);
                i = j;
            }

            shared_object.version = version;
        }

        Ok(shared_object_arc)
    }
}
