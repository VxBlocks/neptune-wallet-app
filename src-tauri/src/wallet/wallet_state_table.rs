use std::sync::atomic::Ordering;

use anyhow::Result;
use neptune_privacy::api::export::Timestamp;
use neptune_privacy::prelude::tasm_lib::prelude::Digest;
use neptune_privacy::state::wallet::expected_utxo::ExpectedUtxo;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Row;
use sqlx::Sqlite;
use sqlx::SqliteConnection;
use sqlx_migrator::Info;
use sqlx_migrator::Migrate;
use sqlx_migrator::Migrator;
use sqlx_migrator::Plan;
use tracing::info;

use super::UtxoRecoveryData;
use super::WalletState;

struct CreateWalletStateNumKeysMigration;

sqlx_migrator::sqlite_migration!(
    CreateWalletStateNumKeysMigration,
    "wallet_state",
    "create_wallet_state_keys",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE wallet_state_keys (id TEXT PRIMARY KEY, value TEXT NOT NULL)", //up
        "DROP TABLE wallet_state_keys"                                               //down
    )]
);

struct CreateWalletStateUtxosMigration;
sqlx_migrator::sqlite_migration!(
    CreateWalletStateUtxosMigration,
    "wallet_state",
    "create_wallet_state_utxos",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE wallet_state_utxos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        hash TEXT NOT NULL,
        recovery_data BLOB NOT NULL,
        spent_in_block TEXT DEFAULT NULL,
        confirmed_in_block TEXT NOT NULL,
        confirmed_txid TEXT DEFAULT NULL,
        spent_txid TEXT DEFAULT NULL,
        confirm_height INTEGER NOT NULL,
        spent_height INTEGER DEFAULT NULL
        )", //up
        "DROP TABLE wallet_state_utxos" //down
    )]
);

struct CreateWalletStateExpectedUtxoMigration;
sqlx_migrator::sqlite_migration!(
    CreateWalletStateExpectedUtxoMigration,
    "wallet_state",
    "create_wallet_state_expected_utxos",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE wallet_state_expected_utxos (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        txid TEXT NOT NULL,
        data BLOB NOT NULL,
        timestamp INTEGER NOT NULL
        )",
        "DROP TABLE wallet_state_expected_utxos"
    )]
);

struct CreateWalletStateKnownRawHashKeysMigration;
sqlx_migrator::sqlite_migration!(
    CreateWalletStateKnownRawHashKeysMigration,
    "wallet_state",
    "create_wallet_state_raw_hash_keys",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE wallet_state_raw_hash_keys (
            key TEXT PRIMARY KEY
        )",
        "DROP TABLE wallet_state_raw_hash_keys"
    )]
);

#[derive(Debug, Clone, Serialize)]
pub struct UtxoDbData {
    pub id: i64,
    pub hash: String,
    pub recovery_data: UtxoRecoveryData,
    // hash of the block, if any, in which this UTXO was spent
    pub spent_in_block: Option<UtxoBlockInfo>,

    // hash of the block, if any, in which this UTXO was confirmed
    pub confirmed_in_block: UtxoBlockInfo,

    // this two values are used to rollback
    pub confirm_height: i64,
    pub spent_height: Option<i64>,

    pub confirmed_txid: Option<String>,
    pub spent_txid: Option<String>,
}

impl UtxoDbData {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoBlockInfo {
    pub block_height: u64,
    pub block_digest: Digest,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Tip {
    pub height: u64,
    pub digest: Digest,
}

impl UtxoDbData {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<Self> {
        let recovery_data = row.get::<Vec<u8>, _>("recovery_data");
        let recovery_data = bincode::deserialize(&recovery_data)?;

        let spent_in_block = row.get::<Option<String>, _>("spent_in_block");
        let comfirmed_in_block = row.get::<String, _>("confirmed_in_block");

        let spent_in_block = match spent_in_block {
            Some(spent_in_block) => Some(serde_json::from_str::<UtxoBlockInfo>(&spent_in_block)?),
            None => None,
        };
        let confirmed_in_block = serde_json::from_str::<UtxoBlockInfo>(&comfirmed_in_block)?;

