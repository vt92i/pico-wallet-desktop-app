const satoshiToBtc = (satoshi: number): number => {
  return satoshi / 1e8;
};

const btcToSatoshi = (btc: number): number => {
  return Math.round(btc * 1e8);
};

export { satoshiToBtc, btcToSatoshi };
