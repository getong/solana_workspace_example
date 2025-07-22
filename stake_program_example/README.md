# Solana Stake Program Example with IDL-Based Rust Client

This project demonstrates a complete Solana staking program built with Anchor framework, along with a comprehensive Rust client that utilizes the generated IDL (Interface Definition Language) for program interaction.

## Project Structure

```
stake_program_example/
├── programs/
│   └── stake_program_example/
│       ├── src/
│       │   └── lib.rs                 # Main program code
│       └── Cargo.toml                 # Program dependencies
├── stake_client/
│   ├── src/
│   │   ├── main.rs                    # Client examples
│   │   └── idl_based_client.rs        # IDL-based raw client
│   └── Cargo.toml                     # Client dependencies
├── target/
│   └── idl/
│       └── stake_program_example.json # Generated IDL
├── Anchor.toml                        # Anchor configuration
└── README.md                          # This file
```

## Features

### Stake Program Features
- **Pool Initialization**: Create staking pools with configurable reward rates and lock periods
- **User Stake Management**: Initialize user stake accounts with PDA derivation
- **Staking Operations**: Stake tokens with reward calculation
- **Unstaking Operations**: Unstake tokens with lock period enforcement
- **Reward Distribution**: Claim accumulated rewards with compound interest calculation
- **Pool Funding**: Fund reward pools for distribution
- **Event Emissions**: Emit events for all major operations
- **Comprehensive Error Handling**: Custom error codes for all failure scenarios

### Client Features
- **High-Level Anchor Client**: Type-safe, convenient methods using anchor-client
- **IDL-Based Raw Client**: Direct instruction construction using generated IDL
- **PDA Derivation**: Automatic Program Derived Address calculation
- **Token Account Management**: Helper methods for token account creation
- **Account Data Retrieval**: Methods to fetch and deserialize program account data

## Prerequisites

- Rust 1.70+
- Solana CLI 1.18+
- Anchor Framework 0.31+
- Node.js (for Anchor workspace)

## Installation & Setup

1. **Clone and navigate to the project:**
   ```bash
   git clone <repository-url>
   cd stake_program_example
   ```

2. **Install dependencies:**
   ```bash
   # Install Anchor dependencies
   npm install
   
   # Build the program
   anchor build
   
   # Generate IDL
   anchor idl build
   ```

3. **Setup Solana configuration:**
   ```bash
   # Configure Solana for local development
   solana config set --url localhost
   solana config set --keypair ~/.config/solana/id.json
   
   # Start local validator (in separate terminal)
   solana-test-validator
   ```

## Program Architecture

### Core Accounts

#### `StakePool`
```rust
pub struct StakePool {
    pub authority: Pubkey,           // Pool authority
    pub staking_mint: Pubkey,        // Token mint for staking
    pub reward_mint: Pubkey,         // Token mint for rewards
    pub staking_vault: Pubkey,       // Vault holding staked tokens
    pub reward_vault: Pubkey,        // Vault holding reward tokens
    pub reward_rate: u64,            // Rewards per second
    pub lock_period: i64,            // Lock period in seconds
    pub total_staked: u64,           // Total tokens staked
    pub accumulated_reward_per_share: u64, // Reward calculation
    pub last_update_time: i64,       // Last reward update
    pub bump: u8,                    // PDA bump seed
}
```

#### `UserStake`
```rust
pub struct UserStake {
    pub owner: Pubkey,           // User's pubkey
    pub pool: Pubkey,            // Associated pool
    pub staked_amount: u64,      // User's staked amount
    pub reward_debt: u64,        // Reward calculation debt
    pub pending_reward: u64,     // Unclaimed rewards
    pub last_stake_time: i64,    // Last stake timestamp
    pub bump: u8,                // PDA bump seed
}
```

### Instructions

1. **initialize_pool** - Create a new staking pool
2. **initialize_user_stake** - Initialize user stake account
3. **stake** - Stake tokens into the pool
4. **unstake** - Withdraw staked tokens (after lock period)
5. **claim_reward** - Claim accumulated rewards
6. **fund_reward_pool** - Add rewards to the pool

