use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
  commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Keypair,
  signer::Signer, transaction::Transaction,
};
use spl_memo::build_memo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let client = RpcClient::new_with_commitment(
    String::from("http://127.0.0.1:8899"),
    CommitmentConfig::confirmed(),
  );

  let signer_keypair = Keypair::new();
  let memo = String::from("Memo message to be logged in this transaction");

  let memo_ix = build_memo(memo.as_bytes(), &[&signer_keypair.pubkey()]);
  let mut transaction = Transaction::new_with_payer(&[memo_ix], Some(&signer_keypair.pubkey()));
  transaction.sign(&[&signer_keypair], client.get_latest_blockhash().await?);

  let transaction_signature = client
    .request_airdrop(&signer_keypair.pubkey(), 5 * LAMPORTS_PER_SOL)
    .await?;
  loop {
    if client.confirm_transaction(&transaction_signature).await? {
      break;
    }
  }

  let estimated_units = estimate_cu_used(&client, &transaction).await?;
  println!("Transaction estimated to consumed {estimated_units} compute units");

  let tx_cost = client.get_fee_for_message(transaction.message()).await?;
  println!("Transaction is estimated to cost {tx_cost} lamports");

  match client.send_and_confirm_transaction(&transaction).await {
    Ok(signature) => println!("Transaction Signature: {}", signature),
    Err(err) => eprintln!("Error sending transaction: {}", err),
  }

  Ok(())
}

async fn estimate_cu_used(client: &RpcClient, tx: &Transaction) -> anyhow::Result<u64> {
  let sim_res = client.simulate_transaction(tx).await?;

  let units_consumed = sim_res
    .value
    .units_consumed
    .expect("couldn't estimate CUs used");

  Ok(units_consumed)
}

// copy from https://solana.com/zh/developers/cookbook/transactions/calculate-cost