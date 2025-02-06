import { createSolanaRpc, devnet } from "@solana/web3.js";

async function getGenesisHash() {
  // Connect to the local Solana node
  const rpc = createSolanaRpc(devnet("http://127.0.0.1:8899"));

  // Get the genesis hash (blockhash of the genesis block)
  try {
    const genesisHash = await rpc.getGenesisHash().send();
    console.log("Genesis Hash:", genesisHash);
  } catch (error) {
    console.error("Error fetching genesis hash:", error);
  }
}

getGenesisHash();
