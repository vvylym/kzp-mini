# Getting Started

Install tools, build the program, deploy, and fund a devnet wallet. For a minimal path, see [Quick Start](../README.md#quick-start) in the README.

## Prerequisites

| Tool | Version / notes |
|------|-----------------|
| [Rust](https://rustup.rs/) | 1.89 (`rust-toolchain.toml`) — `rustfmt`, `clippy` |
| [Solana CLI](https://docs.anza.xyz/cli/install) | 3.x (Solana 4.x not supported) |
| [Anchor](https://www.anchor-lang.com/docs/installation) | 1.0.2 (`avm install 1.0.2`) |
| [pnpm](https://pnpm.io/) | Anchor workspace metadata |
| [cargo-nextest](https://nexte.st/) | `cargo install cargo-nextest --locked` |

```bash
rustup component add rustfmt clippy
cargo install cargo-nextest --locked
```

## Clone and build

```bash
git clone https://github.com/vvylym/kzp-mini.git
cd kzp-mini

mkdir -p target/deploy
cp keys/program.json target/deploy/kzp_mini-keypair.json   # see keys/README.md
NO_DNA=1 anchor build --ignore-keys

cargo build -p kzp-cli --release
export PATH="$PWD/target/release:$PATH"
```

Program and demo keypairs live under `keys/` and are **not** in git — see [keys/README.md](../keys/README.md).

## Deploy

```bash
bash scripts/deploy.sh devnet    # or localnet / mainnet
```

Requires `keys/program.json` matching the program ID in `declare_id!`. First devnet deploy needs ~3 SOL on the deploy wallet.

## Devnet wallet and SPL mint

After build you still need a funded wallet, the program live on devnet, and an SPL mint with tokens in member ATAs:

```bash
solana-keygen new                                    # if needed
solana config set --url devnet
solana airdrop 2

# Skip if program already deployed at README program ID
bash scripts/deploy.sh devnet

spl-token create-token --decimals 6
spl-token create-account <MINT>
spl-token mint <MINT> 1000000
```

Or run the full scripted lifecycle:

```bash
bash scripts/devnet-demo.sh
```

## Verify

```bash
kzp config
```

Prints RPC URL, cluster, wallet, payer, and program ID.

## Cluster and wallet overrides

Every CLI subcommand accepts:

- `--cluster localnet|devnet|mainnet`
- `--rpc-url <URL>`
- `--wallet /path/to/keypair.json`
- `--dry-run` (simulate without sending)

Default: devnet RPC, `~/.config/solana/id.json`.
