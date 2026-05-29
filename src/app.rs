#![allow(unused_imports)]

use anyhow::{Context, Result};

use crate::{
    block_header::{BITS_OFFSET, UINT32_LEN},
    cli::Config,
    ipc::{IpcMiningClient, bootstrap_hint},
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
    println!("target: compact bits {bits:08x}");

    template.destroy().await?;
    Ok(())
}
