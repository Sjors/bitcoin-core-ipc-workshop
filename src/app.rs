use anyhow::Result;

use crate::{cli::Config, ipc::IpcMiningClient};

pub async fn run(config: Config) -> Result<()> {
    let clients = IpcMiningClient::connect(&config.socket).await?;
    let tip = clients.tip().await?;

    println!("tip height: {}", tip.height);
    println!("tip hash: {}", tip.hash);

    Ok(())
}
