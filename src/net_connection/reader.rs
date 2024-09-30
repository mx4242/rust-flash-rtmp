use crate::{
    chunk::{
        packets::MessageTypeId,
        reader::RTMPDechunker,
    }, context::NetConnectionContext, net_connection::{
        packets::{
            AMFCommandMessage, PeerBandwidthLimitType, RTMPMessageType, SetChunkSize, SetPeerBandwidth, WindowAcknowledgementSize
        },
        user_control_messages::reader::UserControlMessageReader
    }, shared_object::reader::SharedObjectReader, transport::Transport, utils::nom::RTMPResult, errors::Error
};

use flash_lso::{
    amf0::read::AMF0Decoder, types::Value
};

use nom::number::complete::{be_u32, be_u8};

use std::rc::Rc;

#[derive(Debug)]

pub struct RTMPReader {}

impl RTMPReader {
    fn read_window_acknowledgement_size(
        payload: &[u8]
    ) -> RTMPResult<'_, WindowAcknowledgementSize> {
        let (i, size) = be_u32(payload)?;

        Ok((i, WindowAcknowledgementSize { size }))
    }

    fn read_set_peer_bandwidth(payload: &[u8]) -> RTMPResult<'_, SetPeerBandwidth> {
        let (i, size) = be_u32(payload)?;
        let (i, bandwidth_limit_byte) = be_u8(i)?;

        let limit_type = PeerBandwidthLimitType::try_from(bandwidth_limit_byte)
            .map_err(|_| nom::Err::Failure(Error::IoError("Invalid peer bandwidth limit type".to_string(), std::io::ErrorKind::InvalidData)))?;


        Ok((i, SetPeerBandwidth { size, limit_type }))
    }

    fn read_set_chunk_size(payload: &[u8]) -> RTMPResult<'_, SetChunkSize> {
        let (i, size) = be_u32(payload)?;

        Ok((i, SetChunkSize { size }))
    }

    fn read_amf0_command(payload: Vec<u8>) -> std::io::Result<AMFCommandMessage> {
        let mut amf_decoder = AMF0Decoder::default();

        let (i, procedure_name) = amf_decoder
            .parse_single_element(payload.as_slice())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse procedure name: {:?}", e)))?;

        let procedure_name = match Rc::try_unwrap(procedure_name) {
            Ok(value) => value,
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to unwrap procedure name")),
        };

        let procedure_name = match procedure_name {
            Value::String(s) => s,
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid procedure name type")),
        };

        let (i, transaction_id) = amf_decoder
            .parse_single_element(i)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse transaction ID: {:?}", e)))?;

        let transaction_id = match Rc::try_unwrap(transaction_id) {
            Ok(value) => value,
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to unwrap transaction ID")),
        };

        let transaction_id = match transaction_id {
            Value::Number(n) => n as u32,
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid transaction ID type")),
        };

        let (mut i, command_object) = amf_decoder
            .parse_single_element(i)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse command object: {:?}", e)))?;

        let command_object = (*command_object).clone();

        let command_object = match command_object {
            Value::Null => None,
            _ => Some(command_object),
        };

        let mut optional_arguments = Vec::new();

        while !i.is_empty() {
            let (j, optional_argument) = amf_decoder
                .parse_single_element(i)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse optional argument: {:?}", e)))?;

            let optional_argument = (*optional_argument).clone();

            optional_arguments.push(optional_argument);

            i = j;
        }

        Ok(AMFCommandMessage {
            procedure_name,
            transaction_id,
            command_object,
            optional_arguments,
        })
    }

    pub fn read<'b, T: Transport>(context: &mut NetConnectionContext<T>) -> std::io::Result<RTMPMessageType> {
        let message = RTMPDechunker::read_chunks(context)?;

        let parsed_message = match message.message_type_id {
            MessageTypeId::WindowAcknowledgementSize => {
                let (_, window_acknowledgement_size) = RTMPReader::read_window_acknowledgement_size(message.payload.as_slice())
                    .expect("Failed to parse window acknowledgement size");
                RTMPMessageType::WindowAcknowledgementSize(window_acknowledgement_size)
            }
            MessageTypeId::SetPeerBandwidth => {
                let (_, set_peer_bandwidth) = RTMPReader::read_set_peer_bandwidth(message.payload.as_slice())
                    .expect("Failed to parse set peer bandwidth");
                RTMPMessageType::SetPeerBandwidth(set_peer_bandwidth)
            }
            MessageTypeId::UserControlMessage => {
                let (_, user_control_message) = UserControlMessageReader::read(message.payload.as_slice())
                    .expect("Failed to parse user control message");
                RTMPMessageType::UserControlMessage(user_control_message)
            }
            MessageTypeId::SetChunkSize => {
                let (_, chunk_size) = RTMPReader::read_set_chunk_size(message.payload.as_slice())
                    .expect("Failed to parse set chunk size");
                RTMPMessageType::SetChunkSize(chunk_size)
            }
            MessageTypeId::CommandAMF0 => {
                let command = RTMPReader::read_amf0_command(message.payload)?;
                RTMPMessageType::AMF0Command(command)
            }
            MessageTypeId::SharedObjectAMF0 => {
                let (_, shared_object) = SharedObjectReader::new().read(context, message.payload.as_slice())
                    .expect("Failed to parse shared object");
                RTMPMessageType::AMF3SharedObject(shared_object)
            }
            _ => todo!("Parsing Message type {:?} not implemented", message.message_type_id),
        };

        Ok(parsed_message)
    }
}
