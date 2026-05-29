#![allow(dead_code)]

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Instant,
};

use bitcoin::{BlockHash, CompactTarget, Target};
use bitcoin_hashes::sha256d;

use crate::{
    block_header::{NONCE_OFFSET, TIMESTAMP_OFFSET, UINT32_LEN, VERSION_OFFSET},
    mining_job::MiningWork,
};

const TIMESTAMP_REFRESH_INTERVAL_NONCES: u64 = 1_000_000;

#[derive(Clone)]
pub struct FoundBlock {
    pub version: u32,
    pub timestamp: u32,
    pub nonce: u32,
}

/// Grind the nonce space for a given extraNonce. The work item's coinbase and merkle
/// root are fixed. Worker threads advance nTime from the template value by elapsed
/// mining time. If no thread finds a solution, the caller should provide a new
/// extraNonce.
pub fn mine_round(
    work: &MiningWork,
    target: &Target,
    threads: usize,
    _extra_nonce: u64,
) -> Option<FoundBlock> {
    let found = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::with_capacity(threads);

    for thread_id in 0..threads {
        let stop = Arc::clone(&found);
        let header = work.header;
        let target = *target;
        handles.push(thread::spawn(move || {
            mine_thread(header, target, threads, thread_id, stop)
        }));
    }

    for handle in handles {
        if let Some(found_block) = handle.join().expect("mining thread panicked") {
            return Some(found_block);
        }
    }

    None
}

fn mine_thread(
    mut header: [u8; 80],
    target: Target,
    threads: usize,
    thread_id: usize,
    stop: Arc<AtomicBool>,
) -> Option<FoundBlock> {
    let version = u32::from_le_bytes(
        header[VERSION_OFFSET..VERSION_OFFSET + UINT32_LEN]
            .try_into()
            .unwrap(),
    );
    let base_timestamp = u32::from_le_bytes(
        header[TIMESTAMP_OFFSET..TIMESTAMP_OFFSET + UINT32_LEN]
            .try_into()
            .unwrap(),
    );
    let mut timestamp = base_timestamp;
    let started = Instant::now();
    let mut nonce = thread_id as u64;
    let mut attempts = 0u64;
    while nonce <= u32::MAX as u64 && !stop.load(Ordering::Relaxed) {
        let nonce_u32 = nonce as u32;
        header[NONCE_OFFSET..NONCE_OFFSET + UINT32_LEN].copy_from_slice(&nonce_u32.to_le_bytes());
        let hash = BlockHash::from_byte_array(sha256d::Hash::hash(&header).to_byte_array());
        if target.is_met_by(hash) {
            stop.store(true, Ordering::Relaxed);
            return Some(FoundBlock {
                version,
                timestamp,
                nonce: nonce_u32,
            });
        }
        attempts += 1;
        refresh_timestamp(
            &mut header,
            &mut timestamp,
            base_timestamp,
            started,
            attempts,
        );
        nonce += threads as u64;
    }
    None
}

pub fn target_from_bits(bits: u32) -> Target {
    Target::from_compact(CompactTarget::from_consensus(bits))
}

fn refresh_timestamp(
    header: &mut [u8; 80],
    timestamp: &mut u32,
    base_timestamp: u32,
    started: Instant,
    attempts: u64,
) {
    if attempts % TIMESTAMP_REFRESH_INTERVAL_NONCES != 0 {
        return;
    }

    // The miner should not choose block time from its local clock. It only advances the
    // template-provided nTime by the monotonic time spent mining this work item.
    let elapsed = started.elapsed().as_secs().min(u32::MAX.into()) as u32;
    let new_timestamp = base_timestamp.saturating_add(elapsed);
    if new_timestamp != *timestamp {
        *timestamp = new_timestamp;
        header[TIMESTAMP_OFFSET..TIMESTAMP_OFFSET + UINT32_LEN]
            .copy_from_slice(&timestamp.to_le_bytes());
    }
}
