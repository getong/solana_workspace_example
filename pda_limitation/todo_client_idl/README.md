# Todo Client IDL

A Rust CLI client for interacting with the Solana Todo Program. This client uses the IDL (Interface Definition Language) to dynamically construct program interactions.

## Program Overview

The Todo Program is a Solana smart contract that manages a simple todo list with the following constraints:
- **Single PDA per user**: All todos are stored in one account
- **Maximum 10 todos**: Limited by on-chain storage constraints
- **Title limit**: 50 characters maximum
- **Description limit**: 200 characters maximum

## Architecture

The program uses a single Program Derived Address (PDA) pattern:
- **Seed**: `TODO_ACC` + user's public key
- **Account**: `TodoState` containing all user's todos

## Features

- Initialize a personal todo list account
- Add new todos with title and description
- Update todo completion status
- Remove todos by index
- List all todos
- Get specific todo details

## Usage

### Build

```bash
cargo build --release
```

### Commands

#### Initialize Todo List
Creates a new todo list account for the user.

```bash
cargo run -- init
```

#### Create Todo
Adds a new todo item to your list.

```bash
cargo run -- create -t "Buy groceries" -d "Milk, eggs, bread, and vegetables"
```

#### List Todos
Displays all todos in your list.

```bash
cargo run -- list
```

Output:
```
Todo List:
[0] Buy groceries | Milk, eggs, bread, and vegetables - ☐
[1] Finish report | Complete quarterly report by Friday - ☐
[2] Exercise | 30 minutes of jogging - ✓
```

#### Update Todo
Marks a todo as completed or uncompleted.

```bash
# Mark todo at index 0 as completed
cargo run -- update -i 0 -c true

# Mark todo at index 0 as not completed
cargo run -- update -i 0 -c false
```

#### Delete Todo
Removes a todo from the list by index.

```bash
cargo run -- delete -i 1
```

#### Get Todo Details
Displays detailed information about a specific todo.

```bash
cargo run -- get -i 0
```

Output:
```
Todo Item [0]:
  Title: Buy groceries
  Description: Milk, eggs, bread, and vegetables
  Completed: No
```

### Configuration Options

- `--keypair`: Path to Solana keypair file (default: `~/.config/solana/id.json`)
- `--url`: RPC URL (default: `http://localhost:8899`)
- `--program-id`: Program ID (default: `6Cjd4PNSWMyFbsA2MTXtEkxhnAgWzjDQV969kFjQJukL`)

### Example with Custom Configuration

```bash
cargo run -- --keypair ~/my-wallet.json --url https://api.devnet.solana.com create -t "Test" -d "Testing on devnet"
```

## Error Handling

The program includes comprehensive error handling for:
- `TitleTooLong`: Title exceeds 50 characters
- `DescriptionTooLong`: Description exceeds 200 characters
- `MaxTodosReached`: Attempting to add more than 10 todos
- `InvalidTodoIndex`: Accessing a todo that doesn't exist

## Technical Details

### IDL Structure
The client reads the `pda_limitation.json` IDL file to understand:
- Instruction discriminators
- Account structures
- Argument types
- Error codes

### Account Structure
```rust
TodoState {
    key: Pubkey,           // User's public key
    bump: u8,              // PDA bump seed
    todos: Vec<Todo>,      // List of todos (max 10)
    total_todos: u64,      // Total number of todos created
}

Todo {
    title: String,         // Max 50 characters
    description: String,   // Max 200 characters
    is_completed: bool,    // Completion status
}
```

## Development

This client demonstrates:
- IDL-based program interaction
- PDA derivation and account management
- Instruction construction and sending
- Account deserialization
- Error handling and user feedback

## cli example

``` shell
# Initialize
cargo run --bin todo_client_raw -- --keypair ~/solana-wallets/alice.json init

# Create a todo
cargo run --bin todo_client_raw -- --keypair ~/solana-wallets/alice.json create -t "New task" -d "Task description"

# List todos
cargo run --bin todo_client_raw -- --keypair ~/solana-wallets/alice.json list
```

## Dependencies

- `anchor-lang`: Anchor framework
- `solana-sdk`: Solana SDK
- `serde_json`: JSON parsing for IDL
- `clap`: Command-line argument parsing
- `shellexpand`: Path expansion