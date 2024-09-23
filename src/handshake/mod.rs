pub mod packets;
pub mod errors;

pub const RTMP_PROTOCOL_VERSION: u8 = 3;
pub const RANDOM_ECHO_SIZE: usize = 1528;

use crate::{
    context::NetConnectionContext,
    transport::Transport,
    handshake::packets::{C1S1Packet, C2S2Packet, ClientAckAndConnect, ClientHello, ServerHelloAck}
};

struct SentReceivedPackets {
    client_hello: Option<ClientHello>,
    server_hello_ack: Option<ServerHelloAck>,
}

impl Default for SentReceivedPackets {
    fn default() -> Self {
        SentReceivedPackets {
            client_hello: None,
            server_hello_ack: None,
        }
    }
}

pub struct RTMPHandshake {
    packets: SentReceivedPackets,
}

impl RTMPHandshake {
    pub fn new() -> Self {
        RTMPHandshake {
            packets: SentReceivedPackets::default(),
        }
    }

    fn send_client_hello<T: Transport>(&mut self, context: &mut NetConnectionContext<T>) -> std::io::Result<()> {
        let client_hello = self.create_client_hello();

        context.transport.write_data(client_hello.to_bytes())?;

        self.packets.client_hello = Some(client_hello.clone());

        Ok(())
    }
    fn handle_server_hello_ack<T: Transport>(&mut self, context: &mut NetConnectionContext<T>) -> std::io::Result<()> {
        let payload = context.transport.read_data(3073)?;

        let (_, server_hello_ack) = ServerHelloAck::from_bytes(&payload)
            .expect("Failed to parse ServerHelloAck");

        self.packets.server_hello_ack = Some(server_hello_ack.clone());

        if server_hello_ack.s0.version != RTMP_PROTOCOL_VERSION {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "RTMP version mismatch",
            ));
        }

        let c1_random_data = self.packets.client_hello.as_ref().unwrap().c1.random_data;
        let s2_random_data = server_hello_ack.s2.random_echo;

        if c1_random_data != s2_random_data {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Echo mismatch",
            ));
        }

        let client_ack_and_connect = self.create_client_ack_and_connect(server_hello_ack.s1);

        context.transport.write_data(client_ack_and_connect.to_bytes())
    }

    fn create_client_hello(&self) -> ClientHello {
        let handshake_bytes: Vec<u8> = (0..RANDOM_ECHO_SIZE).map(|_| b'x').collect();

        ClientHello::new(
            RTMP_PROTOCOL_VERSION,
            0,
            handshake_bytes.try_into().expect("Failed to convert handshake bytes")
        )
    }

    fn create_client_ack_and_connect(&self, s1: C1S1Packet) -> ClientAckAndConnect {
        ClientAckAndConnect::new(C2S2Packet {
            time: s1.time,
            time2: 0,
            random_echo: s1.random_data,
        })
    }

    pub fn do_handshake<T: Transport>(&mut self, context: &mut NetConnectionContext<T>) -> std::io::Result<()> {
        self.send_client_hello(context)?;

        self.handle_server_hello_ack(context)?;

        Ok(())
    }
}