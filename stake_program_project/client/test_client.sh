#!/bin/bash

# Example script showing how to use the stake client

echo "Stake Program Rust Client Examples"
echo "=================================="
echo ""

# Show help
echo "1. Show available commands:"
echo "cargo run -- --help"
echo ""

# Initialize a stake account
echo "2. Initialize a stake account:"
echo "cargo run -- initialize \\"
echo "  --staker 11111111111111111111111111111111 \\"
echo "  --withdrawer 11111111111111111111111111111111 \\"
echo "  --keypair-path ~/.config/solana/id.json"
echo ""

# Delegate stake
echo "3. Delegate stake to a validator:"
echo "cargo run -- delegate \\"
echo "  --stake-account <STAKE_ACCOUNT> \\"
echo "  --vote-account <VOTE_ACCOUNT> \\"
echo "  --keypair-path ~/.config/solana/id.json"
echo ""

# Check stake info
echo "4. View stake account information:"
echo "cargo run -- info \\"
echo "  --stake-account <STAKE_ACCOUNT> \\"
echo "  --keypair-path ~/.config/solana/id.json"
echo ""

# Deactivate stake
echo "5. Deactivate stake:"
echo "cargo run -- deactivate \\"
echo "  --stake-account <STAKE_ACCOUNT> \\"
echo "  --keypair-path ~/.config/solana/id.json"
echo ""

# Withdraw from stake
echo "6. Withdraw from stake account:"
echo "cargo run -- withdraw \\"
echo "  --stake-account <STAKE_ACCOUNT> \\"
echo "  --to <DESTINATION_ADDRESS> \\"
echo "  --lamports 1000000000 \\"
echo "  --keypair-path ~/.config/solana/id.json"
echo ""

echo "Note: Replace <STAKE_ACCOUNT>, <VOTE_ACCOUNT>, etc. with actual addresses"
echo "Note: Make sure you have a local Solana validator running or specify a different RPC with --rpc-url"