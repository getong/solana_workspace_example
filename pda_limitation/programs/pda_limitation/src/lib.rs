use anchor_lang::prelude::*;

pub mod contexts;
use contexts::*;

declare_id!("9oFMiFWGhKmuXN4wVhhFRGZVQpgcBjoHoPEBTDaxMGRA");

#[program]
pub mod pda_limitation {
  use super::*;

  pub fn initialize_pda(ctx: Context<InitializaPda>) -> Result<()> {
    let todo_account = &mut ctx.accounts.todo_account;
    todo_account.key = ctx.accounts.signer.key();
    todo_account.bump = ctx.bumps.todo_account;
    todo_account.todos = Vec::new();
    todo_account.total_todos = 0;
    Ok(())
  }

  pub fn add_todo(ctx: Context<AddTodo>, description: String) -> Result<()> {
    require!(description.len() <= 50, TodoError::DescriptionTooLong);
    require!(
      ctx.accounts.todo_account.todos.len() < 10,
      TodoError::MaxTodosReached
    );

    let todo_account = &mut ctx.accounts.todo_account;

    let new_todo = Todo {
      description,
      is_completed: false,
    };

    todo_account.todos.push(new_todo);
    todo_account.total_todos += 1;

    Ok(())
  }

  pub fn update_todo(ctx: Context<UpdateTodo>, index: u64, is_completed: bool) -> Result<()> {
    let todo_account = &mut ctx.accounts.todo_account;

    require!(
      (index as usize) < todo_account.todos.len(),
      TodoError::InvalidTodoIndex
    );

    todo_account.todos[index as usize].is_completed = is_completed;

    Ok(())
  }

  pub fn remove_todo(ctx: Context<UpdateTodo>, index: u64) -> Result<()> {
    let todo_account = &mut ctx.accounts.todo_account;

    require!(
      (index as usize) < todo_account.todos.len(),
      TodoError::InvalidTodoIndex
    );

    todo_account.todos.remove(index as usize);

    Ok(())
  }
}

#[error_code]
pub enum TodoError {
  #[msg("Description is too long. Maximum 50 characters.")]
  DescriptionTooLong,
  #[msg("Maximum number of todos (10) reached.")]
  MaxTodosReached,
  #[msg("Invalid todo index.")]
  InvalidTodoIndex,
}
