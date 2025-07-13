import { invoke } from "@tauri-apps/api/core";

import type { Device } from "@app/types";

const scanDevices = (): Promise<Device[]> => invoke("scan_devices");

const connect = (port: string): Promise<boolean> => invoke("connect", { port });
const disconnect = (): Promise<boolean> => invoke("disconnect");

const initializeWallet = (): Promise<string[]> => invoke("initialize_wallet");
const resetWallet = (): Promise<void> => invoke("reset_wallet");

const getWalletStatus = (): Promise<boolean> => invoke("get_wallet_status");
const getAddress = (index: number): Promise<string> => invoke("get_address", { index });

export { scanDevices, connect, disconnect, initializeWallet, resetWallet, getWalletStatus, getAddress };
