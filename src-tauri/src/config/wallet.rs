use anyhow::Result;
use neptune_privacy::api::export::Network;
use neptune_privacy::api::export::SpendingKey;
use neptune_privacy::state::wallet::secret_key_material::SecretKeyMaterial;
use neptune_privacy::state::wallet::wallet_entropy::WalletEntropy;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Row;

use super::Config;

impl Config {
    pub async fn get_current_wallet(&self) -> Result<WalletConfig> {
        let mut conn = self.db.acquire().await?;

        let id = self.get_wallet_id().await?;

        let row = sqlx::query("select id,secret_key,scan_config from wallets where id = ?")
            .bind(&id)
            .fetch_one(&mut *conn)
            .await?;

        let secret = row.get::<Vec<u8>, _>("secret_key");
        let mnemonic = self.secret_to_mnemonic(secret).await?;
        let secret = SecretKeyMaterial::from_phrase(&mnemonic)?;
        let key = WalletEntropy::new(secret);

        let scan_config = row.get::<String, _>("scan_config");
        let scan_config = serde_json::from_str::<ScanConfig>(&scan_config)?;

        let network = self.get_network().await?;

        let id = row.get::<i64, _>("id");

        Ok(WalletConfig {
            id,
            key,
            scan_config,
            network,
        })
    }

    pub async fn get_wallet_mnemonic(&self, id: i64) -> Result<Vec<String>> {
        let mut conn = self.db.acquire().await?;

        let row = sqlx::query("select secret_key from wallets where id = ?")
            .bind(&id)
            .fetch_one(&mut *conn)
            .await?;

        let secret = row.get::<Vec<u8>, _>("secret_key");
        self.secret_to_mnemonic(secret).await
    }

    pub async fn add_wallet(
        &self,
        name: &str,
        mnemonic: Vec<String>,
        scan_config: ScanConfig,
    ) -> Result<i64> {
        let mut conn = self.db.acquire().await?;

        let network = self.get_network().await?;

        let address = mnemonic_to_address(&mnemonic, network)?;

        let scan_config = serde_json::to_string(&scan_config)?;

        let secret = self.mnemonic_to_secret(mnemonic).await?;

        let res =  sqlx::query(
            "INSERT INTO wallets (name, secret_key, scan_config, address, balance) VALUES (?,?,?,?,?)",
        )
        .bind(&name)
        .bind(&secret)
        .bind(&scan_config)
        .bind(&address)
        .bind(&"".to_string())
        .execute(&mut *conn)
        .await?;

        Ok(res.last_insert_rowid())
    }

    pub async fn remove_wallet(&self, id: i64) -> Result<()> {
        let mut conn = self.db.acquire().await?;
        sqlx::query("delete from wallets where id = ?")
            .bind(&id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    pub async fn get_wallets(&self) -> Result<Vec<WalletData>> {
        let mut conn = self.db.acquire().await?;

        let rows = sqlx::query("select id,name,address,balance from wallets")
            .fetch_all(&mut *conn)
            .await?;

        let mut wallets = vec![];
        for row in rows {
            let id = row.get::<i64, _>("id");
            let name = row.get::<String, _>("name");
            let address = row.get::<String, _>("address");
            let balance = row.get::<String, _>("balance");
            wallets.push(WalletData {
                id,
                name,
                address,
                balance,
            })
        }
        Ok(wallets)
    }

    pub async fn update_wallet_balance(&self, id: i64, balance: String) -> Result<()> {
        let mut conn = self.db.acquire().await?;
        sqlx::query("update wallets set balance = ? where id = ?")
            .bind(&balance)
            .bind(&id)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    pub async fn mnemonic_to_secret(&self, mnemonic: Vec<String>) -> Result<Vec<u8>> {
        let phase = mnemonic.join(" ");
        let encoded =
            crate::rpc::tls::aes::aes_encode(&self.decrypt_key.lock().await, &phase.as_bytes())?;
        Ok(encoded)
    }

    pub async fn secret_to_mnemonic(&self, secret: Vec<u8>) -> Result<Vec<String>> {
        let decode_key = self.decrypt_key.lock().await.clone();
        let phrase = crate::rpc::tls::aes::aes_decode(&decode_key, &secret)?;
        let phrase = String::from_utf8(phrase)?;
        let phrase = phrase.split(" ").map(|v| v.to_string()).collect::<Vec<_>>();
        Ok(phrase)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletData {
    id: i64,
    name: String,
    address: String,
    balance: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanConfig {
    #[serde(default = "default_num_keys")]
    pub num_keys: u64,
    #[serde(default)]
    pub start_height: u64,
}

pub struct WalletConfig {
    pub id: i64,
    pub key: WalletEntropy,
    pub scan_config: ScanConfig,
    pub network: Network,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            num_keys: default_num_keys(),
            start_height: 0,
        }
    }
}

fn default_num_keys() -> u64 {
    25
}
pub fn mnemonic_to_address(mnemonic: &[String], network: Network) -> Result<String> {
    let secret = SecretKeyMaterial::from_phrase(mnemonic)?;
    let key = WalletEntropy::new(secret);
    let generation_spending_key = key.nth_generation_spending_key(0);
    let spending_key = SpendingKey::from(generation_spending_key);

    spending_key.to_address().to_bech32m(network)
}
