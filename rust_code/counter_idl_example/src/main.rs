use std::sync::Arc;

use anchor_client::{
  Client, Cluster,
  solana_client::rpc_client::RpcClient,
  solana_sdk::{
    commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, pubkey::Pubkey,
    signature::Keypair, signer::Signer,
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

  // Generate Keypair and request airdrop
  let payer = Keypair::new();

  // Find the PDA for the counter account using the same seeds as in lib.rs
  let (counter_pda, _bump) =
    Pubkey::find_program_address(&[b"counter", payer.pubkey().as_ref()], &counter::ID);

  println!("Generated Keypair:");
  println!("   Payer: {}", payer.pubkey());
  println!("   Counter PDA: {}", counter_pda);

  println!("\nRequesting 10 SOL airdrop to payer");
  let airdrop_signature = connection.request_airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)?;

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
  println!("\nSend transaction with initialize, 7 increments, and 5 decrements instructions");
  let initialize_ix = program
    .request()
    .accounts(accounts::Initialize {
      counter: counter_pda,
      payer: program.payer(),
      system_program: system_program::ID,
    })
    .args(args::Initialize)
    .instructions()?
    .remove(0);

  // Create 7 increment instructions (so we can safely do 5 decrements)
  let increment_ix_1 = program
    .request()
    .accounts(accounts::Increment {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Increment)
    .instructions()?
    .remove(0);

  let increment_ix_2 = program
    .request()
    .accounts(accounts::Increment {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Increment)
    .instructions()?
    .remove(0);

  let increment_ix_3 = program
    .request()
    .accounts(accounts::Increment {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Increment)
    .instructions()?
    .remove(0);

  let increment_ix_4 = program
    .request()
    .accounts(accounts::Increment {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Increment)
    .instructions()?
    .remove(0);

  let increment_ix_5 = program
    .request()
    .accounts(accounts::Increment {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Increment)
    .instructions()?
    .remove(0);

  let increment_ix_6 = program
    .request()
    .accounts(accounts::Increment {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Increment)
    .instructions()?
    .remove(0);

  let increment_ix_7 = program
    .request()
    .accounts(accounts::Increment {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Increment)
    .instructions()?
    .remove(0);

  // Create 5 decrement instructions
  let decrement_ix_1 = program
    .request()
    .accounts(accounts::Decrement {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Decrement)
    .instructions()?
    .remove(0);

  let decrement_ix_2 = program
    .request()
    .accounts(accounts::Decrement {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Decrement)
    .instructions()?
    .remove(0);

  let decrement_ix_3 = program
    .request()
    .accounts(accounts::Decrement {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Decrement)
    .instructions()?
    .remove(0);

  let decrement_ix_4 = program
    .request()
    .accounts(accounts::Decrement {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Decrement)
    .instructions()?
    .remove(0);

  let decrement_ix_5 = program
    .request()
    .accounts(accounts::Decrement {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::Decrement)
    .instructions()?
    .remove(0);

  let signature = program
    .request()
    .instruction(initialize_ix)
    .instruction(increment_ix_1)
    .instruction(increment_ix_2)
    .instruction(increment_ix_3)
    .instruction(increment_ix_4)
    .instruction(increment_ix_5)
    .instruction(increment_ix_6)
    .instruction(increment_ix_7)
    .instruction(decrement_ix_1)
    .instruction(decrement_ix_2)
    .instruction(decrement_ix_3)
    .instruction(decrement_ix_4)
    .instruction(decrement_ix_5)
    .send()
    .await?;
  println!("   Transaction confirmed: {}", signature);

  // Call get_count instruction to check current value
  println!("\nCalling get_count instruction to check current counter value");
  let get_count_ix = program
    .request()
    .accounts(accounts::GetCount {
      counter: counter_pda,
      payer: program.payer(),
    })
    .args(args::GetCount)
    .instructions()?
    .remove(0);

  let get_count_signature = program.request().instruction(get_count_ix).send().await?;
  println!(
    "   Get count transaction confirmed: {}",
    get_count_signature
  );

  println!("\nFetch counter account data");
  let counter_account: Counter = program.account::<Counter>(counter_pda).await?;
  println!("   Final counter value: {}", counter_account.count);
  println!("   Expected: 0 + 7 (increments) - 5 (decrements) = 2");

  if counter_account.count == 2 {
    println!("   ✅ Counter value matches expected result!");
  } else {
    println!("   ❌ Counter value doesn't match expected result");
  }

  Ok(())
}

// use std::sync::Arc;

// use anchor_client::{
//   Client, Cluster,
//   solana_client::rpc_client::RpcClient,
//   solana_sdk::{
//     commitment_config::CommitmentConfig, native_token::LAMPORTS_PER_SOL, signature::Keypair,
//     signer::Signer,
//   },
// };
// use anchor_lang::prelude::*;
// use solana_system_interface::program as system_program;

// declare_program!(counter);
// use counter::{
//   accounts::Counter,
//   client::{accounts, args},
// };

// // Debug function to load and verify IDL
// async fn debug_program_id() -> anyhow::Result<()> {
//   println!("🔍 Debugging Program ID Configuration:");

//   // Read the IDL file to check the program ID
//   let idl_path = "idls/counter.json";
//   match std::fs::read_to_string(idl_path) {
//     Ok(idl_content) => {
//       if let Ok(idl_json) = serde_json::from_str::<serde_json::Value>(&idl_content) {
//         if let Some(address) = idl_json.get("address") {
//           println!("   IDL Program ID: {}", address);
//           println!("   Declared Program ID: {}", counter::ID);

//           if address.as_str() == Some(&counter::ID.to_string()) {
//             println!("   ✅ Program IDs match!");
//           } else {
//             println!("   ❌ Program ID MISMATCH!");
//             println!("   This is likely the cause of your error.");
//           }
//         }
//       }
//     }
//     Err(e) => {
//       println!("   ⚠️  Could not read IDL file: {}", e);
//     }
//   }

//   // Check the actual counter program source
//   let counter_lib_path = "../../counter/programs/counter/src/lib.rs";
//   match std::fs::read_to_string(counter_lib_path) {
//     Ok(source_content) => {
//       println!("\n🔍 Counter Program Source Analysis:");
//       // Look for declare_id! in the source
//       for line in source_content.lines() {
//         if line.contains("declare_id!") {
//           println!("   Source Program ID: {}", line.trim());

//           // Extract the program ID from declare_id!
//           if let Some(start) = line.find('"') {
//             if let Some(end) = line.rfind('"') {
//               let source_program_id = &line[start + 1 .. end];
//               println!("   Extracted Source ID: {}", source_program_id);

//               if source_program_id == counter::ID.to_string() {
//                 println!("   ✅ Source and declared IDs match!");
//               } else {
//                 println!("   ❌ SOURCE MISMATCH FOUND!");
//                 println!(
//                   "   The counter program source has ID: {}",
//                   source_program_id
//                 );
//                 println!("   But your client expects ID: {}", counter::ID);
//                 println!("\n   💡 SOLUTION:");
//                 println!("   You need to either:");
//                 println!(
//                   "   1. Update the counter program's declare_id! to: {}",
//                   counter::ID
//                 );
//                 println!("   2. OR rebuild/redeploy the counter program");
//                 println!("   3. OR update your IDL to match the source program ID");
//               }
//             }
//           }
//           break;
//         }
//       }
//     }
//     Err(e) => {
//       println!("   ⚠️  Could not read counter source: {}", e);
//       println!("   Try checking the counter program source manually");
//     }
//   }

//   Ok(())
// }

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//   // First, debug the program ID configuration
//   debug_program_id().await?;

//   let connection = RpcClient::new_with_commitment(
//     "http://127.0.0.1:8899", // Local validator URL
//     CommitmentConfig::confirmed(),
//   );

//   // Generate Keypairs and request airdrop
//   let payer = Keypair::new();
//   let receiver = Arc::new(Keypair::new());
//   println!("Generated Keypairs:");
//   println!("   Payer: {}", payer.pubkey());
//   println!("   Counter: {}", receiver.pubkey());

//   println!("\nRequesting 1 SOL airdrop to payer");
//   let airdrop_signature = connection.request_airdrop(&payer.pubkey(), LAMPORTS_PER_SOL)?;

//   // Wait for airdrop confirmation
//   while !connection.confirm_transaction(&airdrop_signature)? {
//     std::thread::sleep(std::time::Duration::from_millis(100));
//   }
//   println!("   Airdrop confirmed!");

//   // Create program client
//   let provider = Client::new_with_options(
//     Cluster::Localnet,
//     Arc::new(payer),
//     CommitmentConfig::confirmed(),
//   );
//   let program = provider.program(counter::ID)?;

//   // Debug: Print payer balance
//   let payer_balance = connection.get_balance(&program.payer())?;
//   println!(
//     "   Payer balance: {} SOL",
//     payer_balance as f64 / LAMPORTS_PER_SOL as f64
//   );

//   // Debug: Print program information
//   println!("\nProgram Debug Information:");
//   println!("   Declared Program ID: {}", counter::ID);
//   println!("   Program Payer: {}", program.payer());

//   // Debug: Check if program account exists and get its info
//   match connection.get_account(&counter::ID) {
//     Ok(account) => {
//       println!("   Program account found!");
//       println!("   Program account owner: {}", account.owner);
//       println!("   Program account executable: {}", account.executable);
//       println!("   Program account data length: {}", account.data.len());

//       // Additional debug: Check if this is the right program
//       println!("   Program data hash: {:x}", md5::compute(&account.data));
//     }
//     Err(e) => {
//       println!("   ❌ Program account NOT found: {}", e);
//       println!("   This suggests the program may not be deployed or the ID is incorrect");

//       // Try to find if there are any counter programs deployed
//       println!("\n🔍 Searching for deployed counter programs...");
//       // This is a simple approach - in practice you'd want to use getProgramAccounts
//     }
//   }

//   // Build and send instructions
//   println!("\nSend transaction with initialize and increment instructions");

//   // Debug: Check if counter account already exists
//   match connection.get_account(&receiver.pubkey()) {
//     Ok(account) => {
//       println!("   ⚠️  Counter account already exists!");
//       println!("   Account owner: {}", account.owner);
//       println!("   Account data length: {}", account.data.len());
//     }
//     Err(_) => {
//       println!("   ✅ Counter account does not exist (good for initialization)");
//     }
//   }

//   println!("   Building initialize instruction...");
//   let initialize_ix = match program
//     .request()
//     .accounts(accounts::Initialize {
//       counter: receiver.pubkey(),
//       payer: program.payer(),
//       system_program: system_program::ID,
//     })
//     .args(args::Initialize)
//     .instructions()
//   {
//     Ok(mut instructions) => {
//       println!("   ✅ Initialize instruction built successfully");
//       instructions.remove(0)
//     }
//     Err(e) => {
//       println!("   ❌ Failed to build initialize instruction: {}", e);
//       return Err(e.into());
//     }
//   };

//   println!("   Building increment instruction...");
//   let increment_ix = match program
//     .request()
//     .accounts(accounts::Increment {
//       counter: receiver.pubkey(),
//     })
//     .args(args::Increment)
//     .instructions()
//   {
//     Ok(mut instructions) => {
//       println!("   ✅ Increment instruction built successfully");
//       instructions.remove(0)
//     }
//     Err(e) => {
//       println!("   ❌ Failed to build increment instruction: {}", e);
//       return Err(e.into());
//     }
//   };

//   println!("   Sending transaction...");
//   let signature = match program
//     .request()
//     .instruction(initialize_ix)
//     .instruction(increment_ix)
//     .signer(receiver.clone())
//     .send()
//     .await
//   {
//     Ok(sig) => {
//       println!("   ✅ Transaction sent successfully!");
//       sig
//     }
//     Err(e) => {
//       println!("   ❌ Transaction failed: {}", e);

//       // Enhanced error analysis
//       println!("   Error type: {:?}", std::any::type_name_of_val(&e));

//       if e.to_string().contains("DeclaredProgramIdMismatch") {
//         println!("\n   🔍 PROGRAM ID MISMATCH DETECTED!");
//         println!("   This means the program ID in your code doesn't match the deployed
// program.");         println!("   Solutions:");
//         println!("   1. Check if the counter program is properly deployed");
//         println!("   2. Verify the program ID in your IDL matches the deployed program");
//         println!("   3. Redeploy the program if necessary");
//       }

//       return Err(e.into());
//     }
//   };
//   println!("   Transaction confirmed: {}", signature);

//   println!("\nFetch counter account data");
//   let counter_account: Counter = match program.account::<Counter>(receiver.pubkey()).await {
//     Ok(account) => {
//       println!("   ✅ Counter account fetched successfully");
//       account
//     }
//     Err(e) => {
//       println!("   ❌ Failed to fetch counter account: {}", e);
//       return Err(e.into());
//     }
//   };
//   println!("   Counter value: {}", counter_account.count);

//   // Final debug: Show transaction details
//   println!("\n🎉 Transaction Summary:");
//   println!("   Signature: {}", signature);
//   println!("   Counter Account: {}", receiver.pubkey());
//   println!("   Final Counter Value: {}", counter_account.count);

//   Ok(())
// }

// anchor keys sync
