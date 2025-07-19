use std::sync::Arc;

use anchor_client::{
  Client, Cluster,
  solana_client::rpc_client::RpcClient,
  solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Keypair,
    signer::Signer,
  },
};
use anchor_lang::prelude::*;
use solana_system_interface::program as system_program;

declare_program!(counter);
use counter::{
  accounts::Counter,
  client::{accounts, args},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let connection = RpcClient::new_with_commitment(
    "http://127.0.0.1:8899", // Local validator URL
    CommitmentConfig::confirmed(),
  );

  // Generate Keypairs and request airdrop
  let payer = Keypair::new();
  let receiver = Arc::new(Keypair::new());
  println!("Generated Keypairs:");
  println!("   Payer: {}", payer.pubkey());
  println!("   Counter: {}", receiver.pubkey());

  println!("\nRequesting 1 SOL airdrop to payer");
  let airdrop_signature = connection.request_airdrop(&payer.pubkey(), LAMPORTS_PER_SOL)?;

  // Wait for airdrop confirmation
  while !connection.confirm_transaction(&airdrop_signature)? {
    std::thread::sleep(std::time::Duration::from_millis(100));
  }
  println!("   Airdrop confirmed!");

  // Create program client
  let provider = Client::new_with_options(
    Cluster::Localnet,
    Arc::new(payer),
    CommitmentConfig::confirmed(),
  );
  let program = provider.program(counter::ID)?;

  // Build and send instructions
  println!("\nSend transaction with initialize and increment instructions");
  let initialize_ix = program
    .request()
    .accounts(accounts::Initialize {
      counter: receiver.pubkey(),
      payer: program.payer(),
      system_program: system_program::ID,
    })
    .args(args::Initialize)
    .instructions()?
    .remove(0);

  let increment_ix = program
    .request()
    .accounts(accounts::Increment {
      counter: receiver.pubkey(),
    })
    .args(args::Increment)
    .instructions()?
    .remove(0);

  let signature = program
    .request()
    .instruction(initialize_ix)
    .instruction(increment_ix)
    .signer(receiver.clone())
    .send()
    .await?;
  println!("   Transaction confirmed: {}", signature);

  println!("\nFetch counter account data");
  let counter_account: Counter = program.account::<Counter>(receiver.pubkey()).await?;
  println!("   Counter value: {}", counter_account.count);
  Ok(())
}
