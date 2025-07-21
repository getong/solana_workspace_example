use anchor_lang::prelude::*;

declare_id!("CwvrwTvvUhyQu1TFuBthxyEbMyk8W6iNA6xWH1AammPb");

#[program]
pub mod counter {
  use super::*;

  pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let counter = &ctx.accounts.counter;
    msg!("Counter account created! Current count: {}", counter.count);
    Ok(())
  }

  pub fn increment(ctx: Context<Increment>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    msg!("Previous counter: {}", counter.count);

    counter.count = counter.count.checked_add(1).unwrap();
    msg!("Counter incremented! Current count: {}", counter.count);
    Ok(())
  }

  pub fn decrement(ctx: Context<Decrement>) -> Result<()> {
    let counter = &mut ctx.accounts.counter;
    msg!("Previous counter: {}", counter.count);

    counter.count = counter.count.checked_sub(1).unwrap();
    msg!("Counter decremented! Current count: {}", counter.count);
    Ok(())
  }

  pub fn get_count(ctx: Context<GetCount>) -> Result<()> {
    let counter = &ctx.accounts.counter;
    msg!("Current counter value: {}", counter.count);
    Ok(())
  }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
        init,
        payer = payer,
        space = 8 + 8,
        seeds = [b"counter", payer.key().as_ref()],
        bump
    )]
  pub counter: Account<'info, Counter>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
  #[account(
    mut,
    seeds = [b"counter", payer.key().as_ref()],
    bump
  )]
  pub counter: Account<'info, Counter>,

  pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct Decrement<'info> {
  #[account(
    mut,
    seeds = [b"counter", payer.key().as_ref()],
    bump
  )]
  pub counter: Account<'info, Counter>,

  pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetCount<'info> {
  #[account(
    seeds = [b"counter", payer.key().as_ref()],
    bump
  )]
  pub counter: Account<'info, Counter>,

  pub payer: Signer<'info>,
}

#[account]
pub struct Counter {
  pub count: u64,
}
