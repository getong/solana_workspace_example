use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

declare_id!("8Bv63d7LKuxYipEWycEHz263LKgq1qTwE6PaxsmE2Vmx");

#[program]
pub mod stake_program_example {
  use super::*;

  pub fn initialize_pool(
    ctx: Context<InitializePool>,
    reward_rate: u64,
    lock_period: i64,
  ) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.staking_mint = ctx.accounts.staking_mint.key();
    pool.reward_mint = ctx.accounts.reward_mint.key();
    pool.staking_vault = ctx.accounts.staking_vault.key();
    pool.reward_vault = ctx.accounts.reward_vault.key();
    pool.reward_rate = reward_rate;
    pool.lock_period = lock_period;
    pool.total_staked = 0;
    pool.last_update_time = Clock::get()?.unix_timestamp;
    pool.bump = ctx.bumps.pool;

    Ok(())
  }

  pub fn initialize_user_stake(ctx: Context<InitializeUserStake>) -> Result<()> {
    let user_stake = &mut ctx.accounts.user_stake;
    user_stake.owner = ctx.accounts.owner.key();
    user_stake.pool = ctx.accounts.pool.key();
    user_stake.staked_amount = 0;
    user_stake.reward_debt = 0;
    user_stake.last_stake_time = 0;
    user_stake.bump = ctx.bumps.user_stake;

    Ok(())
  }

  pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let user_stake = &mut ctx.accounts.user_stake;
    let clock = Clock::get()?;

    // Update pool rewards
    update_pool_rewards(pool, clock.unix_timestamp)?;

    // Calculate pending rewards for user
    let pending_reward = calculate_pending_reward(
      user_stake.staked_amount,
      pool.accumulated_reward_per_share,
      user_stake.reward_debt,
    );

    // Transfer tokens from user to vault
    let cpi_accounts = token::Transfer {
      from: ctx.accounts.user_token_account.to_account_info(),
      to: ctx.accounts.staking_vault.to_account_info(),
      authority: ctx.accounts.owner.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Update user stake
    user_stake.staked_amount = user_stake
      .staked_amount
      .checked_add(amount)
      .ok_or(ErrorCode::Overflow)?;
    user_stake.reward_debt =
      calculate_reward_debt(user_stake.staked_amount, pool.accumulated_reward_per_share);
    user_stake.pending_reward = user_stake
      .pending_reward
      .checked_add(pending_reward)
      .ok_or(ErrorCode::Overflow)?;
    user_stake.last_stake_time = clock.unix_timestamp;

    // Update pool
    pool.total_staked = pool
      .total_staked
      .checked_add(amount)
      .ok_or(ErrorCode::Overflow)?;

    emit!(StakeEvent {
      user: ctx.accounts.owner.key(),
      amount,
      timestamp: clock.unix_timestamp,
    });

    Ok(())
  }

  pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    let clock = Clock::get()?;

    // Check lock period
    require!(
      clock.unix_timestamp
        >= ctx.accounts.user_stake.last_stake_time + ctx.accounts.pool.lock_period,
      ErrorCode::StillLocked
    );

    // Check sufficient balance
    require!(
      ctx.accounts.user_stake.staked_amount >= amount,
      ErrorCode::InsufficientBalance
    );

    // Update pool rewards
    update_pool_rewards(&mut ctx.accounts.pool, clock.unix_timestamp)?;

    // Calculate pending rewards
    let pending_reward = calculate_pending_reward(
      ctx.accounts.user_stake.staked_amount,
      ctx.accounts.pool.accumulated_reward_per_share,
      ctx.accounts.user_stake.reward_debt,
    );

    // Transfer tokens from vault to user
    let pool_seeds = &[
      b"pool",
      ctx.accounts.pool.staking_mint.as_ref(),
      &[ctx.accounts.pool.bump],
    ];
    let signer_seeds = &[&pool_seeds[..]];

    let cpi_accounts = token::Transfer {
      from: ctx.accounts.staking_vault.to_account_info(),
      to: ctx.accounts.user_token_account.to_account_info(),
      authority: ctx.accounts.pool.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    token::transfer(cpi_ctx, amount)?;

    // Update user stake
    ctx.accounts.user_stake.staked_amount = ctx
      .accounts
      .user_stake
      .staked_amount
      .checked_sub(amount)
      .ok_or(ErrorCode::Underflow)?;
    ctx.accounts.user_stake.reward_debt = calculate_reward_debt(
      ctx.accounts.user_stake.staked_amount,
      ctx.accounts.pool.accumulated_reward_per_share,
    );
    ctx.accounts.user_stake.pending_reward = ctx
      .accounts
      .user_stake
      .pending_reward
      .checked_add(pending_reward)
      .ok_or(ErrorCode::Overflow)?;

    // Update pool
    ctx.accounts.pool.total_staked = ctx
      .accounts
      .pool
      .total_staked
      .checked_sub(amount)
      .ok_or(ErrorCode::Underflow)?;

    emit!(UnstakeEvent {
      user: ctx.accounts.owner.key(),
      amount,
      timestamp: clock.unix_timestamp,
    });

    Ok(())
  }

  pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    let clock = Clock::get()?;

    // Update pool rewards
    update_pool_rewards(&mut ctx.accounts.pool, clock.unix_timestamp)?;

    // Calculate total rewards
    let pending_reward = calculate_pending_reward(
      ctx.accounts.user_stake.staked_amount,
      ctx.accounts.pool.accumulated_reward_per_share,
      ctx.accounts.user_stake.reward_debt,
    );
    let total_reward = ctx
      .accounts
      .user_stake
      .pending_reward
      .checked_add(pending_reward)
      .ok_or(ErrorCode::Overflow)?;

    require!(total_reward > 0, ErrorCode::NoRewardsToClaim);

    // Transfer rewards
    let pool_seeds = &[
      b"pool",
      ctx.accounts.pool.staking_mint.as_ref(),
      &[ctx.accounts.pool.bump],
    ];
    let signer_seeds = &[&pool_seeds[..]];

    let cpi_accounts = token::Transfer {
      from: ctx.accounts.reward_vault.to_account_info(),
      to: ctx.accounts.user_reward_account.to_account_info(),
      authority: ctx.accounts.pool.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
    token::transfer(cpi_ctx, total_reward)?;

    // Update user state
    ctx.accounts.user_stake.reward_debt = calculate_reward_debt(
      ctx.accounts.user_stake.staked_amount,
      ctx.accounts.pool.accumulated_reward_per_share,
    );
    ctx.accounts.user_stake.pending_reward = 0;

    emit!(ClaimRewardEvent {
      user: ctx.accounts.owner.key(),
      amount: total_reward,
      timestamp: clock.unix_timestamp,
    });

    Ok(())
  }

  pub fn fund_reward_pool(ctx: Context<FundRewardPool>, amount: u64) -> Result<()> {
    let cpi_accounts = token::Transfer {
      from: ctx.accounts.funder_token_account.to_account_info(),
      to: ctx.accounts.reward_vault.to_account_info(),
      authority: ctx.accounts.funder.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    Ok(())
  }
}

