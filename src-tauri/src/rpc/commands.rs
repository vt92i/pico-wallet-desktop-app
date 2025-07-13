use crate::rpc::response::Response;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("execution failed")]
    ExecutionError,
}

pub trait Command {
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;

    fn cmd_id(&self) -> u8;
    fn encode(&self) -> Vec<u8>;
    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error>;
}

pub struct PingCommand {}

impl Command for PingCommand {
    type Output = ();
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xFF
    }

    fn encode(&self) -> Vec<u8> {
        vec![]
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        Ok(())
    }
}

pub struct HashMD5Command {
    pub data: Vec<u8>,
}

impl Command for HashMD5Command {
    type Output = String;
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xF4
    }

    fn encode(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        // if response.data.len() != 16 {
        //     return Err(ParseError::InvalidLength);
        // }

        Ok(response.data.iter().map(|b| format!("{:02x}", b)).collect())
    }
}

pub struct HashSHA256Command {
    pub data: Vec<u8>,
}

impl Command for HashSHA256Command {
    type Output = String;
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xF5
    }

    fn encode(&self) -> Vec<u8> {
        self.data.clone()
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        // if response.data.len() != 32 {
        //     return Err(ParseError::InvalidLength);
        // }

        Ok(response.data.iter().map(|b| format!("{:02x}", b)).collect())
    }
}

pub struct InitiliazeWalletCommand {}

impl Command for InitiliazeWalletCommand {
    type Output = Vec<String>;
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xA0
    }

    fn encode(&self) -> Vec<u8> {
        vec![]
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        Ok(response
            .data
            .split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .collect())
    }
}

pub struct ResetWalletCommand {}

impl Command for ResetWalletCommand {
    type Output = ();
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xA1
    }

    fn encode(&self) -> Vec<u8> {
        vec![]
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        Ok(())
    }
}

pub struct GetWalletStatusCommand {}

impl Command for GetWalletStatusCommand {
    type Output = bool;
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xA2
    }

    fn encode(&self) -> Vec<u8> {
        vec![]
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        Ok(response.data.get(0).map_or(false, |&b| b != 0))
    }
}

pub struct GetAddressCommand {
    pub index: u8,
}

impl Command for GetAddressCommand {
    type Output = String;
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xA3
    }

    fn encode(&self) -> Vec<u8> {
        vec![self.index]
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        Ok(response
            .data
            .split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .next()
            .unwrap_or_default())
    }
}

pub struct GetPublicKeyCommand {
    pub index: u8,
}

impl Command for GetPublicKeyCommand {
    type Output = String;
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xA4
    }

    fn encode(&self) -> Vec<u8> {
        vec![self.index]
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        Ok(response.data.iter().map(|b| format!("{:02x}", b)).collect())
    }
}

pub struct SignTransactionCommand {
    pub index: u8,
    pub preimage_hash: Vec<u8>,
}

impl Command for SignTransactionCommand {
    type Output = Vec<u8>;
    type Error = CommandError;

    fn cmd_id(&self) -> u8 {
        0xA5
    }

    fn encode(&self) -> Vec<u8> {
        std::iter::once(self.index)
            .chain(self.preimage_hash.clone())
            .collect()
    }

    fn decode(&self, response: Response) -> Result<Self::Output, Self::Error> {
        if response.status != 0x9000 {
            return Err(CommandError::ExecutionError);
        }

        Ok(response.data.clone())
    }
}
