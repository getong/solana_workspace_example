use anchor_lang::prelude::*;

declare_id!("E5usXUWu4XR7rPJS6WLiYKWGj1BtUYLZL7TGc2mL78ZB");

const TODO_LIST_SEED: &[u8] = b"todo_list";
const TODO_ITEM_SEED: &[u8] = b"todo_item";
const MAX_TODO_SIZE: usize = 10 * 1024 * 1024; // 10MB

#[program]
pub mod pda_limitation {
  use super::*;

  pub fn initialize_todo_list(ctx: Context<InitializeTodoList>) -> Result<()> {
    let todo_list = &mut ctx.accounts.todo_list;
    todo_list.owner = ctx.accounts.user.key();
    todo_list.todo_count = 0;
    Ok(())
  }

  pub fn create_todo(ctx: Context<CreateTodo>, title: String, description: String) -> Result<()> {
    require!(title.len() <= 100, TodoError::TitleTooLong);
    require!(
      description.len() <= MAX_TODO_SIZE - 200,
      TodoError::DescriptionTooLong
    );

    let todo_item = &mut ctx.accounts.todo_item;
    let todo_list = &mut ctx.accounts.todo_list;

    todo_item.owner = ctx.accounts.user.key();
    todo_item.id = todo_list.todo_count;
    todo_item.title = title;
    todo_item.description = description;
    todo_item.completed = false;
    todo_item.created_at = Clock::get()?.unix_timestamp;

    todo_list.todo_count += 1;

    Ok(())
  }

  pub fn update_todo(
    ctx: Context<UpdateTodo>,
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
  ) -> Result<()> {
    let todo_item = &mut ctx.accounts.todo_item;

    if let Some(new_title) = title {
      require!(new_title.len() <= 100, TodoError::TitleTooLong);
      todo_item.title = new_title;
    }

    if let Some(new_description) = description {
      require!(
        new_description.len() <= MAX_TODO_SIZE - 200,
        TodoError::DescriptionTooLong
      );
      todo_item.description = new_description;
    }

    if let Some(is_completed) = completed {
      todo_item.completed = is_completed;
    }

    Ok(())
  }

  pub fn delete_todo(ctx: Context<DeleteTodo>) -> Result<()> {
    Ok(())
  }
}

#[account]
pub struct TodoList {
  pub owner: Pubkey,
  pub todo_count: u64,
}

#[account]
pub struct TodoItem {
  pub owner: Pubkey,
  pub id: u64,
  pub title: String,
  pub description: String,
  pub completed: bool,
  pub created_at: i64,
}

#[derive(Accounts)]
pub struct InitializeTodoList<'info> {
  #[account(
        init,
        payer = user,
        space = 8 + 32 + 8,
        seeds = [TODO_LIST_SEED, user.key().as_ref()],
        bump
    )]
  pub todo_list: Account<'info, TodoList>,
  #[account(mut)]
  pub user: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateTodo<'info> {
  #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 104 + MAX_TODO_SIZE + 1 + 8,
        seeds = [TODO_ITEM_SEED, user.key().as_ref(), &todo_list.todo_count.to_le_bytes()],
        bump
    )]
  pub todo_item: Account<'info, TodoItem>,
  #[account(
        mut,
        seeds = [TODO_LIST_SEED, user.key().as_ref()],
        bump
    )]
  pub todo_list: Account<'info, TodoList>,
  #[account(mut)]
  pub user: Signer<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(todo_id: u64)]
pub struct UpdateTodo<'info> {
  #[account(
        mut,
        seeds = [TODO_ITEM_SEED, user.key().as_ref(), &todo_id.to_le_bytes()],
        bump,
        has_one = owner @ TodoError::Unauthorized
    )]
  pub todo_item: Account<'info, TodoItem>,
  #[account(mut)]
  pub user: Signer<'info>,
  /// CHECK: This is safe as we're only using it for seed derivation
  pub owner: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(todo_id: u64)]
pub struct DeleteTodo<'info> {
  #[account(
        mut,
        close = user,
        seeds = [TODO_ITEM_SEED, user.key().as_ref(), &todo_id.to_le_bytes()],
        bump,
        has_one = owner @ TodoError::Unauthorized
    )]
  pub todo_item: Account<'info, TodoItem>,
  #[account(mut)]
  pub user: Signer<'info>,
  /// CHECK: This is safe as we're only using it for seed derivation
  pub owner: AccountInfo<'info>,
}

#[error_code]
pub enum TodoError {
  #[msg("Title is too long. Maximum 100 characters.")]
  TitleTooLong,
  #[msg("Description is too long. Maximum size is approximately 10MB.")]
  DescriptionTooLong,
  #[msg("You are not authorized to perform this action.")]
  Unauthorized,
}
