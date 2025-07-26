use std::{rc::Rc, str::FromStr};

use anchor_client::{Client, Cluster, solana_sdk::pubkey::Pubkey};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Your wallet keypair (used to sign tx)
  let payer = anchor_client::solana_sdk::signature::read_keypair_file(
    "/Users/gerald/solana-wallets/bob.json",
  )?;
  let payer = Rc::new(payer);

  // Program ID of your data-reader program
  let program_id = Pubkey::from_str("59cnY1rjumvuLbab9MRqFmk62QALDmjFRBzMEupqeyzH")?;

  // Create client and program instance
  let client = Client::new(Cluster::Localnet, payer.clone());
  let program = client.program(program_id)?;

  // The other account address to pass (same as in TypeScript)
  let other_storage_pubkey = Pubkey::from_str("HwUtrTmLy6aHdDWSNN38rpRZy6icDhyCiVYD7XHV3LZF")?;

  // Build the transaction using proper Anchor client pattern
  let tx = program
    .request()
    .accounts(vec![
      anchor_client::solana_sdk::instruction::AccountMeta::new_readonly(
        other_storage_pubkey,
        false,
      ),
    ])
    .send()?;

  println!("Transaction signature: {:?}", tx);

  Ok(())
}
