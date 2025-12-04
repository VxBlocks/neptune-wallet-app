use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::SocketAddrV4;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::extract::Path;
use axum::extract::Request;
use axum::http;
use axum::middleware::Next;
use axum::middleware::{self};
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum::Json;
use axum_extra::response::ErasedJson;
use block::get_tip_height;
use error::RestError;
use http::StatusCode;
use neptune_privacy::api::export::NativeCurrencyAmount;
use neptune_privacy::api::export::ReceivingAddress;
use neptune_privacy::api::export::Timestamp;
use neptune_privacy::state::wallet::utxo_notification::UtxoNotificationMedium;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::*;
use transaction_status::forget_tx;
use transaction_status::get_pending_transaction;

use crate::config::consts::RPC_PORT;
use crate::config::Config;
use crate::service::get_state;
use crate::wallet::balance::WalletHistory;
use crate::wallet::sync::SyncState;
use crate::wallet::sync::SyncStatus;
use crate::wallet::InputSelectionRule;
// mod middleware;
mod block;
#[cfg(feature = "cli")]
pub mod client;
pub mod commands;
mod error;
pub mod tls;
mod transaction_status;

static RPC_CLOSER: Lazy<Mutex<Option<RpcHandler>>> = Lazy::new(|| Mutex::new(None));

pub async fn stop_rpc_server() -> Result<()> {
    info!("stopping rpc server");
    let mut closer = RPC_CLOSER.lock().await;
    if let Some(handler) = closer.take() {
        handler.stop().await?;
    }
    Ok(())
}

#[derive(Debug)]
pub struct RpcHandler {
    pub closer: Sender<()>,
    pub handler: JoinHandle<()>,
}

impl RpcHandler {
    pub async fn stop(self) -> Result<()> {
        self.closer
            .send(())
            .map_err(|_| anyhow!("the receiver dropped"))?;
        let abort = self.handler.abort_handle();
        let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), self.handler).await;
        if let Err(err) = timeout {
            error!("rpc server handler timeout: {:?}", err);
            abort.abort();
        }
        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        self.handler.is_finished()
    }
}

#[derive(Debug, Serialize)]
pub struct WalletBalance {
    pub available_balance: String,
    pub total_balance: String,
}

pub struct WalletRpcImpl;
impl WalletRpc for WalletRpcImpl {}

//TODO: move to crate::command
pub trait WalletRpc {
    async fn sync_state() -> SyncStatus {
        crate::service::get_state::<Arc<SyncState>>().status().await
    }

    async fn wallet_balance() -> Result<WalletBalance, RestError> {
        let wallet = &get_state::<Arc<SyncState>>().wallet;
        let (available_balance, total_balance) = wallet.get_all_balance().await?;
        Ok(WalletBalance {
            available_balance: available_balance.display_lossless(),
            total_balance: total_balance.display_lossless(),
        })
    }
    async fn current_wallet_address(index: u64) -> Result<String, RestError> {
        let wallet = &get_state::<Arc<SyncState>>().wallet;
        let address = wallet.get_address(index).await?;
        Ok(address)
    }
    async fn history() -> Result<Vec<WalletHistory>, RestError> {
        let wallet = &get_state::<Arc<SyncState>>().wallet;
        let history = wallet.get_balance_history().await?;
        Ok(history)
    }
    async fn avaliable_utxos() -> Result<Vec<Utxo>, RestError> {
        let wallet = &get_state::<Arc<SyncState>>().wallet;
        let mut utxos = wallet.get_unspent_utxos().await?;
        utxos.sort_by_key(|v| v.recovery_data.utxo.get_native_currency_amount());
        let now = Timestamp::now();
        let utxos = utxos
            .into_iter()
            .map(|v| Utxo {
                id: v.id,
                hash: v.hash,
                confirm_timestamp: v.confirmed_in_block.timestamp,
                confirm_height: v.confirm_height,
                confirmed_txid: v.confirmed_txid,
                amount: v
                    .recovery_data
                    .utxo
                    .get_native_currency_amount()
                    .display_lossless(),
                locked: match v.recovery_data.utxo.release_date() {
                    Some(v) => v > now,
                    None => false,
                },
            })
            .collect::<Vec<_>>();
        Ok(utxos)
    }
    async fn send_to_address(params: SendToAddressParams) -> Result<SendResponse, RestError> {
        let mut outputs = Vec::with_capacity(params.outputs.len());

        let wallet = &get_state::<Arc<SyncState>>().wallet;
        for output in params.outputs {
            let address = ReceivingAddress::from_bech32m(&output.address, wallet.network)?;
            let amount = NativeCurrencyAmount::coins_from_str(&output.amount)?;
            outputs.push((address, amount));
        }

        let utxo_notification_media = (
            UtxoNotificationMedium::OnChain,
            UtxoNotificationMedium::OnChain,
        );

        let fee = NativeCurrencyAmount::coins_from_str(&params.fee)?;

        let rule = if let Some(input_rule) = params.input_rule {
            InputSelectionRule::from_str(&input_rule).unwrap_or_default()
        } else {
            InputSelectionRule::default()
        };

        let tx = wallet
            .send_to_address(outputs, utxo_notification_media, fee, rule, params.inputs)
            .await
            .map_err(|e| anyhow!("{}", e))?;

        info!("proven tx {}", tx.txid());

        Ok(SendResponse {
            txid: tx.txid().to_string(),
            outputs: tx
                .kernel
                .outputs
                .iter()
                .map(|v| v.canonical_commitment.to_hex())
                .collect::<Vec<_>>(),
        })
    }
}

