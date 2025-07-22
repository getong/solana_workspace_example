use anchor_lang::prelude::*;

declare_id!("FPQBMD5Q5fRTqRt3ttb461VnzESGmJDHanas7VFgos9W");

#[program]
pub mod stake_program_project {
  use super::*;

  pub fn initialize_stake_pool(
    ctx: Context<InitializeStakePool>,
    reward_rate: u64,
    min_stake_amount: u64,
  ) -> Result<()> {
    let stake_pool = &mut ctx.accounts.stake_pool;
    stake_pool.authority = ctx.accounts.authority.key();
    stake_pool.reward_rate = reward_rate;
    stake_pool.min_stake_amount = min_stake_amount;
    stake_pool.total_staked = 0;
    stake_pool.reward_per_token_stored = 0;
    stake_pool.last_update_time = Clock::get()?.unix_timestamp;
    Ok(())
  }

  pub fn create_user_stake(ctx: Context<CreateUserStake>) -> Result<()> {
    let user_stake = &mut ctx.accounts.user_stake;
    user_stake.amount = 0;
    user_stake.reward_per_token_paid = 0;
    user_stake.reward_debt = 0;
    user_stake.last_stake_time = Clock::get()?.unix_timestamp;
    Ok(())
  }

  pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(
      amount >= ctx.accounts.stake_pool.min_stake_amount,
      StakeError::InsufficientAmount
    );

    let clock = Clock::get()?;
    let user_key = ctx.accounts.user.key();

    {
      let stake_pool = &mut ctx.accounts.stake_pool;
      let user_stake = &mut ctx.accounts.user_stake;

      update_rewards(stake_pool, clock.unix_timestamp)?;

      if user_stake.amount > 0 {
        let pending_rewards = calculate_pending_rewards(user_stake, stake_pool)?;
        user_stake.reward_debt += pending_rewards;
      }
    }

    let cpi_accounts = anchor_lang::system_program::Transfer {
      from: ctx.accounts.user.to_account_info(),
      to: ctx.accounts.stake_pool.to_account_info(),
    };
    let cpi_program = ctx.accounts.system_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    anchor_lang::system_program::transfer(cpi_ctx, amount)?;

    let stake_pool = &mut ctx.accounts.stake_pool;
    let user_stake = &mut ctx.accounts.user_stake;

    user_stake.amount += amount;
    user_stake.reward_per_token_paid = stake_pool.reward_per_token_stored;
    user_stake.last_stake_time = clock.unix_timestamp;

    stake_pool.total_staked += amount;

    emit!(StakeEvent {
      user: user_key,
      amount,
      total_staked: user_stake.amount,
    });

    Ok(())
  }

  pub fn unstake(ctx: Context<Unstake>, amount: u64) -> Result<()> {
    let clock = Clock::get()?;
    let user_key = ctx.accounts.user.key();

    {
      let stake_pool = &mut ctx.accounts.stake_pool;
      let user_stake = &mut ctx.accounts.user_stake;

      require!(user_stake.amount >= amount, StakeError::InsufficientStake);

      update_rewards(stake_pool, clock.unix_timestamp)?;

      let pending_rewards = calculate_pending_rewards(user_stake, stake_pool)?;
      user_stake.reward_debt += pending_rewards;
    }

    let stake_pool_lamports = ctx.accounts.stake_pool.to_account_info().lamports();
    **ctx
      .accounts
      .stake_pool
      .to_account_info()
      .try_borrow_mut_lamports()? = stake_pool_lamports - amount;
    let user_lamports = ctx.accounts.user.to_account_info().lamports();
    **ctx
      .accounts
      .user
      .to_account_info()
      .try_borrow_mut_lamports()? = user_lamports + amount;

    let stake_pool = &mut ctx.accounts.stake_pool;
    let user_stake = &mut ctx.accounts.user_stake;

    user_stake.amount -= amount;
    user_stake.reward_per_token_paid = stake_pool.reward_per_token_stored;

    stake_pool.total_staked -= amount;

    emit!(UnstakeEvent {
      user: user_key,
      amount,
      remaining_staked: user_stake.amount,
    });

    Ok(())
  }

  pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let clock = Clock::get()?;
    let user_key = ctx.accounts.user.key();

    let total_rewards = {
      let stake_pool = &mut ctx.accounts.stake_pool;
      let user_stake = &mut ctx.accounts.user_stake;

      update_rewards(stake_pool, clock.unix_timestamp)?;

      let pending_rewards = calculate_pending_rewards(user_stake, stake_pool)?;
      let total_rewards = user_stake.reward_debt + pending_rewards;

      require!(total_rewards > 0, StakeError::NoRewards);

      total_rewards
    };

    let stake_pool_lamports = ctx.accounts.stake_pool.to_account_info().lamports();
    **ctx
      .accounts
      .stake_pool
      .to_account_info()
      .try_borrow_mut_lamports()? = stake_pool_lamports - total_rewards;
    let user_lamports = ctx.accounts.user.to_account_info().lamports();
    **ctx
      .accounts
      .user
      .to_account_info()
      .try_borrow_mut_lamports()? = user_lamports + total_rewards;

    let stake_pool = &mut ctx.accounts.stake_pool;
    let user_stake = &mut ctx.accounts.user_stake;

    user_stake.reward_debt = 0;
    user_stake.reward_per_token_paid = stake_pool.reward_per_token_stored;

    emit!(ClaimRewardsEvent {
      user: user_key,
      amount: total_rewards,
    });

    Ok(())
  }
}

fn update_rewards(stake_pool: &mut StakePool, current_time: i64) -> Result<()> {
  if stake_pool.total_staked == 0 {
    stake_pool.last_update_time = current_time;
    return Ok(());
  }

  let time_diff = current_time - stake_pool.last_update_time;
  let reward_to_distribute = (stake_pool.reward_rate as u128)
    .checked_mul(time_diff as u128)
    .unwrap()
    .checked_div(stake_pool.total_staked as u128)
    .unwrap() as u64;

  stake_pool.reward_per_token_stored += reward_to_distribute;
  stake_pool.last_update_time = current_time;

  Ok(())
}

fn calculate_pending_rewards(user_stake: &UserStake, stake_pool: &StakePool) -> Result<u64> {
  let reward_per_token_diff = stake_pool.reward_per_token_stored - user_stake.reward_per_token_paid;
  let pending = (user_stake.amount as u128)
    .checked_mul(reward_per_token_diff as u128)
    .unwrap()
    .checked_div(1_000_000_000u128)
    .unwrap() as u64;

  Ok(pending)
}

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

#[derive(Accounts)]
pub struct Unstake<'info> {
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
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
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
  const LEN: usize = 32 + 8 + 8 + 8 + 8 + 8;
}

#[account]
pub struct UserStake {
  pub amount: u64,
  pub reward_per_token_paid: u64,
  pub reward_debt: u64,
  pub last_stake_time: i64,
}

impl UserStake {
  const LEN: usize = 8 + 8 + 8 + 8;
}

#[event]
pub struct StakeEvent {
  pub user: Pubkey,
  pub amount: u64,
  pub total_staked: u64,
}

#[event]
pub struct UnstakeEvent {
  pub user: Pubkey,
  pub amount: u64,
  pub remaining_staked: u64,
}

#[event]
pub struct ClaimRewardsEvent {
  pub user: Pubkey,
  pub amount: u64,
}

#[error_code]
pub enum StakeError {
  #[msg("Insufficient amount to stake")]
  InsufficientAmount,
  #[msg("Insufficient staked amount")]
  InsufficientStake,
  #[msg("No rewards available")]
  NoRewards,
}
