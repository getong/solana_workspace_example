# Todo Client IDL

A Rust CLI client for the Solana Todo Program that uses discriminators extracted from the IDL (Interface Definition Language) JSON file for accurate instruction handling.

## Key Advantages of IDL-derived Client

### ✅ **IDL-derived Discriminators**
- Extracts exact instruction discriminators from the generated IDL JSON file
- Ensures perfect compatibility with the deployed program
- No guesswork on instruction hashes

### ✅ **Type Safety & Validation**
- Uses strongly-typed instruction structs based on IDL specification
- Compile-time validation of instruction parameters
- Proper serialization/deserialization handling

### ✅ **Maintainability**
- When the program changes, extract new discriminators from regenerated IDL
- Instruction structures mirror the IDL specification exactly
- Self-documenting code through IDL-derived types

### ✅ **Accurate Integration**
- Uses exact discriminators from deployed program: `[110, 156, 253, 119, 218, 241, 220, 171]`
- Proper account metadata handling
- Compatible with anchor-client API

## Features

- **IDL-derived**: Uses discriminators extracted from IDL for perfect compatibility
- **10MB Storage**: Supports todo items with up to ~10MB of description data
- **Complete CRUD**: Create, Read, Update, Delete operations
- **PDA Management**: Automatic Program Derived Address handling
- **Error Handling**: Comprehensive error messages and validation

## Setup

### Prerequisites
1. Solana program deployed and IDL generated
2. Valid Solana wallet keypair
3. RPC connection to Solana cluster

### Build
```bash
cargo build --release
```

## Usage

### Basic Commands

#### Initialize Todo List
```bash
cargo run -- init
```

#### Create Todo
```bash
cargo run -- create --title "Learn Solana" --description "Study Anchor framework and PDA patterns"
```

#### List All Todos
```bash
cargo run -- list
```

#### Get Specific Todo
```bash
cargo run -- get --id 0
```

#### Update Todo
```bash
# Update title and mark as completed
cargo run -- update --id 0 --title "Learn Solana ✅" --completed true

# Update just description
cargo run -- update --id 0 --description "Completed Anchor tutorial and built todo app"
```

#### Delete Todo
```bash
cargo run -- delete --id 0
```

### Advanced Options

#### Custom Configuration
```bash
# Use custom RPC endpoint
cargo run -- --url https://api.devnet.solana.com create --title "Test" --description "Testing on devnet"

# Use custom keypair
cargo run -- --keypair ./my-wallet.json list

# Use custom program ID
cargo run -- --program-id "YOUR_PROGRAM_ID" init

# Use custom IDL file
cargo run -- --idl-path ./custom-idl.json list
```

#### Large Data Example
```bash
# Create todo with large description (up to ~10MB)
cargo run -- create --title "Documentation" --description "$(cat large-document.txt)"
```

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `--keypair, -k` | `~/.config/solana/id.json` | Path to wallet keypair |
| `--url, -u` | `http://localhost:8899` | Solana RPC URL |
| `--program-id, -p` | `E5usXUWu4XR7rPJS6WLiYKWGj1BtUYLZL7TGc2mL78ZB` | Program ID |
| `--idl-path` | `./pda_limitation.json` | Path to IDL JSON file |

## Architecture

### IDL Integration
```rust
// Load IDL from JSON file
let idl: serde_json::Value = serde_json::from_str(&idl_data)?;

// Create program with IDL
let program = client.program_with_idl(program_id, &idl)?;

// Use named instructions
let tx = program
  .request()
  .instruction("create_todo")  // ← IDL-based instruction name
  .accounts(...)               // ← Named account mapping
  .args((title, description))  // ← Type-safe arguments
  .send()?;
```

### Account Management
- **TodoList PDA**: `seeds = ["todo_list", user_pubkey]`
- **TodoItem PDA**: `seeds = ["todo_item", user_pubkey, todo_id]`
- **Storage**: Each todo item allocated up to 10MB space

### Error Handling
The IDL includes comprehensive error definitions:
- `TitleTooLong`: Title exceeds 100 characters
- `DescriptionTooLong`: Description exceeds ~10MB
- `Unauthorized`: User doesn't own the todo item

## Development

### Regenerating IDL
When the Anchor program changes:
```bash
# In the main project directory
anchor build
cp target/idl/pda_limitation.json todo_client_idl/
```

The client will automatically adapt to program changes through the updated IDL.

### Testing
```bash
# Check compilation
cargo check

# Run with debug logging
RUST_LOG=debug cargo run -- list
```

## Comparison with Manual Client

| Aspect | Manual Client | IDL Client |
|--------|---------------|------------|
| **Setup Complexity** | High - manual discriminators | Low - automatic from IDL |
| **Maintenance** | High - manual updates needed | Low - IDL regeneration |
| **Type Safety** | Manual validation | Automatic validation |
| **Error Prone** | Yes - manual serialization | No - Anchor handles it |
| **Documentation** | Manual | Self-documenting |

The IDL-based approach is significantly more maintainable and less error-prone, making it the recommended approach for production applications.