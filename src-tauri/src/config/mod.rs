use anyhow::{anyhow, Context, Result};
use neptune_cash::config_models::{data_directory::DataDirectory, network::Network};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::{path::PathBuf, str::FromStr};
use tokio::sync::Mutex;

use crate::rpc::tls;

mod config_migrate;
pub mod consts;
pub mod wallet;

#[derive(Debug)]
pub struct Config {
    db: SqlitePool,
    pub password: Mutex<Option<String>>,
    // the key used to decrypt the wallet secret
    // this key is stored in the config file and encoded with the secret key
    // the secret key is generated by the password
    pub decrypt_key: Mutex<Vec<u8>>,
}

const PASSWORD_TEST: &str = "hello world!";
const PASSWORD_TEST_KEY: &str = "password_test";
impl Config {
    pub async fn new(data_dir: &PathBuf) -> Result<Self> {
        DataDirectory::create_dir_if_not_exists(&data_dir).await?;
        let db_path = data_dir.join("config.db");

        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = sqlx::SqlitePool::connect_with(options)
            .await
            .map_err(|err| anyhow::anyhow!("Could not connect to database: {err}"))?;

        let config = Self {
            db: pool,
            password: Mutex::new(None),
            decrypt_key: Mutex::new(Vec::new()),
        };

        config.migrate_tables().await?;

        config.set_data_dir(data_dir).await?;

        Ok(config)
    }

