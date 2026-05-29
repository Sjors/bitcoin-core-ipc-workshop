#![allow(dead_code)]

use bitcoin::{BlockHash, TxMerkleNode, TxOut};

pub struct Tip {
    pub height: i32,
    pub hash: BlockHash,
}

pub struct CoinbaseTemplate {
    pub script_sig_prefix: Vec<u8>,
    pub witness: Option<Vec<u8>>,
    pub block_reward_remaining: u64,
    pub required_outputs: Vec<TxOut>,
}

pub struct MiningWork {
    pub header: [u8; 80],
}

pub type MerklePath = Vec<TxMerkleNode>;
