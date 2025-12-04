use std::collections::HashMap;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::range::Range;
use std::sync::atomic::AtomicPtr;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use anyhow::Context;
use anyhow::Result;
use itertools::Itertools;
use neptune_privacy::api::export::Network;
use neptune_privacy::api::export::Tip5;
use neptune_privacy::api::export::Utxo;
use neptune_privacy::application::config::data_directory::DataDirectory;
use neptune_privacy::application::rest_server::ExportedBlock;
use neptune_privacy::prelude::tasm_lib::prelude::Digest;
use neptune_privacy::protocol::consensus::block::mutator_set_update::MutatorSetUpdate;
use neptune_privacy::protocol::proof_abstractions::mast_hash::MastHash;
use neptune_privacy::state::wallet::incoming_utxo::IncomingUtxo;
use neptune_privacy::state::wallet::wallet_entropy::WalletEntropy;
use neptune_privacy::util_types::mutator_set::mutator_set_accumulator::MutatorSetAccumulator;
use neptune_privacy::util_types::mutator_set::removal_record::absolute_index_set::AbsoluteIndexSet;
use pending::TransactionUpdater;
use rayon::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Pool;
use sqlx::Sqlite;
use tracing::*;
use wallet_file::wallet_dir_by_id;
use wallet_state_table::UtxoBlockInfo;
use wallet_state_table::UtxoDbData;

use crate::config::wallet::ScanConfig;
use crate::config::wallet::WalletConfig;
use crate::config::Config;

// mod archive_state;
pub mod balance;
pub mod fake_archival_state;
pub mod fork;
mod input;
pub use input::InputSelectionRule;
pub mod block_cache;
mod key_cache;
mod keys;
mod pending;
mod spend;
pub mod sync;
pub mod wallet_file;
mod wallet_state_table;

pub struct WalletState {
    key: WalletEntropy,
    scan_config: ScanConfig,
    pub network: Network,
    num_symmetric_keys: AtomicU64,
    num_generation_spending_keys: AtomicU64,
    num_future_keys: AtomicU64,
    pool: Pool<Sqlite>,
    updater: TransactionUpdater,
    know_raw_hash_keys: AtomicPtr<Vec<Digest>>,
    key_cache: key_cache::KeyCache,
    id: i64,
    spend_lock: tokio::sync::Mutex<()>,
}

impl WalletState {
    pub async fn new_from_config(config: &Config) -> Result<Self> {
        let wallet_config = config.get_current_wallet().await?;
        let database = Self::wallet_database_path(config, wallet_config.id).await?;
        Self::new(wallet_config, &database).await
    }

    pub async fn wallet_database_path(config: &Config, id: i64) -> Result<PathBuf> {
        let wallet_dir = Self::wallet_path(config, id).await?;
        DataDirectory::create_dir_if_not_exists(&wallet_dir).await?;
        Ok(wallet_dir.join("wallet_state.db"))
    }

    pub async fn wallet_path(config: &Config, id: i64) -> Result<PathBuf> {
        let data_dir = config.get_data_dir().await?;
        let network = config.get_network().await?;
        let wallet_dir = wallet_dir_by_id(&data_dir, network, id);
        Ok(wallet_dir)
    }

    pub async fn new(wallet_config: WalletConfig, database: &PathBuf) -> Result<Self> {
        #[allow(unused)]
        let pool = {
            let options = sqlx::sqlite::SqliteConnectOptions::new()
                .filename(database)
                .create_if_missing(true);

            sqlx::SqlitePool::connect_with(options)
                .await
                .map_err(|err| anyhow::anyhow!("Could not connect to database: {err}"))?
        };

        #[cfg(test)]
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await?;

        let num_future_keys = wallet_config.scan_config.num_keys;

        let updater = TransactionUpdater::new(pool.clone()).await?;

        let state = Self {
            key: wallet_config.key,
            scan_config: wallet_config.scan_config,
            network: wallet_config.network,
            num_symmetric_keys: AtomicU64::new(0),
            num_generation_spending_keys: AtomicU64::new(0),
            num_future_keys: AtomicU64::new(num_future_keys),
            pool: pool.clone(),
            updater,
            know_raw_hash_keys: AtomicPtr::new(null_mut()),
            key_cache: key_cache::KeyCache::new(),
            id: wallet_config.id,
            spend_lock: tokio::sync::Mutex::new(()),
        };

        state.migrate_tables().await.context("migrate_tables")?;
        state.num_generation_spending_keys.store(
            state.get_num_generation_spending_keys().await?,
            Ordering::Relaxed,
        );
        state
            .num_symmetric_keys
            .store(state.get_num_symmetric_keys().await?, Ordering::Relaxed);

        state
            .init_raw_hash_keys()
            .await
            .context("init_raw_hash_keys")?;

        debug!("Wallet state initialized");

        Ok(state)
    }

