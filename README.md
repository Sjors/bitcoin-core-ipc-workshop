# Bitcoin Core IPC Workshop

This repository is a small end-to-end demo for using the Bitcoin Core IPC mining
interface. In this workshop you'll build a simple Rust CPU miner that connects to
Bitcoin Core over IPC, asks for a block template, mines a (custom signet) block,
and submits the solution.

The proof-of-work difficulty is low enough that mining on a laptop should not be a
problem.

The Rust application uses the
[`2140-dev/bitcoin-capnp-types`](https://github.com/2140-dev/bitcoin-capnp-types)
crate for Bitcoin Core IPC bindings and the
[`rust-bitcoin`](https://github.com/rust-bitcoin/rust-bitcoin) crates for Bitcoin
data structures, consensus encoding, and hashing.

This workshop was generated with the help of this [skill](https://github.com/Sjors/skills/tree/master/tagged-workshop) and this [script](https://github.com/Sjors/dev-utils/blob/master/tagged_workshop_retag.py).

## Step 1 - Bitcoin Core

Download Bitcoin Core v31.0 from:
https://bitcoincore.org/bin/bitcoin-core-31.0/

Extract it in `bitcoin-core/` in this repo, e.g. for macOS:

```sh
curl -O https://bitcoincore.org/bin/bitcoin-core-31.0/bitcoin-31.0-arm64-apple-darwin.tar.gz
mkdir -p bitcoin-core
tar -xzf bitcoin-31.0-arm64-apple-darwin.tar.gz \
  -C bitcoin-core \
  --strip-components=1
```

Or build from source. If you use the `master` branch, then in the instructions
below you'll need to use the `master` branch instead of  `31.x` for
`bitcoin-capnp-types`.

## Step 2 - Your very own signet

Start a fresh custom signet node. Use `bitcoin/` in this repository
as the data directory.

```sh
bitcoin-core/bin/bitcoin node \
  -datadir="$(pwd)/bitcoin"
```

The signet challenge `51` is `OP_1`, so any block only needs to satisfy proof of
work; see [BIP325](https://github.com/bitcoin/bips/blob/master/bip-0325.mediawiki).

If you compiled master from source you can skip the next step, as this was fixed in
[bitcoin/bitcoin#34860](https://github.com/bitcoin/bitcoin/pull/34860).

Bitcoin Core's IPC `createNewBlock` path needs the chain to be past the first 16
blocks because the low-height BIP34 coinbase prefix is too short before the
miner can append its extra nonce.

Bootstrap a fresh workshop chain once (in a new terminal tab):

```sh
bitcoin-core/bin/bitcoin-cli \
  -datadir="$(pwd)/bitcoin" \
  generatetodescriptor 17 "raw(51)" 100000000
```

## Step 3 - Hello World in Rust

We'll leave Bitcoin Core running, so open another terminal tab for our Rust
application.

If you do not have Rust installed yet, follow the official installation
instructions at https://www.rust-lang.org/tools/install.

This step adds a minimal Rust application.

Check that the application runs:

```sh
cargo run
```

## Step 4 - IPC connection

Now connect to Bitcoin Core over IPC and print the current chain tip.

Most of the supporting crates for this step are already in `Cargo.toml`. Add the
`31.x` branch of
[`2140-dev/bitcoin-capnp-types`](https://github.com/2140-dev/bitcoin-capnp-types)
for Rust bindings generated from the Bitcoin Core v31 IPC schemas.

The `init`, `proxy`, and `mining` interfaces used in this step are defined in
`capnp/init.capnp`, `capnp/proxy.capnp`, and `capnp/mining.capnp` in that crate.

The TODOs for this step are in:

- `src/ipc.rs`

## Step 5 - Block template

Next, ask Bitcoin Core for a mining block template and inspect the fields the IPC
interface gives you.

The TODOs for this step are in:

- `src/app.rs`

## Step 6 - Header mining

This step adds the local proof-of-work code for you, so the exercise can stay
focused on IPC.

Run the miner:

```sh
cargo run --release -- --threads 4
```

This step starts from a fake all-zero header and an easy fake target. Replace
those with the IPC block header and target so the miner searches for a valid
nonce for the real template.

The TODOs for this step are in:

- `src/app.rs`

## Step 7 - Submit the block

Mining the IPC header proves the proof-of-work loop works, but Bitcoin Core still
needs the matching coinbase transaction before it can accept the block.

This step provides the coinbase and merkle-root plumbing. Finish the miner by
submitting the solved header fields and serialized coinbase with `submitSolution`.

The TODOs for this step are in:

- `src/app.rs`

By default the miner connects to `./bitcoin/signet/node.sock`. Use `--socket` if
your node uses a different data directory.

Run the miner:

```sh
cargo run --release -- --threads 4
```
