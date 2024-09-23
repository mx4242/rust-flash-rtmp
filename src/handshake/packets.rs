use std::num::NonZero;

use crate::handshake::{RTMP_PROTOCOL_VERSION, RANDOM_ECHO_SIZE};
use crate::utils::nom::RTMPResult;
use nom::number::complete::{be_u8, be_u32};
use nom::bytes::complete::take;

// C0 and S0 Packet (1 byte)
#[derive(Debug, Clone, Copy)]
pub struct Version {
    /// In C0, this field identifies the RTMP version
    /// requested by the client. In S0, this field identifies the RTMP
    /// version selected by the server. The version defined by this
    /// specification is 3. Values 0-2 are deprecated values used by
    /// earlier proprietary products; 4-31 are reserved for future
    /// implementations; and 32-255 are not allowed (to allow
    /// distinguishing RTMP from text-based protocols, which always start
    /// with a printable character). A server that does not recognize the
    /// client’s requested version SHOULD respond with 3. The client MAY
    /// choose to degrade to version 3, or to abandon the handshake.
    pub version: u8,
}

impl Default for Version {
    fn default() -> Self {
        Version {
            version: RTMP_PROTOCOL_VERSION,
        }
    }
}

impl Version {
    pub fn new(version: u8) -> Self {
        Version { version }
    }

    pub fn is_valid(&self) -> bool {
        self.version == RTMP_PROTOCOL_VERSION
    }

    pub fn to_bytes(&self) -> [u8; 1] {
        [self.version]
    }

    pub fn from_bytes(bytes: &[u8]) -> RTMPResult<'_, Self> {
        if bytes.len() < 1 {
            return Err(nom::Err::Incomplete(nom::Needed::Size(NonZero::new(1).unwrap())));
        }

        let (i, version) = be_u8(bytes)?;
        Ok((i, Version { version }))
    }
}

// C1S1 Packet (1536 bytes)
#[derive(Debug, Clone, Copy)]
pub struct C1S1Packet {
    /// This field contains a timestamp, which SHOULD be
    /// used as the epoch for all future chunks sent from this endpoint.
    /// This may be 0, or some arbitrary value. To synchronize multiple
    /// chunkstreams, the endpoint may wish to send the current value of
    /// the other chunkstream’s timestamp.
    pub time: u32,

    /// Original RTMP Documentation: This field MUST be all 0s.
    /// Since FP 9: Should contain the Client/Server version
    pub version: [u8; 4],

    /// This field can contain any arbitrary
    /// values. Since each endpoint has to distinguish between the
    /// response to the handshake it has initiated and the handshake
    /// initiated by its peer, this data SHOULD send something sufficiently
    /// random. But there is no need for cryptographically-secure
    /// randomness, or even dynamic values.
    pub random_data: [u8; RANDOM_ECHO_SIZE],
}


impl Default for C1S1Packet {
    fn default() -> Self {
        C1S1Packet {
            time: 0,
            version: [0, 0, 0, 0],
            random_data: [0; RANDOM_ECHO_SIZE],
        }
    }
}

impl C1S1Packet {
    pub fn new(time: u32, random_data: [u8; RANDOM_ECHO_SIZE]) -> Self {
        C1S1Packet {
            time,
            version: [0, 0, 0, 0],
            random_data,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1536);
        bytes.extend_from_slice(&self.time.to_be_bytes());
        bytes.extend_from_slice(&self.version);
        bytes.extend_from_slice(&self.random_data);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> RTMPResult<'_, Self> {
        if bytes.len() < 1536 {
            return Err(nom::Err::Incomplete(nom::Needed::Size(NonZero::new(1536).unwrap())));
        }

        let (i, time) = be_u32(bytes)?;
        let (i, version) = take(4usize)(i)?;
        let (i, random_data) = take(RANDOM_ECHO_SIZE)(i)?;

        Ok((i, C1S1Packet { time, version: version.try_into().unwrap(), random_data: random_data.try_into().unwrap() }))
    }
}

