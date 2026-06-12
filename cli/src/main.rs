//! KZP CLI - interact with the minimal mutual-aid pool on any Solana cluster.

mod accounts;
mod client;
mod commands;
mod config;
mod ix;
mod pda;

use anyhow::Result;
use clap::{Parser, Subcommand};
use commands::CommandContext;

use crate::config::Cluster;

#[derive(Parser, Debug)]
#[command(
    name = "kzp",
    about = "KZP minimal pool CLI - localnet, devnet, or mainnet",
    version,
    propagate_version = true
)]
struct Cli {
    /// Target cluster (overrides config file).
    #[arg(long, global = true, value_enum)]
    cluster: Option<Cluster>,

    /// RPC URL (overrides cluster default).
    #[arg(long, global = true)]
    rpc_url: Option<String>,

    /// Path to the signing keypair JSON file.
    #[arg(long, global = true)]
    wallet: Option<String>,

    /// Simulate the transaction without sending.
    #[arg(long, global = true)]
    dry_run: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show resolved RPC URL, cluster, wallet, and program ID.
    Config,
    /// Pool lifecycle commands.
    Pool {
        #[command(subcommand)]
        action: PoolCommands,
    },
    /// Loan lifecycle commands.
    Loan {
        #[command(subcommand)]
        action: LoanCommands,
    },
}

#[derive(Subcommand, Debug)]
enum PoolCommands {
    /// Create a new pool and vault for an SPL token mint.
    Initialize {
        #[arg(long)]
        name: String,
        #[arg(long)]
        entry_fee: u64,
        #[arg(long)]
        mint: String,
        /// Admin pubkey (defaults to wallet).
        #[arg(long)]
        admin: Option<String>,
    },
    /// Pay entry fee and join a pool.
    Join {
        #[arg(long)]
        pool: String,
        #[arg(long)]
        entry_fee: u64,
    },
    /// Deposit savings into the pool vault.
    Deposit {
        #[arg(long)]
        pool: String,
        #[arg(long)]
        amount: u64,
    },
    /// Withdraw savings and close the member account.
    Exit {
        #[arg(long)]
        pool: String,
    },
}

#[derive(Subcommand, Debug)]
enum LoanCommands {
    /// Request a new loan with two guarantors.
    Request {
        #[arg(long)]
        pool: String,
        #[arg(long)]
        nonce: u64,
        #[arg(long)]
        amount: u64,
        #[arg(long)]
        guarantor_a: String,
        #[arg(long)]
        guarantor_b: String,
    },
    /// Co-sign a pending loan (pool and borrower are read from the loan account).
    Cosign {
        #[arg(long)]
        loan: String,
        /// Borrower token ATA (derived from the loan's pool mint if omitted).
        #[arg(long)]
        borrower_ata: Option<String>,
    },
    /// Repay an active loan (pool and guarantors are read from the loan account).
    Repay {
        #[arg(long)]
        loan: String,
        #[arg(long)]
        amount: u64,
    },
    /// Cancel a pending loan (borrower only).
    Cancel {
        #[arg(long)]
        loan: String,
    },
    /// Withdraw a partial co-sign on a pending loan.
    WithdrawCosign {
        #[arg(long)]
        loan: String,
    },
    /// Admin marks an active loan as defaulted (50/50 from guarantor savings + SPL to vault).
    SettleDefault {
        #[arg(long)]
        loan: String,
        #[arg(long)]
        pool: String,
        #[arg(long)]
        guarantor_a_wallet: String,
        #[arg(long)]
        guarantor_b_wallet: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = CommandContext::new(cli.cluster, cli.rpc_url, cli.wallet, cli.dry_run)?;

    match cli.command {
        Commands::Config => commands::show_config(&ctx),
        Commands::Pool { action } => match action {
            PoolCommands::Initialize {
                name,
                entry_fee,
                mint,
                admin,
            } => commands::pool::initialize_pool(&ctx, &name, entry_fee, &mint, admin.as_deref()),
            PoolCommands::Join { pool, entry_fee } => {
                commands::pool::join_pool(&ctx, &pool, entry_fee)
            }
            PoolCommands::Deposit { pool, amount } => {
                commands::pool::deposit_savings(&ctx, &pool, amount)
            }
            PoolCommands::Exit { pool } => commands::pool::exit_pool(&ctx, &pool),
        },
        Commands::Loan { action } => match action {
            LoanCommands::Request {
                pool,
                nonce,
                amount,
                guarantor_a,
                guarantor_b,
            } => {
                commands::loan::request_loan(&ctx, &pool, nonce, amount, &guarantor_a, &guarantor_b)
            }
            LoanCommands::Cosign { loan, borrower_ata } => {
                commands::loan::co_sign_loan(&ctx, &loan, borrower_ata.as_deref())
            }
            LoanCommands::Repay { loan, amount } => commands::loan::repay_loan(&ctx, &loan, amount),
            LoanCommands::Cancel { loan } => commands::loan::cancel_loan(&ctx, &loan),
            LoanCommands::WithdrawCosign { loan } => commands::loan::withdraw_cosign(&ctx, &loan),
            LoanCommands::SettleDefault {
                loan,
                pool,
                guarantor_a_wallet,
                guarantor_b_wallet,
            } => commands::loan::settle_default(
                &ctx,
                &loan,
                &pool,
                &guarantor_a_wallet,
                &guarantor_b_wallet,
            ),
        },
    }
}
