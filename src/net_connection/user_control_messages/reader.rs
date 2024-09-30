use nom::number::complete::{be_u16, be_u32};

use crate::net_connection::packets::UserControlMessage;
use crate::utils::nom::RTMPResult;

pub struct UserControlMessageReader { }

impl UserControlMessageReader {
    pub fn read_stream_begin(payload: &[u8]) -> RTMPResult<'_, UserControlMessage> {
        let (i, stream_id) = be_u32(payload)?;

        Ok((i, UserControlMessage::StreamBegin {
            stream_id
        }))
    }

    pub fn read_stream_eof(payload: &[u8]) -> RTMPResult<'_, UserControlMessage> {
        let (i, stream_id) = be_u32(payload)?;

        Ok((i, UserControlMessage::StreamEOF {
            stream_id
        }))
    }

    pub fn read_stream_dry(payload: &[u8]) -> RTMPResult<'_, UserControlMessage> {
        let (i, stream_id) = be_u32(payload)?;

        Ok((i, UserControlMessage::StreamDry {
            stream_id
        }))
    }

    pub fn read_set_buffer_length(payload: &[u8]) -> RTMPResult<'_, UserControlMessage> {
        let (i, stream_id) = be_u32(payload)?;

        let (i, buffer_length) = be_u32(i)?;

        Ok((i, UserControlMessage::SetBufferLength {
            stream_id,
            buffer_length
        }))
    }

    pub fn read_stream_is_recorded(payload: &[u8]) -> RTMPResult<'_, UserControlMessage> {
        let (i, stream_id) = be_u32(payload)?;

        Ok((i, UserControlMessage::StreamIsRecorded {
            stream_id
        }))
    }

    pub fn read_ping_request(payload: &[u8]) -> RTMPResult<'_, UserControlMessage> {
        let (i, timestamp) = be_u32(payload)?;

        Ok((i, UserControlMessage::PingRequest {
            timestamp
        }))
    }

    pub fn read_ping_response(payload: &[u8]) -> RTMPResult<'_, UserControlMessage> {
        let (i, timestamp) = be_u32(payload)?;

        Ok((i, UserControlMessage::PingResponse {
            timestamp
        }))
    }

    pub fn read(payload: &[u8]) -> RTMPResult<'_, UserControlMessage> {
        let (i, event_type) = be_u16(payload)?;
        
        let (i, user_control_message) = match event_type {
            0 => UserControlMessageReader::read_stream_begin(i)?,
            1 => UserControlMessageReader::read_stream_eof(i)?,
            2 => UserControlMessageReader::read_stream_dry(i)?,
            3 => UserControlMessageReader::read_set_buffer_length(i)?,
            4 => UserControlMessageReader::read_stream_is_recorded(i)?,
            6 => UserControlMessageReader::read_ping_request(i)?,
            7 => UserControlMessageReader::read_ping_response(i)?,
            _ => unimplemented!("User control message type not implemented, {:?}", event_type),
        };

        Ok((i, user_control_message))
    }
}