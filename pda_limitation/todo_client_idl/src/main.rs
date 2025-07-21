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

const TODO_ACC_SEED: &[u8] = b"TODO_ACC";

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
    default_value = "6Cjd4PNSWMyFbsA2MTXtEkxhnAgWzjDQV969kFjQJukL"
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
    index: u64,
    #[arg(short, long)]
    completed: bool,
  },
  Delete {
    #[arg(short, long)]
    index: u64,
  },
  List,
  Get {
    #[arg(short, long)]
    index: u64,
  },
}

#[derive(Debug, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
struct Todo {
  title: String,
  description: String,
  is_completed: bool,
}

#[derive(Debug, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
struct TodoState {
  key: Pubkey,
  bump: u8,
  todos: Vec<Todo>,
  total_todos: u64,
}

impl AccountDeserialize for TodoState {
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

impl Discriminator for TodoState {
  const DISCRIMINATOR: &'static [u8] = &[232, 39, 87, 92, 45, 186, 14, 13];
}

impl Owner for TodoState {
  fn owner() -> Pubkey {
    // This should be your program ID
    "6Cjd4PNSWMyFbsA2MTXtEkxhnAgWzjDQV969kFjQJukL"
      .parse()
      .unwrap()
  }
}

// Instruction structures based on IDL
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializePdaInstruction {}

impl Discriminator for InitializePdaInstruction {
  const DISCRIMINATOR: &'static [u8] = &[178, 254, 136, 212, 127, 85, 171, 210];
}

impl InstructionData for InitializePdaInstruction {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddTodoInstruction {
  pub title: String,
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
      Cluster::Custom(cluster_url.to_string(), cluster_url.replace("http", "ws")),
      payer.clone(),
      CommitmentConfig::processed(),
    );

    let program_id = program_id.parse::<Pubkey>()?;

    // Create program without IDL (using manual instruction structs based on IDL discriminators)
    let program = client.program(program_id)?;

    Ok(Self { program, payer })
  }

  fn get_todo_account_pda(&self) -> (Pubkey, u8) {
    let user_pubkey = self.payer.pubkey();
    Pubkey::find_program_address(&[TODO_ACC_SEED, user_pubkey.as_ref()], &self.program.id())
  }

  fn initialize_todo_account(&self) -> Result<String> {
    let (todo_account_pda, _) = self.get_todo_account_pda();
    let system_program = solana_sdk::system_program::ID;

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_account_pda, false),
      AccountMeta::new_readonly(system_program, false),
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

  fn add_todo(&self, title: String, description: String) -> Result<String> {
    let (todo_account_pda, _) = self.get_todo_account_pda();

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_account_pda, false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(AddTodoInstruction { title, description })
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  fn update_todo(&self, index: u64, is_completed: bool) -> Result<String> {
    let (todo_account_pda, _) = self.get_todo_account_pda();

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_account_pda, false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(UpdateTodoInstruction {
        index,
        is_completed,
      })
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  fn remove_todo(&self, index: u64) -> Result<String> {
    let (todo_account_pda, _) = self.get_todo_account_pda();

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_account_pda, false),
    ];

    let tx = self
      .program
      .request()
      .accounts(accounts)
      .args(RemoveTodoInstruction { index })
      .signer(&*self.payer)
      .send()?;

    Ok(tx.to_string())
  }

  fn get_todo_state(&self) -> Result<TodoState> {
    let (todo_account_pda, _) = self.get_todo_account_pda();
    let account: TodoState = self.program.account(todo_account_pda)?;
    Ok(account)
  }
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  let client = TodoClientIdl::new(&cli.url, &cli.keypair, &cli.program_id, &cli.idl_path)?;

  match cli.command {
    Commands::Init => {
      let tx = client.initialize_todo_account()?;
      println!("Todo account initialized. Transaction: {}", tx);
    }
    Commands::Create { title, description } => {
      let tx = client.add_todo(title, description)?;
      println!("Todo created. Transaction: {}", tx);
    }
    Commands::Update { index, completed } => {
      let tx = client.update_todo(index, completed)?;
      println!("Todo updated. Transaction: {}", tx);
    }
    Commands::Delete { index } => {
      let tx = client.remove_todo(index)?;
      println!("Todo deleted. Transaction: {}", tx);
    }
    Commands::List => match client.get_todo_state() {
      Ok(todo_state) => {
        if todo_state.todos.is_empty() {
          println!("No todos found.");
        } else {
          println!("Todo List:");
          for (idx, todo) in todo_state.todos.iter().enumerate() {
            let status = if todo.is_completed { "✓" } else { "☐" };
            println!(
              "[{}] {} | {} - {}",
              idx, todo.title, todo.description, status
            );
          }
          println!("\nTotal todos created: {}", todo_state.total_todos);
        }
      }
      Err(e) => {
        println!("Todo account not initialized. Error: {}", e);
        println!("Run 'cargo run -- init' to initialize your todo account.");
      }
    },
    Commands::Get { index } => match client.get_todo_state() {
      Ok(todo_state) => {
        if let Some(todo) = todo_state.todos.get(index as usize) {
          println!("Todo Item [{}]:", index);
          println!("  Title: {}", todo.title);
          println!("  Description: {}", todo.description);
          println!(
            "  Completed: {}",
            if todo.is_completed { "Yes" } else { "No" }
          );
        } else {
          println!("Todo with index {} not found.", index);
        }
      }
      Err(e) => {
        println!("Todo account not initialized. Error: {}", e);
      }
    },
  }

  Ok(())
}
