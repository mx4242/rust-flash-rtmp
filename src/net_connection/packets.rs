use flash_lso::types::Value;
use crate::{chunk::packets::{ChunkImportance, MessageTypeId}, shared_object::SharedObject};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct WindowAcknowledgementSize {
    pub size: u32,
}

#[derive(Debug)]
pub enum UserControlMessage {
    StreamBegin {
        stream_id: u32,
    },
    StreamEOF {
        stream_id: u32,
    },
    StreamDry {
        stream_id: u32,
    },
    SetBufferLength {
        stream_id: u32,
        buffer_length: u32,
    },
    StreamIsRecorded {
        stream_id: u32,
    },
    PingRequest {
        timestamp: u32,
    },
    PingResponse {
        timestamp: u32,
    },
}

#[derive(Debug)]
pub enum PeerBandwidthLimitType {
    Hard = 0,
    Soft = 1,
    Dynamic = 2,
}

impl TryFrom<u8> for PeerBandwidthLimitType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PeerBandwidthLimitType::Hard),
            1 => Ok(PeerBandwidthLimitType::Soft),
            2 => Ok(PeerBandwidthLimitType::Dynamic),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct SetPeerBandwidth {
    pub size: u32,
    pub limit_type: PeerBandwidthLimitType,
}

#[derive(Debug)]
pub struct SetChunkSize {
    pub size: u32,
}

#[derive(Debug)]
pub struct AMFCommandMessage {
    /// Name of the remote procedure that is
    /// called.
    pub procedure_name: String,

    /// If a response is expected we give a
    /// transaction Id. Else we pass a value of
    /// 0
    pub transaction_id: u32,

    /// If there exists any command info this
    /// is set, else this is set to null type.
    pub command_object: Option<Value>,

    /// Any optional arguments to be provided 
    pub optional_arguments: Vec<Value>
}

#[derive(Debug)]
pub enum RTMPMessageType {
    SetChunkSize(SetChunkSize),
    UserControlMessage(UserControlMessage),
    WindowAcknowledgementSize(WindowAcknowledgementSize),
    SetPeerBandwidth(SetPeerBandwidth),
    AMF0Command(AMFCommandMessage),  
    AMF3SharedObject(Arc<Mutex<SharedObject>>),
}

#[derive(Debug)]
pub struct RTMPMessage {
    pub timestamp: u32,
    pub message_type_id: MessageTypeId,
    pub message_stream_id: u32,
    // todo: chunk_stream_id should be a u32
    pub chunk_stream_id: ChunkImportance,
    pub payload: Vec<u8>,
}