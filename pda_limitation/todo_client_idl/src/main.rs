use std::rc::Rc;

use anchor_client::{
  anchor_lang::{prelude::*, AccountDeserialize, Discriminator, InstructionData},
  solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
  },
  Client, Cluster, Program,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use solana_sdk::signature::read_keypair_file;
use solana_system_interface::program as system_program;

const TODO_LIST_SEED: &[u8] = b"todo_list";
const TODO_ITEM_SEED: &[u8] = b"todo_item";

#[derive(Parser)]
#[command(name = "todo-client-idl")]
#[command(about = "A CLI client for the Solana Todo Program using IDL")]
struct Cli {
  #[command(subcommand)]
  command: Commands,

  #[arg(short, long, default_value = "~/.config/solana/id.json")]
  keypair: String,

  #[arg(short, long, default_value = "http://localhost:8899")]
  url: String,

  #[arg(
    short,
    long,
    default_value = "E5usXUWu4XR7rPJS6WLiYKWGj1BtUYLZL7TGc2mL78ZB"
  )]
  program_id: String,

  #[arg(long, default_value = "./pda_limitation.json")]
  idl_path: String,
}

#[derive(Subcommand)]
enum Commands {
  Init,
  Create {
    #[arg(short, long)]
    title: String,
    #[arg(short, long)]
    description: String,
  },
  Update {
    #[arg(short, long)]
    id: u64,
    #[arg(short, long)]
    title: Option<String>,
    #[arg(short, long)]
    description: Option<String>,
    #[arg(short, long)]
    completed: Option<bool>,
  },
  Delete {
    #[arg(short, long)]
    id: u64,
  },
  List,
  Get {
    #[arg(short, long)]
    id: u64,
  },
}

#[derive(Debug, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
struct TodoList {
  owner: Pubkey,
  todo_count: u64,
}

#[derive(Debug, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
struct TodoItem {
  owner: Pubkey,
  id: u64,
  title: String,
  description: String,
  completed: bool,
  created_at: i64,
}

impl AccountDeserialize for TodoList {
  fn try_deserialize_unchecked(
    buf: &mut &[u8],
  ) -> Result<Self, anchor_client::anchor_lang::error::Error> {
    <Self as AnchorDeserialize>::deserialize(buf).map_err(|_| {
      anchor_client::anchor_lang::error::Error::from(
        anchor_client::anchor_lang::error::ErrorCode::AccountDidNotDeserialize,
      )
    })
  }
}

impl AccountDeserialize for TodoItem {
  fn try_deserialize_unchecked(
    buf: &mut &[u8],
  ) -> Result<Self, anchor_client::anchor_lang::error::Error> {
    <Self as AnchorDeserialize>::deserialize(buf).map_err(|_| {
      anchor_client::anchor_lang::error::Error::from(
        anchor_client::anchor_lang::error::ErrorCode::AccountDidNotDeserialize,
      )
    })
  }
}

// Instruction structures based on IDL
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeTodoListInstruction {}

impl Discriminator for InitializeTodoListInstruction {
  const DISCRIMINATOR: &'static [u8] = &[110, 156, 253, 119, 218, 241, 220, 171];
}

impl InstructionData for InitializeTodoListInstruction {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateTodoInstruction {
  pub title: String,
  pub description: String,
}

impl Discriminator for CreateTodoInstruction {
  const DISCRIMINATOR: &'static [u8] = &[250, 161, 142, 148, 131, 48, 194, 181];
}

impl InstructionData for CreateTodoInstruction {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateTodoInstruction {
  pub title: Option<String>,
  pub description: Option<String>,
  pub completed: Option<bool>,
}

impl Discriminator for UpdateTodoInstruction {
  const DISCRIMINATOR: &'static [u8] = &[105, 8, 31, 183, 159, 73, 203, 134];
}

impl InstructionData for UpdateTodoInstruction {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DeleteTodoInstruction {}

impl Discriminator for DeleteTodoInstruction {
  const DISCRIMINATOR: &'static [u8] = &[224, 212, 234, 177, 90, 57, 219, 115];
}

impl InstructionData for DeleteTodoInstruction {}

struct TodoClientIdl {
  program: Program<Rc<Keypair>>,
  payer: Rc<Keypair>,
}

impl TodoClientIdl {
  fn new(cluster_url: &str, keypair_path: &str, program_id: &str, _idl_path: &str) -> Result<Self> {
    let keypair_path = shellexpand::tilde(keypair_path).to_string();
    let payer = Rc::new(
      read_keypair_file(&keypair_path)
        .map_err(|e| anyhow::anyhow!("Failed to read keypair: {}", e))?,
    );

    let client = Client::new_with_options(
      Cluster::Custom(cluster_url.to_string(), "ws://localhost:8900".to_string()),
      payer.clone(),
      CommitmentConfig::processed(),
    );

    let program_id = program_id.parse::<Pubkey>()?;

    // Create program without IDL (using manual instruction structs based on IDL discriminators)
    let program = client.program(program_id)?;

    Ok(Self { program, payer })
  }

