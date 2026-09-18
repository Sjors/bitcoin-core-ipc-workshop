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

## How this workshop works

The workshop is a series of git tags. You are looking at `step.1`, which is also
the `master` branch. Every step ends with the `git checkout` command that takes
you to the next tag. Always start from `master`: the `workshop` branch only exists
to hold the tags, and its tip is the finished miner.

The `README.md` grows as you go: each tag adds the instructions for its own step
at the bottom, so after every checkout scroll down to the last section (or
follow the link to read it on GitHub).

Steps 1 to 3 are setup. From step 4 onwards every step is an exercise, and comes
as a pair of tags:

| Tag               | What you get                                                |
| ----------------- | ----------------------------------------------------------- |
| `step.N`          | The instructions for step N and starter code with `TODO`s.   |
| `step.N.solution` | The reference solution for step N. No new instructions.      |

So the loop is:

1. `git checkout step.N` and read the new section at the bottom of the README.
2. Implement the `TODO`s yourself. The starter code always compiles and runs, so
   you can use `cargo run` to try things out as you go.
3. `git checkout step.N.solution` to see the reference solution.
4. `git checkout step.N+1`, which continues from the reference solution (not from
   your code), so you can keep going even if you got stuck.

Git refuses to check out a tag while you have uncommitted changes to the same
files. Pick one:

```sh
# Compare your attempt with the solution, without switching
git diff step.4.solution -- src

# Keep your attempt on a branch of your own, then move on
git switch -c my-step-4
git commit -am "My step 4"
git checkout step.4.solution

# Or throw your attempt away
git checkout --force step.4.solution
```

Checking out a tag puts git in "detached HEAD" state. That's expected.

## Step 1 - Bitcoin Core

You need Bitcoin Core v32. Look in
https://bitcoincore.org/bin/bitcoin-core-32.0/ for a release, or a release
candidate in one of the `test.rc*` directories, and download the archive for
your platform.

Extract it in `bitcoin-core/` in this repo:

```sh
mkdir -p bitcoin-core
tar -xzf bitcoin-32.*.tar.gz \
  -C bitcoin-core \
  --strip-components=1
```

If there are no binaries (yet), build the `32.x` branch from source. Install the
dependencies for your platform, see the
[build docs](https://github.com/bitcoin/bitcoin/tree/32.x/doc#building). Those
include Cap'n Proto, which you'll also need later in this workshop. The workshop
does not need a wallet, GUI or tests, so a minimal build will do:

```sh
git clone --depth 1 --branch 32.x https://github.com/bitcoin/bitcoin.git bitcoin-src
cd bitcoin-src
cmake -B build -DENABLE_WALLET=OFF -DBUILD_TESTS=OFF
cmake --build build -j 8 --target bitcoin bitcoin-node bitcoin-cli
cd ..
ln -s bitcoin-src/build bitcoin-core
```

Either way, check that this works:

```sh
bitcoin-core/bin/bitcoin --version
```

The workshop does not work with v31 or older. If you use the Bitcoin Core
`master` branch, then later on you'll need to use the `master` branch instead of
`32.x` for `bitcoin-capnp-types`.

## Step 2 - Your very own signet

Start a fresh custom signet node. Use `bitcoin/` in this repository
as the data directory.

```sh
bitcoin-core/bin/bitcoin node \
  -datadir="$(pwd)/bitcoin"
```

The configuration is in `bitcoin/bitcoin.conf`. The `ipcbind=unix` setting makes
the node listen for IPC connections on the Unix socket
`bitcoin/signet/node.sock`.

The signet challenge `51` is `OP_1`, so any block only needs to satisfy proof of
work; see [BIP325](https://github.com/bitcoin/bips/blob/master/bip-0325.mediawiki).

Leave the node running for the rest of the workshop.

## Step 3 - Hello World in Rust

We'll leave Bitcoin Core running, so open another terminal tab for our Rust
application.

If you do not have Rust installed yet, follow the official installation
instructions at https://www.rust-lang.org/tools/install.

The `step.3` tag you just checked out added a minimal Rust application for you:
`Cargo.toml` and `src/main.rs`. There is nothing to write yet.

Check that the application runs:

```sh
cargo run
```

## Step 4 - IPC connection

This is the first exercise: connect to Bitcoin Core over IPC and print the
current chain tip.

Bitcoin Core's IPC interface uses [Cap'n Proto](https://capnproto.org/). The
[`2140-dev/bitcoin-capnp-types`](https://github.com/2140-dev/bitcoin-capnp-types)
crate generates Rust bindings from the Bitcoin Core v32 schemas, and is already in
`Cargo.toml` (using its `32.x` branch). Building it requires the `capnp`
compiler:

```sh
# macOS
brew install capnp
# Debian / Ubuntu
sudo apt-get install capnproto libcapnp-dev
```

The interfaces used in this step are defined in `capnp/init.capnp`,
`capnp/proxy.capnp` and `capnp/mining.capnp` in that crate. Every method `foo` in
a schema becomes a `foo_request()` method on the Rust client, which you use like
this:

```rust
let mut request = client.some_method_request();
request.get().set_some_param(42);
let response = request.send().promise.await?;
let result = response.get()?.get_result()?;
```

The starter code already opens the Unix socket and gives you the `Init` client.
From there:

1. Call `Init.construct`. The result contains a `ThreadMap`.
2. Call `ThreadMap.makePool` to have Bitcoin Core start a few worker threads for
   your connection. Bitcoin Core executes every IPC call on one of them.
3. Call `Init.makeMining` to get the `Mining` client.
4. Call `Mining.getTip` and return its height and hash.

Most methods take a `context :Proxy.Context` parameter, which can be used to pick
a specific worker thread. Thanks to `makePool` you can simply leave it unset.

The TODOs for this step are in:

- `src/ipc.rs`

Run the application with:

```sh
cargo run
```

The starter code prints a placeholder tip. When you're done it should match:

```sh
bitcoin-core/bin/bitcoin-cli -datadir="$(pwd)/bitcoin" getbestblockhash
```

By default the application connects to `./bitcoin/signet/node.sock`. Use
`--socket` if your node uses a different data directory.

When you're done, or stuck, use `git checkout step.4.solution` to see the
[Step 4 solution](https://github.com/Sjors/bitcoin-core-ipc-workshop/tree/step.4.solution).
