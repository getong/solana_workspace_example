use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct TodoState {
  pub key: Pubkey,
  pub bump: u8,
  #[max_len(10, 5)]
  pub todos: Vec<Todo>,
  pub total_todos: u64,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug, InitSpace)]
pub struct Todo {
  #[max_len(50)]
  pub description: String,
  pub is_completed: bool,
}

#[derive(Accounts)]
pub struct InitializaPda<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,

  #[account(
        init,
        seeds=[b"TODO_ACC", signer.key().as_ref()],
        payer=signer,
        bump,
        space=8+TodoState::INIT_SPACE,
    )]
  pub todo_account: Account<'info, TodoState>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddTodo<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,
  #[account(
        mut,
        seeds=[b"TODO_ACC", signer.key().as_ref()],
        bump=todo_account.bump
    )]
  pub todo_account: Account<'info, TodoState>,
}

#[derive(Accounts)]
pub struct UpdateTodo<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,
  #[account(
        mut,
        seeds=[b"TODO_ACC", signer.key().as_ref()],
        bump=todo_account.bump
    )]
  pub todo_account: Account<'info, TodoState>,
}