// Helper functions
fn update_pool_rewards(pool: &mut StakePool, current_time: i64) -> Result<()> {
  if pool.total_staked == 0 {
    pool.last_update_time = current_time;
    return Ok(());
  }

  let time_elapsed = current_time
    .checked_sub(pool.last_update_time)
    .ok_or(ErrorCode::Underflow)? as u64;

  let reward_amount = pool
    .reward_rate
    .checked_mul(time_elapsed)
    .ok_or(ErrorCode::Overflow)?;

  let reward_per_share = reward_amount
    .checked_mul(PRECISION)
    .ok_or(ErrorCode::Overflow)?
    .checked_div(pool.total_staked)
    .ok_or(ErrorCode::DivisionByZero)?;

  pool.accumulated_reward_per_share = pool
    .accumulated_reward_per_share
    .checked_add(reward_per_share)
    .ok_or(ErrorCode::Overflow)?;

  pool.last_update_time = current_time;

  Ok(())
}

fn calculate_pending_reward(
  staked_amount: u64,
  accumulated_reward_per_share: u64,
  reward_debt: u64,
) -> u64 {
  staked_amount
    .checked_mul(accumulated_reward_per_share)
    .unwrap_or(0)
    .checked_div(PRECISION)
    .unwrap_or(0)
    .saturating_sub(reward_debt)
}

fn calculate_reward_debt(staked_amount: u64, accumulated_reward_per_share: u64) -> u64 {
  staked_amount
    .checked_mul(accumulated_reward_per_share)
    .unwrap_or(0)
    .checked_div(PRECISION)
    .unwrap_or(0)
}

const PRECISION: u64 = 1_000_000;

