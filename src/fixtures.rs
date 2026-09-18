use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use bitcoin::{BlockHash, Target};
use serde::Deserialize;

use crate::pow::FoundBlock;

#[derive(Deserialize)]
struct BlockFixture {
    height: i32,
    hash: String,
    version: u32,
    timestamp: u32,
    nonce: u32,
}

pub fn ci_solution(
    fixtures: &Path,
    enabled: bool,
    height: i32,
    target: &Target,
) -> Result<Option<FoundBlock>> {
    if !enabled {
        return Ok(None);
    }

    let path = fixtures.join(format!("block_{height}.json"));
    if !path.exists() {
        if *target == Target::MAX_ATTAINABLE_REGTEST {
            return Ok(None);
        }
        bail!(
            "missing CI fixture {}; refusing to mine non-trivial target",
            path.display()
        );
    }

    let fixture: BlockFixture = serde_json::from_str(
        &fs::read_to_string(&path)
            .with_context(|| format!("could not read CI fixture {}", path.display()))?,
    )
    .with_context(|| format!("could not parse CI fixture {}", path.display()))?;
    if fixture.height != height {
        bail!(
            "CI fixture {} is for height {}, expected {height}",
            path.display(),
            fixture.height
        );
    }

    let hash: BlockHash = fixture
        .hash
        .parse()
        .with_context(|| format!("invalid hash in CI fixture {}", path.display()))?;
    if !target.is_met_by(hash) {
        bail!(
            "CI fixture {} does not satisfy the current target",
            path.display()
        );
    }

    Ok(Some(FoundBlock {
        version: fixture.version,
        timestamp: fixture.timestamp,
        nonce: fixture.nonce,
    }))
}
