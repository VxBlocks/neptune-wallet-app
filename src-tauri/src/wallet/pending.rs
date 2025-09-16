use anyhow::Context;
use anyhow::Result;
use neptune_cash::api::export::Timestamp;
use neptune_cash::api::export::TransactionDetails;
use neptune_cash::api::export::TxProvingCapability;
use neptune_cash::protocol::consensus::block::Block;
use neptune_cash::state::wallet::expected_utxo::UtxoNotifier;
use sqlx::Row;
use sqlx::SqliteConnection;
use sqlx::SqlitePool;
use sqlx_migrator::Info;
use sqlx_migrator::Migrate;
use sqlx_migrator::Migrator;
use sqlx_migrator::Plan;
use tracing::*;

use super::WalletState;
use crate::rpc_client;

impl super::WalletState {
    // txid, amount
    pub async fn get_pending_transactions(&self) -> Result<Vec<String>> {
        self.updater.get_pending_transaction_ids().await
    }

    pub async fn forget_tx(&self, txid: &str) -> Result<()> {
        self.updater.delete_transaction(txid).await
    }
}

struct CreatePendingTxMigration;

sqlx_migrator::sqlite_migration!(
    CreatePendingTxMigration,
    "wallet_state",
    "create_pengind_tx",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE wallet_state_pending (
        id TEXT PRIMARY KEY,
        details BLOB NOT NULL,
        finished INTEGER NOT NULL DEFAULT 0
        )", //up
        "DROP TABLE wallet_state_pending" //down
    )]
);

struct CreatePendingTxDbIdsMigration;

sqlx_migrator::sqlite_migration!(
    CreatePendingTxDbIdsMigration,
    "wallet_state",
    "create_pengind_tx_dbids",
    sqlx_migrator::vec_box![],
    sqlx_migrator::vec_box![(
        "CREATE TABLE wallet_state_pending_ids (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        txid TEXT NOT NULL,
        utxo_id INTEGER NOT NULL,
        finished INTEGER NOT NULL DEFAULT 0
        )", //up
        "DROP TABLE wallet_state_pending_ids" //down
    )]
);

pub struct TransactionUpdater {
    pool: SqlitePool,
}

// upgrade transaction after new block
impl TransactionUpdater {
    pub async fn new(pool: SqlitePool) -> anyhow::Result<Self> {
        let updater = Self { pool };

        updater.migrate_tables().await?;
        Ok(updater)
    }

    pub async fn migrate_tables(&self) -> anyhow::Result<()> {
        let mut migrator = Migrator::default();
        // Adding migration can fail if another migration with same app and name and different values gets added
        // Adding migrations add its parents, replaces and not before as well
        migrator.add_migration(Box::new(CreatePendingTxMigration))?;
        migrator.add_migration(Box::new(CreatePendingTxDbIdsMigration))?;

        let mut conn = self.pool.acquire().await?;
        // use apply all to apply all pending migration
        migrator.run(&mut *conn, &Plan::apply_all()).await?;

        Ok(())
    }

    pub async fn update_transactions(&self, wallet_state: &WalletState) {
        info!("Updating transactions");
        let mut tx = match self.pool.acquire().await {
            Ok(conn) => conn,
            Err(err) => {
                error!("Error acquiring database connection: {}", err);
                return;
            }
        };

        let transactions = match self.get_pending_transactions(&mut *tx).await {
            Ok(transactions) => transactions,
            Err(err) => {
                error!("Error getting pending transactions: {}", err);
                return;
            }
        };

        for (txid, transaction, _) in transactions {
            info!("updating transaction {}", txid);
            match self
                .update_transaction(txid.to_owned(), wallet_state, transaction)
                .await
            {
                Ok(detail) => {
                    if let Err(e) = self.update_detail(&txid, &detail).await {
                        error!("Error updating transaction: {}", e);
                    }
                }
                Err(err) => {
                    error!("error update transaction {} : {:#?}", txid, err);
                }
            };
        }
    }

    // update transaction to tip and broadcast to node
    async fn update_transaction(
        &self,
        tx_id: String,
        wallet_state: &WalletState,
        detail: TransactionDetails,
    ) -> Result<TransactionDetails> {
        info!("update transaction {}", tx_id);
        let tx_inputs = detail.tx_inputs;
        let mut tx_outputs = detail.tx_outputs;
        let fee = detail.fee;
        let timestamp = Timestamp::now();

        let mut recovery_data_list = Vec::with_capacity(tx_inputs.len());
        for tx_input in tx_inputs.iter() {
            let recovery_data = wallet_state
                .get_recovery_data_from_utxo(&tx_input.utxo)
                .await?;
            recovery_data_list.push(recovery_data);
        }

        let (unlocked_new, tip_digest) = wallet_state.unlock_utxos(recovery_data_list).await?;

        let tip: Block = rpc_client::node_rpc_client()
            .request_block_by_digest(&tip_digest.to_hex())
            .await?
            .context("Failed to get tip block")?
            .to_block();

        let tip_mutator_set_accumulator = tip.mutator_set_accumulator_after()?;

        let block_height = tip.header().height;
        for tx_output in tx_outputs.iter_mut() {
            let new_sender_randomness = wallet_state
                .key
                .generate_sender_randomness(block_height, tx_output.receiver_digest());
            tx_output.set_sender_randomness(new_sender_randomness);
        }

        let expected_utxo = wallet_state.extract_expected_utxos(&tx_outputs, UtxoNotifier::Myself);

        wallet_state
            .update_new_generation_expected_utxos(&tx_id, timestamp, expected_utxo)
            .await?;

        let transaction_details = TransactionDetails::new_without_coinbase(
            unlocked_new,
            tx_outputs,
            fee,
            timestamp,
            tip_mutator_set_accumulator,
            wallet_state.network,
        );

        let transaction = wallet_state
            .create_raw_transaction(&transaction_details, TxProvingCapability::ProofCollection)
            .await?;

        let _new_id = rpc_client::node_rpc_client()
            .broadcast_transaction(&transaction)
            .await?;

        Ok(transaction_details)
    }

