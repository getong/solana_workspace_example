use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let client = RpcClient::new_with_commitment(
    String::from("https://api.mainnet-beta.solana.com"),
    CommitmentConfig::confirmed(),
  );

  let address = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
  let balance = client.get_balance(&address).await?;

  println!("Balance: {} SOL", balance as f64 / LAMPORTS_PER_SOL as f64);

  Ok(())
}
