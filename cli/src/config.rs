//! Cluster and wallet configuration.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, read_keypair_file};

/// Supported Solana clusters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Cluster {
    Localnet,
    Devnet,
    Mainnet,
}

impl Cluster {
    pub fn default_rpc_url(self) -> &'static str {
        match self {
            Cluster::Localnet => "http://127.0.0.1:8899",
            Cluster::Devnet => "https://api.devnet.solana.com",
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com",
        }
    }

    pub fn from_rpc_url(rpc_url: &str) -> Self {
        if rpc_url.contains("devnet") {
            Cluster::Devnet
        } else if rpc_url.contains("mainnet") {
            Cluster::Mainnet
        } else {
            Cluster::Localnet
        }
    }
}

/// Resolved runtime configuration for RPC calls and signing.
pub struct AppConfig {
    pub cluster: Cluster,
    pub rpc_url: String,
    pub wallet_path: PathBuf,
    pub keypair: Keypair,
}

impl AppConfig {
    pub fn load(
        cluster: Option<Cluster>,
        rpc_url: Option<String>,
        wallet: Option<String>,
    ) -> Result<Self> {
        let wallet_path = wallet
            .map(PathBuf::from)
            .or_else(default_wallet_path)
            .context("wallet path not provided and Solana config not found")?;

        let keypair = read_keypair_file(&wallet_path)
            .map_err(|e| anyhow::anyhow!("failed to read wallet {}: {e}", wallet_path.display()))?;

        let rpc_url = rpc_url.unwrap_or_else(|| {
            let cluster = cluster.unwrap_or_else(detect_cluster_from_solana_config);
            cluster.default_rpc_url().to_string()
        });

        let cluster = cluster.unwrap_or_else(|| Cluster::from_rpc_url(&rpc_url));

        Ok(Self {
            cluster,
            rpc_url,
            wallet_path,
            keypair,
        })
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn default_wallet_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".config/solana/id.json"))
}

fn detect_cluster_from_solana_config() -> Cluster {
    let config_path = home_dir().map(|h| h.join(".config/solana/cli/config.yml"));
    if let Some(path) = config_path.filter(|p| p.exists())
        && let Ok(contents) = std::fs::read_to_string(path)
    {
        if contents.contains("devnet") {
            return Cluster::Devnet;
        }
        if contents.contains("mainnet") {
            return Cluster::Mainnet;
        }
    }
    Cluster::Devnet
}
