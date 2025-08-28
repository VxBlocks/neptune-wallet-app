use std::sync::atomic::{AtomicPtr, Ordering};

use anyhow::{Context, Result};
use neptune_cash::{
    models::blockchain::{
        block::{block_height::BlockHeight, block_info::BlockInfo, Block},
        transaction::Transaction,
    },
    prelude::twenty_first::prelude::Digest,
    prelude::twenty_first::prelude::MmrMembershipProof,
    util_types::mutator_set::removal_record::chunk_dictionary::ChunkDictionary,
};

use once_cell::sync::Lazy;
use reqwest;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

static NODE_RPC_CLIENT: Lazy<NodeRpcClient> = Lazy::new(|| NodeRpcClient::new(""));

pub fn node_rpc_client() -> &'static NodeRpcClient {
    return &NODE_RPC_CLIENT;
}

pub struct NodeRpcClient {
    rest_server: AtomicPtr<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMsMembershipProofEx {
    pub swbf_indices: Vec<u128>,
    pub aocl_leaf_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMsMembershipProofEx {
    pub height: BlockHeight,
    pub block_id: Digest,
    pub proofs: Vec<MsMembershipProofEx>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsMembershipProofEx {
    pub auth_path_aocl: MmrMembershipProof,
    pub target_chunks: ChunkDictionary,
}

#[derive(Debug, Serialize, Clone)]
pub struct BroadcastTx<'a> {
    pub transaction: &'a Transaction,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ResponseSendTx {
    status: u64,
    message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RpcBlock {
    pub block: Block,
    pub hash: Digest,
}

impl RpcBlock {
    pub fn to_block(&self) -> Block {
        let block = self.block.clone();
        block.digest.get_or_init(|| self.hash);
        block
    }

    pub fn from_block(block: Block) -> Self {
        let hash = block.hash();
        Self {
            block: block,
            hash: hash,
        }
    }
}

impl NodeRpcClient {
    pub fn new(rest_server: &str) -> Self {
        Self {
            rest_server: AtomicPtr::new(Box::into_raw(Box::new(rest_server.to_string()))),
        }
    }

    fn rest_server(&self) -> &str {
        let url = unsafe { &*self.rest_server.load(Ordering::Relaxed) };
        &url
    }

    pub fn set_rest_server(&self, rest: String) {
        self.rest_server.store(
            Box::into_raw(Box::new(rest)),
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    fn get_client() -> reqwest::Client {
        reqwest::Client::new()
    }

    pub async fn request_block(&self, height: u64) -> Result<Option<RpcBlock>> {
        let block = Self::get_client()
            .get(format!(
                "{}/rpc/block/height/{}",
                self.rest_server(),
                height
            ))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .json::<Option<RpcBlock>>()
            .await?;

        Ok(block)
    }

    pub async fn get_tip_info(&self) -> Result<Option<BlockInfo>> {
        let block = Self::get_client()
            .get(format!("{}/rpc/block_info/tip", self.rest_server()))
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await?
            .error_for_status()?
            .json::<Option<BlockInfo>>()
            .await?;
        Ok(block)
    }

    pub async fn get_block_info(&self, digest: &str) -> Result<Option<BlockInfo>> {
        let block = Self::get_client()
            .get(format!(
                "{}/rpc/block_info/digest/{}",
                self.rest_server(),
                digest
            ))
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .await?
            .error_for_status()?
            .json::<Option<BlockInfo>>()
            .await?;
        Ok(block)
    }

    pub async fn request_block_by_digest(&self, digest: &str) -> Result<Option<RpcBlock>> {
        let block = Self::get_client()
            .get(format!(
                "{}/rpc/block/digest/{}",
                self.rest_server(),
                digest
            ))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .json::<Option<RpcBlock>>()
            .await?;
        Ok(block)
    }

    pub async fn request_block_by_height_range(
        &self,
        height: u64,
        batch_size: u64,
    ) -> Result<Vec<RpcBlock>> {
        let body = Self::get_client()
            .get(format!(
                "{}/rpc/batch_block/{}/{}",
                self.rest_server(),
                height,
                batch_size
            ))
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        let blocks: Vec<RpcBlock> = bincode::deserialize(&body)?;

        Ok(blocks)
    }

    pub async fn broadcast_transaction(&self, tx: &Transaction) -> Result<String, BroadcastError> {
        let tx_req = BroadcastTx { transaction: tx };

        let tx_b = bincode::serialize(&tx_req).context("serialize tx req")?;

        info!("proven tx size: {}", tx_b.len());

        let resp = Self::get_client()
            .post(format!("{}/rpc/broadcast_tx", self.rest_server()))
            .body(tx_b)
            .send()
            .await?
            .error_for_status()?
            .json::<ResponseSendTx>()
            .await?;

        if resp.status != 0 {
            if resp.message == "proof machine is busy" {
                return Err(BroadcastError::Busy);
            };
            return Err(BroadcastError::Server(anyhow::anyhow!(resp.message)));
        }
        Ok(tx.kernel.txid().to_string())
    }

    pub async fn restore_msmp(
        &self,
        request: Vec<RequestMsMembershipProofEx>,
    ) -> Result<ResponseMsMembershipProofEx> {
        let body = bincode::serialize(&request)?;

        let kernel = Self::get_client()
            .post(format!(
                "{}/rpc/generate_membership_proof",
                self.rest_server()
            ))
            .body(body)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        Ok(bincode::deserialize(&kernel)?)
    }
}

#[derive(Error, Debug)]
pub enum BroadcastError {
    #[error("proof machine is busy")]
    Busy,
    #[error("Connection timeout")]
    Timeout,
    #[error("Connection error: {0}")]
    Connection(reqwest::Error),
    #[error("Server error: {0}")]
    Server(anyhow::Error),
    #[error("Internal error: {0}")]
    Internal(anyhow::Error),
}

impl From<reqwest::Error> for BroadcastError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            BroadcastError::Timeout
        } else if e.is_connect() {
            BroadcastError::Connection(e)
        } else {
            BroadcastError::Server(e.into())
        }
    }
}

impl From<anyhow::Error> for BroadcastError {
    fn from(e: anyhow::Error) -> Self {
        BroadcastError::Internal(e)
    }
}
