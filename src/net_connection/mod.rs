pub mod transaction_manager;
pub mod user_control_messages;

pub mod packets;
pub mod reader;
pub mod writer;

use crate::context::{
    allocate_net_connection_context, ConnectionArgs, NetConnectionContext, ObjectEncoding,
};
use crate::handshake::RTMPHandshake;
use crate::net_connection::packets::{AMFCommandMessage, RTMPMessageType};
use crate::net_connection::transaction_manager::TransactionResult;
use crate::shared_object::SharedObject;
use crate::transport::Transport;
use crate::utils::url::parse_tc_url;

use flash_lso::types::{Element, Value};
use packets::{SetChunkSize, SetPeerBandwidth, UserControlMessage, WindowAcknowledgementSize};
use reader::RTMPReader;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use writer::RTMPWriter;

pub struct NetConnection<T: Transport> {
    pub(crate) context: NetConnectionContext<T>,
}

impl<T: Transport> NetConnection<T> {
    pub fn new(transport: T) -> Self {
        NetConnection {
            context: allocate_net_connection_context(transport),
        }
    }

    pub(crate) fn get_context(&mut self) -> &mut NetConnectionContext<T> {
        &mut self.context
    }

    fn send_connect_request<F: Fn(Value, &[Value]) + 'static>(
        &mut self,
        callback: F,
    ) -> std::io::Result<()>
    where
        F: Fn(Value, &[Value]) -> (),
    {
        let connection_args = self.context.connection_args.as_ref().unwrap();

        let transaction_id = self
            .context
            .transaction_manager
            .initialize_transaction(Box::new(callback));

        let command = RTMPMessageType::AMF0Command(AMFCommandMessage {
            procedure_name: "connect".to_string(),
            transaction_id: transaction_id,
            command_object: Some(Value::Object(
                vec![
                    Element {
                        name: String::from("videoCodecs"),
                        value: Rc::new(Value::Number(connection_args.video_codecs.clone() as f64)),
                    },
                    Element {
                        name: String::from("audioCodecs"),
                        value: Rc::new(Value::Number(connection_args.audio_codecs.clone() as f64)),
                    },
                    Element {
                        name: String::from("flashVer"),
                        value: Rc::new(Value::String(connection_args.flash_ver.clone())),
                    },
                    Element {
                        name: String::from("app"),
                        value: Rc::new(Value::String(connection_args.app.clone())),
                    },
                    Element {
                        name: String::from("tcUrl"),
                        value: Rc::new(Value::String(connection_args.tc_url.clone())),
                    },
                    Element {
                        name: String::from("videoFunction"),
                        value: Rc::new(
                            Value::Number(connection_args.video_function.clone() as f64),
                        ),
                    },
                    Element {
                        name: String::from("capabilities"),
                        value: Rc::new(Value::Number(239.0)),
                    },
                    Element {
                        name: String::from("pageUrl"),
                        value: Rc::new(Value::String(connection_args.page_url.clone())),
                    },
                    Element {
                        name: String::from("fpad"),
                        value: Rc::new(Value::Bool(connection_args.fpad.clone())),
                    },
                    Element {
                        name: String::from("swfUrl"),
                        value: Rc::new(Value::String(connection_args.swf_url.clone())),
                    },
                    Element {
                        name: String::from("objectEncoding"),
                        value: Rc::new(Value::Number(
                            connection_args.object_encoding.clone() as i32 as f64,
                        )),
                    },
                ],
                None,
            )),
            optional_arguments: connection_args.additional_args.clone(),
        });

        RTMPWriter::write(command, &mut self.context)?;

        Ok(())
    }

    pub fn connect<F: Fn(Value, &[Value]) + 'static>(
        &mut self,
        tc_url: &str,
        callback: F,
    ) -> std::io::Result<()>
    where
        F: Fn(Value, &[Value]) -> (),
    {
        let tc_url = parse_tc_url(tc_url)?;

        assert_eq!(
            tc_url.protocol, "rtmp",
            "Currently only base RTMP flavor is supported"
        );

        let connection_args = ConnectionArgs {
            app: tc_url.app,
            flash_ver: "WIN 32,0,0,465".to_string(),
            swf_url: "".to_string(),
            tc_url: tc_url.full_url,
            fpad: false,
            audio_codecs: 3575,
            video_codecs: 252,
            video_function: 1,
            page_url: "".to_string(),
            object_encoding: ObjectEncoding::AMF0,
            additional_args: vec![],
        };

        self.context.connection_args = Some(connection_args);

        self.context.transport.connect(tc_url.host, tc_url.port)?;
        RTMPHandshake::new().do_handshake(&mut self.context)?;

        self.send_connect_request(callback)?;

        Ok(())
    }

    pub(crate) fn send_shared_object(&mut self, name: String) -> std::io::Result<()> {
        let shared_object_arc = self.context.get_shared_object(&name)?;

        RTMPWriter::write(
            RTMPMessageType::AMF3SharedObject(shared_object_arc),
            &mut self.context,
        )?;

        Ok(())
    }

    fn process_window_ack_size(&mut self, window_ack_size: WindowAcknowledgementSize) {
        self.context.window_ack_size = Some(window_ack_size.size);
    }

    fn process_set_chunk_size(&mut self, set_chunk_size: SetChunkSize) {
        self.context.chunk_size = set_chunk_size.size;
    }

    fn process_set_peer_bandwidth(&mut self, _peer_bandwidth: SetPeerBandwidth) {
        // println!("not how to implement");
    }

    fn process_user_control_message(&mut self, user_control_message: UserControlMessage) {
        // println!("not how to implement");
        match user_control_message {
            UserControlMessage::PingRequest { timestamp } => {
                let response = UserControlMessage::PingResponse { timestamp };
                RTMPWriter::write(
                    RTMPMessageType::UserControlMessage(response),
                    &mut self.context,
                )
                .unwrap();
            }

            _ => todo!(
                "user control message not implemented, {:?}",
                user_control_message
            ),
        }
    }

    fn process_amf0_command(&mut self, command: AMFCommandMessage) {
        if command.procedure_name == "_result" || command.procedure_name == "_error" {
            let result = if command.procedure_name == "_result" {
                TransactionResult::Result
            } else {
                TransactionResult::Error
            };

            return self
                .context
                .transaction_manager
                .finalize_transaction(command.transaction_id, result, command)
                .unwrap();
        }

        todo!(
            "server tried to call a client command, procedure name: {}",
            command.procedure_name
        );
    }

    fn process_shared_object(&mut self, shared_object: Arc<Mutex<SharedObject>>) {
        let mut shared_object = shared_object.lock().unwrap();
        shared_object.process_events();
    }

    pub fn process_messages<'b>(&mut self) -> std::io::Result<()> {
        let rtmp_message;

        rtmp_message = RTMPReader::read(&mut self.context)?;

        match rtmp_message {
            RTMPMessageType::SetChunkSize(set_chunk_size) => self.process_set_chunk_size(set_chunk_size),
            RTMPMessageType::WindowAcknowledgementSize(window_ack_size) => self.process_window_ack_size(window_ack_size),
            RTMPMessageType::SetPeerBandwidth(peer_bandwidth) => self.process_set_peer_bandwidth(peer_bandwidth),
            RTMPMessageType::UserControlMessage(user_control_message) => self.process_user_control_message(user_control_message),
            RTMPMessageType::AMF0Command(command) => self.process_amf0_command(command),
            RTMPMessageType::AMF3SharedObject(shared_object) => self.process_shared_object(shared_object),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::tcp_transport::TcpTransport;

    #[test]
    fn test_connect() {
        let transport: TcpTransport = TcpTransport::new();
        let mut connection = NetConnection::new(transport);

        let result = connection.connect(
            "rtmp://127.0.0.1/",
            |properties, information| {
                println!("properties: {:?}", properties);
                println!("information: {:?}", information);
            },
        );

        assert!(result.is_ok());

        loop {
            connection.process_messages().unwrap();
        }
    }
}
