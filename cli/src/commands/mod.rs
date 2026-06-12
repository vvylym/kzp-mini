//! CLI command handlers.

pub mod loan;
pub mod pool;

use anyhow::Result;
use serde_json::json;

use crate::client::KzpClient;
use crate::config::{AppConfig, Cluster};
use kzp_mini::ID as PROGRAM_ID;

/// Shared context for all subcommands.
pub struct CommandContext {
    pub client: KzpClient,
    pub dry_run: bool,
}

impl CommandContext {
    pub fn new(
        cluster: Option<Cluster>,
        rpc_url: Option<String>,
        wallet: Option<String>,
        dry_run: bool,
    ) -> Result<Self> {
        let config = AppConfig::load(cluster, rpc_url, wallet)?;
        let client = KzpClient::new(config);
        Ok(Self { client, dry_run })
    }
}

/// Prints resolved configuration as JSON.
pub fn show_config(ctx: &CommandContext) -> Result<()> {
    let cfg = &ctx.client.config;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "cluster": format!("{:?}", cfg.cluster).to_lowercase(),
            "rpc_url": cfg.rpc_url,
            "wallet": cfg.wallet_path.display().to_string(),
            "payer": ctx.client.payer_pubkey().to_string(),
            "program_id": PROGRAM_ID.to_string(),
        }))?
    );
    Ok(())
}
