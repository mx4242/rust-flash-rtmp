
pub mod tcp_transport;

use std::io::Result;

pub trait Transport: Send {
    fn connect(&mut self, ip: String, port: u16) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;

    fn read_data(&mut self, size: usize) -> Result<Vec<u8>>;
    fn write_data(&mut self, data: Vec<u8>) -> Result<()>;

    // those functions are ok for now, but we need to move them to utils later
    fn read_u8(&mut self) -> Result<u8> {
        let data = self.read_data(1)?;
        Ok(data[0])
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let data = self.read_data(4)?;
        Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
    }

    fn read_u32_be(&mut self) -> Result<u32> {
        let data = self.read_data(4)?;
        Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]))
    }

    fn read_u16_be(&mut self) -> Result<u16> {
        let data = self.read_data(2)?;
        Ok(u16::from_be_bytes([data[0], data[1]]))
    }
    
    // fn get_bytes_read(&self) -> u64;
    // fn get_bytes_sent(&self) -> u64;
}

