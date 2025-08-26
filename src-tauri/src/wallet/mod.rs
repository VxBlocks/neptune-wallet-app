use std::{
    collections::HashMap,
    path::PathBuf,
    ptr::null_mut,
    range::Range,
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};

use anyhow::{Context, Result};
use itertools::Itertools;
use neptune_cash::{
    config_models::{data_directory::DataDirectory, network::Network},
    models::{
        blockchain::{
            block::{mutator_set_update::MutatorSetUpdate, Block},
            shared::Hash,
            transaction::utxo::Utxo,
        },
        proof_abstractions::mast_hash::MastHash,
        state::wallet::{incoming_utxo::IncomingUtxo, wallet_entropy::WalletEntropy},
    },
    prelude::tasm_lib::prelude::Digest,
    util_types::mutator_set::{
        mutator_set_accumulator::MutatorSetAccumulator,
        removal_record::{AbsoluteIndexSet, RemovalRecord},
    },
};
use pending::TransactionUpdater;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tracing::*;
use wallet_file::wallet_dir_by_id;
use wallet_state_table::{UtxoBlockInfo, UtxoDbData};

use crate::config::{
    wallet::{ScanConfig, WalletConfig},
    Config,
};

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
        block: &Block,
        should_update: bool,
    ) -> Result<Option<u64>> {
        let mut msa_state = previous_mutator_set_accumulator.clone();
        let height: u64 = block.header().height.into();

        let mut tx = self.pool.begin().await?;

        let _spend_guard = self.spend_lock.lock().await;

        debug!("check fork");
        // if let Some(fork_point) = self.check_fork(&block).await.context("check fork")? {
        //     info!(
        //         "reorganize_to_height: {} {}",
        //         fork_point.0,
        //         fork_point.1.to_hex()
        //     );
        //     self.reorganize_to_height(&mut *tx, fork_point.0, fork_point.1)
        //         .await
        //         .context("reorganize_to_height")?;
        //     tx.commit().await.context("commit db")?;
        //     return Ok(Some(fork_point.0));
        // }
        debug!("update mutator set");

        let MutatorSetUpdate {
            additions: addition_records,
            removals: removal_records,
        } = block.mutator_set_update()?;

        debug!("get removal_records");
        let mut removal_records = removal_records;
        removal_records.reverse();
        let mut removal_records: Vec<&mut RemovalRecord> =
            removal_records.iter_mut().collect::<Vec<_>>();

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
            RemovalRecord::batch_update_from_addition(&mut removal_records, &msa_state);

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

        debug!("iterate removal records");
        while let Some(removal_record) = removal_records.pop() {
            RemovalRecord::batch_update_from_remove(&mut removal_records, removal_record);
            msa_state.remove(removal_record);
        }

        debug!("append utxos");
        let mut db_datas = vec![];
        for recovery_data in recovery_datas {
            let digest = Hash::hash(&recovery_data.utxo);
            let db_data = UtxoDbData {
                id: 0,
                hash: digest.to_hex(),
                recovery_data,
                spent_in_block: None,
                confirmed_in_block: UtxoBlockInfo {
                    block_height: height,
                    block_digest: block.hash(),
                    timestamp: block.header().timestamp,
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
            block_height: block.header().height.into(),
            block_digest: block.hash(),
            timestamp: block.header().timestamp,
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
                let digest = Hash::hash(&recovery.utxo);
                (digest, txid)
            })
            .collect_vec();

        debug!("update utxos with expected utxos");
        self.update_utxos_with_expected_utxos(&mut *tx, expected, height.try_into()?)
            .await?;

        debug!(
            "set tip {} {}",
            block.header().height.value(),
            block.kernel.mast_hash().to_hex()
        );
        self.set_tip(&mut *tx, (block.header().height.into(), block.hash()))
            .await?;

        tx.commit().await?;

        self.clean_old_expected_utxos().await?;

        if should_update {
            self.updater.update_transactions(&self).await;
        }

        info!("sync finished: {}", height);
        Ok(None)
    }

    #[allow(unused)]
    async fn scan_for_incoming_utxo(&self, block: &Block) -> anyhow::Result<Vec<IncomingUtxo>> {
        let transactions = &block.body().transaction_kernel();

        let mut utxos: Vec<IncomingUtxo> = Vec::new();
        let spendingkeys = self.get_future_generation_spending_keys(Range {
            start: 0,
            end: self.num_generation_spending_keys() + self.num_future_keys(),
        });

        let mut max_spending_key = 0u64;
        spendingkeys.iter().for_each(|spendingkey| {
            let utxo = spendingkey.1.scan_for_announced_utxos(&transactions);

            for utxo in utxo {
                utxos.push(utxo);
                max_spending_key = max_spending_key.max(spendingkey.0);
            }
        });

        let symmetric_keys = self.get_future_symmetric_keys(Range {
            start: 0,
            end: self.num_symmetric_keys() + self.num_future_keys(),
        });

        let mut max_symmetric_key = 0u64;
        symmetric_keys.iter().for_each(|spendingkey| {
            let utxo = spendingkey.1.scan_for_announced_utxos(&transactions);
            for utxo in utxo {
                utxos.push(utxo);
                max_symmetric_key = max_symmetric_key.max(spendingkey.0);
            }
        });

        if self.num_symmetric_keys.load(Ordering::Relaxed) < max_symmetric_key {
            self.set_num_symmetric_keys(max_symmetric_key).await?;
        }

        if self.num_generation_spending_keys.load(Ordering::Relaxed) < max_spending_key {
            self.set_num_generation_spending_keys(max_spending_key)
                .await?;
        }

        let receiver_preimage = self.key.prover_fee_address().privacy_digest();
        let gusser_incoming_utxos =
            if block.header().guesser_receiver_data.receiver_digest == receiver_preimage {
                let sender_randomness = block.hash();
                block
                    .guesser_fee_utxos()?
                    .into_iter()
                    .map(|utxo| IncomingUtxo {
                        utxo,
                        sender_randomness,
                        receiver_preimage,
                        is_guesser_fee: true,
                    })
                    .collect_vec()
            } else {
                vec![]
            };

        utxos.extend(gusser_incoming_utxos);

        Ok(utxos)
    }

    async fn par_scan_for_incoming_utxo(&self, block: &Block) -> anyhow::Result<Vec<IncomingUtxo>> {
        let transactions = &block.body().transaction_kernel();

        let spendingkeys = self.get_future_generation_spending_keys(Range {
            start: 0,
            end: self.num_generation_spending_keys() + self.num_future_keys(),
        });

        let spend_to_spendingkeys = spendingkeys.par_iter().flat_map(|spendingkey| {
            let utxo = spendingkey.1.scan_for_announced_utxos(&transactions);
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
            let utxo = spendingkey.1.scan_for_announced_utxos(&transactions);
            if utxo.len() > 0 {
                self.num_symmetric_keys
                    .fetch_max(spendingkey.0, Ordering::SeqCst);
            }
            utxo
        });

        self.set_num_symmetric_keys(self.num_symmetric_keys())
            .await?;

        let receiver_preimage = self.key.prover_fee_address();
        let receiver_preimage = receiver_preimage.privacy_digest();
        let gusser_incoming_utxos =
            if block.header().guesser_receiver_data.receiver_digest == receiver_preimage {
                let sender_randomness = block.hash();
                block
                    .guesser_fee_utxos()?
                    .into_iter()
                    .map(|utxo| IncomingUtxo {
                        utxo,
                        sender_randomness,
                        receiver_preimage,
                        is_guesser_fee: true,
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
        block: &Block,
    ) -> Result<Vec<(Utxo, AbsoluteIndexSet, i64)>> {
        let confirmed_absolute_index_sets = block
            .body()
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
        block: &Block,
    ) -> Result<Vec<(IncomingUtxo, String)>> {
        let MutatorSetUpdate {
            additions: addition_records,
            removals: _removal_records,
        } = block.mutator_set_update()?;

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
        let utxo_digest = Hash::hash(&self.utxo);

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
    let utxo_digest = Hash::hash(&utxo.utxo);
    let new_own_membership_proof =
        msa_state.prove(utxo_digest, utxo.sender_randomness, utxo.receiver_preimage);

    let aocl_index = new_own_membership_proof.aocl_leaf_index;

    UtxoRecoveryData {
        utxo: utxo.utxo,
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
