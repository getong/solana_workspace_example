use std::{rc::Rc, str::FromStr};

use anchor_client::{
  solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
  },
  Client, Cluster,
};
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Your wallet keypair (used to sign tx)
  let payer = anchor_client::solana_sdk::signature::read_keypair_file(
    "/Users/gerald/solana-wallets/bob.json",
  )?;
  let payer = Rc::new(payer);

  // Program ID of your data-reader program
  let program_id = Pubkey::from_str("59cnY1rjumvuLbab9MRqFmk62QALDmjFRBzMEupqeyzH")?;

  // Load IDL from JSON file
  let idl_str = include_str!("../idl/data_reader.json");
  let idl: Value = serde_json::from_str(idl_str)?;

  // Find the read_other_data instruction discriminator from IDL
  let instructions = idl["instructions"].as_array().unwrap();
  let read_other_data_ix = instructions
    .iter()
    .find(|ix| ix["name"] == "read_other_data") // Changed from "readOtherData" to "read_other_data"
    .expect("read_other_data instruction not found in IDL");

  // Get the discriminator (first 8 bytes)
  let discriminator = read_other_data_ix["discriminator"]
    .as_array()
    .unwrap()
    .iter()
    .map(|v| v.as_u64().unwrap() as u8)
    .collect::<Vec<u8>>();

  // Create client and program instance
  let client = Client::new(Cluster::Localnet, payer.clone());
  let program = client.program(program_id)?;

  // The other account address to pass (same as in TypeScript)
  let other_storage_pubkey = Pubkey::from_str("HwUtrTmLy6aHdDWSNN38rpRZy6icDhyCiVYD7XHV3LZF")?;

  // Build the instruction manually using IDL data
  let ix = Instruction {
    program_id,
    accounts: vec![AccountMeta::new_readonly(other_storage_pubkey, false)],
    data: discriminator, // Use discriminator from IDL
  };

  // Send the transaction
  let tx = program.request().instruction(ix).send()?;

  println!("Transaction signature: {:?}", tx);

  Ok(())
}