        Ok(Self {
            id: row.get("id"),
            hash: row.get("hash"),
            recovery_data,
            spent_in_block,
            confirmed_in_block,
            confirm_height: row.get("confirm_height"),
            spent_height: row.get("spent_height"),
            confirmed_txid: row.get("confirmed_txid"),
            spent_txid: row.get("spent_txid"),
        })
    }

    pub async fn create<'c, E>(&self, executor: E) -> anyhow::Result<()>
    where
        E: sqlx::Executor<'c, Database = Sqlite>,
    {
        let query = "INSERT INTO wallet_state_utxos (hash, recovery_data, confirmed_in_block, confirm_height) VALUES (?, ?, ?, ?)";

        let data = bincode::serialize(&self.recovery_data)?;

        let confirmed_in_block = serde_json::to_string(&self.confirmed_in_block)?;

        sqlx::query(query)
            .bind(&self.hash)
            .bind(&data)
            .bind(&confirmed_in_block)
            .bind(&self.confirm_height)
            .execute(executor)
            .await?;
        Ok(())
    }

    pub fn display_pretty(&self) -> String {
        let amount = self
            .recovery_data
            .utxo
            .get_native_currency_amount()
            .display_n_decimals(6);

        let aocl_index = self.recovery_data.aocl_index;

        format!(
            "WalletStateUtxo {{ id: {}, hash: {}, amount: {}, index: {}, spent_in_block: {:?}, confirmed_in_block: {:?}, abs_i : {:?}, sender_randomness: {}, receiver_preimage: {} }}",
            self.id,
            self.hash,
            amount,
            aocl_index,
            self.spent_in_block,
            self.confirmed_in_block,
            self.recovery_data.abs_i(),
            self.recovery_data.sender_randomness.to_hex(),
            self.recovery_data.receiver_preimage.to_hex()
        )
    }
}

pub struct ExpectedUtxoData {
    pub id: i64,
    pub txid: String,
    pub expected_utxo: ExpectedUtxo,
    /// created time, used to clean outdated data
    pub timestamp: Timestamp,
}

impl ExpectedUtxoData {
    fn from_row(row: sqlx::sqlite::SqliteRow) -> anyhow::Result<Self> {
        let expected_utxo = row.get::<Vec<u8>, _>("data");
        let expected_utxo = bincode::deserialize(&expected_utxo)?;

        Ok(Self {
            id: row.get("id"),
            txid: row.get("txid"),
            expected_utxo,
            timestamp: Timestamp::seconds(row.get::<i64, _>("timestamp").try_into()?),
        })
    }

    pub async fn create<'c, E>(&self, executor: E) -> anyhow::Result<()>
    where
        E: sqlx::Executor<'c, Database = Sqlite>,
    {
        let query =
            "INSERT INTO wallet_state_expected_utxos (txid, data, timestamp) VALUES (?, ?, ?)";

        let data = bincode::serialize(&self.expected_utxo)?;

        let timestamp: i64 = (self.timestamp.to_millis() / 1000) as i64;

        sqlx::query(query)
            .bind(&self.txid)
            .bind(&data)
            .bind(&timestamp)
            .execute(executor)
            .await?;
        Ok(())
    }
}

impl WalletState {
    pub async fn migrate_tables(&self) -> anyhow::Result<()> {
        let mut migrator = Migrator::default();
        // Adding migration can fail if another migration with same app and name and different values gets added
        // Adding migrations add its parents, replaces and not before as well
        migrator.add_migration(Box::new(CreateWalletStateNumKeysMigration))?;
        migrator.add_migration(Box::new(CreateWalletStateUtxosMigration))?;
        migrator.add_migration(Box::new(CreateWalletStateExpectedUtxoMigration))?;
        migrator.add_migration(Box::new(CreateWalletStateKnownRawHashKeysMigration))?;

        let mut conn = self.pool.acquire().await?;
        // use apply all to apply all pending migration
        migrator.run(&mut *conn, &Plan::apply_all()).await?;

        Ok(())
    }

    pub async fn set_num_symmetric_keys(&self, value: u64) -> Result<()> {
        let value_db = value.to_string();
        sqlx::query("INSERT INTO wallet_state_keys (id, value) VALUES ('num_symmetric_keys', ?) ON CONFLICT(id) DO UPDATE SET value = ?")
            .bind(&value_db)
            .bind(&value_db)
            .execute(&self.pool)
            .await?;

        self.num_symmetric_keys.store(value, Ordering::Relaxed);

        Ok(())
    }

