use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::{Keypair, Signer};
use anchor_client::solana_sdk::system_program;
use anchor_client::{Client, Cluster};
use anyhow::Result;
use std::rc::Rc;
use std::str::FromStr;

use anchor_client::anchor_lang::declare_id;

declare_id!("FPQBMD5Q5fRTqRt3ttb461VnzESGmJDHanas7VFgos9W");

#[tokio::main]
async fn main() -> Result<()> {
    let payer = Keypair::new();
    let client = Client::new_with_options(
        Cluster::Localnet,
        Rc::new(payer),
        CommitmentConfig::processed(),
    );
    
    let program = client.program(ID)?;
    
    println!("Running Stake Program Client");
    
    let stake_pool_pda = Pubkey::find_program_address(&[b"stake_pool"], &ID).0;
    let authority = Keypair::new();
    
    println!("Initializing stake pool...");
    let tx = program
        .request()
        .accounts(stake_program_project::accounts::InitializeStakePool {
            stake_pool: stake_pool_pda,
            authority: authority.pubkey(),
            system_program: system_program::id(),
        })
        .args(stake_program_project::instruction::InitializeStakePool {
            reward_rate: 1000,
            min_stake_amount: 1000000,
        })
        .signer(&authority)
        .send()?;
    
    println!("Initialize stake pool transaction: {}", tx);
    
    let user = Keypair::new();
    let user_stake_pda = Pubkey::find_program_address(&[b"user_stake", user.pubkey().as_ref()], &ID).0;
    
    println!("Creating user stake account...");
    let tx = program
        .request()
        .accounts(stake_program_project::accounts::CreateUserStake {
            user_stake: user_stake_pda,
            user: user.pubkey(),
            system_program: system_program::id(),
        })
        .args(stake_program_project::instruction::CreateUserStake {})
        .signer(&user)
        .send()?;
    
    println!("Create user stake transaction: {}", tx);
    
    println!("Staking tokens...");
    let tx = program
        .request()
        .accounts(stake_program_project::accounts::Stake {
            stake_pool: stake_pool_pda,
            user_stake: user_stake_pda,
            user: user.pubkey(),
            system_program: system_program::id(),
        })
        .args(stake_program_project::instruction::Stake {
            amount: 5000000,
        })
        .signer(&user)
        .send()?;
    
    println!("Stake transaction: {}", tx);
    
    println!("Getting stake pool account data...");
    let stake_pool_account: stake_program_project::StakePool = program.account(stake_pool_pda)?;
    println!("Stake Pool:");
    println!("  Authority: {}", stake_pool_account.authority);
    println!("  Reward Rate: {}", stake_pool_account.reward_rate);
    println!("  Min Stake Amount: {}", stake_pool_account.min_stake_amount);
    println!("  Total Staked: {}", stake_pool_account.total_staked);
    
    println!("Getting user stake account data...");
    let user_stake_account: stake_program_project::UserStake = program.account(user_stake_pda)?;
    println!("User Stake:");
    println!("  Amount: {}", user_stake_account.amount);
    println!("  Reward Per Token Paid: {}", user_stake_account.reward_per_token_paid);
    println!("  Reward Debt: {}", user_stake_account.reward_debt);
    println!("  Last Stake Time: {}", user_stake_account.last_stake_time);
    
    Ok(())
}

pub mod stake_program_project {
    use anchor_client::anchor_lang::prelude::*;

    #[derive(Accounts)]
    pub struct InitializeStakePool<'info> {
        #[account(
            init,
            payer = authority,
            space = 8 + StakePool::LEN,
            seeds = [b"stake_pool"],
            bump
        )]
        pub stake_pool: Account<'info, StakePool>,
        #[account(mut)]
        pub authority: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    pub struct CreateUserStake<'info> {
        #[account(
            init,
            payer = user,
            space = 8 + UserStake::LEN,
            seeds = [b"user_stake", user.key().as_ref()],
            bump
        )]
        pub user_stake: Account<'info, UserStake>,
        #[account(mut)]
        pub user: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    pub struct Stake<'info> {
        #[account(mut)]
        pub stake_pool: Account<'info, StakePool>,
        #[account(
            mut,
            seeds = [b"user_stake", user.key().as_ref()],
            bump
        )]
        pub user_stake: Account<'info, UserStake>,
        #[account(mut)]
        pub user: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    #[account]
    pub struct StakePool {
        pub authority: Pubkey,
        pub reward_rate: u64,
        pub min_stake_amount: u64,
        pub total_staked: u64,
        pub reward_per_token_stored: u64,
        pub last_update_time: i64,
    }

    impl StakePool {
        pub const LEN: usize = 32 + 8 + 8 + 8 + 8 + 8;
    }

    #[account]
    pub struct UserStake {
        pub amount: u64,
        pub reward_per_token_paid: u64,
        pub reward_debt: u64,
        pub last_stake_time: i64,
    }

    impl UserStake {
        pub const LEN: usize = 8 + 8 + 8 + 8;
    }

    pub mod instruction {
        use super::*;

        pub struct InitializeStakePool {
            pub reward_rate: u64,
            pub min_stake_amount: u64,
        }

        pub struct CreateUserStake {}

        pub struct Stake {
            pub amount: u64,
        }

        pub struct Unstake {
            pub amount: u64,
        }

        pub struct ClaimRewards {}
    }

    pub mod accounts {
        pub use super::{InitializeStakePool, CreateUserStake, Stake};
    }
}

