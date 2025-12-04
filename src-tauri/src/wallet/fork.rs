use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use neptune_privacy::application::rest_server::ExportedBlock;
use neptune_privacy::prelude::tasm_lib::prelude::Digest;
use tracing::debug;

use crate::rpc_client;

impl super::WalletState {
    pub async fn check_fork(&self, block: &ExportedBlock) -> Result<Option<(u64, Digest)>> {
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
                            if prev.is_canonical {
                                let blk_before_fork = rpc_client::node_rpc_client()
                                    .get_block_info(&prev.prev_block_digest.to_hex())
                                    .await?
                                    .context("try get_block_info before fork")?;
                                return Ok(Some((
                                    blk_before_fork.height.into(),
                                    blk_before_fork.digest,
                                )));
                            } else {
                                prev_digest = prev.prev_block_digest;
                            }
                        }
                        None => return Err(anyhow!("Block not found")),
                    }
                }
            }
        }
        return Ok(None);
    }
}