pub async fn start_rpc_server() -> Result<(), anyhow::Error> {
    let address: SocketAddr =
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), RPC_PORT));

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    let router = {
        let routes = axum::Router::new()
            .route("/rpc/scan/{start}/{end}", get(scan_blocks))
            .route("/rpc/scan/state", get(sync_state))
            .route("/rpc/wallet/balance", get(wallet_balance))
            .route("/rpc/wallet/address/{index}", get(wallet_address))
            .route("/rpc/wallet/history", get(history))
            .route("/rpc/wallet/available_utxos", get(avaliable_utxos))
            .route("/rpc/mempool/pendingtx", get(get_pending_transaction))
            .route("/rpc/forget_tx/{id}", get(forget_tx))
            .route("/rpc/send", post(send_to_address))
            .route("/rpc/block/tip_height", get(get_tip_height));

        routes
            // Pass in `Rest` to make t
            // Enable tower-http tracing.
            .layer(TraceLayer::new_for_http())
            .layer(middleware::from_fn(auth_middleware))
            .layer(middleware::from_fn(log_middleware))
            // Enable CORS.
            .layer(cors)
    };

    let listener = TcpListener::bind(address).await?;

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let handler = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = rx.await;
        })
        .await
        .unwrap();
    });

    let mut rpc_handler = RPC_CLOSER.lock().await;
    rpc_handler.replace(RpcHandler {
        closer: tx,
        handler: handler,
    });

    Ok(())
}

async fn log_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    match path.as_str() {
        "/rpc/scan/state" | "/rpc/block/tip_height" => {}
        _ => {
            info!(
                "Received '{} {}' from '{addr}'",
                request.method(),
                request.uri()
            );
        }
    }

    let response = next.run(request).await;
    let (res_parts, res_body) = response.into_parts();

    let body_bytes = axum::body::to_bytes(res_body, usize::MAX).await.unwrap();

    if res_parts.status != StatusCode::OK {
        error!(
            "Response error: '{}' {}",
            path,
            String::from_utf8_lossy(&body_bytes)
        );
    }
    let res = Response::from_parts(res_parts, Body::from(body_bytes));

    Ok(res)
}

async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let config = crate::service::get_state::<Arc<Config>>();
    let secret = config
        .get_secret_key()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = tls::get_p256_pubkey(&secret);
    let token = hex::encode(token);

    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());
    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if auth_header != format!("Bearer {}", token) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

async fn scan_blocks(Path((_start, end)): Path<(u64, u64)>) -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(end))
}

async fn sync_state() -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(WalletRpcImpl::sync_state().await))
}

async fn wallet_balance() -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(WalletRpcImpl::wallet_balance().await?))
}

async fn wallet_address(Path(index): Path<u64>) -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(
        WalletRpcImpl::current_wallet_address(index).await?,
    ))
}

async fn history() -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(WalletRpcImpl::history().await?))
}

#[derive(Serialize, Deserialize)]
pub struct SendToAddressParams {
    pub outputs: Vec<Output>,
    pub fee: String,
    pub input_rule: Option<String>,
    #[serde(default)]
    pub inputs: Vec<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    pub address: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendResponse {
    txid: String,
    outputs: Vec<String>,
}

async fn send_to_address(Json(params): Json<SendToAddressParams>) -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(
        WalletRpcImpl::send_to_address(params).await?,
    ))
}

#[derive(Serialize)]
pub struct Utxo {
    pub id: i64,
    pub hash: String,
    pub confirm_timestamp: Timestamp,
    // this two values are used to rollback
    pub confirm_height: i64,
    pub confirmed_txid: Option<String>,
    pub amount: String,
    pub locked: bool,
}

async fn avaliable_utxos() -> Result<ErasedJson, RestError> {
    Ok(ErasedJson::pretty(WalletRpcImpl::avaliable_utxos().await?))
}