    async fn set_data<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let data = serde_json::to_vec(value)?;
        sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)")
            .bind(key)
            .bind(data)
            .execute(&self.db)
            .await
            .context("Failed to insert or replace data")?;
        Ok(())
    }

    async fn get_data<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let row = sqlx::query("SELECT value FROM config WHERE key = ?1")
            .bind(key)
            .fetch_optional(&self.db)
            .await
            .context("Failed to query data")?;

        match row {
            Some(row) => {
                let data: Vec<u8> = row.get(0);
                Ok(Some(serde_json::from_slice(&data)?))
            }
            None => Ok(None),
        }
    }

    pub async fn set_data_dir(&self, path: &PathBuf) -> Result<()> {
        self.set_data("data_dir", &path).await
    }

    pub async fn get_data_dir(&self) -> Result<PathBuf> {
        match self.get_data::<PathBuf>("data_dir").await? {
            Some(p) => Ok(p),
            None => Err(anyhow!("data_dir not set")),
        }
    }

    pub async fn set_wallet_id(&self, id: i64) -> Result<()> {
        self.set_data("wallet_id", &id).await
    }

    pub async fn get_wallet_id(&self) -> Result<i64> {
        match self.get_data::<i64>("wallet_id").await? {
            Some(v) => Ok(v),
            None => Ok(1),
        }
    }

    pub async fn set_network(&self, network: Network) -> Result<()> {
        self.set_data("network", &network.to_string()).await
    }

    pub async fn get_network(&self) -> Result<Network> {
        if let Some(n) = self.get_data::<String>("network").await? {
            return Ok(Network::from_str(&n).map_err(|e| anyhow!("{}", e))?);
        }
        Ok(Network::Main)
    }

    pub async fn set_disk_cache(&self, enabled: bool) -> Result<()> {
        self.set_data("disk_cache", &enabled).await
    }

    pub async fn get_disk_cache(&self) -> Result<bool> {
        match self.get_data::<bool>("disk_cache").await? {
            Some(v) => Ok(v),
            None => Ok(true),
        }
    }

    async fn remote_rest_key(&self) -> Result<&str> {
        let network = self.get_network().await?;
        match network {
            Network::Main => Ok("remote_rest"),
            Network::Testnet => Ok("remote_rest_testnet"),
            Network::RegTest => Ok("remote_rest_regtest"),
            _ => Ok("remote_rest"),
        }
    }
    pub async fn set_remote_rest(&self, rest: &String) -> Result<()> {
        let key = self.remote_rest_key().await?;
        self.set_data::<String>(key, &rest).await
    }

    pub async fn get_remote_rest(&self) -> Result<String> {
        let key = self.remote_rest_key().await?;
        Ok(self
            .get_data::<String>(key)
            .await?
            .unwrap_or("https://nptwallet.vxb.ai".to_string()))
    }

    pub async fn decrypt_config(&self, password: &str) -> Result<()> {
        let pass_test = self
            .get_data::<Vec<u8>>(PASSWORD_TEST_KEY)
            .await?
            .context("password not set")?;
        match password {
            "" => {
                if pass_test != PASSWORD_TEST.as_bytes().to_vec() {
                    return Err(anyhow!("password is wrong"));
                }
            }
            _ => {
                let encrypt_key = hash(password);
                let decrypted = crate::rpc::tls::aes::aes_decode(&encrypt_key, &pass_test)
                    .context("cant decode db")?;
                if decrypted != PASSWORD_TEST.as_bytes() {
                    return Err(anyhow!("password is wrong"));
                }
            }
        };

        {
            let mut password_guard = self.password.lock().await;
            password_guard.replace(password.to_string());
        }

        {
            let decrypt_key = self.get_decrypt_key().await.context("get_decrypt_key")?;
            let mut decrypt_key_guard = self.decrypt_key.lock().await;
            *decrypt_key_guard = decrypt_key;
        }

        Ok(())
    }

    // used to set or change the password
    pub async fn set_password(&self, old: &str, password: &str) -> Result<()> {
        // make sure config is decrypted, otherwise decrypt_key will be generated and old key will be lost!
        if self.has_password().await? {
            self.decrypt_config(old)
                .await
                .context("failed to decrypt config")?;
        }

        self.password.lock().await.replace(password.to_string());
        let secret_key = self
            .create_secret_key()
            .await
            .context("create server secret")?;
        match password {
            "" => {
                self.set_data::<Vec<u8>>(PASSWORD_TEST_KEY, &PASSWORD_TEST.as_bytes().to_vec())
                    .await?;
            }
            _ => {
                let encrypt_key = hash(password);
                let encrypted = tls::aes::aes_encode(&encrypt_key, PASSWORD_TEST.as_bytes())?;
                self.set_data::<Vec<u8>>(PASSWORD_TEST_KEY, &encrypted)
                    .await?;
            }
        };

        self.update_decrypt_key(secret_key)
            .await
            .context("update_decrypt_key")?;

        Ok(())
    }

    pub async fn has_password(&self) -> Result<bool> {
        Ok(self.get_data::<Vec<u8>>(PASSWORD_TEST_KEY).await?.is_some())
    }

    pub async fn create_secret_key(&self) -> Result<Vec<u8>> {
        let secret_key = tls::generate_p256_secret().context("generate secret")?;

        match self.password.lock().await.as_ref() {
            Some(v) => {
                if v == "" {
                    self.set_data::<Vec<u8>>("secret_key", &secret_key)
                        .await
                        .context("cant write to db")?;
                } else {
                    let encrypt_key = hash(v);
                    let encrypted = tls::aes::aes_encode(&encrypt_key, &secret_key)?;
                    self.set_data::<Vec<u8>>("secret_key", &encrypted)
                        .await
                        .context("cant write to db")?;
                }
            }

            None => {
                return Err(anyhow!("no password set!"));
            }
        }

        Ok(secret_key)
    }

    // secret_key is encoded with the password, it will be changed when the password is changed
    // it is not stable to use it as the key to decrypt the wallet secret, but can be used to validate access via rpc
    pub async fn get_secret_key(&self) -> Result<Vec<u8>> {
        let value = self
            .get_data::<Vec<u8>>("secret_key")
            .await
            .context("cant read db")?
            .context("secret not set!")?;

        match self
            .password
            .lock()
            .await
            .clone()
            .ok_or(anyhow!("no password set!"))?
            .as_str()
        {
            "" => return Ok(value),
            str => {
                let encrypt_key = hash(str);
                let decrypted =
                    tls::aes::aes_decode(&encrypt_key, &value).context("decode secret key")?;
                return Ok(decrypted);
            }
        }
    }

    async fn get_decrypt_key(&self) -> Result<Vec<u8>> {
        let secret_key = self.get_secret_key().await.context("get secret key")?;

        let encoded = self
            .get_data::<Vec<u8>>("wallet_secret")
            .await?
            .context("wallet secret not set")?;

        let decoded = tls::aes::aes_decode(&secret_key, &encoded).context("decode decrypt_key")?;
        Ok(decoded)
    }

    /// used to init or update the decrypt key, should be called after [`decrypt_config`] or first time set the password
    async fn update_decrypt_key(&self, secret_key: Vec<u8>) -> Result<()> {
        let mut decrypt_key_guard = self.decrypt_key.lock().await;
        let mut old_decrypt_key = decrypt_key_guard.clone();
        if old_decrypt_key.is_empty() {
            old_decrypt_key = tls::aes::generate_aes_256_key();
            *decrypt_key_guard = old_decrypt_key.clone();
        }

        let encoded = tls::aes::aes_encode(&secret_key, &old_decrypt_key)?;
        self.set_data::<Vec<u8>>("wallet_secret", &encoded).await?;
        Ok(())
    }

    pub async fn set_log_level(&self, level: &str) -> Result<()> {
        self.set_data::<String>("log_level", &level.to_string())
            .await
    }

    pub async fn get_log_level(&self) -> Result<Option<String>> {
        self.get_data::<String>("log_level").await
    }
}

pub fn hash(str: &str) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(str);
    let result = hasher.finalize();
    result.to_vec()
}
