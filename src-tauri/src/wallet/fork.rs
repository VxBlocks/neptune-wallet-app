use anyhow::{anyhow, Result};
use neptune_cash::{models::blockchain::block::Block, prelude::tasm_lib::prelude::Digest};

use crate::rpc_client;

impl super::WalletState {
    pub async fn check_fork(&self, block: &Block) -> Result<Option<(u64, Digest)>> {
        if block.header().height.is_genesis() {
            return Ok(None);
        }

        if let Some((_, prev_digest)) = self.get_tip().await? {
            if block.header().prev_block_digest != prev_digest {
                let mut prev_digest = block.header().prev_block_digest;
                loop {
                    let prev = rpc_client::node_rpc_client()
                        .get_block_info(&prev_digest.to_hex())
                        .await?;

                    match prev {
                        Some(prev) => {
                            if prev.is_canonical {
                                return Ok(Some((prev.height.into(), prev.digest)));
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
