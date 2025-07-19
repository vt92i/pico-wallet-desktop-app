<script lang="ts">
  import { AppBar } from "@skeletonlabs/skeleton-svelte";
  import { Modal } from "@skeletonlabs/skeleton-svelte";
  import { invoke } from "@tauri-apps/api/core";
  import QRCode from "qrcode";

  import { broadcastTransaction } from "@app/api";
  import { devicesStore as devices, walletStore as wallet } from "@app/stores";
  import type { Transaction } from "@app/types";
  import { btcToSatoshi, satoshiToBtc } from "@app/utils/btc";
  import { toaster } from "@app/utils/toaster";

  let selectedPort = $state<string>("");

  let modalAddressesState = $state(false);
  let modalSendState = $state(false);
  let modalReceiveState = $state(false);
  let modalRestoreWalletState = $state(false);
  let modalInitWalletState = $state(false);

  let mnemonic = $state<string[]>([]);
  let balance = $state(0);
  let transactions = $state<Transaction[]>([]);

  let currentAddressQR = $state<string>("");

  let isUnlocked = $state(false);

  const socket = new WebSocket("wss://mempool.space/testnet4/api/v1/ws");
  socket.addEventListener("message", () => {
    void (async () => {
      try {
        if (!$wallet.isConnected || !$wallet.isInitialized) return;

        const currentAddress = $wallet.currentAddress;
        if (!currentAddress) return;

        balance = await wallet.getBalance(currentAddress);
        transactions = await wallet.getTransactions(currentAddress);
      } catch (err) {
        console.error("WebSocket message handler error:", err);
      }
    })();
  });
</script>

