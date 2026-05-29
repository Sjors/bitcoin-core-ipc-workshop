# Bitcoin Core IPC Workshop

This repository is a small end-to-end demo for using the Bitcoin Core IPC mining
interface. In this workshop you'll build a simple Rust CPU miner that connects to
Bitcoin Core over IPC, asks for a block template, mines a (custom signet) block,
and submits the solution.

The proof-of-work difficulty is low enough that mining on a laptop should not be a
problem.

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

## Step 2 ...

Use `git checkout step.2` to move to [step 2](https://github.com/Sjors/bitcoin-core-ipc-workshop/tree/step.2).
