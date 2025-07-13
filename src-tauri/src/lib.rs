mod rpc;
mod transport;

use std::str::FromStr;
use std::sync::Mutex;

use bitcoin::absolute::LockTime;
use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin::consensus::serialize;
use bitcoin::hashes::Hash;
use bitcoin::sighash::SighashCache;
use bitcoin::{
    Address, Amount, CompressedPublicKey, EcdsaSighashType, Network, OutPoint, ScriptBuf, Txid,
    Witness,
};
use serialport;
use tauri::{Manager, State};

use crate::{rpc::RPC, transport::Transport};

#[derive(Default)]
struct AppState {
    rpc: Option<RPC>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct DeviceInfo {
    port: String,
    vid: u16,
    pid: u16,
    manufacturer: Option<String>,
    product: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UTXO {
    txid: String,
    vout: u32,
    value: u64,
}

#[tauri::command]
fn scan_devices() -> Result<Vec<DeviceInfo>, String> {
    let ports = serialport::available_ports().map_err(|e| e.to_string())?;

    let devices: Vec<DeviceInfo> = ports
        .into_iter()
        .filter_map(|p| {
            if let serialport::SerialPortType::UsbPort(info) = p.port_type {
                if info.vid == 0x2E8A && info.pid == 0x10D8 {
                    return Some(DeviceInfo {
                        port: p.port_name,
                        vid: info.vid,
                        pid: info.pid,
                        manufacturer: info.manufacturer,
                        product: info.product,
                    });
                }
            }
            None
        })
        .collect();

    if devices.is_empty() {
        Err("no compatible devices found".to_string())
    } else {
        Ok(devices)
    }
}

#[tauri::command]
fn connect(state: State<'_, Mutex<AppState>>, port: String) -> Result<bool, String> {
    let mut state = state.lock().unwrap();

    if port.is_empty() {
        return Err("port name cannot be empty".to_string());
    }

    if state.rpc.is_some() {
        return Err("already connected to a device".to_string());
    }

    match Transport::new(&port, 115200, 10) {
        Ok(transport) => {
            state.rpc = Some(RPC { transport });
            Ok(true)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn disconnect(state: State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let mut state = state.lock().unwrap();
    if state.rpc.is_some() {
        state.rpc = None;
        Ok(true)
    } else {
        Err("not connected to a device".to_string())
    }
}

#[tauri::command]
fn reset_wallet(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();

    let rpc = match state.rpc.as_mut() {
        Some(rpc) => rpc,
        None => return Err("not connected to a device".to_string()),
    };

    let command = rpc::commands::ResetWalletCommand {};

    rpc.send_command(&command).map_err(|e| e.to_string())
}

#[tauri::command]
fn initialize_wallet(state: State<'_, Mutex<AppState>>) -> Result<Vec<String>, String> {
    let mut state = state.lock().unwrap();

    let rpc = match state.rpc.as_mut() {
        Some(rpc) => rpc,
        None => return Err("not connected to a device".to_string()),
    };

    let command = rpc::commands::InitiliazeWalletCommand {};

    rpc.send_command(&command).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_wallet_status(state: State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let mut state = state.lock().unwrap();

    let rpc = match state.rpc.as_mut() {
        Some(rpc) => rpc,
        None => return Err("not connected to a device".to_string()),
    };

    let command = rpc::commands::GetWalletStatusCommand {};

    rpc.send_command(&command).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_address(state: State<'_, Mutex<AppState>>, index: u8) -> Result<String, String> {
    let mut state = state.lock().unwrap();

    let rpc = match state.rpc.as_mut() {
        Some(rpc) => rpc,
        None => return Err("not connected to a device".to_string()),
    };

    let command = rpc::commands::GetAddressCommand { index };

    rpc.send_command(&command).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_transaction(
    state: State<'_, Mutex<AppState>>,
    address: String,
    address_index: u8,
    recipient_address: String,
    utxos: Vec<UTXO>,
    amount: u64,
    fee: u64,
) -> Result<String, String> {
    let mut state = state.lock().unwrap();

    let rpc = match state.rpc.as_mut() {
        Some(rpc) => rpc,
        None => return Err("not connected to a device".to_string()),
    };

    let command = rpc::commands::GetPublicKeyCommand {
        index: address_index,
    };

    let sender_address = address;
    let sender_address_public_key = rpc.send_command(&command).map_err(|e| e.to_string())?;
    let sender_address_public_key =
        CompressedPublicKey::from_str(&sender_address_public_key).map_err(|e| e.to_string())?;

    let total_utxo_value: u64 = utxos.iter().map(|utxo| utxo.value).sum();

    let inputs = utxos
        .iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: Txid::from_str(&utxo.txid).unwrap(),
                vout: utxo.vout,
            },
            script_sig: ScriptBuf::new(),
            sequence: bitcoin::Sequence(0xFFFFFFFF),
            witness: Witness::new(),
        })
        .collect::<Vec<_>>();

    let outputs = vec![
        TxOut {
            value: Amount::from_sat(amount),
            script_pubkey: Address::script_pubkey(
                &recipient_address
                    .parse::<Address<_>>()
                    .unwrap()
                    .require_network(Network::Testnet4)
                    .unwrap(),
            ),
        },
        TxOut {
            value: Amount::from_sat(total_utxo_value - amount - fee),
            script_pubkey: Address::script_pubkey(
                &sender_address
                    .parse::<Address<_>>()
                    .unwrap()
                    .require_network(Network::Testnet4)
                    .unwrap(),
            ),
        },
    ];

    let mut tx = Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: LockTime::ZERO,
        input: inputs,
        output: outputs,
    };

    let p2wpkh_script_pubkey =
        Address::p2wpkh(&sender_address_public_key, Network::Testnet4).script_pubkey();

    let mut sighashes = Vec::new();

    {
        let mut sighash_cache = SighashCache::new(&tx);
        for (i, utxo) in utxos.iter().enumerate() {
            let utxo_amount = Amount::from_sat(utxo.value);

            let sighash = sighash_cache
                .p2wpkh_signature_hash(i, &p2wpkh_script_pubkey, utxo_amount, EcdsaSighashType::All)
                .unwrap();

            sighashes.push(sighash.to_byte_array());
        }
    }

    for (i, sighash_bytes) in sighashes.into_iter().enumerate() {
        let command = rpc::commands::SignTransactionCommand {
            index: address_index,
            preimage_hash: sighash_bytes.to_vec(),
        };

        let signature = rpc.send_command(&command).map_err(|e| e.to_string())?;
        tx.input[i].witness = Witness::from(vec![
            signature,
            sender_address_public_key.to_bytes().to_vec(),
        ]);
    }

    Ok(serialize(&tx)
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_devices,
            connect,
            disconnect,
            initialize_wallet,
            reset_wallet,
            get_wallet_status,
            get_address,
            create_transaction
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
