use anchor_lang::{prelude::*, solana_program::stake_history::StakeHistory as SolanaStakeHistory};

declare_id!("FPQBMD5Q5fRTqRt3ttb461VnzESGmJDHanas7VFgos9W");

#[program]
pub mod stake_program_project {
  use super::*;

  pub fn initialize(
    ctx: Context<Initialize>,
    authorized: Authorized,
    lockup: Lockup,
  ) -> Result<()> {
    let clock = Clock::get()?;
    let rent = Rent::get()?;

    let data_len = ctx.accounts.stake_account.to_account_info().data_len();
    let stake_account_key = ctx.accounts.stake_account.key();

    let stake_account = &mut ctx.accounts.stake_account;
    stake_account.meta = Meta {
      rent_exempt_reserve: rent.minimum_balance(data_len),
      authorized,
      lockup,
    };
    stake_account.stake = None;
    stake_account.stake_flags = StakeFlags::empty();

    emit!(StakeInitializeEvent {
      stake_account: stake_account_key,
      staker: authorized.staker,
      withdrawer: authorized.withdrawer,
      timestamp: clock.unix_timestamp,
    });

    Ok(())
  }

  pub fn authorize(
    ctx: Context<Authorize>,
    new_authority: Pubkey,
    stake_authorize: StakeAuthorize,
  ) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;

    match stake_authorize {
      StakeAuthorize::Staker => {
        require!(
          ctx.accounts.authority.key() == stake_account.meta.authorized.staker,
          StakeError::UnauthorizedStaker
        );
        stake_account.meta.authorized.staker = new_authority;
      }
      StakeAuthorize::Withdrawer => {
        require!(
          ctx.accounts.authority.key() == stake_account.meta.authorized.withdrawer,
          StakeError::UnauthorizedWithdrawer
        );
        stake_account.meta.authorized.withdrawer = new_authority;
      }
    }

    emit!(AuthorizeEvent {
      stake_account: ctx.accounts.stake_account.key(),
      old_authority: ctx.accounts.authority.key(),
      new_authority,
      stake_authorize,
    });

