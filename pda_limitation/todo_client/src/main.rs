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
use borsh::{BorshDeserialize, BorshSerialize};
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
    default_value = "6Cjd4PNSWMyFbsA2MTXtEkxhnAgWzjDQV969kFjQJukL"
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
  title: String,
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

// Borsh structs for manual deserialization debugging
#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct BorshTodoState {
  key: Pubkey,
  bump: u8,
  todos: Vec<BorshTodo>,
  total_todos: u64,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
struct BorshTodo {
  title: String,
  description: String,
  is_completed: bool,
}

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

  fn create_todo(&self, title: String, description: String) -> Result<String> {
    let (todo_list_pda, _) = self.get_todo_list_pda();

    let accounts = vec![
      AccountMeta::new(self.payer.pubkey(), true),
      AccountMeta::new(todo_list_pda, false),
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
      "First 40 bytes: {:?}",
      &account_info.data[.. 40.min(account_info.data.len())]
    );

    // Try manual Borsh deserialization
    let data_without_discriminator = &account_info.data[8 ..]; // Skip discriminator

    // Use a cursor to allow partial deserialization
    use std::io::{Cursor, Read};
    let mut cursor = Cursor::new(data_without_discriminator);

    match BorshTodoState::deserialize_reader(&mut cursor) {
      Ok(borsh_state) => {
        println!("Borsh deserialization successful!");
        println!("Key: {}", borsh_state.key);
        println!("Bump: {}", borsh_state.bump);
        println!("Total todos: {}", borsh_state.total_todos);
        println!("Todos count: {}", borsh_state.todos.len());
        for (i, todo) in borsh_state.todos.iter().enumerate() {
          println!(
            "  Todo {}: [{}] {} - {}",
            i,
            todo.title,
            todo.description,
            if todo.is_completed { "✓" } else { "☐" }
          );
        }
        let bytes_read = cursor.position() as usize;
        let remaining_bytes = data_without_discriminator.len() - bytes_read;
        println!(
          "Bytes read: {}, Remaining bytes (padding): {}",
          bytes_read, remaining_bytes
        );
      }
      Err(e) => {
        println!("Borsh deserialization error: {:?}", e);

        // Try to deserialize individual fields to debug
        println!("\nDebugging field-by-field:");
        let mut debug_cursor = Cursor::new(data_without_discriminator);

        // Try to read the key (Pubkey = 32 bytes)
        let mut key_bytes = [0u8; 32];
        match debug_cursor.read_exact(&mut key_bytes) {
          Ok(_) => {
            let key = Pubkey::from(key_bytes);
            println!("  Key: {} ✓", key);
          }
          Err(e) => println!("  Key deserialization failed: {:?}", e),
        }

        // Try to read the bump (u8 = 1 byte)
        match u8::deserialize_reader(&mut debug_cursor) {
          Ok(bump) => println!("  Bump: {} ✓", bump),
          Err(e) => println!("  Bump deserialization failed: {:?}", e),
        }

        // Try to read the todos length (as part of Vec)
        match u64::deserialize_reader(&mut debug_cursor) {
          Ok(vec_len) => println!("  Todos vector length: {} ✓", vec_len),
          Err(e) => println!("  Todos length deserialization failed: {:?}", e),
        }
      }
    }

    // Try Anchor deserialization
    let mut data = &account_info.data[8 ..]; // Skip discriminator
    let result = TodoState::deserialize(&mut data);
    match result {
      Ok(state) => {
        println!("Anchor deserialization successful!");
        Ok(state)
      }
      Err(e) => {
        println!("Anchor deserialization error: {:?}", e);
        // Fallback to anchor client
        let account = self.program.account::<TodoState>(todo_list_pda)?;
        Ok(account)
      }
    }
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
            "[{}] {} | {} - {}",
            index,
            todo.title,
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
        println!("  Title: {}", todo.title);
        println!("  Description: {}", todo.description);
        println!("  Completed: {}", todo.is_completed);
      } else {
        println!("Todo with index {} not found", id);
      }
    }
  }

  Ok(())
}
