use super::constants::{
    RPC_HEADER_SIZE, RPC_MAX_DATA_LEN, RPC_MAX_TX_PACKET_SIZE, RPC_STATUS_SIZE,
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("response data is too long")]
    TooLong,
    #[error("response data is too short")]
    TooShort,
    #[error("response data has invalid length")]
    InvalidLength,
    #[error("response data is incomplete")]
    IncompleteData,
}

pub struct Response {
    pub data: Vec<u8>,
    pub status: u16,
}

impl Response {
    pub fn from_bytes(resp: &[u8]) -> Result<Self, ResponseError> {
        if resp.len() > RPC_MAX_TX_PACKET_SIZE {
            return Err(ResponseError::TooLong);
        }

        if resp.len() < RPC_HEADER_SIZE + RPC_STATUS_SIZE {
            return Err(ResponseError::TooShort);
        }

        let length = u16::from_be_bytes([resp[0], resp[1]]) as usize;
        if length > RPC_MAX_DATA_LEN {
            return Err(ResponseError::InvalidLength);
        }

        let sw1 = resp[2];
        let sw2 = resp[3];
        let status = ((sw1 as u16) << 8) | (sw2 as u16);

        let data_start = 4;
        let data_end = data_start + length;

        if data_end > resp.len() {
            return Err(ResponseError::IncompleteData);
        }

        let data = resp[data_start..data_end].to_vec();

        Ok(Response { data, status })
    }
}
