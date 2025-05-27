use std::env;

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_sdk::{
  commitment_config::CommitmentConfig,
  signature::{read_keypair_file, Keypair},
  signer::Signer,
  transaction::Transaction,
};
use spl_associated_token_account::instruction as ata_instruction;
use spl_token::{instruction, state::Mint};

const RPC_URL: &str = "http://localhost:8899";
const KEYPAIR_PATH: &str = "~/solana-wallets/bob.json";

#[tokio::main]
async fn main() -> Result<()> {
  // Connect to local Solana cluster
  let client = RpcClient::new_with_commitment(RPC_URL, CommitmentConfig::confirmed());

  // Load the keypair from your configuration with tilde expansion
  let expanded_path = if KEYPAIR_PATH.starts_with("~/") {
    let home =
      env::var("HOME").map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
    KEYPAIR_PATH.replacen("~", &home, 1)
  } else {
    KEYPAIR_PATH.to_string()
  };

  let payer = read_keypair_file(&expanded_path)
    .map_err(|e| anyhow::anyhow!("Failed to read keypair: {}", e))?;

  println!("Payer public key: {}", payer.pubkey());

  // Create a new mint
  let mint_keypair = Keypair::new();
  let mint_pubkey = mint_keypair.pubkey();

  println!("Creating mint: {}", mint_pubkey);

  // Get minimum balance for mint account
  let mint_rent = client.get_minimum_balance_for_rent_exemption(Mint::LEN)?;

  // Create mint account instruction
  let create_mint_account_ix = solana_sdk::system_instruction::create_account(
    &payer.pubkey(),
    &mint_pubkey,
    mint_rent,
    Mint::LEN as u64,
    &spl_token::id(),
  );

  // Initialize mint instruction
  let init_mint_ix = instruction::initialize_mint(
    &spl_token::id(),
    &mint_pubkey,
    &payer.pubkey(),
    Some(&payer.pubkey()),
    9, // decimals
  )?;

  // Create and send mint creation transaction
  let mut transaction = Transaction::new_with_payer(
    &[create_mint_account_ix, init_mint_ix],
    Some(&payer.pubkey()),
  );

  let recent_blockhash = client.get_latest_blockhash()?;
  transaction.sign(&[&payer, &mint_keypair], recent_blockhash);

  let signature = client.send_and_confirm_transaction(&transaction)?;
  println!("Mint created with signature: {}", signature);

  // Create associated token account
  let associated_token_account =
    spl_associated_token_account::get_associated_token_address(&payer.pubkey(), &mint_pubkey);

  println!(
    "Creating associated token account: {}",
    associated_token_account
  );

  let create_ata_ix = ata_instruction::create_associated_token_account(
    &payer.pubkey(),
    &payer.pubkey(),
    &mint_pubkey,
    &spl_token::id(),
  );

  let mut ata_transaction = Transaction::new_with_payer(&[create_ata_ix], Some(&payer.pubkey()));

  let recent_blockhash = client.get_latest_blockhash()?;
  ata_transaction.sign(&[&payer], recent_blockhash);

  let ata_signature = client.send_and_confirm_transaction(&ata_transaction)?;
  println!(
    "Associated token account created with signature: {}",
    ata_signature
  );

  // Mint 9999 tokens
  let mint_amount = 9999 * 10_u64.pow(9); // 9999 tokens with 9 decimals

  let mint_to_ix = instruction::mint_to(
    &spl_token::id(),
    &mint_pubkey,
    &associated_token_account,
    &payer.pubkey(),
    &[],
    mint_amount,
  )?;

  let mut mint_transaction = Transaction::new_with_payer(&[mint_to_ix], Some(&payer.pubkey()));

  let recent_blockhash = client.get_latest_blockhash()?;
  mint_transaction.sign(&[&payer], recent_blockhash);

  let mint_signature = client.send_and_confirm_transaction(&mint_transaction)?;
  println!("Minted 9999 tokens with signature: {}", mint_signature);

  println!("Success! Token mint: {}", mint_pubkey);
  println!("Token account: {}", associated_token_account);

  Ok(())
}
