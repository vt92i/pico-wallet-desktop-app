pub(crate) const RPC_HEADER_SIZE: usize = 2;
pub(crate) const RPC_CMD_SIZE: usize = 1;
pub(crate) const RPC_STATUS_SIZE: usize = 2;

pub(crate) const RPC_MAX_DATA_LEN: usize = 512;
pub(crate) const RPC_MAX_RX_PACKET_SIZE: usize = RPC_HEADER_SIZE + RPC_CMD_SIZE + RPC_MAX_DATA_LEN;
pub(crate) const RPC_MAX_TX_PACKET_SIZE: usize =
    RPC_HEADER_SIZE + RPC_STATUS_SIZE + RPC_MAX_DATA_LEN;
