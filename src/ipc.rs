// TODO: Remove this line when you're done; it silences warnings about the imports
// and variables that your code will use.
#![allow(dead_code, unused_imports, unused_variables)]

use std::path::Path;

use anyhow::{Context, Result, bail};
use bitcoin::BlockHash;
use bitcoin_capnp_types::{init_capnp::init, mining_capnp::mining, proxy_capnp::thread_map};
use capnp_rpc::{RpcSystem, rpc_twoparty_capnp::Side, twoparty::VatNetwork};
use futures::io::BufReader;
use tokio::net::{UnixStream, unix::OwnedReadHalf};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::mining_job::Tip;

/// Number of Bitcoin Core worker threads that serve our IPC requests.
const IPC_THREAD_POOL_SIZE: u32 = 2;

pub struct IpcMiningClient {
    // TODO: Store the mining client (mining::Client) here.
}

impl IpcMiningClient {
    pub async fn connect(socket_path: &Path) -> Result<Self> {
        let network = connect_unix_stream(socket_path).await?;
        let mut rpc_system = RpcSystem::new(Box::new(network), None);
        let init: init::Client = rpc_system.bootstrap(Side::Server);
        tokio::task::spawn_local(rpc_system);

        // TODO: Call construct on the init client and get the thread map
        // (thread_map::Client) from the result.

        // TODO: Call makePool on the thread map, with IPC_THREAD_POOL_SIZE as the
        // count. Requests that do not name a thread in their context are dispatched
        // to this pool.

        // TODO: Call makeMining on the init client, and store the mining client
        // from the result in Self.

        Ok(Self {})
    }

    pub async fn tip(&self) -> Result<Tip> {
        // TODO: Call getTip on the mining client. Bail if hasResult is false,
        // otherwise return the real height and hash instead of this placeholder.
        // The hash is a 32 byte Data field.
        Ok(Tip {
            height: 0,
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
