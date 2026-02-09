use axum_extra::response::ErasedJson;

use crate::{rpc::WalletRpcImpl, rpc_client};

use super::error::RestError;

pub async fn get_tip_height() -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(WalletRpcImpl::get_tip_height().await?))
}

pub trait BlockInfoRpc {
    async fn get_tip_height() -> Result<u64, RestError> {
        let tip = rpc_client::node_rpc_client().get_tip_info().await?;

        Ok(tip.height.into())
    }
}

impl BlockInfoRpc for WalletRpcImpl {}
