use std::{path::PathBuf, thread};

use anyhow::{Result, bail};
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Bitcoin Core IPC Unix socket path.
    #[arg(long, default_value = "./bitcoin/signet/node.sock")]
    socket: PathBuf,

    /// Number of CPU mining threads. Defaults to available parallelism.
    #[arg(long)]
    threads: Option<usize>,

    /// Use deterministic CI fixtures instead of mining real block templates.
    #[arg(long)]
    ci: bool,

    /// Directory containing CI block fixtures.
    #[arg(long, default_value = "test/fixtures")]
    fixtures: PathBuf,
}

pub struct Config {
    pub socket: PathBuf,
    pub threads: usize,
    pub ci: bool,
    pub fixtures: PathBuf,
}

impl Args {
    pub fn into_config(self) -> Result<Config> {
        let threads = match self.threads {
            Some(0) => bail!("--threads must be greater than zero"),
            Some(n) => n,
            None => thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
        };

        Ok(Config {
            socket: self.socket,
            threads,
            ci: self.ci,
            fixtures: self.fixtures,
        })
    }
}
