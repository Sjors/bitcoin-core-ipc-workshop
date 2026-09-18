// TODO: Remove this line when you're done.
#![allow(unused_imports)]

use anyhow::Result;
use bitcoin::Target;

use crate::{
    block_header::{BITS_OFFSET, UINT32_LEN},
    cli::Config,
    fixtures::ci_solution,
    ipc::IpcMiningClient,
    mining_job::MiningWork,
    pow::{mine_round, target_from_bits},
};

pub async fn run(config: Config) -> Result<()> {
    let clients = IpcMiningClient::connect(&config.socket).await?;
    let tip = clients.tip().await?;
    let template = clients.create_block_template().await?;
    let header = template.block_header().await?;
    let coinbase_template = template.coinbase_template().await?;
    let _merkle_path = template.coinbase_merkle_path().await?;
    let bits = u32::from_le_bytes(
        header[BITS_OFFSET..BITS_OFFSET + UINT32_LEN]
            .try_into()
            .unwrap(),
    );

    println!("tip height: {}", tip.height);
    println!("tip hash: {}", tip.hash);
    println!(
        "coinbase remaining reward: {} sats",
        coinbase_template.block_reward_remaining
    );

    // TODO: Build MiningWork from the IPC block header instead of this fake header.
    let work = MiningWork { header: [0u8; 80] };

    // TODO: Convert the IPC block header's nBits value into a mining target with
    // target_from_bits().
    let _bits = bits;
    let target = Target::MAX_ATTAINABLE_REGTEST;
    println!("target: {target:064x}");

    // TODO: If a header was found, print its version, timestamp and nonce.
    let _found = ci_solution(&config.fixtures, config.ci, tip.height + 1, &target)?
        .or_else(|| mine_round(&work, &target, config.threads, 0));

    template.destroy().await?;
    Ok(())
}