// Contexts
#[derive(Accounts)]
pub struct InitializePool<'info> {
  #[account(
        init,
        payer = authority,
        space = 8 + StakePool::INIT_SPACE,
        seeds = [b"pool", staking_mint.key().as_ref()],
        bump
    )]
  pub pool: Account<'info, StakePool>,

  #[account(mut)]
  pub authority: Signer<'info>,

  pub staking_mint: Account<'info, Mint>,
  pub reward_mint: Account<'info, Mint>,

  #[account(
        init,
        payer = authority,
        token::mint = staking_mint,
        token::authority = pool,
        seeds = [b"staking_vault", pool.key().as_ref()],
        bump
    )]
  pub staking_vault: Account<'info, TokenAccount>,

  #[account(
        init,
        payer = authority,
        token::mint = reward_mint,
        token::authority = pool,
        seeds = [b"reward_vault", pool.key().as_ref()],
        bump
    )]
  pub reward_vault: Account<'info, TokenAccount>,

  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeUserStake<'info> {
  #[account(
        init,
        payer = owner,
        space = 8 + UserStake::INIT_SPACE,
        seeds = [b"user_stake", pool.key().as_ref(), owner.key().as_ref()],
        bump
    )]
  pub user_stake: Account<'info, UserStake>,

  pub pool: Account<'info, StakePool>,

  #[account(mut)]
  pub owner: Signer<'info>,

  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
  #[account(mut)]
  pub pool: Account<'info, StakePool>,

  #[account(
        mut,
        has_one = owner,
        has_one = pool,
    )]
  pub user_stake: Account<'info, UserStake>,

  #[account(
        mut,
        constraint = user_token_account.mint == pool.staking_mint,
        constraint = user_token_account.owner == owner.key(),
    )]
  pub user_token_account: Account<'info, TokenAccount>,

  #[account(
        mut,
        constraint = staking_vault.key() == pool.staking_vault,
    )]
  pub staking_vault: Account<'info, TokenAccount>,

  pub owner: Signer<'info>,
  pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
  #[account(mut)]
  pub pool: Account<'info, StakePool>,

  #[account(
        mut,
        has_one = owner,
        has_one = pool,
    )]
  pub user_stake: Account<'info, UserStake>,

  #[account(
        mut,
        constraint = user_token_account.mint == pool.staking_mint,
        constraint = user_token_account.owner == owner.key(),
    )]
  pub user_token_account: Account<'info, TokenAccount>,

  #[account(
        mut,
        constraint = staking_vault.key() == pool.staking_vault,
    )]
  pub staking_vault: Account<'info, TokenAccount>,

  pub owner: Signer<'info>,
  pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
  #[account(mut)]
  pub pool: Account<'info, StakePool>,

  #[account(
        mut,
        has_one = owner,
        has_one = pool,
    )]
  pub user_stake: Account<'info, UserStake>,

  #[account(
        mut,
        constraint = user_reward_account.mint == pool.reward_mint,
        constraint = user_reward_account.owner == owner.key(),
    )]
  pub user_reward_account: Account<'info, TokenAccount>,

  #[account(
        mut,
        constraint = reward_vault.key() == pool.reward_vault,
    )]
  pub reward_vault: Account<'info, TokenAccount>,

  pub owner: Signer<'info>,
  pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct FundRewardPool<'info> {
  pub pool: Account<'info, StakePool>,

  #[account(
        mut,
        constraint = funder_token_account.mint == pool.reward_mint,
    )]
  pub funder_token_account: Account<'info, TokenAccount>,

  #[account(
        mut,
        constraint = reward_vault.key() == pool.reward_vault,
    )]
  pub reward_vault: Account<'info, TokenAccount>,

  pub funder: Signer<'info>,
  pub token_program: Program<'info, Token>,
}

// State
#[account]
#[derive(InitSpace)]
pub struct StakePool {
  pub authority: Pubkey,
  pub staking_mint: Pubkey,
  pub reward_mint: Pubkey,
  pub staking_vault: Pubkey,
  pub reward_vault: Pubkey,
  pub reward_rate: u64,
  pub lock_period: i64,
  pub total_staked: u64,
  pub accumulated_reward_per_share: u64,
  pub last_update_time: i64,
  pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserStake {
  pub owner: Pubkey,
  pub pool: Pubkey,
  pub staked_amount: u64,
  pub reward_debt: u64,
  pub pending_reward: u64,
  pub last_stake_time: i64,
  pub bump: u8,
}

// Events
#[event]
pub struct StakeEvent {
  pub user: Pubkey,
  pub amount: u64,
  pub timestamp: i64,
}

#[event]
pub struct UnstakeEvent {
  pub user: Pubkey,
  pub amount: u64,
  pub timestamp: i64,
}

#[event]
pub struct ClaimRewardEvent {
  pub user: Pubkey,
  pub amount: u64,
  pub timestamp: i64,
}

// Errors
#[error_code]
pub enum ErrorCode {
  #[msg("Arithmetic overflow")]
  Overflow,
  #[msg("Arithmetic underflow")]
  Underflow,
  #[msg("Division by zero")]
  DivisionByZero,
  #[msg("Insufficient balance")]
  InsufficientBalance,
  #[msg("Tokens are still locked")]
  StillLocked,
  #[msg("No rewards to claim")]
  NoRewardsToClaim,
}
