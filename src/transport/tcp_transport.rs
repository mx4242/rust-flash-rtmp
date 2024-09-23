use crate::transport::Transport;
use std::{
    io::{self, ErrorKind, Write, Read}, net::{
        IpAddr, Shutdown, SocketAddr, TcpStream
    }, time::Duration
};

#[derive(Debug)]
pub struct TcpTransport {
    stream: Option<TcpStream>,

    written_bytes: u32,
    received_bytes: u32,
}

impl TcpTransport {
    pub fn new() -> Self {
        TcpTransport {
            stream: None,

            written_bytes: 0,
            received_bytes: 0
        }
    }
}

impl Transport for TcpTransport {
    fn connect(&mut self, ip: String, port: u16) -> std::io::Result<()> {
        let ip_addr: IpAddr = ip.parse().map_err(|_| {
            io::Error::new(ErrorKind::InvalidInput, "Failed to parse IP address")
        })?;

        // Create a socket addr object
        let socket_addr = SocketAddr::new(ip_addr, port);

        // Attempt to connect with a timeout of 10s
        // TODO: maybe make this configurable?

        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(10)) {
            Ok(stream) => {
                self.stream = Some(stream);
                Ok(())
            },
            Err(e) => Err(e),
        }
    }

    fn disconnect(&mut self) -> std::io::Result<()> {
        if let Some(stream) = self.stream.take() {
            stream.shutdown(Shutdown::Both)?;
            Ok(())
        } else {
            Err(io::Error::new(ErrorKind::BrokenPipe, "Stream hasn't been opened yet or was already closed."))
        }
    }

    fn read_data(&mut self, size: usize) -> std::io::Result<Vec<u8>> {
        if let Some(ref mut stream) = self.stream {
            let mut buffer = vec![0; size];
            let mut read_bytes = 0;

            while read_bytes < size {
                match stream.read(&mut buffer[read_bytes..]) {
                    Ok(bytes) => {
                        read_bytes += bytes;
                    },
                    Err(e) => return Err(e),
                }
            }

            self.received_bytes += read_bytes as u32;

            Ok(buffer)
        } else {
            Err(io::Error::new(ErrorKind::BrokenPipe, "Stream hasn't been opened yet or was closed."))
        }
    }

    fn write_data(&mut self, data: Vec<u8>) -> std::io::Result<()> {
        if let Some(ref mut stream) = self.stream {
            return match stream.write_all(data.as_slice()) {
                Ok(_) => {
                    self.written_bytes += data.len() as u32;

                    Ok(())
                },
                Err(e) => Err(e),
            }
        }

        Err(io::Error::new(ErrorKind::BrokenPipe, "Stream hasn't been opened yet or was closed."))
    }
}