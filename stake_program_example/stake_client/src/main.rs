use std::{rc::Rc, str::FromStr};

use anchor_client::{
  solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
  },
  Client, Cluster,
};
use anyhow::Result;
use solana_system_interface::program as system_program;
use spl_associated_token_account::get_associated_token_address;

// Program ID from the IDL
const PROGRAM_ID: &str = "8Bv63d7LKuxYipEWycEHz263LKgq1qTwE6PaxsmE2Vmx";

/// Simplified client for demonstrating IDL-based interaction
pub struct IdlStakeClient {
  pub client: Client<Rc<Keypair>>,
  pub payer: Rc<Keypair>,
  pub program_id: Pubkey,
}

impl IdlStakeClient {
  pub fn new(cluster: Cluster, payer_path: &str) -> Result<Self> {
    let payer = Rc::new(read_keypair_file(payer_path).map_err(|e| anyhow::anyhow!("{}", e))?);
    let client = Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str(PROGRAM_ID)?;

    Ok(Self {
      client,
      payer,
      program_id,
    })
  }

  /// PDA derivation functions
  pub fn derive_pool_pda(&self, staking_mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"pool", staking_mint.as_ref()], &self.program_id)
  }

  pub fn derive_user_stake_pda(&self, pool: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
      &[b"user_stake", pool.as_ref(), user.as_ref()],
      &self.program_id,
    )
  }

  pub fn derive_staking_vault_pda(&self, pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"staking_vault", pool.as_ref()], &self.program_id)
  }

  pub fn derive_reward_vault_pda(&self, pool: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"reward_vault", pool.as_ref()], &self.program_id)
  }

  /// Build instruction for initialize_pool using IDL discriminator
  pub fn build_initialize_pool_instruction(
    &self,
    staking_mint: Pubkey,
    reward_mint: Pubkey,
    reward_rate: u64,
    lock_period: i64,
  ) -> Result<Instruction> {
    let (pool, _) = self.derive_pool_pda(&staking_mint);
    let (staking_vault, _) = self.derive_staking_vault_pda(&pool);
    let (reward_vault, _) = self.derive_reward_vault_pda(&pool);

    // Instruction discriminator for initialize_pool (from IDL)
    let discriminator = [95, 180, 10, 172, 84, 174, 232, 40];
    let mut instruction_data = discriminator.to_vec();
    instruction_data.extend_from_slice(&reward_rate.to_le_bytes());
    instruction_data.extend_from_slice(&lock_period.to_le_bytes());

    let accounts = vec![
      AccountMeta::new(pool, false),
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new_readonly(staking_mint, false),
      AccountMeta::new_readonly(reward_mint, false),
      AccountMeta::new(staking_vault, false),
      AccountMeta::new(reward_vault, false),
      AccountMeta::new_readonly(spl_token::id(), false),
      AccountMeta::new_readonly(system_program::id(), false),
    ];

    Ok(Instruction {
      program_id: self.program_id,
      accounts,
      data: instruction_data,
    })
  }

  /// Build instruction for initialize_user_stake using IDL discriminator
  pub fn build_initialize_user_stake_instruction(
    &self,
    pool: Pubkey,
    user: Pubkey,
  ) -> Result<Instruction> {
    let (user_stake, _) = self.derive_user_stake_pda(&pool, &user);

    // Instruction discriminator for initialize_user_stake (from IDL)
    let discriminator = [248, 96, 76, 185, 77, 56, 18, 0];
    let instruction_data = discriminator.to_vec(); // No additional args

    let accounts = vec![
      AccountMeta::new(user_stake, false),
      AccountMeta::new_readonly(pool, false),
      AccountMeta::new(user, true),
      AccountMeta::new_readonly(system_program::id(), false),
    ];

    Ok(Instruction {
      program_id: self.program_id,
      accounts,
      data: instruction_data,
    })
  }

  /// Build instruction for stake using IDL discriminator
  pub fn build_stake_instruction(
    &self,
    pool: Pubkey,
    user: Pubkey,
    user_token_account: Pubkey,
    amount: u64,
  ) -> Result<Instruction> {
    let (user_stake, _) = self.derive_user_stake_pda(&pool, &user);
    let (staking_vault, _) = self.derive_staking_vault_pda(&pool);

    // Instruction discriminator for stake (from IDL)
    let discriminator = [206, 176, 202, 18, 200, 209, 179, 108];
    let mut instruction_data = discriminator.to_vec();
    instruction_data.extend_from_slice(&amount.to_le_bytes());

    let accounts = vec![
      AccountMeta::new(pool, false),
      AccountMeta::new(user_stake, false),
      AccountMeta::new(user_token_account, false),
      AccountMeta::new(staking_vault, false),
      AccountMeta::new_readonly(user, true),
      AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
      program_id: self.program_id,
      accounts,
      data: instruction_data,
    })
  }

  /// Build instruction for unstake using IDL discriminator
  pub fn build_unstake_instruction(
    &self,
    pool: Pubkey,
    user: Pubkey,
    user_token_account: Pubkey,
    amount: u64,
  ) -> Result<Instruction> {
    let (user_stake, _) = self.derive_user_stake_pda(&pool, &user);
    let (staking_vault, _) = self.derive_staking_vault_pda(&pool);

    // Instruction discriminator for unstake (from IDL)
    let discriminator = [90, 95, 107, 42, 205, 124, 50, 225];
    let mut instruction_data = discriminator.to_vec();
    instruction_data.extend_from_slice(&amount.to_le_bytes());

    let accounts = vec![
      AccountMeta::new(pool, false),
      AccountMeta::new(user_stake, false),
      AccountMeta::new(user_token_account, false),
      AccountMeta::new(staking_vault, false),
      AccountMeta::new_readonly(user, true),
      AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
      program_id: self.program_id,
      accounts,
      data: instruction_data,
    })
  }

  /// Build instruction for claim_reward using IDL discriminator
  pub fn build_claim_reward_instruction(
    &self,
    pool: Pubkey,
    user: Pubkey,
    user_reward_account: Pubkey,
  ) -> Result<Instruction> {
    let (user_stake, _) = self.derive_user_stake_pda(&pool, &user);
    let (reward_vault, _) = self.derive_reward_vault_pda(&pool);

    // Instruction discriminator for claim_reward (from IDL)
    let discriminator = [149, 95, 181, 242, 94, 90, 158, 162];
    let instruction_data = discriminator.to_vec(); // No additional args

    let accounts = vec![
      AccountMeta::new(pool, false),
      AccountMeta::new(user_stake, false),
      AccountMeta::new(user_reward_account, false),
      AccountMeta::new(reward_vault, false),
      AccountMeta::new_readonly(user, true),
      AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
      program_id: self.program_id,
      accounts,
      data: instruction_data,
    })
  }

  /// Helper method to create associated token account
  pub fn build_create_token_account_instruction(&self, mint: Pubkey, owner: Pubkey) -> Instruction {
    spl_associated_token_account::instruction::create_associated_token_account(
      &self.payer.pubkey(),
      &owner,
      &mint,
      &spl_token::id(),
    )
  }

  /// Print IDL information (simulated for demo)
  pub fn print_idl_info(&self) {
    println!("Program ID: {}", self.program_id);
    println!("Program Name: stake_program_example");

    println!("\nInstructions:");
    println!("  - initialize_pool: [95, 180, 10, 172, 84, 174, 232, 40]");
    println!("  - initialize_user_stake: [248, 96, 76, 185, 77, 56, 18, 0]");
    println!("  - stake: [206, 176, 202, 18, 200, 209, 179, 108]");
    println!("  - unstake: [90, 95, 107, 42, 205, 124, 50, 225]");
    println!("  - claim_reward: [149, 95, 181, 242, 94, 90, 158, 162]");
    println!("  - fund_reward_pool: [85, 49, 108, 245, 204, 70, 243, 3]");

    println!("\nAccount Types:");
    println!("  - StakePool: [121, 34, 206, 21, 79, 127, 255, 28]");
    println!("  - UserStake: [102, 53, 163, 107, 9, 138, 87, 153]");

    println!("\nEvents:");
    println!("  - StakeEvent: [226, 134, 188, 173, 19, 33, 75, 175]");
    println!("  - UnstakeEvent: [162, 104, 137, 228, 81, 3, 79, 197]");
    println!("  - ClaimRewardEvent: [207, 16, 14, 170, 176, 71, 40, 53]");

    println!("\nErrors:");
    println!("  - 6000: Overflow (Arithmetic overflow)");
    println!("  - 6001: Underflow (Arithmetic underflow)");
    println!("  - 6002: DivisionByZero (Division by zero)");
    println!("  - 6003: InsufficientBalance (Insufficient balance)");
    println!("  - 6004: StillLocked (Tokens are still locked)");
    println!("  - 6005: NoRewardsToClaim (No rewards to claim)");
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  println!("=== Stake Program IDL-Based Client Example ===\n");

  // Setup paths
  let keypair_path = std::env::var("KEYPAIR_PATH").unwrap_or_else(|_| {
    dirs::home_dir()
      .unwrap()
      .join(".config/solana/id.json")
      .to_string_lossy()
      .to_string()
  });

  // Example token mints (you would use real mint addresses)
  let staking_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?; // SOL mint
  let reward_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; // USDC mint

  println!("🚀 Initializing IDL-based stake client...");
  let client = IdlStakeClient::new(Cluster::Localnet, &keypair_path)?;

  // Display IDL information
  println!("\n📋 IDL Information:");
  client.print_idl_info();

  // Derive PDAs
  let (pool, pool_bump) = client.derive_pool_pda(&staking_mint);
  let user = client.payer.pubkey();
  let (user_stake, user_stake_bump) = client.derive_user_stake_pda(&pool, &user);
  let (staking_vault, staking_vault_bump) = client.derive_staking_vault_pda(&pool);
  let (reward_vault, reward_vault_bump) = client.derive_reward_vault_pda(&pool);

  println!("\n🔍 Derived Addresses:");
  println!("  Program ID: {}", client.program_id);
  println!("  Pool PDA: {} (bump: {})", pool, pool_bump);
  println!(
    "  User Stake PDA: {} (bump: {})",
    user_stake, user_stake_bump
  );
  println!(
    "  Staking Vault PDA: {} (bump: {})",
    staking_vault, staking_vault_bump
  );
  println!(
    "  Reward Vault PDA: {} (bump: {})",
    reward_vault, reward_vault_bump
  );

  println!("\n🛠️ Building Instructions:");

  // Build initialize pool instruction
  match client.build_initialize_pool_instruction(staking_mint, reward_mint, 100, 86400) {
    Ok(instruction) => {
      println!("✅ Initialize Pool instruction built successfully");
      println!("   - Program ID: {}", instruction.program_id);
      println!("   - Accounts: {} accounts", instruction.accounts.len());
      println!("   - Data: {} bytes", instruction.data.len());
    }
    Err(e) => println!("❌ Failed to build initialize pool instruction: {}", e),
  }

  // Build initialize user stake instruction
  match client.build_initialize_user_stake_instruction(pool, user) {
    Ok(instruction) => {
      println!("✅ Initialize User Stake instruction built successfully");
      println!("   - Program ID: {}", instruction.program_id);
      println!("   - Accounts: {} accounts", instruction.accounts.len());
      println!("   - Data: {} bytes", instruction.data.len());
    }
    Err(e) => println!(
      "❌ Failed to build initialize user stake instruction: {}",
      e
    ),
  }

  // Create token account addresses for demo
  let user_staking_token_account = get_associated_token_address(&user, &staking_mint);
  let user_reward_token_account = get_associated_token_address(&user, &reward_mint);

  println!(
    "   - User Staking Token Account: {}",
    user_staking_token_account
  );
  println!(
    "   - User Reward Token Account: {}",
    user_reward_token_account
  );

  // Build stake instruction
  match client.build_stake_instruction(pool, user, user_staking_token_account, 1000) {
    Ok(instruction) => {
      println!("✅ Stake instruction built successfully");
      println!("   - Program ID: {}", instruction.program_id);
      println!("   - Accounts: {} accounts", instruction.accounts.len());
      println!("   - Data: {} bytes", instruction.data.len());
    }
    Err(e) => println!("❌ Failed to build stake instruction: {}", e),
  }

  // Build unstake instruction
  match client.build_unstake_instruction(pool, user, user_staking_token_account, 500) {
    Ok(instruction) => {
      println!("✅ Unstake instruction built successfully");
      println!("   - Program ID: {}", instruction.program_id);
      println!("   - Accounts: {} accounts", instruction.accounts.len());
      println!("   - Data: {} bytes", instruction.data.len());
    }
    Err(e) => println!("❌ Failed to build unstake instruction: {}", e),
  }

  // Build claim reward instruction
  match client.build_claim_reward_instruction(pool, user, user_reward_token_account) {
    Ok(instruction) => {
      println!("✅ Claim Reward instruction built successfully");
      println!("   - Program ID: {}", instruction.program_id);
      println!("   - Accounts: {} accounts", instruction.accounts.len());
      println!("   - Data: {} bytes", instruction.data.len());
    }
    Err(e) => println!("❌ Failed to build claim reward instruction: {}", e),
  }

  println!("\n💡 Key Concepts Demonstrated:");
  println!("  ✓ IDL-based instruction discriminators");
  println!("  ✓ PDA derivation using program seeds");
  println!("  ✓ Account metadata construction");
  println!("  ✓ Instruction data serialization");
  println!("  ✓ Associated token account addressing");

  println!("\n📝 Next Steps:");
  println!("  • Deploy program to localnet/devnet");
  println!("  • Create token mints for testing");
  println!("  • Execute instructions using anchor-client or web3.js");
  println!("  • Monitor events and account state changes");

  println!("\n🎯 This example shows how to interact with Solana programs using:");
  println!("  1. Generated IDL discriminators for type-safe instruction construction");
  println!("  2. Program Derived Address (PDA) calculation");
  println!("  3. Complete account metadata specification");
  println!("  4. Proper instruction data serialization");

  Ok(())
}
