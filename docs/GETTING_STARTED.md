# Getting Started

Install tools, build the program, deploy, and fund a devnet wallet. For a minimal path, see [Quick Start](../README.md#quick-start) in the README.

## Prerequisites

| Tool | Version / notes |
|------|-----------------|
| [Rust](https://rustup.rs/) | Stable channel from `rust-toolchain.toml` with `rustfmt`, `clippy`, and `llvm-tools-preview` |
| [Solana CLI](https://docs.anza.xyz/cli/install) | 4.x / Agave; workspace crates use `solana-client` 4.1.0, `solana-program-test` 4.1.0, and `solana-sdk` 4.0.1 |
| [Anchor CLI](https://www.anchor-lang.com/docs/installation) | 1.0.2 via `Anchor.toml`; program crates use `anchor-lang` / `anchor-spl` 1.1.2 |
| [pnpm](https://pnpm.io/) | Anchor workspace metadata |
| [cargo-nextest](https://nexte.st/) | `cargo install cargo-nextest --locked` |

```bash
rustup component add rustfmt clippy
cargo install cargo-nextest --locked
```

### SBF compiler note

Host builds use the stable Rust channel, but `anchor build` delegates to Solana's SBF toolchain, which ships its own Rust compiler. Because Cargo validates `package.rust-version` with that bundled compiler, this workspace intentionally does not set a workspace `rust-version` higher than the SBF compiler supports. If integration tests panic with `Program file data not available for kzp_mini`, rebuild the SBF artifact first:

```bash
mkdir -p target/deploy
cp keys/program.json target/deploy/kzp_mini-keypair.json
NO_DNA=1 anchor build --ignore-keys
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

Program and demo keypairs live under `keys/` and are **not** in git - see [keys/README.md](../keys/README.md).

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
