#![allow(unused_imports)]

use anyhow::{Context, Result};

use crate::{
    block_header::{BITS_OFFSET, UINT32_LEN},
    cli::Config,
    fixtures::ci_solution,
    ipc::{IpcMiningClient, bootstrap_hint},
    mining_job::MiningWork,
    pow::{mine_round, target_from_bits},
};

pub async fn run(config: Config) -> Result<()> {
    let clients = IpcMiningClient::connect(&config.socket).await?;
    let tip = clients.tip().await?;
    let template = clients
        .create_block_template()
        .await
        .with_context(|| bootstrap_hint(tip.height))?;
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

    let work = MiningWork { header };
    let target = target_from_bits(bits);
    println!("target: {target:064x}");

    let found = ci_solution(&config.fixtures, config.ci, tip.height + 1, &target)?
        .or_else(|| mine_round(&work, &target, config.threads, 0));
    if let Some(found) = found {
        println!(
            "found header: version={:#x} timestamp={} nonce={}",
            found.version, found.timestamp, found.nonce
        );
    }

    template.destroy().await?;
    Ok(())
}