<div class="card border-surface-100-900 grid h-screen w-full">
  <div class="flex w-full flex-col items-center gap-4">
    {#if $wallet.isConnected}
      {#if $wallet.isInitialized}
        <AppBar>
          {#snippet lead()}
            <button class="btn btn-sm preset-filled" onclick={() => (modalAddressesState = true)}>Manage Address</button
            >
            <Modal
              backdropClasses="backdrop-blur-sm"
              closeOnInteractOutside={false}
              contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
              onOpenChange={(e) => (modalAddressesState = e.open)}
              open={modalAddressesState}
              triggerBase="btn preset-tonal"
            >
              {#snippet content()}
                <header class="flex justify-between">
                  <h2 class="text-2xl font-semibold">Manage Addresses</h2>
                </header>
                <div class="table-wrap">
                  <table class="table caption-bottom">
                    <caption class="pt-4">List of Addresses.</caption>
                    <thead>
                      <tr>
                        <th>Index</th>
                        <th>Address</th>
                        <th>Action</th>
                      </tr>
                    </thead>
                    <tbody>
                      {#if $wallet.addresses.length === 0}
                        <tr>
                          <td class="text-center" colspan="3">
                            <button
                              class="btn btn-sm preset-filled-primary-500 w-full"
                              onclick={async () => {
                                await wallet.getAddresses();
                                toaster.success({
                                  title: "Addresses Loaded",
                                  description: "Click on an address to set it as active.",
                                });
                              }}>Load Addresses</button
                            >
                          </td>
                        </tr>
                      {:else}
                        {#each $wallet.addresses as address, index (index)}
                          <tr>
                            <td>{index}</td>
                            <td>{address}</td>
                            <td>
                              <button
                                class="btn btn-sm preset-filled-primary-500"
                                disabled={address === $wallet.currentAddress}
                                onclick={async () => {
                                  modalAddressesState = false;
                                  wallet.setCurrentAddress(address);
                                  socket.send(
                                    JSON.stringify({
                                      "track-address": address,
                                    }),
                                  );
                                  toaster.success({
                                    title: "Current Address Set",
                                    description: `${address}`,
                                  });

                                  balance = await wallet.getBalance($wallet.currentAddress);
                                  transactions = await wallet.getTransactions($wallet.currentAddress);
                                  currentAddressQR = await QRCode.toDataURL(address, {
                                    errorCorrectionLevel: "M",
                                    type: "image/png",
                                    margin: 1,
                                    width: 400,
                                  });
                                }}>Set Current Address</button
                              >
                            </td>
                          </tr>
                        {/each}
                      {/if}
                    </tbody>
                  </table>
                </div>
              {/snippet}
            </Modal>
          {/snippet}
          {#snippet trail()}
            <span class="badge preset-filled-warning-500">TestNet4</span>
          {/snippet}
        </AppBar>

        <Modal
          backdropClasses="backdrop-blur-sm"
          closeOnEscape={false}
          closeOnInteractOutside={false}
          contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
          open={!isUnlocked}
          triggerBase="btn preset-tonal"
        >
          {#snippet content()}
            <header class="flex justify-between">
              <h2 class="text-2xl font-semibold">Unlock Wallet</h2>
            </header>
            <form
              class="w-full space-y-4"
              onsubmit={async (e) => {
                const formEvent = e as SubmitEvent;
                formEvent.preventDefault();

                const target = formEvent.target as HTMLFormElement;
                const formData = new FormData(target);

                const password = formData.get("password") as string;

                try {
                  await invoke("unlock_wallet", { password });
                  isUnlocked = true;

                  toaster.success({
                    title: "Wallet Unlocked",
                    description: "You can now use your wallet.",
                  });

                  await wallet.getStatus();
                } catch (error) {
                  toaster.error({
                    title: "Unlock Failed",
                    description: "Wallet is resetted, please reinitialize.",
                  });
                  // await wallet.reset();
                  await wallet.getStatus();
                }
              }}
            >
              <label class="label">
                <span class="label-text">Password</span>
                <input name="password" class="input" placeholder="Password" required type="text" />
              </label>
              <button class="btn btn-sm preset-filled-primary-500 w-full" type="submit">Unlock</button>
            </form>
          {/snippet}
        </Modal>

        <Modal
          backdropClasses="backdrop-blur-sm"
          closeOnInteractOutside={false}
          contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
          onOpenChange={(e) => (modalSendState = e.open)}
          open={modalSendState}
          triggerBase="btn preset-tonal"
        >
          {#snippet content()}
            <header class="flex justify-between">
              <h2 class="text-2xl font-semibold">Send Bitcoin</h2>
            </header>
            <form
              class="w-full space-y-4"
              onsubmit={async (e) => {
                const formEvent = e as SubmitEvent;
                formEvent.preventDefault();

                const target = formEvent.target as HTMLFormElement;
                const formData = new FormData(target);

                const addressIndex = $wallet.addresses.findIndex((address) => address === $wallet.currentAddress);

                const recipientAddress = formData.get("address") as string;
                if (recipientAddress === $wallet.currentAddress) {
                  toaster.error({
                    title: "Invalid Address",
                    description: "You cannot send Bitcoin to your own address.",
                  });
                  return;
                }

                const amount = btcToSatoshi(Number(formData.get("amount")));
                const fee = Number(formData.get("fee"));

                let utxos = await wallet.getUtxos($wallet.currentAddress);
                utxos = utxos.filter((utxo) => utxo.status.confirmed);

                if (utxos.length === 0) {
                  toaster.error({
                    title: "No UTXOs Found",
                    description: "You need to receive some Bitcoin before sending.",
                  });
                  return;
                }

                const totalBalance = utxos.reduce((acc, utxo) => acc + utxo.value, 0);
                if (totalBalance < amount + fee) {
                  toaster.error({
                    title: "Insufficient Funds",
                    description: "You do not have enough funds to cover the transaction.",
                  });
                  return;
                }

                utxos.sort((a, b) => b.value - a.value);

                const filtered_utxos = [];
                let accumulated = 0;
                for (const utxo of utxos) {
                  accumulated += utxo.value;
                  filtered_utxos.push(utxo);
                  if (accumulated >= amount + fee) break;
                }

                try {
                  const tx_hex = await invoke("create_transaction", {
                    address: $wallet.currentAddress,
                    addressIndex,
                    recipientAddress,
                    utxos: filtered_utxos.map((utxo) => ({
                      txid: utxo.txid,
                      vout: utxo.vout,
                      value: utxo.value,
                    })),
                    amount,
                    fee,
                  });

                  const txid = await broadcastTransaction(tx_hex as string);
                  toaster.success({
                    title: "Transaction Created",
                    description: `Transaction ID: ${txid}`,
                  });

                  target.reset();

                  balance = await wallet.getBalance($wallet.currentAddress);
                  transactions = await wallet.getTransactions($wallet.currentAddress);

                  modalSendState = false;
                } catch (error) {
                  toaster.error({
                    title: "Transaction Creation Failed",
                    description: error,
                  });
                  return;
                }
              }}
            >
              <label class="label">
                <span class="label-text">Recipient Address</span>
                <input name="address" class="input" placeholder="Recipient Address" required type="text" />
              </label>
              <label class="label">
                <span class="label-text">Amount (BTC)</span>
                <input
                  name="amount"
                  class="input"
                  defaultValue="0.00001"
                  min="0.00001"
                  placeholder="Amount in BTC"
                  required
                  step="0.00001"
                  type="number"
                />
              </label>
              <label class="label">
                <span class="label-text">Fee (Satoshi)</span>
                <input
                  name="fee"
                  class="input"
                  defaultValue="150"
                  min="150"
                  placeholder="Fee in Satoshi"
                  required
                  step="1"
                  type="number"
                />
              </label>
              <button class="btn btn-sm preset-filled-primary-500 w-full" type="submit">Send</button>
            </form>
          {/snippet}
        </Modal>

        <Modal
          backdropClasses="backdrop-blur-sm"
          closeOnInteractOutside={false}
          contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
          onOpenChange={(e) => (modalReceiveState = e.open)}
          open={modalReceiveState}
          triggerBase="btn preset-tonal"
        >
          {#snippet content()}
            <header class="flex justify-between">
              <h2 class="text-2xl font-semibold">Receive Bitcoin</h2>
            </header>
            <div class="flex items-center justify-center">
              <img alt={$wallet.currentAddress} src={currentAddressQR} />
            </div>
            <span class="text-center text-lg font-semibold">{$wallet.currentAddress}</span>
          {/snippet}
        </Modal>

        <Modal
          backdropClasses="backdrop-blur-sm"
          closeOnEscape={false}
          closeOnInteractOutside={false}
          contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
          open={mnemonic.length === 24}
          triggerBase="btn preset-tonal"
        >
          {#snippet content()}
            <header class="flex justify-between">
              <h2 class="text-2xl font-semibold">Mnemonic Phrase</h2>
            </header>
            <article class="">
              <p class="text-warning-500">
                Please write down your mnemonic phrase and keep it safe. It is the only way to recover your wallet.
              </p>
              <div class="mt-4">
                <textarea
                  class="textarea preset-filled-surface-100-900 h-32 w-full resize-none"
                  readonly
                  value={mnemonic.join(" ")}
                ></textarea>
              </div>
            </article>
            <footer class="flex justify-end gap-4">
              <button class="btn btn-sm preset-filled-primary-500 mt-4" onclick={() => (mnemonic = [])}>Close</button>
            </footer>
          {/snippet}
        </Modal>

        <div class="card preset-filled-surface-100-900 flex w-full max-w-5xl justify-between gap-2 p-4">
          {#if $wallet.currentAddress}
            <div class="flex w-full max-w-5xl justify-start gap-2">
              <button
                class="btn btn-sm preset-filled-primary-500"
                disabled={balance <= 0}
                onclick={() => (modalSendState = true)}>Send</button
              >
              <button class="btn btn-sm preset-filled-primary-500" onclick={() => (modalReceiveState = true)}
                >Receive</button
              >
            </div>
            <div class="flex w-full max-w-5xl justify-center gap-2">
              <span class="text-lg font-semibold">Balance: {satoshiToBtc(balance)} BTC</span>
            </div>
          {/if}
          <div class="flex w-full max-w-5xl justify-end gap-2">
            <button
              class="btn btn-sm preset-filled-error-500"
              onclick={async () => {
                try {
                  await wallet.reset();
                  isUnlocked = false;
                  toaster.success({
                    title: "Wallet Reset",
                    description: "The wallet has been reset successfully.",
                  });
                } catch (error) {
                  toaster.error({
                    title: "Reset Failed",
                    description: error,
                  });
                }
              }}>Reset Wallet</button
            >
            <button
              class="btn btn-sm preset-filled"
              onclick={async () => {
                try {
                  await wallet.disconnect();
                  isUnlocked = false;
                  toaster.success({
                    title: "Wallet Disconnected",
                    description: "The wallet has been disconnected successfully.",
                  });
                } catch (error) {
                  toaster.error({
                    title: "Disconnection Failed",
                    description: error,
                  });
                }
              }}>Disconnect</button
            >
          </div>
        </div>

        {#if $wallet.currentAddress}
          <div class="card preset-filled-surface-100-900 w-full max-w-5xl p-4 text-left">
            <div class="table-wrap">
              <table class="table caption-bottom">
                <caption class="pt-4">List of Transactions.</caption>
                <thead>
                  <tr>
                    <th>Transaction ID</th>
                    <th>Amount</th>
                    <th>Status</th>
                    <th>Block Time</th>
                  </tr>
                </thead>
                <tbody class="[&>tr]:hover:preset-tonal-primary">
                  {#each transactions as tx (tx.txid)}
                    <tr>
                      <td>
                        <a
                          href={`https://mempool.space/testnet4/tx/${tx.txid}`}
                          rel="noopener noreferrer"
                          target="_blank"
                        >
                          {tx.txid}
                        </a>
                      </td>
                      <td>
                        {(() => {
                          const currentAddress = $wallet.currentAddress;

                          let tx_amount = 0;
                          let rx_amount = 0;

                          for (const vin of tx.vin) {
                            if (vin.prevout?.scriptpubkey_address === currentAddress) {
                              tx_amount += vin.prevout.value;
                            }
                          }

                          for (const vout of tx.vout) {
                            if (vout.scriptpubkey_address === currentAddress) {
                              rx_amount += vout.value;
                            }
                          }

                          return tx_amount > rx_amount
                            ? satoshiToBtc(-(tx_amount - rx_amount))
                            : satoshiToBtc(+rx_amount);
                        })()}
                      </td>
                      <td>{tx.status.confirmed ? "Confirmed" : "Pending"}</td>
                      <td>
                        {tx.status.confirmed
                          ? new Date(tx.status.block_time * 1000).toLocaleString("en-US", {
                              year: "numeric",
                              month: "long",
                              day: "numeric",
                              hour: "numeric",
                              minute: "2-digit",
                              hour12: true,
                            })
                          : "N/A"}</td
                      >
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      {:else}
        <div class="flex min-h-screen w-full flex-row items-center justify-center gap-2 px-64">
          <Modal
            backdropClasses="backdrop-blur-sm"
            closeOnInteractOutside={false}
            contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
            onOpenChange={(e) => (modalInitWalletState = e.open)}
            open={modalInitWalletState}
            triggerBase="btn preset-tonal"
          >
            {#snippet content()}
              <header class="flex justify-between">
                <h2 class="text-2xl font-semibold">Initialize Wallet</h2>
              </header>
              <form
                class="w-full space-y-4"
                onsubmit={async (e) => {
                  const formEvent = e as SubmitEvent;
                  formEvent.preventDefault();

                  const target = formEvent.target as HTMLFormElement;
                  const formData = new FormData(target);

                  const password = formData.get("password") as string;

                  try {
                    mnemonic = await wallet.initialize(password);

                    modalInitWalletState = false;
                    isUnlocked = true;

                    toaster.success({
                      title: "Wallet Initialized",
                      description: `${mnemonic.length} words mnemonic generated. `,
                    });

                    await wallet.getStatus();
                  } catch (error) {
                    toaster.error({
                      title: "Initialization Failed",
                      description: error,
                    });
                  }
                }}
              >
                <label class="label">
                  <span class="label-text">Password</span>
                  <input name="password" class="input" placeholder="Password" required type="text" />
                </label>
                <button class="btn btn-sm preset-filled-primary-500 w-full" type="submit">Initialize</button>
              </form>
            {/snippet}
          </Modal>

          <button
            class="btn btn-sm preset-filled-warning-500"
            onclick={async () => {
              try {
                await wallet.disconnect();
                isUnlocked = false;
                toaster.success({
                  title: "Wallet Disconnected",
                  description: "The wallet has been disconnected successfully.",
                });
              } catch (error) {
                toaster.error({
                  title: "Disconnection Failed",
                  description: error,
                });
              }
            }}>Disconnect</button
          >

          <button class="btn btn-sm preset-filled" onclick={() => (modalInitWalletState = true)}
            >Initialize Wallet</button
          >

          <Modal
            backdropClasses="backdrop-blur-sm"
            closeOnInteractOutside={false}
            contentBase="card bg-surface-100-900 p-4 space-y-4 shadow-xl max-w-screen-sm"
            onOpenChange={(e) => (modalRestoreWalletState = e.open)}
            open={modalRestoreWalletState}
            triggerBase="btn preset-tonal"
          >
            {#snippet content()}
              <header class="flex justify-between">
                <h2 class="text-2xl font-semibold">Mnemonic Phrase</h2>
              </header>
              <article class="">
                <div class="mt-4">
                  <textarea
                    id="mnemonic-restore"
                    class="textarea preset-filled-surface-100-900 h-32 w-full resize-none"
                    placeholder="24 words mnemonic"
                  ></textarea>
                </div>
                <div class="mt-4">
                  <input name="password-restore" class="input" placeholder="Password" required type="text" />
                </div>
              </article>
              <footer class="flex justify-end gap-4">
                <button
                  class="btn btn-sm preset-filled-primary-500 mt-4"
                  onclick={async () => {
                    const textarea = document.getElementById("mnemonic-restore") as HTMLTextAreaElement;
                    const words = textarea.value.trim().split(/\s+/);
                    if (words.length !== 24) {
                      toaster.error({
                        title: "Invalid Mnemonic",
                        description: "Please enter a valid 24-word mnemonic phrase.",
                      });
                      return;
                    }

                    const passwordInput = document.querySelector('input[name="password-restore"]') as HTMLInputElement;
                    const password = passwordInput.value.trim();

                    if (!password) {
                      toaster.error({
                        title: "Password Required",
                        description: "Please enter a password to restore your wallet.",
                      });
                      return;
                    }

                    try {
                      await invoke("restore_wallet", { mnemonic: words, password });

                      modalRestoreWalletState = false;
                      isUnlocked = true;

                      toaster.success({
                        title: "Wallet Restored",
                        description: "Your wallet has been restored successfully.",
                      });
                    } catch (error) {
                      toaster.error({
                        title: "Restore Failed",
                        description: error,
                      });
                      return;
                    }

                    await wallet.getStatus();
                  }}>Restore</button
                >
              </footer>
            {/snippet}
          </Modal>

          <button class="btn btn-sm preset-filled" onclick={() => (modalRestoreWalletState = true)}
            >Restore Wallet</button
          >
        </div>
      {/if}
    {:else}
      <div class="flex min-h-screen w-full flex-col items-center justify-center gap-4 px-64">
        <form
          class="w-full space-y-4"
          onsubmit={async (e) => {
            e.preventDefault();

            try {
              await wallet.connect(selectedPort);
              toaster.success({
                title: "Wallet Connected",
                description: "Successfully connected to the wallet.",
              });
            } catch (error) {
              toaster.error({
                title: "Connection Failed",
                description: error,
              });
            }

            await wallet.getStatus();
          }}
        >
          <div class="input-group flex">
            <button
              class="ig-btn btn-sm preset-filled"
              onclick={async () => {
                await devices.scan();
                toaster.success({
                  title: "Devices Scanned",
                  description: `Found ${$devices.length} devices.`,
                });
                if ($devices.length > 0) {
                  selectedPort = $devices[0].port;
                }
              }}
              type="button"
            >
              Scan Devices</button
            >
            <select class="ig-select" disabled={$devices.length === 0} bind:value={selectedPort}>
              {#each $devices as port (port.port)}
                <option value={port.port}>
                  {port.port} - ({port.vid.toString(16)}:{port.pid.toString(16)}) {port.manufacturer} - {port.product}
                </option>
              {/each}
            </select>
            <button class="ig-btn btn-sm preset-filled-primary-500" disabled={$devices.length === 0} type="submit">
              Connect
            </button>
          </div>
        </form>
      </div>
    {/if}
  </div>
</div>
