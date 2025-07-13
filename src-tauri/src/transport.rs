use serialport::Error as SerialPortError;
use serialport::SerialPort;
use std::io::Error as IoError;
use std::time;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("serial port error: {0}")]
    Serial(#[from] SerialPortError),

    #[error("io error: {0}")]
    Io(#[from] IoError),
}

pub struct Transport {
    port: Box<dyn SerialPort>,
}

impl Transport {
    pub fn new(port_name: &str, baud_rate: u32, timeout: u64) -> Result<Self, TransportError> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(time::Duration::from_secs(timeout))
            .open()?;

        Ok(Self { port })
    }

    pub fn send(&mut self, command: &[u8], buffer: &mut [u8]) -> Result<usize, TransportError> {
        self.port.write(command)?;

        let mut n = self.port.read(buffer)?;
        let total_len = u16::from_be_bytes([buffer[0], buffer[1]]) as usize;

        while n < total_len {
            let bytes_read = self.port.read(&mut buffer[n..])?;
            if bytes_read == 0 {
                break;
            }
            n += bytes_read;
        }

        Ok(n)
    }
}
