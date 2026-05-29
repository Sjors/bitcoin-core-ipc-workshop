#![allow(unused_imports)]

use anyhow::{Context, Result};

use crate::{
    block_header::{BITS_OFFSET, UINT32_LEN},
    cli::Config,
    ipc::{IpcMiningClient, bootstrap_hint},
    mining_job::{CoinbaseTemplate, MerklePath},
};

pub async fn run(config: Config) -> Result<()> {
    let clients = IpcMiningClient::connect(&config.socket).await?;
    let tip = clients.tip().await?;

    // TODO: Create a block template with clients.create_block_template().

    // TODO: Replace these placeholders by fetching the header, coinbase template,
    // and merkle path from the block template.
    let header = [0u8; 80];
    let coinbase_template = CoinbaseTemplate {
        script_sig_prefix: Vec::new(),
        witness: None,
        block_reward_remaining: 0,
        required_outputs: Vec::new(),
    };
    let _merkle_path: MerklePath = Vec::new();
    let bits = u32::from_le_bytes(
        header[BITS_OFFSET..BITS_OFFSET + UINT32_LEN]
            .try_into()
            .unwrap(),
    );
    let _ = bootstrap_hint(tip.height);

    println!("tip height: {}", tip.height);
    println!("tip hash: {}", tip.hash);
    println!(
        "coinbase remaining reward: {} sats",
        coinbase_template.block_reward_remaining
    );
    println!("target: compact bits {bits:08x}");

    // TODO: Destroy the block template before exiting.

    Ok(())
}
