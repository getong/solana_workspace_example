use std::{rc::Rc, str::FromStr};

use anchor_client::{
  solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
  },
  Client, Cluster, Program,
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use solana_system_interface::program as system_program;
use stake_program_project::{
  accounts, instruction, Authorized, Lockup, LockupArgs, StakeAuthorize,
};

#[derive(Parser)]
#[command(name = "stake-client")]
#[command(about = "Solana Stake Program Client", long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,

  #[arg(short, long, default_value = "http://localhost:8899")]
  rpc_url: String,

  #[arg(short, long)]
  keypair_path: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
  Initialize {
    #[arg(long)]
    staker: String,
    #[arg(long)]
    withdrawer: String,
    #[arg(long, default_value = "0")]
    lockup_timestamp: i64,
    #[arg(long, default_value = "0")]
    lockup_epoch: u64,
    #[arg(long)]
    custodian: Option<String>,
  },
  Authorize {
    #[arg(long)]
    stake_account: String,
    #[arg(long)]
    new_authority: String,
    #[arg(long, value_enum)]
    stake_authorize: AuthorizeType,
  },
  Delegate {
    #[arg(long)]
    stake_account: String,
    #[arg(long)]
    vote_account: String,
  },
  Split {
    #[arg(long)]
    source_account: String,
    #[arg(long)]
    lamports: u64,
  },
  Withdraw {
    #[arg(long)]
    stake_account: String,
    #[arg(long)]
    to: String,
    #[arg(long)]
    lamports: u64,
  },
  Deactivate {
    #[arg(long)]
    stake_account: String,
  },
  SetLockup {
    #[arg(long)]
    stake_account: String,
    #[arg(long)]
    unix_timestamp: Option<i64>,
    #[arg(long)]
    epoch: Option<u64>,
    #[arg(long)]
    custodian: Option<String>,
  },
  Merge {
    #[arg(long)]
    source_account: String,
    #[arg(long)]
    dest_account: String,
  },
  GetMinimumDelegation,
  Info {
    #[arg(long)]
    stake_account: String,
  },
}

#[derive(clap::ValueEnum, Clone)]
enum AuthorizeType {
  Staker,
  Withdrawer,
}

