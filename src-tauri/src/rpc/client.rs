use anyhow::Result;

use crate::{config, wallet::balance::WalletHistory};

use super::{SendResponse, SendToAddressParams};
pub struct RestRpcClient {
    client: reqwest::Client,
}

impl RestRpcClient {
    pub fn new(token: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        RestRpcClient { client }
    }

    fn api_url() -> String {
        format!("http://localhost:{}", config::consts::RPC_PORT)
    }

    pub async fn history(&self) -> Result<Vec<WalletHistory>> {
        let url = format!("{}/rpc/wallet/history", Self::api_url());

        let resp = self.client.get(url).send().await?.json().await?;

        Ok(resp)
    }

    pub async fn send(&self, params: &SendToAddressParams) -> Result<String> {
        let url = format!("{}/rpc/send", Self::api_url());

        let resp = self
            .client
            .post(url)
            .json(&params)
            .send()
            .await?
            .json::<SendResponse>()
            .await?;

        Ok(resp.txid)
    }
}
