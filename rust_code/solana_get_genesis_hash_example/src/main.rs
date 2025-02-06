use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

fn main() {
  // Connect to the local Solana node
  let rpc_url = "http://localhost:8899";
  let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());

  // Fetch the genesis hash
  match client.get_genesis_hash() {
    Ok(genesis_hash) => {
      println!("Genesis hash: {}", genesis_hash);
    }
    Err(err) => {
      eprintln!("Failed to fetch genesis hash: {}", err);
    }
  }
}
