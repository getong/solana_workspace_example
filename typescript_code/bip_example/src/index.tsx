import * as bip39 from "bip39";

const mnemonic = bip39.generateMnemonic();
console.log("mnemonic is: \n" + mnemonic);

// copy from https://solana.com/zh/developers/cookbook/wallets/generate-mnemonic
