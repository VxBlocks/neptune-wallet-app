use std::net::SocketAddr;

use anyhow::Result;
use hyper::header::{CONTENT_TYPE, HeaderValue};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Request, Response, StatusCode, body::Body};
use jsonrpc_core::types::request::Request as JsonRpcRequest;
use jsonrpc_core::{Id, MethodCall, Params};
use serde_json::{Value, from_slice};
use tokio::sync::oneshot::Sender;

use crate::tls;

const CODE_AES_DECRYPT_ERROR: i64 = 1234;
const CODE_NO_PUBKEY_FOUND: i64 = 1235;
const CODE_ECDH_ERROR: i64 = 1236;

const HEADER_PUBLIC_KEY: &str = "Public-Key";

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::OPTIONS, _) => {
            let resp = Response::builder()
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                .header("Access-Control-Allow-Headers", "Content-Type, Public-Key")
                .body(Body::empty())
                .unwrap();
            Ok(resp)
        }
        (&hyper::Method::GET, "/discovery") => {
            let res = super::rpc::RPC_HANDER
                .handle_rpc_request(JsonRpcRequest::Single(jsonrpc_core::Call::MethodCall(
                    MethodCall {
                        jsonrpc: Some(jsonrpc_core::Version::V2),
                        method: "discovery".to_string(),
                        params: Params::None,
                        id: Id::Num(1),
                    },
                )))
                .await;

            if let Some(response) = res {
                let response_body = serde_json::to_string(&response).unwrap_or_default();
                let body = Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
                    .header("Access-Control-Allow-Origin", "*")
                    .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                    .header("Access-Control-Allow-Headers", "Content-Type, Public-Key")
                    .body(Body::from(response_body))
                    .unwrap();
                return Ok(body);
            }

            // 返回响应
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
                .body(Body::from("Response"))
                .unwrap();

            Ok(response)
        }
        (&hyper::Method::POST, "/") => {
            let mut response = aes_decode_middleware(req).await;
            let headers = response.headers_mut();
            headers.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
            headers.insert(
                "Access-Control-Allow-Methods",
                "GET, POST, OPTIONS".parse().unwrap(),
            );
            headers.insert(
                "Access-Control-Allow-Headers",
                "Content-Type, Public-Key".parse().unwrap(),
            );

            Ok(response)
        }
        _ => {
            let response = Response::builder().status(404).body(Body::empty()).unwrap();
            Ok(response)
        }
    }
}

async fn aes_decode_middleware(req: Request<Body>) -> Response<Body> {
    let (parts, body) = req.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.unwrap();

    let pk = match read_header_hex_as_bytes(parts.headers.get(HEADER_PUBLIC_KEY)) {
        Ok(v) => v,
        Err(e) => {
            let mut error = jsonrpc_core::error::Error::new(jsonrpc_core::ErrorCode::ServerError(
                CODE_NO_PUBKEY_FOUND,
            ));
            error.data = Some(Value::String(format!("{:#?}", e)));
            error.message = "no public key found".to_string();
            return jsonrpc_error_to_response(error);
        }
    };

    let shared = match tls::generate_p256_shared_secret(&pk) {
        Ok(v) => v,
        Err(e) => {
            let mut error = jsonrpc_core::error::Error::new(jsonrpc_core::ErrorCode::ServerError(
                CODE_ECDH_ERROR,
            ));
            error.data = Some(Value::String(format!("{:#?}", e)));
            error.message = "failed to generate shared secret".to_string();
            return jsonrpc_error_to_response(error);
        }
    };

    let aes_key = tls::shared_secret_to_symmetric_secret(&shared);

    let decoded_body = match tls::aes::aes_decode(&aes_key, &body_bytes) {
        Ok(v) => v,
        Err(e) => {
            let mut error = jsonrpc_core::error::Error::new(jsonrpc_core::ErrorCode::ServerError(
                CODE_AES_DECRYPT_ERROR,
            ));
            error.data = Some(serde_json::Value::String(format!("{:#?}", e)));
            error.message = e.to_string();
            return jsonrpc_error_to_response(error);
        }
    };

    let new_body = Body::from(decoded_body);

    let new_req = Request::from_parts(parts, new_body);

    handle_rpc(new_req).await
}

fn read_header_hex_as_bytes(header: Option<&HeaderValue>) -> Result<Vec<u8>> {
    match header {
        Some(header_value) => match header_value.to_str() {
            Ok(v) => {
                let byt = hex::decode(v)?;
                Ok(byt.to_vec())
            }
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        },
        None => Err(anyhow::anyhow!("no value")),
    }
}

async fn handle_rpc(req: Request<Body>) -> Response<Body> {
    let (_, body) = req.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await.unwrap();

    // Parse the request body as JSON-RPC request
    let decoded_body: JsonRpcRequest = match from_slice(&body_bytes) {
        Ok(request) => request,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap();
        }
    };

    let res = super::rpc::RPC_HANDER
        .handle_rpc_request(decoded_body)
        .await;

    if let Some(response) = res {
        let response_body = serde_json::to_string(&response).unwrap_or_default();
        let body = Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .body(Body::from(response_body))
            .unwrap();
        return body;
    }

    // 返回响应
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("text/plain"))
        .body(Body::from("Response"))
        .unwrap();

    response
}

pub fn start_hyper(address: &SocketAddr) -> Sender<()> {
    // Create the Hyper server
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handle_request)) });
    let server = hyper::Server::bind(address).serve(make_svc);

    // Prepare some signal for when the server should start shutting down...
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let graceful = server.with_graceful_shutdown(async {
        rx.await.ok();
    });

    tokio::spawn(async {
        // Start the server
        if let Err(err) = graceful.await {
            tracing::error!("server error: {}", err);
            // crate::service::app::error_dialog(&format!("server error: {}", err));
        }
    });

    tx
}

pub fn jsonrpc_error_to_response(err: jsonrpc_core::Error) -> hyper::Response<Body> {
    let response_body = serde_json::to_string(&err).unwrap_or_default();
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .body(Body::from(response_body))
        .unwrap()
}
