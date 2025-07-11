use std::{fs, io::Write};

use bs58;
use solana_sdk::{signature::Keypair, signer::Signer};

fn main() {
  // Create a new keypair
  let keypair = Keypair::new();
  let address = keypair.pubkey();

  println!("address: {address}");
  println!("base58: {}", bs58::encode(address.to_bytes()).into_string());

  // Display secret key
  let secret_key = keypair.to_bytes();
  println!("secret key: {:?}", secret_key);
  println!(
    "secret key base58: {}",
    bs58::encode(&secret_key).into_string()
  );

  // Save keypair to file
  save_keypair_to_file(&keypair, "keypair.json").expect("Failed to save keypair");
  println!("Keypair saved to keypair.json");

  // Load keypair from file
  let loaded_keypair = load_keypair_from_file("keypair.json").expect("Failed to load keypair");
  println!("Loaded keypair address: {}", loaded_keypair.pubkey());

  // Verify they're the same
  println!(
    "Keypairs match: {}",
    keypair.pubkey() == loaded_keypair.pubkey()
  );
}

fn save_keypair_to_file(
  keypair: &Keypair,
  filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let secret_key_bytes = keypair.to_bytes();
  let secret_key_array: Vec<u8> = secret_key_bytes.to_vec();

  let json_string = serde_json::to_string(&secret_key_array)?;
  let mut file = fs::File::create(filename)?;
  file.write_all(json_string.as_bytes())?;

  Ok(())
}

fn load_keypair_from_file(filename: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
  let file_content = fs::read_to_string(filename)?;
  let secret_key_array: Vec<u8> = serde_json::from_str(&file_content)?;

  let keypair = Keypair::try_from(&secret_key_array[..])?;
  Ok(keypair)
}
