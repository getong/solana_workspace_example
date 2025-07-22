# Solana Stake Program Rust Client

A comprehensive Rust client for interacting with the Solana Stake Program built with Anchor framework.

## Features

This client provides a command-line interface for all stake program operations:

- Initialize stake accounts
- Authorize stake operations
- Delegate stake to validators
- Split stake accounts
- Withdraw from stake accounts
- Deactivate stake
- Set lockup periods
- Merge stake accounts
- Query minimum delegation amount
- View stake account information

## Building

```bash
cd client
cargo build --release
```

## Usage

The client requires a keypair for signing transactions. You can specify one with `--keypair-path` or it will use a random keypair (for testing).

### Initialize a Stake Account

```bash
cargo run -- initialize \
  --staker <STAKER_PUBKEY> \
  --withdrawer <WITHDRAWER_PUBKEY> \
  --lockup-timestamp 0 \
  --lockup-epoch 0 \
  --custodian <CUSTODIAN_PUBKEY>
```

### Authorize a New Authority

```bash
cargo run -- authorize \
  --stake-account <STAKE_ACCOUNT_PUBKEY> \
  --new-authority <NEW_AUTHORITY_PUBKEY> \
  --stake-authorize staker  # or 'withdrawer'
```

### Delegate Stake to a Validator

```bash
cargo run -- delegate \
  --stake-account <STAKE_ACCOUNT_PUBKEY> \
  --vote-account <VOTE_ACCOUNT_PUBKEY>
```

### Split a Stake Account

```bash
cargo run -- split \
  --source-account <SOURCE_STAKE_ACCOUNT> \
  --lamports <AMOUNT_TO_SPLIT>
```

### Withdraw from Stake Account

```bash
cargo run -- withdraw \
  --stake-account <STAKE_ACCOUNT_PUBKEY> \
  --to <DESTINATION_PUBKEY> \
  --lamports <AMOUNT>
```

### Deactivate Stake

```bash
cargo run -- deactivate \
  --stake-account <STAKE_ACCOUNT_PUBKEY>
```

### Set Lockup

```bash
cargo run -- set-lockup \
  --stake-account <STAKE_ACCOUNT_PUBKEY> \
  --unix-timestamp <TIMESTAMP> \
  --epoch <EPOCH> \
  --custodian <CUSTODIAN_PUBKEY>
```

### Merge Stake Accounts

```bash
cargo run -- merge \
  --source-account <SOURCE_STAKE_ACCOUNT> \
  --dest-account <DESTINATION_STAKE_ACCOUNT>
```

### Get Minimum Delegation Amount

```bash
cargo run -- get-minimum-delegation
```

### View Stake Account Info

```bash
cargo run -- info \
  --stake-account <STAKE_ACCOUNT_PUBKEY>
```

## Configuration Options

- `--rpc-url`: RPC endpoint URL (default: http://localhost:8899)
- `--keypair-path`: Path to keypair file for signing transactions

## Example

Here's a complete example of creating and delegating a stake account:

```bash
# First, create a stake account
cargo run -- initialize \
  --staker 11111111111111111111111111111111 \
  --withdrawer 11111111111111111111111111111111 \
  --keypair-path ~/.config/solana/id.json

# Output will show the created stake account address
# Stake account created: ABC123...

# Then delegate to a vote account
cargo run -- delegate \
  --stake-account ABC123... \
  --vote-account VOTE123... \
  --keypair-path ~/.config/solana/id.json

# Check the stake account info
cargo run -- info \
  --stake-account ABC123... \
  --keypair-path ~/.config/solana/id.json
```

## Notes

- The minimum delegation amount is 1 SOL (1,000,000,000 lamports)
- Stake activation and deactivation follow epoch boundaries
- Lockup periods can prevent withdrawals until the specified time/epoch