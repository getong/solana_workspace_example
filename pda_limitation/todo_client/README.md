# Todo Client

A Rust CLI client for interacting with the Solana Todo Program that supports PDAs with 10MB storage limitations.

## Features

- Initialize todo list
- Create, Read, Update, Delete (CRUD) operations for todos
- Support for large descriptions (up to ~10MB)
- Command-line interface with easy-to-use commands

## Usage

### Build the client

```bash
cargo build --release
```

### Commands

#### Initialize Todo List
```bash
cargo run -- init
```

#### Create a Todo
```bash
cargo run -- create --title "My First Todo" --description "This is a detailed description"
```

#### List All Todos
```bash
cargo run -- list
```

#### Get Specific Todo
```bash
cargo run -- get --id 0
```

#### Update a Todo
```bash
# Update title and mark as completed
cargo run -- update --id 0 --title "Updated Title" --completed true

# Update just the description
cargo run -- update --id 0 --description "New description"
```

#### Delete a Todo
```bash
cargo run -- delete --id 0
```

### Options

- `--keypair, -k`: Path to keypair file (default: ~/.config/solana/id.json)
- `--url, -u`: RPC URL (default: http://localhost:8899)
- `--program-id, -p`: Program ID (default: E5usXUWu4XR7rPJS6WLiYKWGj1BtUYLZL7TGc2mL78ZB)

### Example with custom options

```bash
cargo run -- --keypair ./my-keypair.json --url https://api.devnet.solana.com create --title "Test" --description "Test description"
```

## Program Details

- Each todo item is stored in a separate PDA with up to 10MB storage capacity
- Todo items are identified by incremental IDs
- Users can only modify their own todos
- Deleted todos free up the storage space by closing the account