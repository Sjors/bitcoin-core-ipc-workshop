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
    mining: mining::Client,
}

impl IpcMiningClient {
    pub async fn connect(socket_path: &Path) -> Result<Self> {
        let network = connect_unix_stream(socket_path).await?;
        let mut rpc_system = RpcSystem::new(Box::new(network), None);
        let init: init::Client = rpc_system.bootstrap(Side::Server);
        tokio::task::spawn_local(rpc_system);

        let construct_response = init
            .construct_request()
            .send()
            .promise
            .await
            .context("construct IPC request failed")?;
        let thread_map: thread_map::Client = construct_response
            .get()?
            .get_thread_map()
            .context("missing IPC thread map")?;

        // Ask Bitcoin Core for a pool of worker threads. Requests that do not name
        // a thread in their context are dispatched to this pool.
        let mut pool_request = thread_map.make_pool_request();
        pool_request.get().set_count(IPC_THREAD_POOL_SIZE);
        pool_request
            .send()
            .promise
            .await
            .context("makePool IPC request failed")?;

        let mining_response = init
            .make_mining_request()
            .send()
            .promise
            .await
            .context("makeMining IPC request failed")?;
        let mining = mining_response
            .get()?
            .get_result()
            .context("missing mining client")?;

        Ok(Self { mining })
    }

    pub async fn tip(&self) -> Result<Tip> {
        let response = self
            .mining
            .get_tip_request()
            .send()
            .promise
            .await
            .context("getTip IPC request failed")?;
        let results = response.get()?;
        if !results.get_has_result() {
            bail!("Bitcoin Core did not return a chain tip");
        }
        let tip = results.get_result()?;
        let hash = tip.get_hash()?.to_vec();
        let hash = hash.try_into().map_err(|hash: Vec<u8>| {
            anyhow::anyhow!("expected 32-byte chain tip hash, got {}", hash.len())
        })?;
        Ok(Tip {
            height: tip.get_height(),
            hash: BlockHash::from_byte_array(hash),
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
