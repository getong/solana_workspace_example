use anchor_client::{
    solana_sdk::{
        commitment_config::CommitmentConfig,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        system_program,
        transaction::Transaction,
    },
    Client, Cluster,
};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::{rc::Rc, str::FromStr};

/// IDL-based stake client that uses the generated IDL JSON directly
pub struct IdlStakeClient {
    pub client: Client<Rc<Keypair>>,
    pub payer: Rc<Keypair>,
    pub program_id: Pubkey,
    pub idl: Value,
}

impl IdlStakeClient {
    pub fn new(cluster: Cluster, payer_path: &str, idl_path: &str) -> Result<Self> {
        let payer = Rc::new(read_keypair_file(payer_path).map_err(|e| anyhow!("{}", e))?);
        let client = Client::new_with_options(cluster, payer.clone(), CommitmentConfig::confirmed());
        
        // Load IDL from file
        let idl_content = std::fs::read_to_string(idl_path)?;
        let idl: Value = serde_json::from_str(&idl_content)?;
        
        let program_id = Pubkey::from_str(
            idl["address"]
                .as_str()
                .ok_or_else(|| anyhow!("Invalid program ID in IDL"))?
        )?;

        Ok(Self {
            client,
            payer,
            program_id,
            idl,
        })
    }

    /// Helper function to find instruction discriminator
    fn get_instruction_discriminator(&self, instruction_name: &str) -> Result<Vec<u8>> {
        let instructions = self.idl["instructions"]
            .as_array()
            .ok_or_else(|| anyhow!("No instructions found in IDL"))?;

        for instruction in instructions {
            if instruction["name"].as_str() == Some(instruction_name) {
                let discriminator = instruction["discriminator"]
                    .as_array()
                    .ok_or_else(|| anyhow!("No discriminator found"))?
                    .iter()
                    .map(|v| v.as_u64().unwrap() as u8)
                    .collect();
                return Ok(discriminator);
            }
        }
        
        Err(anyhow!("Instruction {} not found in IDL", instruction_name))
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

    /// Initialize pool using raw instruction data
    pub async fn initialize_pool_raw(
        &self,
        staking_mint: Pubkey,
        reward_mint: Pubkey,
        reward_rate: u64,
        lock_period: i64,
    ) -> Result<String> {
        let discriminator = self.get_instruction_discriminator("initialize_pool")?;
        let (pool, _) = self.derive_pool_pda(&staking_mint);
        let (staking_vault, _) = self.derive_staking_vault_pda(&pool);
        let (reward_vault, _) = self.derive_reward_vault_pda(&pool);

        // Construct instruction data: discriminator + args
        let mut instruction_data = discriminator;
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

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&*self.payer],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    /// Initialize user stake using raw instruction data
    pub async fn initialize_user_stake_raw(&self, pool: Pubkey, user: Pubkey) -> Result<String> {
        let discriminator = self.get_instruction_discriminator("initialize_user_stake")?;
        let (user_stake, _) = self.derive_user_stake_pda(&pool, &user);

        let instruction_data = discriminator; // No additional args for this instruction

        let accounts = vec![
            AccountMeta::new(user_stake, false),
            AccountMeta::new_readonly(pool, false),
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ];

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&*self.payer],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    /// Stake tokens using raw instruction data
    pub async fn stake_raw(
        &self,
        pool: Pubkey,
        user: Pubkey,
        user_token_account: Pubkey,
        amount: u64,
    ) -> Result<String> {
        let discriminator = self.get_instruction_discriminator("stake")?;
        let (user_stake, _) = self.derive_user_stake_pda(&pool, &user);
        let (staking_vault, _) = self.derive_staking_vault_pda(&pool);

        let mut instruction_data = discriminator;
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(pool, false),
            AccountMeta::new(user_stake, false),
            AccountMeta::new(user_token_account, false),
            AccountMeta::new(staking_vault, false),
            AccountMeta::new_readonly(user, true),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&*self.payer],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    /// Unstake tokens using raw instruction data
    pub async fn unstake_raw(
        &self,
        pool: Pubkey,
        user: Pubkey,
        user_token_account: Pubkey,
        amount: u64,
    ) -> Result<String> {
        let discriminator = self.get_instruction_discriminator("unstake")?;
        let (user_stake, _) = self.derive_user_stake_pda(&pool, &user);
        let (staking_vault, _) = self.derive_staking_vault_pda(&pool);

        let mut instruction_data = discriminator;
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            AccountMeta::new(pool, false),
            AccountMeta::new(user_stake, false),
            AccountMeta::new(user_token_account, false),
            AccountMeta::new(staking_vault, false),
            AccountMeta::new_readonly(user, true),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&*self.payer],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    /// Claim rewards using raw instruction data
    pub async fn claim_reward_raw(
        &self,
        pool: Pubkey,
        user: Pubkey,
        user_reward_account: Pubkey,
    ) -> Result<String> {
        let discriminator = self.get_instruction_discriminator("claim_reward")?;
        let (user_stake, _) = self.derive_user_stake_pda(&pool, &user);
        let (reward_vault, _) = self.derive_reward_vault_pda(&pool);

        let instruction_data = discriminator; // No additional args

        let accounts = vec![
            AccountMeta::new(pool, false),
            AccountMeta::new(user_stake, false),
            AccountMeta::new(user_reward_account, false),
            AccountMeta::new(reward_vault, false),
            AccountMeta::new_readonly(user, true),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&*self.payer],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    /// Fund reward pool using raw instruction data
    pub async fn fund_reward_pool_raw(
        &self,
        pool: Pubkey,
        funder: Pubkey,
        funder_token_account: Pubkey,
        amount: u64,
    ) -> Result<String> {
        let discriminator = self.get_instruction_discriminator("fund_reward_pool")?;
        let (reward_vault, _) = self.derive_reward_vault_pda(&pool);

        let mut instruction_data = discriminator;
        instruction_data.extend_from_slice(&amount.to_le_bytes());

        let accounts = vec![
            AccountMeta::new_readonly(pool, false),
            AccountMeta::new(funder_token_account, false),
            AccountMeta::new(reward_vault, false),
            AccountMeta::new_readonly(funder, true),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        let instruction = Instruction {
            program_id: self.program_id,
            accounts,
            data: instruction_data,
        };

        let recent_blockhash = self.client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&*self.payer],
            recent_blockhash,
        );

        let signature = self.client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    /// Display IDL information
    pub fn print_idl_info(&self) {
        println!("Program ID: {}", self.idl["address"].as_str().unwrap_or("Unknown"));
        println!("Program Name: {}", self.idl["metadata"]["name"].as_str().unwrap_or("Unknown"));
        
        println!("\nInstructions:");
        if let Some(instructions) = self.idl["instructions"].as_array() {
            for instruction in instructions {
                let name = instruction["name"].as_str().unwrap_or("Unknown");
                let discriminator: Vec<u8> = instruction["discriminator"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|v| v.as_u64().unwrap_or(0) as u8)
                    .collect();
                println!("  - {}: {:?}", name, discriminator);
            }
        }

        println!("\nAccount Types:");
        if let Some(accounts) = self.idl["accounts"].as_array() {
            for account in accounts {
                let name = account["name"].as_str().unwrap_or("Unknown");
                let discriminator: Vec<u8> = account["discriminator"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|v| v.as_u64().unwrap_or(0) as u8)
                    .collect();
                println!("  - {}: {:?}", name, discriminator);
            }
        }

        println!("\nEvents:");
        if let Some(events) = self.idl["events"].as_array() {
            for event in events {
                let name = event["name"].as_str().unwrap_or("Unknown");
                let discriminator: Vec<u8> = event["discriminator"]
                    .as_array()
                    .unwrap_or(&Vec::new())
                    .iter()
                    .map(|v| v.as_u64().unwrap_or(0) as u8)
                    .collect();
                println!("  - {}: {:?}", name, discriminator);
            }
        }

        println!("\nErrors:");
        if let Some(errors) = self.idl["errors"].as_array() {
            for error in errors {
                let code = error["code"].as_u64().unwrap_or(0);
                let name = error["name"].as_str().unwrap_or("Unknown");
                let msg = error["msg"].as_str().unwrap_or("No message");
                println!("  - {}: {} ({})", code, name, msg);
            }
        }
    }
}