use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    /// Bitcoin Core IPC Unix socket path.
    #[arg(long, default_value = "./bitcoin/signet/node.sock")]
    socket: PathBuf,
}

pub struct Config {
    pub socket: PathBuf,
}

impl Args {
    pub fn into_config(self) -> Result<Config> {
        Ok(Config {
            socket: self.socket,
        })
    }
}
