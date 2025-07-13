import axios from "axios";

import type { Transaction, UTXO } from "@app/types";

const BASE_URL = "https://api.mempool.space/testnet4/api";

const broadcastTransaction = async (tx_hex: string): Promise<string> => {
  const response = await axios.post(`${BASE_URL}/tx`, tx_hex, {
    headers: {
      "Content-Type": "text/plain",
    },
  });

  if (response.status !== 200) {
    throw new Error("Failed to broadcast transaction");
  }

  return response.data as string;
};

const getAddressUtxos = async (address: string): Promise<UTXO[]> => {
  const response = await axios.get<UTXO[]>(`${BASE_URL}/address/${address}/utxo`);
  return response.data;
};

const getAddressTransactions = async (address: string): Promise<Transaction[]> => {
  const response = await axios.get<Transaction[]>(`${BASE_URL}/address/${address}/txs`);
  return response.data;
};

export { broadcastTransaction, getAddressUtxos, getAddressTransactions };
