use anyhow::{Context, Result, bail};
use bitcoin::{
    Amount, BlockHash, OutPoint, ScriptPubKeyBuf, ScriptSigBuf, Sequence, Transaction,
    TransactionVersion, TxIn, TxMerkleNode, TxOut, Witness, absolute,
};
use bitcoin_hashes::sha256d;

use crate::block_header::{BLOCK_HASH_LEN, MERKLE_ROOT_OFFSET};

pub struct Tip {
    pub height: i32,
    pub hash: BlockHash,
}

pub struct CoinbaseTemplate {
    pub version: u32,
    pub sequence: u32,
    pub script_sig_prefix: Vec<u8>,
    pub witness: Option<Vec<u8>>,
    pub block_reward_remaining: u64,
    pub required_outputs: Vec<TxOut>,
    pub lock_time: u32,
}

pub struct MiningWork {
    pub header: [u8; 80],
    pub coinbase: Vec<u8>,
}

pub fn build_mining_work(
    template_header: &[u8; 80],
    coinbase_template: &CoinbaseTemplate,
    merkle_path: &[TxMerkleNode],
    coinbase_message: &[u8],
    extra_nonce: u64,
) -> Result<MiningWork> {
    let transaction = build_coinbase_transaction(coinbase_template, coinbase_message, extra_nonce)?;
    let coinbase_txid = transaction.compute_txid().to_byte_array();
    let coinbase = encoding::encode_to_vec(&transaction);
    let merkle_root = merkle_root_from_path(coinbase_txid, merkle_path);

    let mut header = *template_header;
    header[MERKLE_ROOT_OFFSET..MERKLE_ROOT_OFFSET + BLOCK_HASH_LEN]
        .copy_from_slice(merkle_root.as_byte_array());
    Ok(MiningWork { header, coinbase })
}

fn merkle_root_from_path(mut hash: [u8; 32], merkle_path: &[TxMerkleNode]) -> TxMerkleNode {
    for sibling in merkle_path {
        let mut pair = Vec::with_capacity(64);
        pair.extend_from_slice(&hash);
        pair.extend_from_slice(sibling.as_byte_array());
        hash = sha256d::Hash::hash(&pair).to_byte_array();
    }
    TxMerkleNode::from_byte_array(hash)
}

fn build_coinbase_transaction(
    template: &CoinbaseTemplate,
    coinbase_message: &[u8],
    extra_nonce: u64,
) -> Result<Transaction> {
    let mut script_sig = template.script_sig_prefix.clone();
    script_sig.extend_from_slice(coinbase_message);
    script_sig.extend_from_slice(&extra_nonce.to_le_bytes());
    if script_sig.len() > 100 {
        bail!(
            "coinbase scriptSig is {} bytes; maximum is 100",
            script_sig.len()
        );
    }

    let witness = template
        .witness
        .as_ref()
        .map(|witness| Witness::from_slice(&[witness]))
        .unwrap_or_default();
    let mut outputs = Vec::with_capacity(1 + template.required_outputs.len());
    outputs.push(TxOut {
        amount: Amount::from_sat(template.block_reward_remaining)
            .context("block reward remaining is outside the valid money range")?,
        script_pubkey: ScriptPubKeyBuf::new(),
    });
    outputs.extend(template.required_outputs.iter().cloned());

    Ok(Transaction {
        version: TransactionVersion::maybe_non_standard(template.version),
        lock_time: absolute::LockTime::from_consensus(template.lock_time),
        inputs: vec![TxIn {
            previous_output: OutPoint::COINBASE_PREVOUT,
            script_sig: ScriptSigBuf::from_bytes(script_sig),
            sequence: Sequence::from_consensus(template.sequence),
            witness,
        }],
        outputs,
    })
}
