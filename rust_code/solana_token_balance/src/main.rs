//$ spl-token create-token --url http://localhost:8899
// Creating token GoCSyuoVDKZiiFK7D8BvWFxvw1ySudk9Kgeoo7qbNJXV under program TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
// Address:  GoCSyuoVDKZiiFK7D8BvWFxvw1ySudk9Kgeoo7qbNJXV
// Decimals:  9
// Signature: 4hrNUreaHnfi1HThnLit4edbasK8K5zNb4bG2HbohXaVce8LfVAFeMYjG8B4BQCRHwQyMLMouZ3N7KSYpbmJXzpd

//$ spl-token create-account 5iKzsyCbfnyvGYpBxTtjDsK74B695CmiDKX7EW7ShDxe  --url http://localhost:8899
// Creating account fyXns3iqStQ3wBaLoGv5gZGQYHq3BXFnKkF3FPabTJX
// Signature: GnocVsitSHd47t5Z1SeXbb4B17g2TcWpcohi7pPsKiPAt4voqLdJQWPgbdqAegWzC99B8xXSDU4pzfDhGexbXvs

// $solana airdrop 99 fyXns3iqStQ3wBaLoGv5gZGQYHq3BXFnKkF3FPabTJX --url http://localhost:8899
// Requesting airdrop of 99 SOL
// Signature: 315e7VYnoUC7Z4wFogz2XgaDG2jxa2GePpoesCk3puckcrwhjdrgWzFcXUCt2bneTXQcMFAggHuSZZmiiVbcaLrp
// 99.00203928 SOL

// $solana balance fyXns3iqStQ3wBaLoGv5gZGQYHq3BXFnKkF3FPabTJX --url http://localhost:8899
// 99.00203928 SOL

// $spl-token accounts --url http://localhost:8899
// Token                                         Balance
// -----------------------------------------------------
// 5iKzsyCbfnyvGYpBxTtjDsK74B695CmiDKX7EW7ShDxe  0
// GoCSyuoVDKZiiFK7D8BvWFxvw1ySudk9Kgeoo7qbNJXV  0

// $spl-token mint GoCSyuoVDKZiiFK7D8BvWFxvw1ySudk9Kgeoo7qbNJXV 100 fyXns3iqStQ3wBaLoGv5gZGQYHq3BXFnKkF3FPabTJX --url http://localhost:8899
// Minting 100 tokens
// Token: GoCSyuoVDKZiiFK7D8BvWFxvw1ySudk9Kgeoo7qbNJXV
// Recipient: fyXns3iqStQ3wBaLoGv5gZGQYHq3BXFnKkF3FPabTJX
// Signature: 36dSryKG4FNvUF8NzCMQku5wLbJ9rAxuf8fe4zMqT9hXGAKLRWgB5iCDPTTURWJ9KeMWj6cM9tXoZaCa1n7HFPb7

use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;
use std::str::FromStr;

fn main() {
  // let rpc_url = String::from("https://api.devnet.solana.com");
  let rpc_url = String::from("http://localhost:8899");
  let connection = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

  // let token_account = Pubkey::from_str("FWZedVtyKQtP4CXhT7XDnLidRADrJknmZGA2qNjpTPg8").unwrap();
  let token_account = Pubkey::from_str("fyXns3iqStQ3wBaLoGv5gZGQYHq3BXFnKkF3FPabTJX").unwrap();
  let balance = connection
    .get_token_account_balance(&token_account)
    .unwrap();

  println!("amount: {}, decimals: {}", balance.amount, balance.decimals);
}

// copy from https://solana.com/zh/developers/cookbook/tokens/get-token-balance
