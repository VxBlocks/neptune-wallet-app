use anyhow::{anyhow, Context, Result};
use neptune_cash::{models::blockchain::block::Block, prelude::tasm_lib::prelude::Digest};

use crate::rpc_client;

impl super::WalletState {
    pub async fn check_fork(&self, block: &Block) -> Result<Option<(u64, Digest)>> {
        if block.header().height.is_genesis() {
            return Ok(None);
        }

        if let Some((_, prev_digest)) = self.get_tip().await? {
            //prev is forked
            if block.header().prev_block_digest != prev_digest {
                let mut prev_digest = block.header().prev_block_digest;
                loop {
                    let prev = rpc_client::node_rpc_client()
                        .get_block_info(&prev_digest.to_hex())
                        .await?;

                    match prev {
                        Some(prev) => {
                            if prev.is_canonical {
                                let blk_before_fork = rpc_client::node_rpc_client()
                                    .get_block_info(&prev.prev_block_digest.to_hex())
                                    .await?
                                    .context("")?;
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
