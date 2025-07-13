pub mod commands;
mod constants;
mod response;

use crate::rpc::commands::Command;
use crate::rpc::constants::{
    RPC_CMD_SIZE, RPC_HEADER_SIZE, RPC_MAX_DATA_LEN, RPC_MAX_RX_PACKET_SIZE, RPC_MAX_TX_PACKET_SIZE,
};
use crate::rpc::response::{Response, ResponseError};
use crate::transport::{Transport, TransportError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("transport error: {0}")]
    Transport(TransportError),
    #[error("response error: {0}")]
    Response(ResponseError),
    #[error("command error: {0}")]
    Command(Box<dyn std::error::Error + Send + Sync>),

    #[error("data too long: {0} bytes")]
    TooMuchData(usize),
}

pub struct RPC {
    pub transport: Transport,
}

impl RPC {
    fn build_packet(cmd: u8, data: &[u8]) -> Result<Vec<u8>, RpcError> {
        if data.len() > RPC_MAX_DATA_LEN {
            return Err(RpcError::TooMuchData(data.len()));
        }

        let length = data.len() as u16;
        let mut packet = Vec::with_capacity(RPC_HEADER_SIZE + RPC_CMD_SIZE + data.len());

        packet.push((length >> 8) as u8); // MSB
        packet.push((length & 0xFF) as u8); // LSB
        packet.push(cmd);

        packet.extend_from_slice(data);

        if packet.len() > RPC_MAX_RX_PACKET_SIZE {
            return Err(RpcError::TooMuchData(packet.len()));
        }

        Ok(packet)
    }

    pub fn send_command<C: Command>(&mut self, cmd: &C) -> Result<C::Output, RpcError> {
        let data = cmd.encode();
        let packet = RPC::build_packet(cmd.cmd_id(), &data)?;

        let mut buffer = [0u8; RPC_MAX_TX_PACKET_SIZE];
        let bytes_read = self
            .transport
            .send(&packet, &mut buffer)
            .map_err(RpcError::Transport)?;

        let response = &buffer[..bytes_read];
        let parsed = Response::from_bytes(response).map_err(RpcError::Response)?;

        let decoded = cmd
            .decode(parsed)
            .map_err(|e| RpcError::Command(Box::new(e)))?;

        Ok(decoded)
    }
}
