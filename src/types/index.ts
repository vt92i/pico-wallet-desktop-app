interface Device {
  port: string;
  vid: number;
  pid: number;
  manufacturer: string;
  product: string;
}

interface Wallet {
  isConnected: boolean;
  isInitialized: boolean;
  addresses: string[];
  currentAddress: string;
}

interface UTXO {
  txid: string;
  vout: number;
  status: {
    confirmed: boolean;
    block_height: number;
    block_hash: string;
    block_time: number;
  };
  value: number;
}

interface Transaction {
  txid: string;
  vin: {
    prevout: {
      scriptpubkey_address: string;
      value: number;
    };
  }[];
  vout: {
    scriptpubkey_address: string;
    value: number;
  }[];
  status: {
    confirmed: boolean;
    block_height: number;
    block_hash: string;
    block_time: number;
  };
}

export type { Device, Wallet, UTXO, Transaction };
