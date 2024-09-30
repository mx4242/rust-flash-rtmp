use std::sync::{Arc, Mutex, MutexGuard};

use crate::errors::Error;
use crate::utils::nom::RTMPResult;
use crate::{
    context::NetConnectionContext,
    shared_object::{SharedObject, SharedObjectEvent},
    transport::Transport,
};
use nom::bytes::complete::take;
use nom::number::complete::{be_u16, be_u32, be_u8};
use nom::combinator::map_res;

use flash_lso::amf0::read::AMF0Decoder;
use std::rc::Rc;

pub struct SharedObjectReader<'a> {
    shared_object: Option<MutexGuard<'a, SharedObject>>,
}

impl<'a> SharedObjectReader<'a> {
    pub fn new() -> Self {
        SharedObjectReader {
            shared_object: None,
        }
    }
    
    pub fn read<'b, T: Transport>(
        &'b self,
        context: &mut NetConnectionContext<T>,
        payload: &'b [u8],
    ) -> RTMPResult<'b, Arc<Mutex<SharedObject>>> {
        let (i, name) = self.read_string(payload)?;
        let (i, version) = be_u32(i)?;

        let (i, _flags) = be_u32(i)?;
        let (mut i, _flags1) = be_u32(i)?;

        let shared_object_arc = context.get_shared_object(&name)
            .map_err(|e| nom::Err::Error(Error::IoError(e.to_string(), e.kind())))?;

        {
            let mut shared_object = shared_object_arc.lock().unwrap();

            while !i.is_empty() {
                let (j, event) = self.read_event(i)?;
                shared_object.dispatch_event(event);
                i = j;
            }

            shared_object.version = version;
        }

        Ok((i, shared_object_arc))
    }

    fn read_string<'b>(&self, payload: &'b [u8]) -> RTMPResult<'b, String> {
        let (i, length) = be_u16(payload)?;
        let (i, string) = map_res(take(length), std::str::from_utf8)(i)?;

        Ok((i, string.to_string()))
    }

    fn read_change_event<'b>(&self, payload: &'b [u8]) -> RTMPResult<'b, SharedObjectEvent> {
        // TODO: handle case where the event payload size is 0
        let (i, key) = self.read_string(payload)?;
        let (i, value) = AMF0Decoder::default().parse_single_element(i).unwrap();

        if !i.is_empty() {
            println!(
                "for some reason, there are still bytes left in the payload: {:?}",
                i
            );
        }

        Ok((i, SharedObjectEvent::Change {
            key,
            value: Rc::try_unwrap(value).unwrap(),
        }))
    }

    fn read_event<'b>(&self, payload: &'b [u8]) -> RTMPResult<'b, SharedObjectEvent> {
        let (i, event_type) = be_u8(payload)?;
        let (i, event_length) = be_u32(i)?;
        let (i, event_payload) = take(event_length as usize)(i)?;

        let (i, event) = match event_type {
            // 0x01 => self.read_use_event(event_payload)?,
            // 0x02 => self.read_release_event(event_payload)?,
            0x04 => self.read_change_event(event_payload)?,
            // 0x06 => self.send_message_event(event_payload)?,
            // 0x08 => self.read_clear_event(event_payload)?,
            // 0x09 => self.read_remove_event(event_payload)?,
            0x0b => (i, SharedObjectEvent::UseSuccess),
            // _ => Err(std::io::Error::new(
            //     std::io::ErrorKind::InvalidData,
            //     format!("Unknown event type: {:?}", event_type),
            // ))?
            _ => {
                println!("Unknown event type: {:?}", event_type);
                (i, SharedObjectEvent::UseSuccess)
            }
        };

        Ok((i, event))
    }
}
