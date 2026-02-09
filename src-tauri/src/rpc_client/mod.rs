use std::sync::atomic::AtomicPtr;
use std::sync::atomic::Ordering;

use anyhow::anyhow;
use anyhow::Result;
use neptune_cash::api::export::Digest;
use neptune_cash::api::export::Transaction;
use neptune_cash::application::json_rpc::core::api::rpc::RpcApi;
use neptune_cash::application::json_rpc::core::model::block::header::RpcBlockHeader;
use neptune_cash::application::json_rpc::core::model::wallet::mutator_set::RpcMsMembershipSnapshot;
use neptune_cash::protocol::consensus::block::block_selector::BlockSelector;
use neptune_cash::util_types::mutator_set::removal_record::absolute_index_set::AbsoluteIndexSet;
use neptune_rpc_client::http::HttpClient;
use once_cell::sync::Lazy;
use reqwest;
use serde::Serialize;
use thiserror::Error;

use crate::wallet::block::WalletBlock;

static NODE_RPC_CLIENT: Lazy<NodeRpcClient> = Lazy::new(|| NodeRpcClient::new(""));

pub fn node_rpc_client() -> &'static NodeRpcClient {
    return &NODE_RPC_CLIENT;
}

pub struct NodeRpcClient {
    client: AtomicPtr<HttpClient>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BroadcastTx<'a> {
    pub transaction: &'a Transaction,
}

impl NodeRpcClient {
    pub fn new(rpc_server: &str) -> Self {
        Self {
            client: AtomicPtr::new(Box::into_raw(Box::new(HttpClient::new(
                rpc_server.to_string(),
            )))),
        }
    }

    pub fn set_rest_server(&self, rest: String) {
        let _old = unsafe { Box::from_raw(self.client.load(Ordering::Relaxed)) };
        self.client.store(
            Box::into_raw(Box::new(HttpClient::new(rest.clone()))),
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    fn get_client() -> &'static HttpClient {
        unsafe { &*NODE_RPC_CLIENT.client.load(Ordering::Relaxed) }
    }

    pub async fn request_block(&self, height: u64) -> Result<Option<WalletBlock>> {
        let block = Self::get_client()
            .get_blocks(height.into(), height.into())
            .await?;

        if block.blocks.is_empty() {
            Ok(None)
        } else {
            Ok(Some(WalletBlock::from(&block.blocks[0])))
        }
    }

    pub async fn get_tip_info(&self) -> Result<RpcBlockHeader> {
        let tip_header = Self::get_client().tip_header().await?;

        Ok(tip_header.header)
    }

    pub async fn get_block_info(&self, digest: &str) -> Result<Option<RpcBlockHeader>> {
        let block_header = Self::get_client()
            .get_block_header(BlockSelector::Digest(Digest::try_from_hex(digest)?))
            .await?;

        Ok(block_header.header)
    }

    pub async fn request_block_by_digest(&self, digest: &str) -> Result<Option<WalletBlock>> {
        let selector = BlockSelector::Digest(Digest::try_from_hex(digest)?);
        let block_header = Self::get_client().get_block_header(selector).await?.header;

        if block_header.is_none() {
            return Ok(None);
        }

        return self
            .request_block(block_header.unwrap().height.into())
            .await;
    }

    pub async fn request_block_by_height_range(
        &self,
        height: u64,
        batch_size: u64,
    ) -> Result<Vec<WalletBlock>> {
        let block = Self::get_client()
            .get_blocks(height.into(), (height + batch_size - 1).into())
            .await?;

        Ok(block.blocks.iter().map(WalletBlock::from).collect())
    }

    pub async fn broadcast_transaction(&self, tx: &Transaction) -> Result<String> {
        let resp = Self::get_client()
            .submit_transaction(tx.clone().into())
            .await?;

        if resp.success != true {
            return Err(anyhow!("failed to send to main loop"));
        }
        Ok(tx.txid().to_string())
    }

    pub async fn restore_msmps(
        &self,
        request: Vec<AbsoluteIndexSet>,
    ) -> Result<RpcMsMembershipSnapshot> {
        let msmp_recovery_resp = Self::get_client().restore_membership_proof(request).await?;

        Ok(msmp_recovery_resp.snapshot)
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
