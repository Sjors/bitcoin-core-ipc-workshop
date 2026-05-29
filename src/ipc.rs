use std::path::Path;

use anyhow::{Context, Result};
use bitcoin::BlockHash;
use capnp_rpc::{rpc_twoparty_capnp::Side, twoparty::VatNetwork};
use futures::io::BufReader;
use tokio::net::{UnixStream, unix::OwnedReadHalf};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::mining_job::Tip;

// TODO: After adding bitcoin-capnp-types to Cargo.toml, uncomment these imports.
// use bitcoin_capnp_types::{
//     init_capnp::init,
//     mining_capnp::mining,
//     proxy_capnp::{thread as ipc_thread, thread_map},
// };
// use capnp_rpc::RpcSystem;

pub struct IpcMiningClient {
    // TODO: Store the IPC thread and mining client here.
}

impl IpcMiningClient {
    pub async fn connect(socket_path: &Path) -> Result<Self> {
        let network = connect_unix_stream(socket_path).await?;
        let _connected = &network;

        // TODO: After adding bitcoin-capnp-types to Cargo.toml, construct the IPC
        // service, make a thread, and call makeMining with that thread in the
        // request context.

        Ok(Self {})
    }

    pub async fn tip(&self) -> Result<Tip> {
        // TODO: Call getTip on the mining client with the IPC thread in the request
        // context, then return the height and hash from the result.
        Ok(Tip {
            height: 17,
            hash: BlockHash::from_byte_array([0u8; 32]),
        })
    }
}

async fn connect_unix_stream(
    socket_path: &Path,
) -> Result<VatNetwork<BufReader<Compat<OwnedReadHalf>>>> {
    let stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| format!("could not connect to IPC socket {}", socket_path.display()))?;
    let (reader, writer) = stream.into_split();
    let reader = futures::io::BufReader::new(reader.compat());
    let writer = futures::io::BufWriter::new(writer.compat_write());
    Ok(VatNetwork::new(
        reader,
        writer,
        Side::Client,
        Default::default(),
    ))
}