    pub async fn get_num_symmetric_keys(&self) -> Result<u64> {
        let row =
            sqlx::query("SELECT value FROM wallet_state_keys WHERE id = 'num_symmetric_keys'")
                .fetch_one(&self.pool)
                .await;

        match row {
            Ok(row) => Ok(row.get::<String, _>(0).parse()?),
            Err(sqlx::Error::RowNotFound) => Ok(0),
            Err(err) => Err(err)?,
        }
    }

    pub async fn set_num_generation_spending_keys(&self, value: u64) -> Result<()> {
        let value_db = value.to_string();
        sqlx::query("INSERT INTO wallet_state_keys (id, value) VALUES ('num_generation_spending_keys', ?) ON CONFLICT(id) DO UPDATE SET value = ?")
            .bind(&value_db)
            .bind(&value_db)
            .execute(&self.pool).await?;
        self.num_generation_spending_keys
            .store(value, Ordering::Relaxed);
        Ok(())
    }

    pub async fn get_num_generation_spending_keys(&self) -> Result<u64> {
        let row = sqlx::query(
            "SELECT value FROM wallet_state_keys WHERE id = 'num_generation_spending_keys'",
        )
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(row) => Ok(row.get::<String, _>(0).parse()?),
            Err(sqlx::Error::RowNotFound) => Ok(0),
            Err(err) => Err(err)?,
        }
    }

    pub async fn set_tip<'c>(
        &self,
        tx: &'c mut SqliteConnection,
        (height, digest): (u64, Digest),
    ) -> Result<()> {
        let tip = Tip { height, digest };

        let value_db = serde_json::to_string(&tip)?;
        sqlx::query("INSERT INTO wallet_state_keys (id, value) VALUES ('tip', ?) ON CONFLICT(id) DO UPDATE SET value = ?")
            .bind(&value_db)
            .bind(&value_db)
            .execute(&mut *tx).await?;
        Ok(())
    }

