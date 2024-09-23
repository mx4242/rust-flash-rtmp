use crate::{
    chunk::{
        packets::{ChunkImportance, MessageTypeId},
        writer::RTMPChunker,
    },
    context::NetConnectionContext,
    net_connection::packets::{
        AMFCommandMessage, RTMPMessage, RTMPMessageType, UserControlMessage,
    },
    shared_object::writer::SharedObjectWriter,
    transport::Transport,
};

use flash_lso::{amf0::write::write_value, types::Value};
use std::rc::Rc;

#[derive(Debug)]
pub struct RTMPWriter {}

impl RTMPWriter {
    fn write_amf0_command(
        command: AMFCommandMessage,
        payload_vector: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        write_value(
            payload_vector,
            &Rc::new(Value::String(command.procedure_name)),
        )?;

        write_value(
            payload_vector,
            &Rc::new(Value::Number(command.transaction_id as f64)),
        )?;

        if let Some(command_object) = command.command_object {
            write_value(payload_vector, &Rc::new(command_object))?;
        }

        for optional_argument in command.optional_arguments {
            write_value(payload_vector, &Rc::new(optional_argument))?;
        }

        Ok(())
    }

    fn write_user_control_message(
        user_control_message: UserControlMessage,
        payload_vector: &mut Vec<u8>,
    ) -> std::io::Result<()> {
        let (event_type, payload): (i16, Vec<u8>) = match user_control_message {
            UserControlMessage::PingResponse { timestamp } => {
                (0x07, timestamp.to_be_bytes().to_vec())
            }
            _ => todo!(
                "User control message type not implemented, {:?}",
                user_control_message
            ),
        };

        payload_vector.extend_from_slice(&event_type.to_be_bytes());
        payload_vector.extend_from_slice(&payload);

        Ok(())
    }

    pub fn write<T: Transport>(
        payload: RTMPMessageType,
        context: &mut NetConnectionContext<T>,
    ) -> std::io::Result<()> {
        let mut payload_vector: Vec<u8> = Vec::new();

        let (message_type_id, chunk_stream_id) = match payload {
            RTMPMessageType::AMF0Command(command) => {
                RTMPWriter::write_amf0_command(command, &mut payload_vector)?;

                (MessageTypeId::CommandAMF0, ChunkImportance::CommandAMF0AMF3)
            }
            RTMPMessageType::UserControlMessage(user_control_message) => {
                RTMPWriter::write_user_control_message(user_control_message, &mut payload_vector)?;

                (
                    MessageTypeId::UserControlMessage,
                    ChunkImportance::ProtocolUserControl,
                )
            }
            RTMPMessageType::AMF3SharedObject(shared_object) => {
                SharedObjectWriter::new(shared_object).write(&mut payload_vector, context)?;

                (
                    MessageTypeId::SharedObjectAMF3,
                    ChunkImportance::CommandAMF0AMF3,
                )
            }
            _ => {
                todo!("Payload type not implemented")
            }
        };

        let rtmp_message = RTMPMessage {
            timestamp: 0,
            message_type_id,
            chunk_stream_id,
            message_stream_id: 0,
            payload: payload_vector,
        };

        RTMPChunker::write_chunks(rtmp_message, context)?;

        Ok(())
    }
}