  fn get_todo_list_pda(&self) -> (Pubkey, u8) {
    Pubkey::find_program_address(
      &[TODO_LIST_SEED, self.payer.pubkey().as_ref()],
      &self.program.id(),
    )
  }

  fn get_todo_item_pda(&self, todo_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
      &[
        TODO_ITEM_SEED,
        self.payer.pubkey().as_ref(),
        &todo_id.to_le_bytes(),
      ],
      &self.program.id(),
    )
  }

  async fn initialize_todo_list(&self) -> Result<String> {
    let (todo_list_pda, _) = self.get_todo_list_pda();

    let accounts = vec![
      AccountMeta::new(todo_list_pda, false),
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new_readonly(system_program::id(), false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(InitializeTodoListInstruction {})
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  async fn create_todo(&self, title: String, description: String) -> Result<String> {
    let (todo_list_pda, _) = self.get_todo_list_pda();

    // Fetch current todo count
    let todo_list_account: TodoList = self.program.account(todo_list_pda)?;
    let (todo_item_pda, _) = self.get_todo_item_pda(todo_list_account.todo_count);

    let accounts = vec![
      AccountMeta::new(todo_item_pda, false),
      AccountMeta::new(todo_list_pda, false),
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new_readonly(system_program::id(), false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(CreateTodoInstruction { title, description })
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  async fn update_todo(
    &self,
    todo_id: u64,
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
  ) -> Result<String> {
    let (todo_item_pda, _) = self.get_todo_item_pda(todo_id);

    let accounts = vec![
      AccountMeta::new(todo_item_pda, false),
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new_readonly(self.payer.pubkey(), false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(UpdateTodoInstruction {
        title,
        description,
        completed,
      })
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  async fn delete_todo(&self, todo_id: u64) -> Result<String> {
    let (todo_item_pda, _) = self.get_todo_item_pda(todo_id);

    let accounts = vec![
      AccountMeta::new(todo_item_pda, false),
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new_readonly(self.payer.pubkey(), false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(DeleteTodoInstruction {})
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  async fn get_todo_list(&self) -> Result<TodoList> {
    let (todo_list_pda, _) = self.get_todo_list_pda();
    let account: TodoList = self.program.account(todo_list_pda)?;
    Ok(account)
  }

  async fn get_todo_item(&self, todo_id: u64) -> Result<TodoItem> {
    let (todo_item_pda, _) = self.get_todo_item_pda(todo_id);
    let account: TodoItem = self.program.account(todo_item_pda)?;
    Ok(account)
  }

  async fn list_todos(&self) -> Result<Vec<TodoItem>> {
    let todo_list = self.get_todo_list().await?;
    let mut todos = Vec::new();

    for i in 0 .. todo_list.todo_count {
      match self.get_todo_item(i).await {
        Ok(todo) => todos.push(todo),
        Err(_) => continue, // Skip deleted todos
      }
    }

    Ok(todos)
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();

  let client = TodoClientIdl::new(&cli.url, &cli.keypair, &cli.program_id, &cli.idl_path)?;

  match cli.command {
    Commands::Init => {
      let tx = client.initialize_todo_list().await?;
      println!("Todo list initialized. Transaction: {}", tx);
    }
    Commands::Create { title, description } => {
      let tx = client.create_todo(title, description).await?;
      println!("Todo created. Transaction: {}", tx);
    }
    Commands::Update {
      id,
      title,
      description,
      completed,
    } => {
      let tx = client
        .update_todo(id, title, description, completed)
        .await?;
      println!("Todo updated. Transaction: {}", tx);
    }
    Commands::Delete { id } => {
      let tx = client.delete_todo(id).await?;
      println!("Todo deleted. Transaction: {}", tx);
    }
    Commands::List => {
      let todos = client.list_todos().await?;
      if todos.is_empty() {
        println!("No todos found.");
      } else {
        println!("Todo List:");
        for todo in todos {
          println!(
            "ID: {}, Title: {}, Completed: {}, Created: {}",
            todo.id, todo.title, todo.completed, todo.created_at
          );
          if !todo.description.is_empty() {
            println!("  Description: {}", todo.description);
          }
          println!();
        }
      }
    }
    Commands::Get { id } => match client.get_todo_item(id).await {
      Ok(todo) => {
        println!("Todo Item:");
        println!("  ID: {}", todo.id);
        println!("  Title: {}", todo.title);
        println!("  Description: {}", todo.description);
        println!("  Completed: {}", todo.completed);
        println!("  Created: {}", todo.created_at);
      }
      Err(e) => {
        println!("Todo with ID {} not found: {}", id, e);
      }
    },
  }

  Ok(())
}
