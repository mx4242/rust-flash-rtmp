use crate::{chunk::packets::{ChunkBasicHeader, ChunkMessageHeader, RTMPChunk}, context::NetConnectionContext, transport::Transport};
use crate::net_connection::packets::RTMPMessage;

pub struct RTMPChunker {}

impl RTMPChunker {
    fn write_basic_header(buffer: &mut Vec<u8>, basic_header: ChunkBasicHeader) {
        let format = basic_header.chunk_header_format;
        let chunk_stream_id = basic_header.chunk_stream_id;

        if chunk_stream_id >= 64 + 255 {
            buffer.push(format << 6 | 1);
            buffer.push(((chunk_stream_id - 64) >> 8) as u8);
            buffer.push((chunk_stream_id - 64) as u8);
        } else if chunk_stream_id >= 64 {
            buffer.push(format << 6);
            buffer.push((chunk_stream_id - 64) as u8);
        } else {
            buffer.push(format << 6 | chunk_stream_id as u8);
        }
    }

    fn write_message_header(buffer: &mut Vec<u8>, message_header: ChunkMessageHeader) {
        match message_header {
            ChunkMessageHeader::Type0 { 
                absolute_timestamp,
                message_length, 
                message_type_id, 
                message_stream_id 
            } => {
                // TODO: maybe move to a util?
                // write timestamp as u24
                buffer.push((absolute_timestamp >> 16) as u8);
                buffer.push((absolute_timestamp >> 8) as u8);
                buffer.push(absolute_timestamp as u8);  

                // same goes for length
                buffer.push((message_length >> 16) as u8);
                buffer.push((message_length >> 8) as u8);
                buffer.push(message_length as u8);  

                buffer.push(message_type_id as u8);

                // uses little endian
                buffer.extend_from_slice(&message_stream_id.to_le_bytes());
            },
            ChunkMessageHeader::Type1 { 
                timestamp_delta,
                message_length, 
                message_type_id, 
            } => {
                buffer.push((timestamp_delta >> 16) as u8);
                buffer.push((timestamp_delta >> 8) as u8);
                buffer.push(timestamp_delta as u8);

                buffer.push((message_length >> 16) as u8);
                buffer.push((message_length >> 8) as u8);
                buffer.push(message_length as u8);

                buffer.push(message_type_id as u8);
            },
            ChunkMessageHeader::Type2 { 
                timestamp_delta 
            } => {
                buffer.push((timestamp_delta >> 16) as u8);
                buffer.push((timestamp_delta >> 8) as u8);
                buffer.push(timestamp_delta as u8);
            },
            ChunkMessageHeader::Type3 => {
                // no fields
            }
        }
    } 

    pub fn write_chunks<T: Transport>(rtmp_message: RTMPMessage, context: &mut NetConnectionContext<T>) -> std::io::Result<()> {
        let mut buffer: Vec<u8> = Vec::new();
        let mut rtmp_chunks: Vec<RTMPChunk> = Vec::new();

        let payload_chunks = rtmp_message.payload.chunks(context.chunk_size as usize);
        let mut remaining = rtmp_message.payload.len();
    
        let mut first_chunk = true;
        let chunk_stream_id = rtmp_message.chunk_stream_id as u32;

        for payload_chunk in payload_chunks {
            let (chunk_header, chunk_header_format) = if first_chunk {
                first_chunk = false;
                (ChunkMessageHeader::Type0 {
                    absolute_timestamp: 0, 
                    message_length: remaining as u32, 
                    message_type_id: rtmp_message.message_type_id, 
                    message_stream_id: rtmp_message.message_stream_id 
                }, 0)
            } else {
                (ChunkMessageHeader::Type3, 3)
            };

            remaining -= payload_chunk.len();
            rtmp_chunks.push(RTMPChunk {
                basic_header: ChunkBasicHeader { 
                    chunk_header_format,
                    chunk_stream_id
                },
                message_header: chunk_header,
                extended_timestamp: None,
                data: payload_chunk.to_vec()
            })
        }

        for rtmp_chunk in rtmp_chunks {
            RTMPChunker::write_basic_header(&mut buffer, rtmp_chunk.basic_header);
            RTMPChunker::write_message_header(&mut buffer, rtmp_chunk.message_header);

            if let Some(_extended_timestamp) = rtmp_chunk.extended_timestamp {
                // TODO: write extended timestamp
            }

            buffer.extend_from_slice(&rtmp_chunk.data);
        }

        context.transport.write_data(buffer)?;

        Ok(())
    }

    
}