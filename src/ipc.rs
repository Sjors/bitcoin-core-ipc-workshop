use std::path::Path;

use anyhow::{Context, Result, bail};
use bitcoin::BlockHash;
use bitcoin_capnp_types::{
    init_capnp::init,
    mining_capnp::mining,
    proxy_capnp::{thread as ipc_thread, thread_map},
};
use capnp_rpc::{RpcSystem, rpc_twoparty_capnp::Side, twoparty::VatNetwork};
use futures::io::BufReader;
use tokio::net::{UnixStream, unix::OwnedReadHalf};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::mining_job::Tip;

pub struct IpcMiningClient {
    thread: ipc_thread::Client,
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
        let thread_response = thread_map
            .make_thread_request()
            .send()
            .promise
            .await
            .context("makeThread IPC request failed")?;
        let thread = thread_response
            .get()?
            .get_result()
            .context("missing IPC thread")?;

        let mut mining_request = init.make_mining_request();
        mining_request
            .get()
            .get_context()?
            .set_thread(thread.clone());
        let mining_response = mining_request
            .send()
            .promise
            .await
            .context("makeMining IPC request failed")?;
        let mining = mining_response
            .get()?
            .get_result()
            .context("missing mining client")?;

        Ok(Self { thread, mining })
    }

    pub async fn tip(&self) -> Result<Tip> {
        let mut request = self.mining.get_tip_request();
        request.get().get_context()?.set_thread(self.thread.clone());
        let response = request
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