### PDA Seeds

- **Pool**: `["pool", staking_mint]`
- **User Stake**: `["user_stake", pool, user]`
- **Staking Vault**: `["staking_vault", pool]`
- **Reward Vault**: `["reward_vault", pool]`

## Client Usage

### Running the Examples

```bash
# Navigate to client directory
cd stake_client

# Run the comprehensive example
cargo run

# Or set custom paths
KEYPAIR_PATH=/path/to/keypair.json IDL_PATH=/path/to/idl.json cargo run
```

### High-Level Anchor Client Example

```rust
use anchor_client::{Client, Cluster};

// Initialize client
let client = StakeClient::new(Cluster::Localnet, "/path/to/keypair.json")?;

// Initialize a staking pool
client.initialize_pool(
    staking_mint,
    reward_mint,
    100,    // reward rate: 100 tokens per second
    86400   // lock period: 24 hours
).await?;

// Stake tokens
client.stake(pool, user, user_token_account, 1000).await?;

// Claim rewards
client.claim_reward(pool, user, user_reward_account).await?;
```

### IDL-Based Raw Client Example

```rust
// Load IDL and create raw client
let idl_client = IdlStakeClient::new(
    Cluster::Localnet,
    "/path/to/keypair.json",
    "/path/to/idl.json"
)?;

// Display IDL information
idl_client.print_idl_info();

// Use raw instructions
let tx = idl_client.initialize_pool_raw(
    staking_mint,
    reward_mint,
    200,    // reward rate
    172800  // lock period: 48 hours
).await?;
```

## Key Concepts

### Reward Calculation
The program implements a compound reward system:
- Rewards accumulate per second based on `reward_rate`
- `accumulated_reward_per_share` tracks total rewards per staked token
- User rewards = `(staked_amount * accumulated_reward_per_share) - reward_debt`

### PDA (Program Derived Address) Usage
All program accounts use PDAs for security:
- Deterministic addresses based on seeds
- No private key required
- Program can sign transactions on behalf of PDAs

### IDL Benefits
The generated IDL provides:
- **Instruction Discriminators**: 8-byte prefixes for each instruction
- **Account Structure**: Complete type definitions
- **Error Codes**: All custom program errors
- **Event Definitions**: Event structure and discriminators

## Testing

```bash
# Run Anchor tests
anchor test

# Run client tests
cd stake_client
cargo test
```

## Deployment

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Update IDL on-chain
anchor idl init <PROGRAM_ID> --provider.cluster devnet
```

## Security Considerations

1. **PDA Validation**: All PDAs are validated with proper seed derivation
2. **Owner Checks**: User operations require proper ownership verification
3. **Arithmetic Safety**: All calculations use checked arithmetic to prevent overflow
4. **Lock Period Enforcement**: Unstaking respects the configured lock period
5. **Token Account Validation**: All token operations validate mint and ownership

## Common Issues & Solutions

1. **"Account not found"**: Ensure accounts are properly initialized before use
2. **"Insufficient funds"**: Check token balances before staking/unstaking
3. **"Still locked"**: Wait for lock period to expire before unstaking
4. **"Invalid mint"**: Verify token account mints match pool configuration

## Advanced Usage

### Custom Token Mints
Replace the example mints with your own:
```rust
let staking_mint = Pubkey::from_str("YOUR_STAKING_TOKEN_MINT")?;
let reward_mint = Pubkey::from_str("YOUR_REWARD_TOKEN_MINT")?;
```

### Multiple Pools
The program supports multiple independent staking pools:
```rust
// Different pools for different staking tokens
let sol_pool = client.derive_pool_pda(&sol_mint);
let usdc_pool = client.derive_pool_pda(&usdc_mint);
```

### Event Listening
Monitor program events:
```rust
// Events emitted: StakeEvent, UnstakeEvent, ClaimRewardEvent
// Each contains: user, amount, timestamp
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.