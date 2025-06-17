use axum_extra::response::ErasedJson;

use crate::rpc_client;

use super::error::RestError;

pub async fn get_tip_height() -> Result<ErasedJson, RestError> {
    let tip = rpc_client::node_rpc_client().get_tip_info().await?;

    let height: u64 = if let Some(tip) = tip {
        tip.height.into()
    } else {
        0
    };

    Ok(ErasedJson::pretty(height))
}