    pub async fn start_height(&self) -> Result<u64> {
        if let Some(tip) = self.get_tip().await? {
            return Ok(tip.0 + 1);
        }
        info!(
            "new sync, using scan_config height: {}",
            self.scan_config.start_height
        );
        Ok(self.scan_config.start_height)
    }

    pub async fn update_new_tip(
        &self,
        previous_mutator_set_accumulator: &MutatorSetAccumulator,
        block: &ExportedBlock,
        should_update: bool,
    ) -> Result<Option<u64>> {
        let mut msa_state = previous_mutator_set_accumulator.clone();
        let height: u64 = block.kernel.header.height.into();

        let mut tx = self.pool.begin().await?;

        let _spend_guard = self.spend_lock.lock().await;

        debug!("check fork");
        if let Some(fork_point) = self.check_fork(&block).await.context("check fork")? {
            info!(
                "reorganize_to_height: {} {}",
                fork_point.0,
                fork_point.1.to_hex()
            );
            self.reorganize_to_height(&mut *tx, fork_point.0, fork_point.1)
                .await
                .context("reorganize_to_height")?;
            tx.commit().await.context("commit db")?;
            return Ok(Some(fork_point.0));
        }
        debug!("update mutator set");

        let MutatorSetUpdate {
            additions: addition_records,
            removals: _,
        } = block.mutator_set_update();

        debug!("get removal_records");

        debug!("scan for incoming utxo");
        let incommings = self.par_scan_for_incoming_utxo(&block).await?;
        let mut recovery_datas = Vec::with_capacity(incommings.len());

        let incoming = incommings
            .into_iter()
            .map(|v| (v.addition_record(), v))
            .collect::<std::collections::HashMap<_, _>>();

        debug!("iterate addition records");
        let mut gusser_preimage = None;
        for addition_record in &addition_records {
            if let Some(incoming_utxo) = incoming.get(addition_record) {
                let r = incoming_utxo_recovery_data_from_incomming_utxo(
                    incoming_utxo.clone(),
                    &msa_state,
                );
                recovery_datas.push(r);

                if incoming_utxo.is_guesser_fee {
                    gusser_preimage = Some(incoming_utxo.receiver_preimage);
                }
            }

            msa_state.add(addition_record);
        }

        debug!("append utxos");
        let mut db_datas = vec![];
        for recovery_data in recovery_datas {
            let digest = Tip5::hash(&recovery_data.utxo);
            let db_data = UtxoDbData {
                id: 0,
                hash: digest.to_hex(),
                recovery_data,
                spent_in_block: None,
                confirmed_in_block: UtxoBlockInfo {
                    block_height: height,
                    block_digest: block.hash(),
                    timestamp: block.kernel.header.timestamp,
                },
                spent_height: None,
                confirm_height: height.try_into()?,
                confirmed_txid: None,
                spent_txid: None,
            };
            db_datas.push(db_data);
        }

        self.append_utxos(&mut *tx, db_datas).await?;

        if let Some(key) = gusser_preimage {
            debug!("add guesser preimage to raw hash keys");
            self.add_raw_hash_key(&mut *tx, key).await?;
        }

        debug!("scan for spent utxos");
        let spents = self.scan_for_spent_utxos(&block).await?;

        let block_info = UtxoBlockInfo {
            block_height: block.kernel.header.height.into(),
            block_digest: block.hash(),
            timestamp: block.kernel.header.timestamp,
        };

        let spent_updates = spents
            .iter()
            .map(|v| (v.2, block_info.clone()))
            .collect_vec();

        debug!("update spent utxos");
        self.update_spent_utxos(&mut *tx, spent_updates).await?;

        debug!("scan for expected utxos");
        // update expected utxo with txid
        let expected = self
            .scan_for_expected_utxos(block)
            .await?
            .into_iter()
            .map(|(recovery, txid)| {
                let digest = Tip5::hash(&recovery.utxo);
                (digest, txid)
            })
            .collect_vec();

        debug!("update utxos with expected utxos");
        self.update_utxos_with_expected_utxos(&mut *tx, expected, height.try_into()?)
            .await?;

        debug!(
            "set tip {} {}",
            block.kernel.header.height.value(),
            block.kernel.mast_hash().to_hex()
        );
        self.set_tip(&mut *tx, (block.kernel.header.height.into(), block.hash()))
            .await?;

        tx.commit().await?;

        self.clean_old_expected_utxos().await?;

        if should_update {
            self.updater.update_transactions(&self).await;
        }

        info!("sync finished: {}", height);
        Ok(None)
    }

