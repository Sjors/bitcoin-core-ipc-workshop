use anyhow::{Context, Result};

use crate::{
    block_header::{BITS_OFFSET, UINT32_LEN},
    cli::Config,
    fixtures::ci_solution,
    ipc::{IpcMiningClient, bootstrap_hint},
    mining_job::build_mining_work,
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
    let merkle_path = template.coinbase_merkle_path().await?;
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

    let target = target_from_bits(bits);
    println!("target: {target:064x}");

    let mut extra_nonce = 0u64;
    loop {
        let work = build_mining_work(
            &header,
            &coinbase_template,
            &merkle_path,
            config.coinbase_message.as_bytes(),
            extra_nonce,
        )?;
        if config.check_template {
            template.destroy().await?;
            return Ok(());
        }

        match ci_solution(&config.fixtures, config.ci, tip.height + 1, &target)?
            .or_else(|| mine_round(&work, &target, config.threads, extra_nonce))
        {
            Some(found) => {
                println!(
                    "found header: version={:#x} timestamp={} nonce={}",
                    found.version, found.timestamp, found.nonce
                );
                template.submit_solution(&found, &work.coinbase).await?;
                println!("submitted block at height {}", tip.height + 1);
                template.destroy().await?;
                return Ok(());
            }
            None => {
                extra_nonce = extra_nonce.wrapping_add(1);
            }
        }
    }
}
