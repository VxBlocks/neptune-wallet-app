use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use neptune_cash::prelude::tasm_lib::prelude::Digest;
use tracing::debug;

use crate::rpc_client;
use crate::wallet::block::WalletBlock;

impl super::WalletState {
    pub async fn check_fork(&self, block: &WalletBlock) -> Result<Option<(u64, Digest)>> {
        if block.kernel.header.height.value() <= 1 {
            return Ok(None);
        }

        debug!(
            "Checking fork for block {} {} {}",
            block.kernel.header.height,
            block.hash().to_hex(),
            block.kernel.header.prev_block_digest.to_hex()
        );

        if let Some((prev_height, prev_digest)) = self.get_tip().await.context("get tip")? {
            debug!("prev digest: {} {:?}", prev_height, prev_digest.to_hex());
            //prev is forked
            if block.kernel.header.prev_block_digest != prev_digest {
                let mut prev_digest = block.kernel.header.prev_block_digest;
                loop {
                    let prev = rpc_client::node_rpc_client()
                        .get_block_info(&prev_digest.to_hex())
                        .await
                        .context("try get_prev_block_info")?;

                    match prev {
                        Some(prev) => {
                            let prev_height: u64 = prev.height.into();

                            // this rpc always returns the canonical chain so we can check against it
                            let canonical_at_height = rpc_client::node_rpc_client()
                                .request_block(prev_height)
                                .await?
                                .context("try get canonical block by height")?;

                            // If the digest at this height matches the canonical chain, we found
                            // the common ancestor and can reorg to its parent.
                            if canonical_at_height.hash() == prev_digest {
                                return Ok(Some((
                                    prev_height.saturating_sub(1),
                                    prev.prev_block_digest,
                                )));
                            }

                            prev_digest = prev.prev_block_digest;
                        }
                        None => return Err(anyhow!("Block not found")),
                    }
                }
            }
        }
        return Ok(None);
    }
}