    pub async fn add_transaction(
        &self,
        tx_id: String,
        detail: TransactionDetails,
        input_ids: Vec<i64>,
    ) -> Result<()> {
        let mut conn = self.pool.begin().await?;

        let detail = bincode::serialize(&detail)?;

        sqlx::query("INSERT INTO wallet_state_pending (id, details) VALUES (?, ?)")
            .bind(&tx_id)
            .bind(&detail)
            .execute(&mut *conn)
            .await?;

        for utxo_id in input_ids {
            sqlx::query("INSERT INTO wallet_state_pending_ids (txid, utxo_id) VALUES (?, ?)")
                .bind(&tx_id)
                .bind(&utxo_id)
                .execute(&mut *conn)
                .await?;
        }

        conn.commit().await?;

        Ok(())
    }

    async fn update_detail(&self, tx_id: &str, detail: &TransactionDetails) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        let detail = bincode::serialize(&detail)?;
        sqlx::query("UPDATE wallet_state_pending SET details = ? WHERE id = ?")
            .bind(&detail)
            .bind(tx_id)
            .execute(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn get_pending_transactions<'c>(
        &self,
        tx: &'c mut SqliteConnection,
    ) -> Result<Vec<(String, TransactionDetails, Vec<i64>)>> {
        let rows = sqlx::query("SELECT * FROM wallet_state_pending WHERE finished = 0")
            .fetch_all(&mut *tx)
            .await?;

        let mut result = vec![];
        for row in rows {
            let txid = row.get::<String, _>("id");
            let detail = row.get::<Vec<u8>, _>("details");
            let detail = bincode::deserialize::<TransactionDetails>(&detail)?;

            let spent_utxos =
                sqlx::query("SELECT utxo_id FROM wallet_state_pending_ids WHERE txid = ?")
                    .bind(&txid)
                    .fetch_all(&mut *tx)
                    .await?
                    .into_iter()
                    .map(|row| row.get::<i64, _>(0))
                    .collect::<Vec<_>>();

            result.push((txid, detail, spent_utxos));
        }

        Ok(result)
    }

    pub async fn delete_transaction(&self, tx_id: &str) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query("DELETE FROM wallet_state_pending WHERE id = ?")
            .bind(tx_id)
            .execute(&mut *conn)
            .await?;

        sqlx::query("DELETE FROM wallet_state_pending_ids WHERE txid = ?")
            .bind(tx_id)
            .execute(&mut *conn)
            .await?;

        Ok(())
    }

    pub async fn get_pending_transaction_ids(&self) -> Result<Vec<String>> {
        let mut conn = self.pool.acquire().await?;

        let transactions = sqlx::query("SELECT id FROM wallet_state_pending WHERE finished = 0")
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(|row| row.get::<String, _>(0))
            .collect::<Vec<_>>();

        Ok(transactions)
    }

    // returns all spent utxos database index
    pub async fn get_pending_spent_utxos(&self) -> Result<Vec<i64>> {
        let mut conn = self.pool.acquire().await?;

        let spent_utxos =
            sqlx::query("SELECT utxo_id FROM wallet_state_pending_ids WHERE finished = 0")
                .fetch_all(&mut *conn)
                .await?
                .into_iter()
                .map(|row| row.get::<i64, _>(0))
                .collect::<Vec<_>>();

        Ok(spent_utxos)
    }

    // remove pending and returns transaction id
    pub async fn try_remove_pending_by_utxo_id<'c>(
        &self,
        tx: &'c mut SqliteConnection,
        id: i64,
    ) -> Result<Option<String>> {
        let txids = sqlx::query("SELECT txid FROM wallet_state_pending_ids WHERE utxo_id = ?")
            .bind(id)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|row| row.get::<String, _>(0))
            .collect::<Vec<_>>();

        let mut remove = None;
        for txid in txids {
            sqlx::query("UPDATE wallet_state_pending SET finished = 1 WHERE id = ?")
                .bind(&txid)
                .execute(&mut *tx)
                .await?;

            sqlx::query("UPDATE wallet_state_pending_ids SET finished = 1 WHERE txid = ?")
                .bind(&txid)
                .execute(&mut *tx)
                .await?;
            remove = Some(txid);
        }

        Ok(remove)
    }

    pub async fn try_clean_pending_by_utxo<'c>(
        &self,
        tx: &'c mut SqliteConnection,
        utxoid: Vec<i64>,
    ) -> Result<()> {
        let transactions = match self.get_pending_transactions(&mut *tx).await {
            Ok(transactions) => transactions,
            Err(err) => {
                error!("Error getting pending transactions: {}", err);
                vec![]
            }
        };

        for transaction in transactions {
            if let Some(_utxo) = transaction.2.iter().find(|id| utxoid.contains(id)) {
                //should be deleted
                let txid = transaction.0;
                sqlx::query("DELETE FROM wallet_state_pending WHERE id = ?")
                    .bind(&txid)
                    .execute(&mut *tx)
                    .await?;

                sqlx::query("DELETE FROM wallet_state_pending_ids WHERE txid = ?")
                    .bind(&txid)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        Ok(())
    }
}
