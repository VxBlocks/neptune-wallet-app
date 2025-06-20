use std::ops::Deref;
use std::sync::Arc;

use tracing::*;

use super::tls;
use crate::command::{Result, TauriCommandResultExt};
use crate::config::Config;
use crate::rpc::block::BlockInfoRpc;
use crate::rpc::error::RestError;
use crate::rpc::transaction_status::{TransactionStatus, TransactionStatusRpc};
use crate::rpc::{
    SendResponse, SendToAddressParams, Utxo, WalletBalance, WalletRpc, WalletRpcImpl,
};
use crate::wallet::balance::WalletHistory;
use crate::wallet::sync::{SyncState, SyncStatus};

#[cfg_attr(feature = "gui", tauri::command)]
#[cfg_attr(not(feature = "gui"), allow(unused))]
pub async fn get_server_url() -> Result<String> {
    let token = get_token().await?;

    let url = format!(
        "http://{}@{}:{}",
        token,
        "127.0.0.1",
        crate::config::consts::RPC_PORT
    );

    Ok(url)
}

pub async fn get_token() -> Result<String> {
    let config = crate::service::get_state::<Arc<Config>>();
    let sk = config.get_secret_key().await.into_tauri_result()?;
    let public = tls::get_p256_pubkey(&sk);
    return Ok(hex::encode(public));
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn run_rpc_server() -> Result<()> {
    start_rpc_server_inner().await.map_err(|e| {
        let err = e.to_string();
        error!("error start rpc: {}", err);
        err
    })
}

pub async fn start_rpc_server_inner() -> Result<()> {
    let mut rpc_handler = super::RPC_CLOSER.lock().await;
    if let Some(handler) = rpc_handler.deref() {
        if !handler.is_finished() {
            return Err("rpc server is already running".to_string());
        };
        rpc_handler
            .take()
            .unwrap()
            .stop()
            .await
            .into_tauri_result()?;
    }
    drop(rpc_handler);

    if let Some(old) = crate::service::try_get_state::<Arc<SyncState>>() {
        old.cancel_sync().await;
    }

    let config = crate::service::get_state::<Arc<Config>>();

    let sync_state = Arc::new(SyncState::new(&config).await.into_tauri_result()?);
    crate::service::manage_or_replace(sync_state.clone());
    sync_state.sync().await;

    super::start_rpc_server().await.into_tauri_result()?;

    Ok(())
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn stop_rpc_server() -> Result<()> {
    if let Some(sync_state) = crate::service::try_get_state::<Arc<SyncState>>() {
        super::stop_rpc_server().await.into_tauri_result()?;
        sync_state.cancel_sync().await;
    };

    Ok(())
}

impl<T> TauriCommandResultExt for std::result::Result<T, RestError> {
    type Output = T;

    fn into_tauri_result(self) -> std::result::Result<T, String> {
        self.map_err(|e| e.0)
    }
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn sync_state() -> SyncStatus {
    WalletRpcImpl::sync_state().await
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn wallet_balance() -> Result<WalletBalance> {
    WalletRpcImpl::wallet_balance().await.into_tauri_result()
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn current_wallet_address(index: u64) -> Result<String> {
    WalletRpcImpl::current_wallet_address(index)
        .await
        .into_tauri_result()
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn history() -> Result<Vec<WalletHistory>> {
    WalletRpcImpl::history().await.into_tauri_result()
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn avaliable_utxos() -> Result<Vec<Utxo>> {
    WalletRpcImpl::avaliable_utxos().await.into_tauri_result()
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn send_to_address(params: SendToAddressParams) -> Result<SendResponse> {
    WalletRpcImpl::send_to_address(params)
        .await
        .into_tauri_result()
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn pending_transactions() -> Result<Vec<TransactionStatus>> {
    WalletRpcImpl::pending_transactions()
        .await
        .into_tauri_result()
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn forget_tx(txid: String) -> Result<()> {
    WalletRpcImpl::forget_tx(txid).await.into_tauri_result()
}

#[cfg_attr(feature = "gui", tauri::command)]
pub async fn get_tip_height() -> Result<u64> {
    WalletRpcImpl::get_tip_height().await.into_tauri_result()
}