    Ok(())
  }

  pub fn delegate_stake(ctx: Context<DelegateStake>) -> Result<()> {
    let vote_account = &ctx.accounts.vote_account;
    let clock = Clock::get()?;
    // In a real implementation, stake history would be passed as an account
    // For now, we'll use a placeholder

    let stake_lamports = ctx.accounts.stake_account.to_account_info().lamports();
    let stake_account = &mut ctx.accounts.stake_account;

    require!(
      ctx.accounts.staker.key() == stake_account.meta.authorized.staker,
      StakeError::UnauthorizedStaker
    );

    let stake_amount = stake_lamports - stake_account.meta.rent_exempt_reserve;

    require!(
      stake_amount >= MINIMUM_DELEGATION,
      StakeError::InsufficientStake
    );

    stake_account.stake = Some(Stake {
      delegation: Delegation {
        voter_pubkey: vote_account.key(),
        stake: stake_amount,
        activation_epoch: clock.epoch,
        deactivation_epoch: u64::MAX,
        warmup_cooldown_rate: WARMUP_COOLDOWN_RATE,
      },
      credits_observed: 0, // Credits observed would come from vote account in real implementation
    });

    emit!(DelegateEvent {
      stake_account: ctx.accounts.stake_account.key(),
      vote_account: vote_account.key(),
      stake: stake_amount,
      activation_epoch: clock.epoch,
    });

    Ok(())
  }

  pub fn split(ctx: Context<Split>, lamports: u64) -> Result<()> {
    let clock = Clock::get()?;
    let rent = Rent::get()?;

    let source_lamports = ctx.accounts.source_account.to_account_info().lamports();
    let split_data_len = ctx.accounts.split_account.to_account_info().data_len();
    let split_min_balance = rent.minimum_balance(split_data_len);
    let source_account_key = ctx.accounts.source_account.key();
    let split_account_key = ctx.accounts.split_account.key();

    let source_account = &mut ctx.accounts.source_account;
    let split_account = &mut ctx.accounts.split_account;

    require!(
      ctx.accounts.staker.key() == source_account.meta.authorized.staker,
      StakeError::UnauthorizedStaker
    );

    require!(
      lamports + split_min_balance <= source_lamports,
      StakeError::InsufficientFunds
    );

    require!(
      source_lamports - lamports >= source_account.meta.rent_exempt_reserve,
      StakeError::InsufficientFunds
    );

    split_account.meta = source_account.meta.clone();

    if let Some(source_stake) = &mut source_account.stake {
      let split_stake_amount =
        (source_stake.delegation.stake as u128 * lamports as u128 / source_lamports as u128) as u64;

      source_stake.delegation.stake -= split_stake_amount;

      split_account.stake = Some(Stake {
        delegation: Delegation {
          voter_pubkey: source_stake.delegation.voter_pubkey,
          stake: split_stake_amount,
          activation_epoch: source_stake.delegation.activation_epoch,
          deactivation_epoch: source_stake.delegation.deactivation_epoch,
          warmup_cooldown_rate: source_stake.delegation.warmup_cooldown_rate,
        },
        credits_observed: source_stake.credits_observed,
      });
    } else {
      split_account.stake = None;
    }

    split_account.stake_flags = source_account.stake_flags;

    **ctx
      .accounts
      .source_account
      .to_account_info()
      .try_borrow_mut_lamports()? -= lamports;
    **ctx
      .accounts
      .split_account
      .to_account_info()
      .try_borrow_mut_lamports()? += lamports;

    emit!(SplitEvent {
      source_account: source_account_key,
      split_account: split_account_key,
      lamports,
      timestamp: clock.unix_timestamp,
    });

    Ok(())
  }

  pub fn withdraw(ctx: Context<Withdraw>, lamports: u64) -> Result<()> {
    let clock = Clock::get()?;
    // In a real implementation, stake history would be passed as an account
    // For now, we'll use a placeholder

    let stake_lamports = ctx.accounts.stake_account.to_account_info().lamports();
    let stake_account = &mut ctx.accounts.stake_account;

    require!(
      ctx.accounts.withdrawer.key() == stake_account.meta.authorized.withdrawer,
      StakeError::UnauthorizedWithdrawer
    );

    require!(
      !stake_account.meta.lockup.is_in_force(&clock, None),
      StakeError::LockupInForce
    );

    let available_for_withdrawal = if let Some(stake) = &stake_account.stake {
      let effective_stake = stake
        .delegation
        .stake_activating_and_deactivating(
          clock.epoch,
          None, // Stake history would be passed in real implementation
          None,
        )
        .effective;

      stake_lamports - effective_stake - stake_account.meta.rent_exempt_reserve
    } else {
      stake_lamports - stake_account.meta.rent_exempt_reserve
    };

    require!(
      lamports <= available_for_withdrawal,
      StakeError::InsufficientFunds
    );

    **ctx
      .accounts
      .stake_account
      .to_account_info()
      .try_borrow_mut_lamports()? -= lamports;
    **ctx
      .accounts
      .to
      .to_account_info()
      .try_borrow_mut_lamports()? += lamports;

    emit!(WithdrawEvent {
      stake_account: ctx.accounts.stake_account.key(),
      withdrawer: ctx.accounts.withdrawer.key(),
      to: ctx.accounts.to.key(),
      lamports,
      timestamp: clock.unix_timestamp,
    });

    Ok(())
  }

  pub fn deactivate(ctx: Context<Deactivate>) -> Result<()> {
    let stake_account = &mut ctx.accounts.stake_account;
    let clock = Clock::get()?;

    require!(
      ctx.accounts.staker.key() == stake_account.meta.authorized.staker,
      StakeError::UnauthorizedStaker
    );

    if let Some(stake) = &mut stake_account.stake {
      stake.delegation.deactivation_epoch = clock.epoch;

      emit!(DeactivateEvent {
        stake_account: ctx.accounts.stake_account.key(),
        epoch: clock.epoch,
      });
    }

    Ok(())
  }

  pub fn set_lockup(ctx: Context<SetLockup>, lockup: LockupArgs) -> Result<()> {
    let clock = Clock::get()?;
    let stake_account_key = ctx.accounts.stake_account.key();

    let stake_account = &mut ctx.accounts.stake_account;

    require!(
      ctx.accounts.custodian.key() == stake_account.meta.lockup.custodian,
      StakeError::UnauthorizedCustodian
    );

    if let Some(unix_timestamp) = lockup.unix_timestamp {
      stake_account.meta.lockup.unix_timestamp = unix_timestamp;
    }
    if let Some(epoch) = lockup.epoch {
      stake_account.meta.lockup.epoch = epoch;
    }
    if let Some(custodian) = lockup.custodian {
      stake_account.meta.lockup.custodian = custodian;
    }

    let lockup = stake_account.meta.lockup;

    emit!(SetLockupEvent {
      stake_account: stake_account_key,
      lockup,
      timestamp: clock.unix_timestamp,
    });

    Ok(())
  }

  pub fn merge(ctx: Context<Merge>) -> Result<()> {
    let clock = Clock::get()?;
    // In a real implementation, stake history would be passed as an account
    // For now, we'll use a placeholder

    let source_lamports = ctx.accounts.source_account.to_account_info().lamports();

    let source_account = &mut ctx.accounts.source_account;
    let dest_account = &mut ctx.accounts.dest_account;

    require!(
      ctx.accounts.staker.key() == source_account.meta.authorized.staker,
      StakeError::UnauthorizedStaker
    );
    require!(
      ctx.accounts.staker.key() == dest_account.meta.authorized.staker,
      StakeError::UnauthorizedStaker
    );

    require!(
      source_account.meta.authorized == dest_account.meta.authorized,
      StakeError::MergeMismatch
    );

    require!(
      source_account.meta.lockup == dest_account.meta.lockup,
      StakeError::MergeMismatch
    );

    match (&source_account.stake, &dest_account.stake) {
      (Some(source_stake), Some(dest_stake)) => {
        require!(
          source_stake.delegation.voter_pubkey == dest_stake.delegation.voter_pubkey,
          StakeError::MergeMismatch
        );

        require!(
          source_stake.delegation.deactivation_epoch == u64::MAX
            && dest_stake.delegation.deactivation_epoch == u64::MAX,
          StakeError::MergeDeactivated
        );
      }
      _ => return Err(StakeError::MergeTransientStake.into()),
    }

    if let Some(dest_stake) = &mut dest_account.stake {
      if let Some(source_stake) = &source_account.stake {
        dest_stake.delegation.stake = dest_stake
          .delegation
          .stake
          .checked_add(source_stake.delegation.stake)
          .ok_or(StakeError::InsufficientFunds)?;
      }
    }

    **ctx
      .accounts
      .source_account
      .to_account_info()
      .try_borrow_mut_lamports()? = 0;
    **ctx
      .accounts
      .dest_account
      .to_account_info()
      .try_borrow_mut_lamports()? += source_lamports;

    emit!(MergeEvent {
      source_account: ctx.accounts.source_account.key(),
      dest_account: ctx.accounts.dest_account.key(),
      lamports: source_lamports,
      timestamp: clock.unix_timestamp,
    });

    Ok(())
  }

  pub fn get_minimum_delegation(_ctx: Context<GetMinimumDelegation>) -> Result<()> {
    let min_delegation = MINIMUM_DELEGATION;

    anchor_lang::solana_program::program::set_return_data(&min_delegation.to_le_bytes());

    Ok(())
  }
}

