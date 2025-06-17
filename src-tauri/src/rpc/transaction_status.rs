use std::sync::Arc;

use crate::{wallet::sync::SyncState, service::get_state};

use super::error::RestError;
use axum::extract::Path;
use axum_extra::response::ErasedJson;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct TransactionStatus {
    tx_id: String,
    status: TransactionStatusEnum,
}

#[derive(Debug, Serialize)]
enum TransactionStatusEnum {
    Pending,   // 等待执行
    // Proving,   // 生成singleProof中
    // Composing, // 等待确认
}
pub async fn get_pending_transaction() -> Result<ErasedJson, RestError> {
    let wallet = &get_state::<Arc<SyncState>>().wallet;
    let txs = wallet.get_pending_transactions().await?;

    let mut result = vec![];
    for tx in txs {
        let status = TransactionStatus {
            tx_id: tx,
            status: TransactionStatusEnum::Pending,
        };
        result.push(status);
    }

    //TODO: get status from remote

    Ok(ErasedJson::pretty(result))
}

pub async fn forget_tx(Path(id): Path<String>) -> Result<ErasedJson, RestError> {
    let wallet = &get_state::<Arc<SyncState>>().wallet;
    wallet.forget_tx(&id).await?;

    Ok(ErasedJson::pretty(true))
}
