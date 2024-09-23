
// Enum for the Message Type ID
#[derive(Debug, Clone, Copy)]
pub enum MessageTypeId {
    /// Protocol control message 1, Set Chunk Size, is used to notify the
    /// peer of a new maximum chunk size.
    SetChunkSize = 0x01,

    /// Protocol control message 2, Abort Message, is used to notify the peer
    /// if it is waiting for chunks to complete a message, then to discard
    /// the partially received message over a chunk stream. The peer
    /// receives the chunk stream ID as this protocol message’s payload. An
    /// application may send this message when closing in order to indicate
    /// that further processing of the messages is not required.
    AbortMessage = 0x02,

    // The client or the server MUST send an acknowledgment to the peer
    // after receiving bytes equal to the window size. The window size is
    // the maximum number of bytes that the sender sends without receiving
    // acknowledgment from the receiver. This message specifies the
    // sequence number, which is the number of the bytes received so far.
    Acknowledgement = 0x03,

    /// RTMP uses message type ID 4 for User Control messages. These
    /// messages contain information used by the RTMP streaming layer.
    UserControlMessage = 0x04,

    /// The client or the server sends this message to inform the peer of the
    /// window size to use between sending acknowledgments. The sender
    /// expects acknowledgment from its peer after the sender sends window
    /// size bytes. The receiving peer MUST send an Acknowledgement
    /// (Section 5.4.3) after receiving the indicated number of bytes since
    /// the last Acknowledgement was sent, or from the beginning of the
    /// session if no Acknowledgement has yet been sent.
    WindowAcknowledgementSize = 0x05,

    /// The client or the server sends this message to limit the output
    /// bandwidth of its peer. The peer receiving this message limits its
    /// output bandwidth by limiting the amount of sent but unacknowledged
    /// data to the window size indicated in this message. The peer
    /// receiving this message SHOULD respond with a Window Acknowledgement
    /// Size message if the window size is different from the last one sent
    /// to the sender of this message.
    SetPeerBandwidth = 0x06,

    /// The client or the server sends this message to send audio data to the
    /// peer. The message type value of 8 is reserved for audio messages.
    AudioData = 0x08,

    /// The client or the server sends this message to send video data to the
    /// peer. The message type value of 9 is reserved for video messages.
    VideoData = 0x09,

    /// The client or the server sends this message to send Metadata or any
    /// user data to the peer. Metadata includes details about the
    /// data(audio, video etc.) like creation time, duration, theme and so
    /// on.
    DataAMF3 = 0x0F,
    DataAMF0 = 0x12,

    /// A shared object is a Flash object (a collection of name value pairs)
    /// that are in synchronization across multiple clients, instances, and
    /// so on.
    SharedObjectAMF3 = 0x10,
    SharedObjectAMF0 = 0x13,

    /// Command messages carry the AMF-encoded commands between the client
    /// and the server. encoding. These messages are sent to perform some 
    /// operations like connect, createStream, publish, play, pause on the peer. 
    /// Command messages like onstatus, result etc. are used to inform the sender
    /// about the status of the requested commands. A command message
    /// consists of command name, transaction ID, and command object that
    /// contains related parameters. A client or a server can request Remote
    /// Procedure Calls (RPC) over streams that are communicated using the
    /// command messages to the peer.
    CommandAMF0 = 0x14,
    CommandAMF3 = 0x11,

    /// An aggregate message is a single message that contains a series of
    /// RTMP sub-messages using the format described in Section 6.1. Message
    /// type 22 is used for aggregate messages.
    AggregateMessage = 0x16,
}   

impl TryFrom <u8> for MessageTypeId {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(MessageTypeId::SetChunkSize),
            0x02 => Ok(MessageTypeId::AbortMessage),
            0x03 => Ok(MessageTypeId::Acknowledgement),
            0x04 => Ok(MessageTypeId::UserControlMessage),
            0x05 => Ok(MessageTypeId::WindowAcknowledgementSize),
            0x06 => Ok(MessageTypeId::SetPeerBandwidth),
            0x08 => Ok(MessageTypeId::AudioData),
            0x09 => Ok(MessageTypeId::VideoData),
            0x0F => Ok(MessageTypeId::DataAMF3),
            0x12 => Ok(MessageTypeId::DataAMF0),
            0x10 => Ok(MessageTypeId::SharedObjectAMF3),
            0x13 => Ok(MessageTypeId::SharedObjectAMF0),
            0x14 => Ok(MessageTypeId::CommandAMF0),
            0x11 => Ok(MessageTypeId::CommandAMF3),
            0x16 => Ok(MessageTypeId::AggregateMessage),
            _ => Err("Invalid message type ID"),
        }
    }
}

// Represents the basic header chunk stream ID values, as they are basically constants.
#[derive(Debug)]
pub enum ChunkImportance {
    /// Used for protocol messages, such as User Control messages.
    ProtocolUserControl = 2,

    /// Used for command messages that are encoded with either AMF0 or AMF3.
    CommandAMF0AMF3 = 3,

    /// Used for audio data messages.
    Audio = 4,

    /// Used for video data messages.
    Video = 5,

    /// Used for data messages encoded with either AMF0 or AMF3.
    DataAMF0AMF3 = 6,
}

impl TryFrom <u8> for ChunkImportance {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(ChunkImportance::ProtocolUserControl),
            3 => Ok(ChunkImportance::CommandAMF0AMF3),
            4 => Ok(ChunkImportance::Audio),
            5 => Ok(ChunkImportance::Video),
            6 => Ok(ChunkImportance::DataAMF0AMF3),
            _ => Err("Invalid chunk importance ID"),
        }
    }
}

/// Represents the basic header of an RTMP chunk, encoding the chunk stream ID and chunk type (format).
#[derive(Debug)]
pub struct ChunkBasicHeader {
    pub chunk_header_format: u8,
    pub chunk_stream_id: u32,     // Variable length, up to 24 bits, renamed from 'cs_id'
}

/// Represents the different types of message headers in RTMP chunks.
#[derive(Debug)]
pub enum ChunkMessageHeader {
    Type0 {
        absolute_timestamp: u32, // can be extended
        message_length: u32,
        message_type_id: MessageTypeId,
        message_stream_id: u32,       // 4 bytes, little-endian
    },
    Type1 {
        timestamp_delta: u32, // can be extended
        message_length: u32,
        message_type_id: MessageTypeId,
    },
    Type2 {
        timestamp_delta: u32, // can be extended
    },
    Type3, // No fields, takes values from the preceding chunk
}

impl ChunkMessageHeader {
    pub fn is_extended_timestamp(&self) -> bool {
        match self {
            ChunkMessageHeader::Type0 { absolute_timestamp, .. } => *absolute_timestamp == 0xFFFFFF,
            ChunkMessageHeader::Type1 { .. } => false,
            ChunkMessageHeader::Type2 { .. } => false,
            ChunkMessageHeader::Type3 => false,
        }
    }
}

/// Represents the Extended Timestamp used in RTMP chunks.
#[derive(Debug)]
pub struct ExtendedTimestamp(pub u32);

/// Represents an RTMP chunk, containing a header and data.
#[derive(Debug)]
pub struct RTMPChunk {
    pub basic_header: ChunkBasicHeader,
    pub message_header: ChunkMessageHeader,
    pub extended_timestamp: Option<ExtendedTimestamp>, // Optional, depends on the header
    pub data: Vec<u8>, // The chunk payload
}