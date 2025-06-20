use axum_extra::response::ErasedJson;

use crate::{rpc::WalletRpcImpl, rpc_client};

use super::error::RestError;

pub async fn get_tip_height() -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(WalletRpcImpl::get_tip_height().await?))
}

pub trait BlockInfoRpc {
    async fn get_tip_height() -> Result<u64, RestError> {
        let tip = rpc_client::node_rpc_client().get_tip_info().await?;

        let height: u64 = if let Some(tip) = tip {
            tip.height.into()
        } else {
            0
        };

        Ok(height)
    }
}

impl BlockInfoRpc for WalletRpcImpl {}
