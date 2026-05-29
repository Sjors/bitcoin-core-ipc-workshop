mod app;
mod block_header;
mod cli;
mod ipc;
mod mining_job;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use tokio::task::LocalSet;

fn main() -> Result<()> {
    let config = Args::parse().into_config()?;

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local = LocalSet::new();
    runtime.block_on(local.run_until(app::run(config)))
}
