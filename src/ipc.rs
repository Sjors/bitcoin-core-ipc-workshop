use std::path::Path;

use anyhow::{Context, Result, bail};
use bitcoin::{BlockHash, TxMerkleNode, TxOut};
use bitcoin_capnp_types::{
    init_capnp::init,
    mining_capnp::{block_template, mining},
    proxy_capnp::{thread as ipc_thread, thread_map},
};
use capnp_rpc::{RpcSystem, rpc_twoparty_capnp::Side, twoparty::VatNetwork};
use futures::io::BufReader;
use tokio::net::{UnixStream, unix::OwnedReadHalf};
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::{
    block_header::BLOCK_HEADER_LEN,
    mining_job::{CoinbaseTemplate, Tip},
    pow::FoundBlock,
};

pub struct IpcMiningClient {
    thread: ipc_thread::Client,
    mining: mining::Client,
}

pub struct IpcBlockTemplate {
    thread: ipc_thread::Client,
    template: block_template::Client,
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

    pub async fn create_block_template(&self) -> Result<IpcBlockTemplate> {
        let mut request = self.mining.create_new_block_request();
        request.get().get_context()?.set_thread(self.thread.clone());
        request.get().set_cooldown(false);
        request.get().init_options().set_use_mempool(false);

        let response = request
            .send()
            .promise
            .await
            .context("createNewBlock IPC request failed")?;
        let template = response
            .get()?
            .get_result()
            .context("missing block template")?;

        Ok(IpcBlockTemplate {
            thread: self.thread.clone(),
            template,
        })
    }
}

impl IpcBlockTemplate {
    pub async fn block_header(&self) -> Result<[u8; 80]> {
        let mut request = self.template.get_block_header_request();
        request.get().get_context()?.set_thread(self.thread.clone());
        let response = request
            .send()
            .promise
            .await
            .context("getBlockHeader IPC request failed")?;
        let bytes = response.get()?.get_result()?.to_vec();
        bytes.try_into().map_err(|bytes: Vec<u8>| {
            anyhow::anyhow!(
                "expected {BLOCK_HEADER_LEN}-byte block header, got {}",
                bytes.len()
            )
        })
    }

    pub async fn coinbase_template(&self) -> Result<CoinbaseTemplate> {
        let mut request = self.template.get_coinbase_tx_request();
        request.get().get_context()?.set_thread(self.thread.clone());
        let response = request
            .send()
            .promise
            .await
            .context("getCoinbaseTx IPC request failed")?;
        let coinbase = response.get()?.get_result()?;

        let mut required_outputs = Vec::new();
        let outputs = coinbase.get_required_outputs()?;
        for i in 0..outputs.len() {
            required_outputs.push(encoding::decode_from_slice::<TxOut>(outputs.get(i)?)?);
        }

        let witness = coinbase.get_witness()?.to_vec();
        Ok(CoinbaseTemplate {
            version: coinbase.get_version(),
            sequence: coinbase.get_sequence(),
            script_sig_prefix: coinbase.get_script_sig_prefix()?.to_vec(),
            witness: (!witness.is_empty()).then_some(witness),
            block_reward_remaining: coinbase
                .get_block_reward_remaining()
                .try_into()
                .context("negative block reward remaining")?,
            required_outputs,
            lock_time: coinbase.get_lock_time(),
        })
    }

    pub async fn coinbase_merkle_path(&self) -> Result<Vec<TxMerkleNode>> {
        let mut request = self.template.get_coinbase_merkle_path_request();
        request.get().get_context()?.set_thread(self.thread.clone());
        let response = request
            .send()
            .promise
            .await
            .context("getCoinbaseMerklePath IPC request failed")?;
        let path = response.get()?.get_result()?;

        let mut hashes = Vec::new();
        for i in 0..path.len() {
            let bytes = path.get(i)?.to_vec();
            let bytes = bytes.try_into().map_err(|bytes: Vec<u8>| {
                anyhow::anyhow!("expected 32-byte merkle path hash, got {}", bytes.len())
            })?;
            hashes.push(TxMerkleNode::from_byte_array(bytes));
        }
        Ok(hashes)
    }

    pub async fn destroy(&self) -> Result<()> {
        let mut request = self.template.destroy_request();
        request.get().get_context()?.set_thread(self.thread.clone());
        request
            .send()
            .promise
            .await
            .context("destroy BlockTemplate IPC request failed")?;
        Ok(())
    }

    pub async fn submit_solution(&self, found: &FoundBlock, coinbase: &[u8]) -> Result<()> {
        let mut request = self.template.submit_solution_request();
        request.get().get_context()?.set_thread(self.thread.clone());
        request.get().set_version(found.version);
        request.get().set_timestamp(found.timestamp);
        request.get().set_nonce(found.nonce);
        request.get().set_coinbase(coinbase);

        let response = request
            .send()
            .promise
            .await
            .context("submitSolution IPC request failed")?;
        if !response.get()?.get_result() {
            bail!("node rejected submitted block solution");
        }
        Ok(())
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

pub(crate) fn bootstrap_hint(tip_height: i32) -> String {
    format!(
        "createNewBlock failed at chain height {tip_height}; if this is Bitcoin Core v31.0, bootstrap this custom signet past height 16 first, for example: bitcoin-cli -datadir=$(pwd)/bitcoin generatetodescriptor 17 \"raw(51)\" 100000000"
    )
}