// C2 and S2 Packet (1536 bytes)
#[derive(Debug, Clone, Copy)]
pub struct C2S2Packet {
    /// This field MUST contain the timestamp sent by the peer in S1 (for C2) or C1 (for S2).
    pub time: u32,

    /// This field MUST contain the timestamp at which the previous packet (S1 or C1) sent by the peer was read.
    pub time2: u32,

    /// This field MUST contain the random data
    /// field sent by the peer in S1 (for C2) or S2 (for C1). Either peer
    /// can use the time and time2 fields together with the current
    /// timestamp as a quick estimate of the bandwidth and/or latency of
    /// the connection, but this is unlikely to be useful.
    pub random_echo: [u8; RANDOM_ECHO_SIZE],
}

impl Default for C2S2Packet {
    fn default() -> Self {
        C2S2Packet {
            time: 0,
            time2: 0,
            random_echo: [0; RANDOM_ECHO_SIZE],
        }
    }
}

impl C2S2Packet {
    pub fn new(time: u32, time2: u32, random_echo: [u8; RANDOM_ECHO_SIZE]) -> Self {
        C2S2Packet {
            time,
            time2,
            random_echo,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1536);
        bytes.extend_from_slice(&self.time.to_be_bytes());
        bytes.extend_from_slice(&self.time2.to_be_bytes());
        bytes.extend_from_slice(&self.random_echo);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> RTMPResult<'_, Self> {
        if bytes.len() < 1536 {
            return Err(nom::Err::Incomplete(nom::Needed::Size(NonZero::new(1536).unwrap())));
        }

        let (i, time) = be_u32(bytes)?;
        let (i, time2) = be_u32(i)?;
        let (i, random_data) = take(RANDOM_ECHO_SIZE)(i)?;

        Ok((i, C2S2Packet { time, time2, random_echo: random_data.try_into().unwrap() }))
    }
}

// Combined C0 and C1 Packet (Client -> Server)
#[derive(Debug, Clone, Copy)]
pub struct ClientHello {
    pub c0: Version,
    pub c1: C1S1Packet,
}

impl ClientHello {
    pub fn new(version: u8, time: u32, random_data: [u8; RANDOM_ECHO_SIZE]) -> Self {
        ClientHello {
            c0: Version::new(version),
            c1: C1S1Packet::new(time, random_data),
        }
    }

    // No need to implement `from_bytes` for this struct
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + 1536);
        bytes.extend_from_slice(&self.c0.to_bytes());
        bytes.extend_from_slice(&self.c1.to_bytes());
        bytes
    }
}

// Combined S0, S1, and S2 Packet (Server -> Client)
#[derive(Debug, Clone)]
pub struct ServerHelloAck {
    pub s0: Version,
    pub s1: C1S1Packet,
    pub s2: C2S2Packet,
}

impl ServerHelloAck {
    pub fn from_bytes(bytes: &[u8]) -> RTMPResult<'_, Self> {
        if bytes.len() < 1 + 1536 + 1536 {
            return Err(nom::Err::Incomplete(nom::Needed::Size(NonZero::new(3073).unwrap())));
        }

        let (i, s0) = Version::from_bytes(bytes)?;
        let (i, s1) = C1S1Packet::from_bytes(i)?;
        let (i, s2) = C2S2Packet::from_bytes(i)?;

        Ok((i, ServerHelloAck { s0, s1, s2 }))
    }
}

// C2 packet, after this packet a AMF connect command should occur (Client -> Server)
#[derive(Debug, Clone)]
pub struct ClientAckAndConnect {
    pub c2: C2S2Packet,
}

impl ClientAckAndConnect {
    pub fn new(c2: C2S2Packet) -> Self {
        ClientAckAndConnect {
            c2,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1536);
        bytes.extend_from_slice(&self.c2.to_bytes());
        bytes
    }
}