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

  let transaction_signature = client
    .request_airdrop(&signer_keypair.pubkey(), 5 * LAMPORTS_PER_SOL)
    .await?;
  loop {
    if client.confirm_transaction(&transaction_signature).await? {
      break;
    }
  }

  let mut transaction = Transaction::new_with_payer(&[memo_ix], Some(&signer_keypair.pubkey()));
  transaction.sign(&[&signer_keypair], client.get_latest_blockhash().await?);

  match client.send_and_confirm_transaction(&transaction).await {
    Ok(signature) => println!("Transaction Signature: {}", signature),
    Err(err) => eprintln!("Error sending transaction: {}", err),
  }

  Ok(())
}