    async fn par_scan_for_incoming_utxo(
        &self,
        block: &ExportedBlock,
    ) -> anyhow::Result<Vec<IncomingUtxo>> {
        let transaction = &block.kernel.body.transaction_kernel();

        let spendingkeys = self.get_future_generation_spending_keys(Range {
            start: 0,
            end: self.num_generation_spending_keys() + self.num_future_keys(),
        });

        let spend_to_spendingkeys = spendingkeys.par_iter().flat_map(|spendingkey| {
            let utxo = spendingkey.1.scan_for_announced_utxos(&transaction);
            if utxo.len() > 0 {
                self.num_generation_spending_keys
                    .fetch_max(spendingkey.0, Ordering::SeqCst);
            }
            utxo
        });

        self.set_num_generation_spending_keys(self.num_generation_spending_keys())
            .await?;

        let symmetric_keys = self.get_future_symmetric_keys(Range {
            start: 0,
            end: self.num_symmetric_keys() + self.num_future_keys(),
        });

        let spend_to_symmetrickeys = symmetric_keys.par_iter().flat_map(|spendingkey| {
            let utxo = spendingkey.1.scan_for_announced_utxos(&transaction);
            if utxo.len() > 0 {
                self.num_symmetric_keys
                    .fetch_max(spendingkey.0, Ordering::SeqCst);
            }
            utxo
        });

        self.set_num_symmetric_keys(self.num_symmetric_keys())
            .await?;

        let own_guesser_key = self.key.guesser_fee_key();
        let was_guessed_by_us = block
            .kernel
            .header
            .was_guessed_by(&own_guesser_key.to_address().into());

        let gusser_incoming_utxos = if was_guessed_by_us {
            let sender_randomness = block.hash();
            block
                .kernel
                .guesser_fee_utxos()
                .expect("Exported block must have guesser fee UTXOs")
                .into_iter()
                .map(|utxo| {
                    IncomingUtxo{
                        utxo,
                        sender_randomness,
                        receiver_preimage: own_guesser_key.receiver_preimage(),
                        is_guesser_fee: true,
                    }
                })
                .collect_vec()
        } else {
            vec![]
        };

        let receive = spend_to_spendingkeys
            .chain(spend_to_symmetrickeys)
            .chain(gusser_incoming_utxos)
            .collect::<Vec<_>>();

        Ok(receive)
    }

    /// Return a list of UTXOs spent by this wallet in the transaction
    ///
    /// Returns a list of tuples (utxo, absolute-index-set, index-into-database).
    async fn scan_for_spent_utxos(
        &self,
        block: &ExportedBlock,
    ) -> Result<Vec<(Utxo, AbsoluteIndexSet, i64)>> {
        let confirmed_absolute_index_sets = block
            .kernel
            .body
            .transaction_kernel()
            .inputs
            .iter()
            .map(|rr| rr.absolute_indices)
            .collect_vec();

        let monitored_utxos = self.get_unspent_utxos().await?;
        let mut spent_own_utxos = vec![];

        for monitored_utxo in monitored_utxos {
            let utxo: UtxoRecoveryData = monitored_utxo.recovery_data;

            if confirmed_absolute_index_sets.contains(&utxo.abs_i()) {
                spent_own_utxos.push((utxo.utxo.clone(), utxo.abs_i(), monitored_utxo.id));
            }
        }

        Ok(spent_own_utxos)
    }

    // returns IncomingUtxo and
    pub async fn scan_for_expected_utxos(
        &self,
        block: &ExportedBlock,
    ) -> Result<Vec<(IncomingUtxo, String)>> {
        let MutatorSetUpdate {
            additions: addition_records,
            removals: _removal_records,
        } = block.mutator_set_update();

        let expected_utxos = self.expected_utxos().await?;
        let eu_map: HashMap<_, _> = expected_utxos
            .into_iter()
            .map(|eu| (eu.expected_utxo.addition_record, eu))
            .collect();

        let incommings = addition_records
            .iter()
            .filter_map(move |a| {
                eu_map
                    .get(a)
                    .map(|eu| (IncomingUtxo::from(&eu.expected_utxo), eu.txid.to_owned()))
            })
            .collect_vec();
        Ok(incommings)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UtxoRecoveryData {
    pub utxo: Utxo,
    pub sender_randomness: Digest,
    pub receiver_preimage: Digest,
    pub aocl_index: u64,
}

impl UtxoRecoveryData {
    pub fn abs_i(&self) -> AbsoluteIndexSet {
        let utxo_digest = Tip5::hash(&self.utxo);

        AbsoluteIndexSet::compute(
            utxo_digest,
            self.sender_randomness,
            self.receiver_preimage,
            self.aocl_index,
        )
    }
}

fn incoming_utxo_recovery_data_from_incomming_utxo(
    utxo: IncomingUtxo,
    msa_state: &MutatorSetAccumulator,
) -> UtxoRecoveryData {
    let utxo_digest = Tip5::hash(&utxo.utxo);
    let new_own_membership_proof = msa_state.prove(
        utxo_digest,
        utxo.sender_randomness,
        utxo.receiver_preimage,
    );

    let aocl_index = new_own_membership_proof.aocl_leaf_index;

    UtxoRecoveryData {
        utxo: utxo.utxo.to_owned(),
        sender_randomness: utxo.sender_randomness,
        receiver_preimage: utxo.receiver_preimage,
        aocl_index,
    }
}

impl Drop for WalletState {
    fn drop(&mut self) {
        let ptr = self.know_raw_hash_keys.load(Ordering::Acquire);
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }
}