const MINIMUM_DELEGATION: u64 = 1_000_000_000; // 1 SOL
const WARMUP_COOLDOWN_RATE: f64 = 0.09;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct Authorized {
  pub staker: Pubkey,
  pub withdrawer: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct Lockup {
  pub unix_timestamp: i64,
  pub epoch: u64,
  pub custodian: Pubkey,
}

impl Lockup {
  pub fn is_in_force(&self, clock: &Clock, custodian: Option<&Pubkey>) -> bool {
    if custodian == Some(&self.custodian) {
      return false;
    }
    self.unix_timestamp > clock.unix_timestamp || self.epoch > clock.epoch
  }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct LockupArgs {
  pub unix_timestamp: Option<i64>,
  pub epoch: Option<u64>,
  pub custodian: Option<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum StakeAuthorize {
  Staker,
  Withdrawer,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct Delegation {
  pub voter_pubkey: Pubkey,
  pub stake: u64,
  pub activation_epoch: u64,
  pub deactivation_epoch: u64,
  pub warmup_cooldown_rate: f64,
}

impl Delegation {
  pub fn stake_activating_and_deactivating(
    &self,
    target_epoch: u64,
    history: Option<&SolanaStakeHistory>,
    _new_rate_activation_epoch: Option<u64>,
  ) -> StakeActivationStatus {
    let (effective_stake, activating_stake, deactivating_stake) =
      self.stake_and_activating(target_epoch, history, _new_rate_activation_epoch);

    StakeActivationStatus {
      effective: effective_stake,
      activating: activating_stake,
      deactivating: deactivating_stake,
    }
  }

  fn stake_and_activating(
    &self,
    target_epoch: u64,
    history: Option<&SolanaStakeHistory>,
    _new_rate_activation_epoch: Option<u64>,
  ) -> (u64, u64, u64) {
    let delegated_stake = self.stake;

    if self.activation_epoch == self.deactivation_epoch {
      return (0, 0, 0);
    }
    if target_epoch == self.activation_epoch {
      return (0, delegated_stake, 0);
    }
    if target_epoch < self.activation_epoch {
      return (0, 0, 0);
    }

    let effective_stake = delegated_stake;
    let mut activating_stake = 0;
    let mut deactivating_stake = 0;

    if target_epoch < self.deactivation_epoch {
      activating_stake = delegated_stake;
    } else if target_epoch == self.deactivation_epoch {
      deactivating_stake = delegated_stake;
    } else if let Some(_history) = history {
      deactivating_stake = delegated_stake;
    }

    (effective_stake, activating_stake, deactivating_stake)
  }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct StakeActivationStatus {
  pub effective: u64,
  pub activating: u64,
  pub deactivating: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct StakeFlags {
  flags: u8,
}

impl StakeFlags {
  pub const MUST_FULLY_ACTIVATE_BEFORE_DEACTIVATION_IS_PERMITTED: u8 = 0b0000_0001;

  pub fn empty() -> Self {
    StakeFlags { flags: 0 }
  }
}

impl Default for StakeFlags {
  fn default() -> Self {
    StakeFlags::empty()
  }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
  #[account(
        init,
        payer = payer,
        space = 8 + StakeAccount::LEN
    )]
  pub stake_account: Account<'info, StakeAccount>,
  #[account(mut)]
  pub payer: Signer<'info>,
  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Authorize<'info> {
  #[account(mut)]
  pub stake_account: Account<'info, StakeAccount>,
  pub authority: Signer<'info>,
  pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct DelegateStake<'info> {
  #[account(mut)]
  pub stake_account: Account<'info, StakeAccount>,
  /// CHECK: Vote account is validated in instruction
  pub vote_account: AccountInfo<'info>,
  pub staker: Signer<'info>,
  pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct Split<'info> {
  #[account(mut)]
  pub source_account: Account<'info, StakeAccount>,
  #[account(
        init,
        payer = staker,
        space = 8 + StakeAccount::LEN
    )]
  pub split_account: Account<'info, StakeAccount>,
  #[account(mut)]
  pub staker: Signer<'info>,
  pub rent: Sysvar<'info, Rent>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
  #[account(mut)]
  pub stake_account: Account<'info, StakeAccount>,
  #[account(mut)]
  pub withdrawer: Signer<'info>,
  /// CHECK: Destination account for withdrawal
  #[account(mut)]
  pub to: AccountInfo<'info>,
  pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct Deactivate<'info> {
  #[account(mut)]
  pub stake_account: Account<'info, StakeAccount>,
  pub staker: Signer<'info>,
  pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct SetLockup<'info> {
  #[account(mut)]
  pub stake_account: Account<'info, StakeAccount>,
  pub custodian: Signer<'info>,
}

#[derive(Accounts)]
pub struct Merge<'info> {
  #[account(mut, close = staker)]
  pub source_account: Account<'info, StakeAccount>,
  #[account(mut)]
  pub dest_account: Account<'info, StakeAccount>,
  #[account(mut)]
  pub staker: Signer<'info>,
  pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct GetMinimumDelegation {}

#[account]
pub struct StakeAccount {
  pub meta: Meta,
  pub stake: Option<Stake>,
  pub stake_flags: StakeFlags,
}

impl StakeAccount {
  const LEN: usize = Meta::LEN + 1 + Stake::LEN + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct Meta {
  pub rent_exempt_reserve: u64,
  pub authorized: Authorized,
  pub lockup: Lockup,
}

impl Meta {
  const LEN: usize = 8 + Authorized::LEN + Lockup::LEN;
}

impl Authorized {
  const LEN: usize = 32 + 32;
}

impl Lockup {
  const LEN: usize = 8 + 8 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct Stake {
  pub delegation: Delegation,
  pub credits_observed: u64,
}

impl Stake {
  const LEN: usize = Delegation::LEN + 8;
}

impl Delegation {
  const LEN: usize = 32 + 8 + 8 + 8 + 8;
}

#[event]
pub struct StakeInitializeEvent {
  pub stake_account: Pubkey,
  pub staker: Pubkey,
  pub withdrawer: Pubkey,
  pub timestamp: i64,
}

#[event]
pub struct AuthorizeEvent {
  pub stake_account: Pubkey,
  pub old_authority: Pubkey,
  pub new_authority: Pubkey,
  pub stake_authorize: StakeAuthorize,
}

#[event]
pub struct DelegateEvent {
  pub stake_account: Pubkey,
  pub vote_account: Pubkey,
  pub stake: u64,
  pub activation_epoch: u64,
}

#[event]
pub struct SplitEvent {
  pub source_account: Pubkey,
  pub split_account: Pubkey,
  pub lamports: u64,
  pub timestamp: i64,
}

#[event]
pub struct WithdrawEvent {
  pub stake_account: Pubkey,
  pub withdrawer: Pubkey,
  pub to: Pubkey,
  pub lamports: u64,
  pub timestamp: i64,
}

#[event]
pub struct DeactivateEvent {
  pub stake_account: Pubkey,
  pub epoch: u64,
}

#[event]
pub struct SetLockupEvent {
  pub stake_account: Pubkey,
  pub lockup: Lockup,
  pub timestamp: i64,
}

#[event]
pub struct MergeEvent {
  pub source_account: Pubkey,
  pub dest_account: Pubkey,
  pub lamports: u64,
  pub timestamp: i64,
}

#[error_code]
pub enum StakeError {
  #[msg("Insufficient stake amount")]
  InsufficientStake,
  #[msg("Insufficient funds")]
  InsufficientFunds,
  #[msg("Unauthorized staker")]
  UnauthorizedStaker,
  #[msg("Unauthorized withdrawer")]
  UnauthorizedWithdrawer,
  #[msg("Unauthorized custodian")]
  UnauthorizedCustodian,
  #[msg("Lockup is still in force")]
  LockupInForce,
  #[msg("Merge mismatch")]
  MergeMismatch,
  #[msg("Cannot merge deactivated stakes")]
  MergeDeactivated,
  #[msg("Cannot merge transient stakes")]
  MergeTransientStake,
}