impl From<AuthorizeType> for StakeAuthorize {
  fn from(auth_type: AuthorizeType) -> Self {
    match auth_type {
      AuthorizeType::Staker => StakeAuthorize::Staker,
      AuthorizeType::Withdrawer => StakeAuthorize::Withdrawer,
    }
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let cli = Cli::parse();

  let payer = if let Some(path) = cli.keypair_path {
    let keypair_bytes = std::fs::read(&path)?;
    Keypair::try_from(&keypair_bytes[..])?
  } else {
    println!("Warning: Using new random keypair. Specify --keypair-path for a real keypair.");
    Keypair::new()
  };

  let cluster_url = if cli.rpc_url.starts_with("http") {
    Cluster::Custom(cli.rpc_url.clone(), cli.rpc_url.replace("http", "ws"))
  } else {
    match cli.rpc_url.as_str() {
      "devnet" => Cluster::Devnet,
      "testnet" => Cluster::Testnet,
      "mainnet" => Cluster::Mainnet,
      _ => Cluster::Localnet,
    }
  };

  let client = Client::new_with_options(cluster_url, Rc::new(payer), CommitmentConfig::confirmed());

  let program_id = stake_program_project::ID;
  let program = client.program(program_id)?;

  match cli.command {
    Commands::Initialize {
      staker,
      withdrawer,
      lockup_timestamp,
      lockup_epoch,
      custodian,
    } => {
      initialize_stake_account(
        &program,
        staker,
        withdrawer,
        lockup_timestamp,
        lockup_epoch,
        custodian,
      )
      .await?;
    }
    Commands::Authorize {
      stake_account,
      new_authority,
      stake_authorize,
    } => {
      authorize_stake_account(
        &program,
        stake_account,
        new_authority,
        stake_authorize.into(),
      )
      .await?;
    }
    Commands::Delegate {
      stake_account,
      vote_account,
    } => {
      delegate_stake(&program, stake_account, vote_account).await?;
    }
    Commands::Split {
      source_account,
      lamports,
    } => {
      split_stake(&program, source_account, lamports).await?;
    }
    Commands::Withdraw {
      stake_account,
      to,
      lamports,
    } => {
      withdraw_stake(&program, stake_account, to, lamports).await?;
    }
    Commands::Deactivate { stake_account } => {
      deactivate_stake(&program, stake_account).await?;
    }
    Commands::SetLockup {
      stake_account,
      unix_timestamp,
      epoch,
      custodian,
    } => {
      set_lockup(&program, stake_account, unix_timestamp, epoch, custodian).await?;
    }
    Commands::Merge {
      source_account,
      dest_account,
    } => {
      merge_stake(&program, source_account, dest_account).await?;
    }
    Commands::GetMinimumDelegation => {
      get_minimum_delegation(&program).await?;
    }
    Commands::Info { stake_account } => {
      show_stake_info(&program, stake_account).await?;
    }
  }

  Ok(())
}

async fn initialize_stake_account(
  program: &Program<Rc<Keypair>>,
  staker: String,
  withdrawer: String,
  lockup_timestamp: i64,
  lockup_epoch: u64,
  custodian: Option<String>,
) -> Result<()> {
  let stake_account = Keypair::new();
  let staker_pubkey = Pubkey::from_str(&staker)?;
  let withdrawer_pubkey = Pubkey::from_str(&withdrawer)?;
  let custodian_pubkey = custodian
    .map(|c| Pubkey::from_str(&c))
    .transpose()?
    .unwrap_or_else(|| Pubkey::new_unique());

  let authorized = Authorized {
    staker: staker_pubkey,
    withdrawer: withdrawer_pubkey,
  };

  let lockup = Lockup {
    unix_timestamp: lockup_timestamp,
    epoch: lockup_epoch,
    custodian: custodian_pubkey,
  };

  println!("Initializing stake account: {}", stake_account.pubkey());

  let sig = program
    .request()
    .accounts(accounts::Initialize {
      stake_account: stake_account.pubkey(),
      payer: program.payer(),
      rent: anchor_client::solana_sdk::sysvar::rent::id(),
      system_program: system_program::id(),
    })
    .args(instruction::Initialize { authorized, lockup })
    .signer(&stake_account)
    .send()?;

  println!("Transaction signature: {}", sig);
  println!("Stake account created: {}", stake_account.pubkey());

  Ok(())
}

async fn authorize_stake_account(
  program: &Program<Rc<Keypair>>,
  stake_account: String,
  new_authority: String,
  stake_authorize: StakeAuthorize,
) -> Result<()> {
  let stake_account_pubkey = Pubkey::from_str(&stake_account)?;
  let new_authority_pubkey = Pubkey::from_str(&new_authority)?;

  println!("Authorizing stake account: {}", stake_account_pubkey);
  println!("New authority: {}", new_authority_pubkey);

  let sig = program
    .request()
    .accounts(accounts::Authorize {
      stake_account: stake_account_pubkey,
      authority: program.payer(),
      clock: anchor_client::solana_sdk::sysvar::clock::id(),
    })
    .args(instruction::Authorize {
      new_authority: new_authority_pubkey,
      stake_authorize,
    })
    .send()?;

  println!("Transaction signature: {}", sig);

  Ok(())
}

async fn delegate_stake(
  program: &Program<Rc<Keypair>>,
  stake_account: String,
  vote_account: String,
) -> Result<()> {
  let stake_account_pubkey = Pubkey::from_str(&stake_account)?;
  let vote_account_pubkey = Pubkey::from_str(&vote_account)?;

  println!("Delegating stake account: {}", stake_account_pubkey);
  println!("To vote account: {}", vote_account_pubkey);

  let sig = program
    .request()
    .accounts(accounts::DelegateStake {
      stake_account: stake_account_pubkey,
      vote_account: vote_account_pubkey,
      staker: program.payer(),
      clock: anchor_client::solana_sdk::sysvar::clock::id(),
    })
    .args(instruction::DelegateStake {})
    .send()?;

  println!("Transaction signature: {}", sig);

  Ok(())
}

async fn split_stake(
  program: &Program<Rc<Keypair>>,
  source_account: String,
  lamports: u64,
) -> Result<()> {
  let source_account_pubkey = Pubkey::from_str(&source_account)?;
  let split_account = Keypair::new();

  println!("Splitting stake account: {}", source_account_pubkey);
  println!("New split account: {}", split_account.pubkey());
  println!("Amount: {} lamports", lamports);

  let sig = program
    .request()
    .accounts(accounts::Split {
      source_account: source_account_pubkey,
      split_account: split_account.pubkey(),
      staker: program.payer(),
      rent: anchor_client::solana_sdk::sysvar::rent::id(),
      system_program: system_program::id(),
    })
    .args(instruction::Split { lamports })
    .signer(&split_account)
    .send()?;

  println!("Transaction signature: {}", sig);
  println!("Split account created: {}", split_account.pubkey());

  Ok(())
}

async fn withdraw_stake(
  program: &Program<Rc<Keypair>>,
  stake_account: String,
  to: String,
  lamports: u64,
) -> Result<()> {
  let stake_account_pubkey = Pubkey::from_str(&stake_account)?;
  let to_pubkey = Pubkey::from_str(&to)?;

  println!("Withdrawing from stake account: {}", stake_account_pubkey);
  println!("To: {}", to_pubkey);
  println!("Amount: {} lamports", lamports);

  let sig = program
    .request()
    .accounts(accounts::Withdraw {
      stake_account: stake_account_pubkey,
      withdrawer: program.payer(),
      to: to_pubkey,
      clock: anchor_client::solana_sdk::sysvar::clock::id(),
    })
    .args(instruction::Withdraw { lamports })
    .send()?;

  println!("Transaction signature: {}", sig);

  Ok(())
}

async fn deactivate_stake(program: &Program<Rc<Keypair>>, stake_account: String) -> Result<()> {
  let stake_account_pubkey = Pubkey::from_str(&stake_account)?;

  println!("Deactivating stake account: {}", stake_account_pubkey);

  let sig = program
    .request()
    .accounts(accounts::Deactivate {
      stake_account: stake_account_pubkey,
      staker: program.payer(),
      clock: anchor_client::solana_sdk::sysvar::clock::id(),
    })
    .args(instruction::Deactivate {})
    .send()?;

  println!("Transaction signature: {}", sig);

  Ok(())
}

async fn set_lockup(
  program: &Program<Rc<Keypair>>,
  stake_account: String,
  unix_timestamp: Option<i64>,
  epoch: Option<u64>,
  custodian: Option<String>,
) -> Result<()> {
  let stake_account_pubkey = Pubkey::from_str(&stake_account)?;
  let custodian_pubkey = custodian.map(|c| Pubkey::from_str(&c)).transpose()?;

  let lockup_args = LockupArgs {
    unix_timestamp,
    epoch,
    custodian: custodian_pubkey,
  };

  println!("Setting lockup for stake account: {}", stake_account_pubkey);

  let sig = program
    .request()
    .accounts(accounts::SetLockup {
      stake_account: stake_account_pubkey,
      custodian: program.payer(),
    })
    .args(instruction::SetLockup {
      lockup: lockup_args,
    })
    .send()?;

  println!("Transaction signature: {}", sig);

  Ok(())
}

async fn merge_stake(
  program: &Program<Rc<Keypair>>,
  source_account: String,
  dest_account: String,
) -> Result<()> {
  let source_account_pubkey = Pubkey::from_str(&source_account)?;
  let dest_account_pubkey = Pubkey::from_str(&dest_account)?;

  println!("Merging stake accounts:");
  println!("Source: {}", source_account_pubkey);
  println!("Destination: {}", dest_account_pubkey);

  let sig = program
    .request()
    .accounts(accounts::Merge {
      source_account: source_account_pubkey,
      dest_account: dest_account_pubkey,
      staker: program.payer(),
      clock: anchor_client::solana_sdk::sysvar::clock::id(),
    })
    .args(instruction::Merge {})
    .send()?;

  println!("Transaction signature: {}", sig);

  Ok(())
}

async fn get_minimum_delegation(program: &Program<Rc<Keypair>>) -> Result<()> {
  println!("Getting minimum delegation amount...");

  let sig = program
    .request()
    .accounts(accounts::GetMinimumDelegation {})
    .args(instruction::GetMinimumDelegation {})
    .send()?;

  println!("Transaction signature: {}", sig);
  println!(
    "Note: Check the return data in the transaction logs for the minimum delegation amount."
  );

  Ok(())
}

async fn show_stake_info(program: &Program<Rc<Keypair>>, stake_account: String) -> Result<()> {
  let stake_account_pubkey = Pubkey::from_str(&stake_account)?;

  println!("Fetching stake account info: {}", stake_account_pubkey);

  let account: stake_program_project::StakeAccount = program.account(stake_account_pubkey)?;

  println!("\nStake Account Info:");
  println!("==================");
  println!("Meta:");
  println!(
    "  Rent Exempt Reserve: {} lamports",
    account.meta.rent_exempt_reserve
  );
  println!("  Staker: {}", account.meta.authorized.staker);
  println!("  Withdrawer: {}", account.meta.authorized.withdrawer);
  println!("  Lockup:");
  println!("    Unix Timestamp: {}", account.meta.lockup.unix_timestamp);
  println!("    Epoch: {}", account.meta.lockup.epoch);
  println!("    Custodian: {}", account.meta.lockup.custodian);

  if let Some(stake) = &account.stake {
    println!("\nStake:");
    println!("  Voter: {}", stake.delegation.voter_pubkey);
    println!("  Stake: {} lamports", stake.delegation.stake);
    println!("  Activation Epoch: {}", stake.delegation.activation_epoch);
    println!(
      "  Deactivation Epoch: {}",
      stake.delegation.deactivation_epoch
    );
    println!(
      "  Warmup/Cooldown Rate: {}",
      stake.delegation.warmup_cooldown_rate
    );
    println!("  Credits Observed: {}", stake.credits_observed);
  } else {
    println!("\nNo active stake delegation");
  }

  Ok(())
}
