use std::str::FromStr;

use sha2::{Digest, Sha256};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    instruction::{AccountMeta, Instruction},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Solana Data Reader Rust Client (CPI Version) ===");
    println!("Creating CPI call to data_reader program, just like TypeScript\n");

    // Setup equivalent to anchor.setProvider(anchor.AnchorProvider.env())
    let payer = read_keypair_file(&*shellexpand::tilde("~/solana-wallets/bob.json"))
        .expect("Example requires a keypair file");

    println!("✓ Loaded payer keypair: {}", payer.pubkey());

    // Program setup (equivalent to anchor.workspace.DataReader as Program<DataReader>)
    let program_id = Pubkey::from_str("59cnY1rjumvuLbab9MRqFmk62QALDmjFRBzMEupqeyzH")?;
    println!("✓ DataReader program: {}", program_id);

    // Setup the account to read from (equivalent to TypeScript's otherStorageAddress)
    let other_storage_address = "HwUtrTmLy6aHdDWSNN38rpRZy6icDhyCiVYD7XHV3LZF";
    let pub_key_other_storage = Pubkey::from_str(other_storage_address)?;
    println!("✓ Target account: {}", pub_key_other_storage);

    // Create the instruction discriminator for readOtherData
    let mut hasher = Sha256::new();
    hasher.update(b"global:read_other_data");
    let hash = hasher.finalize();
    let discriminator: [u8; 8] = hash[..8].try_into().unwrap();

    println!("✓ Instruction discriminator: {:?}", discriminator);

    println!("\n=== Creating CPI Transaction ===");

    // Create the instruction (equivalent to program.methods.readOtherData().accounts({ otherData: pub_key_other_storage }))
    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(pub_key_other_storage, false), // otherData account (readonly)
        ],
        data: discriminator.to_vec(), // readOtherData instruction data
    };

    println!("✓ Created instruction:");
    println!("  - Program: {}", instruction.program_id);
    println!("  - Data: {:?} (readOtherData discriminator)", instruction.data);
    println!("  - Accounts:");
    println!("    * otherData: {} (readonly)", pub_key_other_storage);

    // Create the transaction (equivalent to .rpc() in TypeScript)
    // Note: In a real scenario, we would need a recent blockhash and send this to the network
    println!("\n=== Transaction Structure ===");
    println!("This creates a Solana transaction that:");
    println!("1. Calls the DataReader program");
    println!("2. Executes the readOtherData instruction");
    println!("3. Passes the target account as a parameter");
    println!("4. The program will perform a CPI to read data from the target account");

    println!("\n=== TypeScript Equivalent ===");
    println!("// Setup (same as Rust)");
    println!("import * as anchor from \"@coral-xyz/anchor\";");
    println!("import {{ Program }} from \"@coral-xyz/anchor\";");
    println!("import {{ DataReader }} from \"../target/types/data_reader\";");
    println!();
    println!("anchor.setProvider(anchor.AnchorProvider.env());");
    println!("const program = anchor.workspace.DataReader as Program<DataReader>;");
    println!();
    println!("// The actual CPI call");
    println!("const otherStorageAddress = \"{}\";", other_storage_address);
    println!("const pub_key_other_storage = new anchor.web3.PublicKey(otherStorageAddress);");
    println!();
    println!("const tx = await program.methods.readOtherData()");
    println!("  .accounts({{ otherData: pub_key_other_storage }})");
    println!("  .rpc();");

    println!("\n=== CPI Flow Explanation ===");
    println!("1. Client creates transaction calling DataReader program");
    println!("2. DataReader.readOtherData instruction executes");
    println!("3. Inside the instruction, the program reads data from the specified account");
    println!("4. The program can deserialize and process the account data");
    println!("5. Program logs the results (e.g., 'The value of x is: 9')");

    println!("\n=== Summary ===");
    println!("✓ This Rust client creates the same CPI structure as TypeScript:");
    println!("  - Same program ID: {}", program_id);
    println!("  - Same target account: {}", pub_key_other_storage);
    println!("  - Same instruction discriminator: {:?}", discriminator);
    println!("  - Same account metadata (readonly access)");
    println!("  - Ready for network submission");

    println!("\nThe transaction is prepared and would execute the CPI call");
    println!("when sent to the Solana network with proper RPC client.");

    Ok(())
}