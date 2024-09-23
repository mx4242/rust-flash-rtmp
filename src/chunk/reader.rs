use crate::chunk::packets::{ChunkBasicHeader, ChunkMessageHeader, ExtendedTimestamp, RTMPChunk};
use crate::context::NetConnectionContext;
use crate::net_connection::packets::RTMPMessage;
use crate::transport::Transport;

use super::packets::MessageTypeId;

pub struct RTMPDechunker {

}

impl RTMPDechunker {
    pub fn read_basic_header<T: Transport>(
        context: &mut NetConnectionContext<T>,
    ) -> std::io::Result<ChunkBasicHeader> {
        let first_byte = context.transport.read_u8()?;
        let format = first_byte >> 6;
        let chunk_stream_id = first_byte & 0b00111111;

        let chunk_stream_id = match chunk_stream_id {
            0 => {
                let second_byte = context.transport.read_u8()?;
                64 + second_byte as u32
            }
            1 => {
                let second_byte = context.transport.read_u8()?;
                let third_byte = context.transport.read_u8()?;
                64 + second_byte as u32 + third_byte as u32
            }
            _ => chunk_stream_id as u32,
        };

        Ok(ChunkBasicHeader {
            chunk_header_format: format,
            chunk_stream_id,
        })
    }

    pub fn read_message_header<T: Transport>(
        context: &mut NetConnectionContext<T>,
        format: u8,
    ) -> std::io::Result<ChunkMessageHeader> {
        match format {
            0 => {
                let timestamp = context.transport.read_u8()? as u32;
                let timestamp = timestamp << 8 | context.transport.read_u8()? as u32;
                let timestamp = timestamp << 8 | context.transport.read_u8()? as u32;

                let message_length = context.transport.read_u8()? as u32;
                let message_length = message_length << 8 | context.transport.read_u8()? as u32;
                let message_length = message_length << 8 | context.transport.read_u8()? as u32;

                let message_type_id = MessageTypeId::try_from(context.transport.read_u8()?)
                    .map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid message type ID",
                        )
                    })?;

                let message_stream_id = context.transport.read_u32_le()?;

                Ok(ChunkMessageHeader::Type0 {
                    absolute_timestamp: timestamp,
                    message_length,
                    message_type_id,
                    message_stream_id,
                })
            }
            1 => {
                let timestamp = context.transport.read_u8()? as u32;
                let timestamp = timestamp << 8 | context.transport.read_u8()? as u32;
                let timestamp = timestamp << 8 | context.transport.read_u8()? as u32;

                let message_length = context.transport.read_u8()? as u32;
                let message_length = message_length << 8 | context.transport.read_u8()? as u32;
                let message_length = message_length << 8 | context.transport.read_u8()? as u32;

                let message_type_id = MessageTypeId::try_from(context.transport.read_u8()?)
                    .map_err(|_| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Invalid message type ID",
                        )
                    })?;

                Ok(ChunkMessageHeader::Type1 {
                    timestamp_delta: timestamp,
                    message_length,
                    message_type_id,
                })
            }
            3 => Ok(ChunkMessageHeader::Type3),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid chunk header format for message header {:?}", format),
            )),
        }
    }

    pub fn read_extended_timestamp<T: Transport>(
        context: &mut NetConnectionContext<T>,
    ) -> std::io::Result<ExtendedTimestamp> {
        let timestamp = context.transport.read_u32_be()?;
        Ok(ExtendedTimestamp(timestamp))
    }

	pub fn read_chunks<T: Transport>(
		context: &mut NetConnectionContext<T>,
	) -> std::io::Result<RTMPMessage> {
		let mut rtmp_chunks: Vec<RTMPChunk> = Vec::new();
		let mut payload_size: Option<usize> = None;

		loop {
			if let Some(size) = payload_size {
				if size == 0 {
					break;
				}
			}

			let basic_header = RTMPDechunker::read_basic_header(context)?;
			let message_header = RTMPDechunker::read_message_header(context, basic_header.chunk_header_format)?;

            if message_header.is_extended_timestamp() {
                let ets = RTMPDechunker::read_extended_timestamp(context)?;
                println!("Extended timestamp: {:?}", ets);
            }

			payload_size = match message_header {
				ChunkMessageHeader::Type0 { message_length, .. } => Some(message_length as usize),
				ChunkMessageHeader::Type1 { message_length, .. } => Some(message_length as usize),
				_ => payload_size,
			};

			let read_size = std::cmp::min(payload_size.unwrap_or(context.chunk_size as usize), context.chunk_size as usize);
			let payload = context.transport.read_data(read_size)?;

			if let Some(size) = payload_size.as_mut() {
				*size -= read_size;
			}

			rtmp_chunks.push(RTMPChunk {
				basic_header,
				message_header,
				extended_timestamp: None,
				data: payload,
			});
		}

        let payload = rtmp_chunks.iter().flat_map(|chunk| chunk.data.iter()).cloned().collect();

        Ok(RTMPMessage {
            timestamp: 0,
            message_type_id: match rtmp_chunks[0].message_header {
                ChunkMessageHeader::Type0 { message_type_id, .. } => message_type_id,
                ChunkMessageHeader::Type1 { message_type_id, .. } => message_type_id,
                _ => todo!("message type id chunk 0 is not type0"),
            },
            // todo: implement chunk stream id
            chunk_stream_id: super::packets::ChunkImportance::CommandAMF0AMF3,
            message_stream_id: match rtmp_chunks[0].message_header {
                ChunkMessageHeader::Type0 { message_stream_id, .. } => message_stream_id,
                ChunkMessageHeader::Type1 { .. } => {
                    println!("TODO: Type1 message stream id");
                    0
                },
                _ => todo!("message type id stream id is not type0"),
            },
            payload,
        })

    }
}
