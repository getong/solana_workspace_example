use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
  commitment_config::CommitmentConfig, compute_budget, native_token::LAMPORTS_PER_SOL,
  signature::Keypair, signer::Signer, system_instruction::transfer, transaction::Transaction,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let client = RpcClient::new_with_commitment(
    String::from("http://127.0.0.1:8899"),
    CommitmentConfig::confirmed(),
  );

  let signer_keypair = Keypair::new();

  let modify_cu_ix = compute_budget::ComputeBudgetInstruction::set_compute_unit_limit(1_000_000);
  let add_priority_fee_ix = compute_budget::ComputeBudgetInstruction::set_compute_unit_price(1);

  let transfer_ix = transfer(
    &signer_keypair.pubkey(),
    &Keypair::new().pubkey(),
    LAMPORTS_PER_SOL,
  );

  let transaction_signature = client
    .request_airdrop(&signer_keypair.pubkey(), 5 * LAMPORTS_PER_SOL)
    .await?;
  loop {
    if client.confirm_transaction(&transaction_signature).await? {
      break;
    }
  }

  let mut transaction = Transaction::new_with_payer(
    &[modify_cu_ix, add_priority_fee_ix, transfer_ix],
    Some(&signer_keypair.pubkey()),
  );
  transaction.sign(&[&signer_keypair], client.get_latest_blockhash().await?);

  match client.send_and_confirm_transaction(&transaction).await {
    Ok(signature) => println!("Transaction Signature: {}", signature),
    Err(err) => eprintln!("Error sending transaction: {}", err),
  }

  Ok(())
}