    pub async fn get_tip(&self) -> Result<Option<(u64, Digest)>> {
        let row = sqlx::query("SELECT value FROM wallet_state_keys WHERE id = 'tip'")
            .fetch_one(&self.pool)
            .await;

        match row {
            Ok(row) => {
                let tip: Tip = serde_json::from_str(&row.get::<String, _>(0))?;
                Ok(Some((tip.height, tip.digest)))
            }
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err)?,
        }
    }

    pub async fn flush(&self) {}

    pub async fn append_utxos<'c>(
        &self,
        tx: &'c mut SqliteConnection,
        utxos: Vec<UtxoDbData>,
    ) -> Result<()> {
        for utxo in utxos {
            let tx = &mut *tx;
            utxo.create(&mut *tx).await?;
        }

        Ok(())
    }

    pub async fn update_spent_utxos<'c>(
        &self,
        tx: &'c mut SqliteConnection,
        utxos: Vec<(i64, UtxoBlockInfo)>,
    ) -> Result<()> {
        for utxo in &utxos {
            let info = serde_json::to_string(&utxo.1)?;

            sqlx::query::<Sqlite>("UPDATE wallet_state_utxos SET spent_in_block = ? WHERE id = ?")
                .bind(&info)
                .bind(&utxo.0)
                .execute(&mut *tx)
                .await?;
        }

        // remove from pending so it will not be updated again
        for (id, _) in utxos {
            info!("checking utxo {} for pending", id);
            if let Some(txid) = self.updater.try_remove_pending_by_utxo_id(tx, id).await? {
                info!("removing pending tx {}", txid);
                sqlx::query::<Sqlite>("UPDATE wallet_state_utxos SET spent_txid = ? WHERE id = ?")
                    .bind(&txid)
                    .bind(&id)
                    .execute(&mut *tx)
                    .await?;
            };
        }

        Ok(())
    }

    pub async fn update_utxos_with_expected_utxos<'c>(
        &self,
        tx: &'c mut SqliteConnection,
        utxos: Vec<(Digest, String)>,
        height: i64,
    ) -> Result<()> {
        for (digest, txid) in utxos {
            let hash = digest.to_hex();
            sqlx::query(
                "UPDATE wallet_state_utxos SET confirmed_txid = ? WHERE hash = ? AND confirm_height = ?",
            )
            .bind(&txid)
            .bind(&hash)
            .bind(&height)
            .execute(&mut *tx)
            .await?;
        }

        Ok(())
    }

    pub async fn get_utxos(&self) -> Result<Vec<UtxoDbData>> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query("SELECT * FROM wallet_state_utxos")
            .fetch_all(&mut *conn)
            .await?;

        let mut utxos: Vec<UtxoDbData> = Vec::new();
        for row in rows {
            let utxo = UtxoDbData::from_row(row)?;
            utxos.push(utxo);
        }

        Ok(utxos)
    }

    pub async fn get_utxo_db_data(&self, hash: &Digest) -> Result<Option<UtxoDbData>> {
        let hash = hash.to_hex();
        let row = sqlx::query("SELECT * FROM wallet_state_utxos WHERE hash =?")
            .bind(&hash)
            .fetch_one(&self.pool)
            .await;

        match row {
            Ok(row) => {
                let data: UtxoDbData = UtxoDbData::from_row(row)?;
                return Ok(Some(data));
            }
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        }
    }

    pub async fn get_unspent_utxos(&self) -> Result<Vec<UtxoDbData>> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query("SELECT * FROM wallet_state_utxos WHERE spent_in_block IS NULL")
            .fetch_all(&mut *conn)
            .await?;

        let mut utxos: Vec<UtxoDbData> = Vec::new();
        for row in rows {
            let utxo = UtxoDbData::from_row(row)?;
            utxos.push(utxo);
        }

        Ok(utxos)
    }

    pub async fn get_unspent_inputs_with_ids(&self, ids: &[i64]) -> Result<Vec<UtxoDbData>> {
        let mut conn = self.pool.acquire().await?;

        let mut utxos = Vec::with_capacity(ids.len());
        for id in ids {
            let row = sqlx::query(
                "SELECT * FROM wallet_state_utxos WHERE spent_in_block IS NULL AND id = ?",
            )
            .bind(&id)
            .fetch_one(&mut *conn)
            .await?;
            let utxo = UtxoDbData::from_row(row)?;
            utxos.push(utxo);
        }

        Ok(utxos)
    }

    pub async fn add_expected_utxo(&self, utxo: Vec<ExpectedUtxoData>) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for expedted in utxo {
            expedted.create(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn expected_utxos(&self) -> Result<Vec<ExpectedUtxoData>> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query("SELECT * FROM wallet_state_expected_utxos")
            .fetch_all(&mut *conn)
            .await?;

        let mut utxos: Vec<ExpectedUtxoData> = Vec::new();
        for row in rows {
            let utxo = ExpectedUtxoData::from_row(row)?;
            utxos.push(utxo);
        }

        Ok(utxos)
    }

    pub async fn update_new_generation_expected_utxos(
        &self,
        txid: &str,
        timestamp: Timestamp,
        expected_utxos: Vec<ExpectedUtxo>,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // sqlx::query("DELETE FROM wallet_state_expected_utxos WHERE txid = ?")
        //     .bind(txid)
        //     .execute(&mut *tx)
        //     .await?;

        for utxo in expected_utxos {
            let expected_data = ExpectedUtxoData {
                id: 0,
                txid: txid.to_owned(),
                expected_utxo: utxo,
                timestamp,
            };

            expected_data.create(&mut *tx).await?;
        }
        Ok(())
    }

    pub async fn clean_old_expected_utxos(&self) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
        let now = Timestamp::now().to_millis() / 1000;
        let begin = now - (2 * 60 * 60);
        let begin: i64 = begin.try_into()?;
        sqlx::query("DELETE FROM wallet_state_expected_utxos WHERE timestamp < ?")
            .bind(&begin)
            .execute(&mut *conn)
            .await?;
        Ok(())
    }

    //  add raw hash key; NOTE: this is unsafe, should only be called when syncing blocks
    pub async fn add_raw_hash_key<'c>(
        &self,
        tx: &'c mut SqliteConnection,
        key: Digest,
    ) -> Result<()> {
        let key_hex = key.to_hex();
        sqlx::query(
            "INSERT INTO wallet_state_raw_hash_keys (key) VALUES (?) ON CONFLICT DO NOTHING",
        )
        .bind(&key_hex)
        .execute(&mut *tx)
        .await?;

        let old_ptr = self.know_raw_hash_keys.load(Ordering::Acquire);
        if old_ptr.is_null() {
            let new_vec = Box::into_raw(Box::new(vec![key]));
            self.know_raw_hash_keys.store(new_vec, Ordering::Release);
        } else {
            let vec_ref = unsafe { &mut *old_ptr };
            if !vec_ref.contains(&key) {
                vec_ref.push(key);
            }
        }

        Ok(())
    }

    pub async fn init_raw_hash_keys(&self) -> Result<()> {
        let mut conn = self.pool.acquire().await?;
        let rows = sqlx::query("SELECT * FROM wallet_state_raw_hash_keys")
            .fetch_all(&mut *conn)
            .await?;
        let mut keys: Vec<Digest> = Vec::with_capacity(rows.len());
        for row in rows {
            let key = Digest::try_from_hex(&row.get::<String, _>("key"))?;
            keys.push(key);
        }

        let new_vec = Box::into_raw(Box::new(keys));
        self.know_raw_hash_keys.store(new_vec, Ordering::Release);

        Ok(())
    }

    // reorganize to the fork point
    pub async fn reorganize_to_height<'c>(
        &self,
        tx: &'c mut SqliteConnection,
        height: u64,
        digest: Digest,
    ) -> Result<()> {
        let height_i64 = height as i64;

        let ids = sqlx::query("SELECT id FROM wallet_state_utxos WHERE confirm_height > ?")
            .bind(&height_i64)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|row| row.get::<i64, _>(0))
            .collect::<Vec<_>>();

        self.updater
            .try_clean_pending_by_utxo(&mut *tx, ids)
            .await?;

        sqlx::query("DELETE FROM wallet_state_utxos WHERE confirm_height > ?")
            .bind(&height_i64)
            .execute(&mut *tx)
            .await?;

        sqlx::query("UPDATE wallet_state_utxos SET spent_height = NULL, spent_txid = NULL, spent_in_block = NULL WHERE spent_height > ?")
            .bind(&height_i64)
            .execute(&mut *tx)
            .await?;

        self.set_tip(&mut *tx, (height, digest)).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use neptune_privacy::api::export::Network;
    use neptune_privacy::state::wallet::wallet_entropy::WalletEntropy;

    use super::*;
    use crate::config::wallet::ScanConfig;
    use crate::config::wallet::WalletConfig;
    #[tokio::test]
    async fn test_migrate_tables() {
        let config = WalletConfig {
            id: 0,
            key: WalletEntropy::devnet_wallet(),
            scan_config: ScanConfig {
                num_keys: 1,
                start_height: 0,
            },
            network: Network::Main,
        };

        let wallet_state = WalletState::new(config, &PathBuf::new()).await.unwrap();

        wallet_state.migrate_tables().await.unwrap();

        wallet_state.set_num_symmetric_keys(1).await.unwrap();
        wallet_state
            .set_num_generation_spending_keys(2)
            .await
            .unwrap();

        assert_eq!(wallet_state.get_num_symmetric_keys().await.unwrap(), 1);
        assert_eq!(
            wallet_state
                .get_num_generation_spending_keys()
                .await
                .unwrap(),
            2
        );

        let mut tx = wallet_state.pool.begin().await.unwrap();

        wallet_state
            .add_raw_hash_key(&mut *tx, Digest::try_from_hex("ded35bd6d93a222591ad88ebaea3ecc63598b30b18851231b50980557989734e362f5494404feb0e").unwrap())
            .await
            .unwrap();

        println!("add 1");

        wallet_state
            .add_raw_hash_key(&mut *tx, Digest::try_from_hex("ded35bd6d93a222591ad88ebaea3ecc63598b30b18851231b50980557989734e362f5494404feb0e").unwrap())
            .await
            .unwrap();

        tx.commit().await.unwrap();
        println!("add 2");

        // assert!(wallet_state.get_known_raw_hash_keys().len() == 1);

        wallet_state.init_raw_hash_keys().await.unwrap();

        // assert!(wallet_state.get_known_raw_hash_keys().len() == 1);
    }
}
