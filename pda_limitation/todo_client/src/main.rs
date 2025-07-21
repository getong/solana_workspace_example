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
use solana_sdk::signature::read_keypair_file;
use solana_system_interface::program as system_program;

const TODO_LIST_SEED: &[u8] = b"TODO_ACC";

#[derive(Parser)]
#[command(name = "todo-client")]
#[command(about = "A CLI client for the Solana Todo Program")]
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
    default_value = "9oFMiFWGhKmuXN4wVhhFRGZVQpgcBjoHoPEBTDaxMGRA"
  )]
  program_id: String,
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

#[derive(Debug, AnchorSerialize, AnchorDeserialize)]
struct TodoState {
  key: Pubkey,
  bump: u8,
  todos: Vec<Todo>,
  total_todos: u64,
}

impl Discriminator for TodoState {
  const DISCRIMINATOR: &'static [u8] = &[232, 39, 87, 92, 45, 186, 14, 13];
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize)]
struct Todo {
  description: String,
  is_completed: bool,
}

impl AccountDeserialize for TodoState {
  fn try_deserialize_unchecked(
    buf: &mut &[u8],
  ) -> Result<Self, anchor_client::anchor_lang::error::Error> {
    Self::deserialize(buf).map_err(|_| {
      anchor_client::anchor_lang::error::Error::from(
        anchor_client::anchor_lang::error::ErrorCode::AccountDidNotDeserialize,
      )
    })
  }
}

impl AccountDeserialize for Todo {
  fn try_deserialize_unchecked(
    buf: &mut &[u8],
  ) -> Result<Self, anchor_client::anchor_lang::error::Error> {
    Self::deserialize(buf).map_err(|_| {
      anchor_client::anchor_lang::error::Error::from(
        anchor_client::anchor_lang::error::ErrorCode::AccountDidNotDeserialize,
      )
    })
  }
}

// Instruction structs with discriminators
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializePdaInstruction {}

impl Discriminator for InitializePdaInstruction {
  const DISCRIMINATOR: &'static [u8] = &[178, 254, 136, 212, 127, 85, 171, 210];
}

impl InstructionData for InitializePdaInstruction {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddTodoInstruction {
  pub description: String,
}

impl Discriminator for AddTodoInstruction {
  const DISCRIMINATOR: &'static [u8] = &[188, 16, 45, 145, 4, 5, 188, 75];
}

impl InstructionData for AddTodoInstruction {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateTodoInstruction {
  pub index: u64,
  pub is_completed: bool,
}

impl Discriminator for UpdateTodoInstruction {
  const DISCRIMINATOR: &'static [u8] = &[105, 8, 31, 183, 159, 73, 203, 134];
}

impl InstructionData for UpdateTodoInstruction {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveTodoInstruction {
  pub index: u64,
}

impl Discriminator for RemoveTodoInstruction {
  const DISCRIMINATOR: &'static [u8] = &[28, 167, 91, 69, 25, 225, 253, 117];
}

impl InstructionData for RemoveTodoInstruction {}

struct TodoClient {
  program: Program<Rc<Keypair>>,
  payer: Rc<Keypair>,
}

impl TodoClient {
  fn new(cluster_url: &str, keypair_path: &str, program_id: &str) -> Result<Self> {
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
    let program = client.program(program_id)?;

    Ok(Self { program, payer })
  }

  fn get_todo_list_pda(&self) -> (Pubkey, u8) {
    Pubkey::find_program_address(
      &[TODO_LIST_SEED, self.payer.pubkey().as_ref()],
      &self.program.id(),
    )
  }

  fn initialize_todo_list(&self) -> Result<String> {
    let (todo_list_pda, _) = self.get_todo_list_pda();
    println!("Todo list PDA: {}", todo_list_pda);

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_list_pda, false),
      AccountMeta::new_readonly(system_program::id(), false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(InitializePdaInstruction {})
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  fn create_todo(&self, _title: String, description: String) -> Result<String> {
    let (todo_list_pda, _) = self.get_todo_list_pda();

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_list_pda, false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(AddTodoInstruction { description })
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  fn update_todo(
    &self,
    todo_id: u64,
    _title: Option<String>,
    _description: Option<String>,
    completed: Option<bool>,
  ) -> Result<String> {
    let (todo_list_pda, _) = self.get_todo_list_pda();
    let is_completed = completed.unwrap_or(false);

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_list_pda, false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(UpdateTodoInstruction {
        index: todo_id,
        is_completed,
      })
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  fn delete_todo(&self, todo_id: u64) -> Result<String> {
    let (todo_list_pda, _) = self.get_todo_list_pda();

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_list_pda, false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(RemoveTodoInstruction { index: todo_id })
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  fn get_todo_list(&self) -> Result<TodoState> {
    let (todo_list_pda, _) = self.get_todo_list_pda();

    // Debug: Let's check the raw account data
    let account_info = self.program.rpc().get_account(&todo_list_pda)?;
    println!("Account data length: {}", account_info.data.len());
    println!(
      "First 16 bytes: {:?}",
      &account_info.data[.. 16.min(account_info.data.len())]
    );

    let account = self.program.account::<TodoState>(todo_list_pda)?;
    Ok(account)
  }

  fn list_todos(&self) -> Result<Vec<Todo>> {
    let todo_state = self.get_todo_list()?;
    Ok(todo_state.todos)
  }
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  let client = TodoClient::new(&cli.url, &cli.keypair, &cli.program_id)?;

  match cli.command {
    Commands::Init => {
      let tx = client.initialize_todo_list()?;
      println!("Todo list initialized. Transaction: {}", tx);
    }
    Commands::Create { title, description } => {
      let tx = client.create_todo(title, description)?;
      println!("Todo created. Transaction: {}", tx);
    }
    Commands::Update {
      id,
      title,
      description,
      completed,
    } => {
      let tx = client.update_todo(id, title, description, completed)?;
      println!("Todo updated. Transaction: {}", tx);
    }
    Commands::Delete { id } => {
      let tx = client.delete_todo(id)?;
      println!("Todo deleted. Transaction: {}", tx);
    }
    Commands::List => {
      let todos = client.list_todos()?;
      if todos.is_empty() {
        println!("No todos found.");
      } else {
        println!("Todo List:");
        for (index, todo) in todos.iter().enumerate() {
          println!(
            "[{}] {} - {}",
            index,
            todo.description,
            if todo.is_completed { "✓" } else { "☐" }
          );
        }
      }
    }
    Commands::Get { id } => {
      let todos = client.list_todos()?;
      if let Some(todo) = todos.get(id as usize) {
        println!("Todo Item [{}]:", id);
        println!("  Description: {}", todo.description);
        println!("  Completed: {}", todo.is_completed);
      } else {
        println!("Todo with index {} not found", id);
      }
    }
  }

  Ok(())
}
