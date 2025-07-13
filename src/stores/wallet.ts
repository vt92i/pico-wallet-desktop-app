import { writable } from "svelte/store";

import {
  connect,
  disconnect,
  getAddress,
  getAddressTransactions,
  getAddressUtxos,
  getWalletStatus,
  initializeWallet,
  resetWallet,
} from "@app/api";
import type { Wallet } from "@app/types";

const initialState: Wallet = {
  isConnected: false,
  isInitialized: false,
  addresses: [],
  currentAddress: "",
};

const createWalletStore = () => {
  const { subscribe, set, update } = writable<Wallet>(initialState);

  const _connect = async (port: string) => {
    try {
      await connect(port);
      update((state) => ({ ...state, isConnected: true }));
    } catch (error) {
      update((state) => ({ ...state, isConnected: false }));
      throw error;
    }
  };

  const _disconnect = async () => {
    await disconnect();
    set(initialState);
  };

  const _initialize = async () => {
    try {
      const mnemonic = (await initializeWallet())[0].split(" ");

      update((state) => ({
        ...state,
        isInitialized: true,
      }));

      return mnemonic;
    } catch (error) {
      update((state) => ({ ...state, isInitialized: false }));
      throw error;
    }
  };

  const _reset = async () => {
    await resetWallet();
    update((state) => ({
      ...state,
      isInitialized: false,
      addresses: [],
      currentAddress: "",
    }));
  };

  const _getStatus = async () => {
    try {
      const status = await getWalletStatus();
      update((state) => ({ ...state, isInitialized: status }));
    } catch (error) {
      update((state) => ({ ...state, isInitialized: false }));
      throw error;
    }
  };

  const _getAddresses = async () => {
    const addresses = await Promise.all(Array.from({ length: 10 }, (_, i) => getAddress(i)));
    update((state) => ({
      ...state,
      addresses: addresses,
    }));
  };

  const _getBalance = async (address: string) => {
    const utxos = await getAddressUtxos(address);
    const confirmedUtxos = utxos.filter((utxo) => utxo.status.confirmed);

    const balance = confirmedUtxos.reduce((acc, utxo) => acc + utxo.value, 0);

    return balance;
  };

  const _getTransactions = async (address: string) => {
    const transactions = await getAddressTransactions(address);
    return transactions;
  };

  const _getUtxos = async (address: string) => {
    const utxos = await getAddressUtxos(address);
    return utxos;
  };

  const _setCurrentAddress = (address: string) => {
    update((state) => ({
      ...state,
      currentAddress: address,
    }));
  };

  return {
    subscribe,
    connect: _connect,
    disconnect: _disconnect,
    initialize: _initialize,
    reset: _reset,
    getStatus: _getStatus,
    getAddresses: _getAddresses,
    getBalance: _getBalance,
    getTransactions: _getTransactions,
    getUtxos: _getUtxos,
    setCurrentAddress: _setCurrentAddress,
  };
};

export const walletStore = createWalletStore();
