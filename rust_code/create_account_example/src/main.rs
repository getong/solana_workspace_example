use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
  commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Keypair,
  signer::Signer, system_instruction::create_account as create_account_ix,
  system_program::ID as SYSTEM_PROGRAM_ID, transaction::Transaction,
};

// SOLANA ACCOUNT CAPABILITIES:
// 1. Store SOL tokens (native cryptocurrency)
// 2. Store data (up to 10MB per account)
// 3. Execute programs (if it's a program account)
// 4. Own other accounts (as data accounts)
// 5. Receive and send transactions
// 6. Store NFTs, tokens, and other digital assets
// 7. Interact with smart contracts/programs
// 8. Participate in DeFi protocols
// 9. Stake SOL for consensus participation
// 10. Hold metadata for various applications
//
// Account Types:
// - System accounts: Hold SOL, owned by System Program
// - Program accounts: Contain executable code
// - Data accounts: Store application state, owned by programs
// - Token accounts: Hold SPL tokens

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Connect to local Solana validator (devnet/testnet)
  let client = RpcClient::new_with_commitment(
    String::from("http://127.0.0.1:8899"),
    CommitmentConfig::confirmed(),
  );

  // Create keypairs for payer and new account
  let from_keypair = Keypair::new(); // payer (will fund the creation)
  let new_account_keypair = Keypair::new(); // the new account being created

  // Define account parameters
  let data_len = 0; // No data storage initially (can be increased later)

  // Calculate minimum balance needed to make account rent-exempt
  // Rent-exempt accounts don't need periodic payments to stay active
  let rent_exemption_amount = client
    .get_minimum_balance_for_rent_exemption(data_len)
    .await?;

  // Create the account creation instruction
  let create_acc_ix = create_account_ix(
    &from_keypair.pubkey(),        // payer - who pays for account creation
    &new_account_keypair.pubkey(), // new account address
    rent_exemption_amount,         // lamports to transfer for rent exemption
    data_len as u64,               // space reserved for account data
    &SYSTEM_PROGRAM_ID,            // program that will own this account
  );

  // Fund the payer account with 1 SOL from faucet (testnet/devnet only)
  let transaction_signature = client
    .request_airdrop(&from_keypair.pubkey(), 1 * LAMPORTS_PER_SOL)
    .await?;

  // Wait for airdrop confirmation
  loop {
    if client.confirm_transaction(&transaction_signature).await? {
      break;
    }
  }

  // Create and sign the transaction
  let mut transaction = Transaction::new_with_payer(&[create_acc_ix], Some(&from_keypair.pubkey()));
  transaction.sign(
    &[&from_keypair, &new_account_keypair], // Both must sign
    client.get_latest_blockhash().await?,
  );

  // Send transaction to create the account
  match client.send_and_confirm_transaction(&transaction).await {
    Ok(signature) => {
      println!("Transaction Signature: {}", signature);
      println!("New account created at: {}", new_account_keypair.pubkey());
      println!("Account capabilities:");
      println!("- Can receive and send SOL transfers");
      println!("- Can store data if reallocated with more space");
      println!("- Can be used as signer for other transactions");
      println!("- Can interact with Solana programs/smart contracts");
      println!("- Can hold SPL tokens if converted to token account");
    }
    Err(err) => eprintln!("Error sending transaction: {}", err),
  }

  Ok(())
}
