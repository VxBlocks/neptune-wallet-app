use itertools::Itertools;
use neptune_privacy::api::export::Timestamp;
use neptune_privacy::api::export::TransactionDetails;
use neptune_privacy::api::export::TransactionProof;
use neptune_privacy::api::export::TxProvingCapability;
use neptune_privacy::prelude::tasm_lib::prelude::Digest;
use neptune_privacy::protocol::consensus::block::block_height::BlockHeight;
use neptune_privacy::protocol::consensus::transaction::primitive_witness::PrimitiveWitness;
use neptune_privacy::protocol::consensus::transaction::utxo::Utxo;
use neptune_privacy::protocol::consensus::transaction::Transaction;
use neptune_privacy::protocol::consensus::type_scripts::native_currency_amount::NativeCurrencyAmount;
use neptune_privacy::state::wallet::address::ReceivingAddress;
use neptune_privacy::state::wallet::address::SpendingKey;
use neptune_privacy::state::wallet::expected_utxo::ExpectedUtxo;
use neptune_privacy::state::wallet::expected_utxo::UtxoNotifier;
use neptune_privacy::state::wallet::transaction_output::TxOutput;
use neptune_privacy::state::wallet::transaction_output::TxOutputList;
use neptune_privacy::state::wallet::unlocked_utxo::UnlockedUtxo;
use neptune_privacy::state::wallet::utxo_notification::UtxoNotificationMedium;
use neptune_privacy::state::wallet::utxo_notification::UtxoNotificationMethod;
use neptune_privacy::util_types::mutator_set::mutator_set_accumulator::MutatorSetAccumulator;
use num_traits::CheckedSub;
use thiserror::Error;
use tracing::*;

use super::input::InputSelectionRule;
use crate::prover::ProofBuilder;
use crate::rpc_client;
use crate::rpc_client::BroadcastError;
use crate::wallet::wallet_state_table::ExpectedUtxoData;

impl super::WalletState {
    pub async fn send_to_address(
        &self,
        outputs: Vec<(ReceivingAddress, NativeCurrencyAmount)>,
        utxo_notification_media: (UtxoNotificationMedium, UtxoNotificationMedium),
        fee: NativeCurrencyAmount,
        rule: InputSelectionRule,
        must_include_utxos: Vec<i64>,
    ) -> anyhow::Result<Transaction, SendError> {
        let _spend_guard = self.spend_lock.lock().await;
        let now = Timestamp::now();
        let tx_proving_capability = TxProvingCapability::ProofCollection;

        let (owned_utxo_notification_medium, unowned_utxo_notification_medium) =
            utxo_notification_media;

        let _ = crate::service::app::emit_event_to(
            "main",
            "send_state",
            "stmi: step 1. get change key.",
        );

        // obtain next unused symmetric key for change utxo
        let change_key = {
            let symmetric_key = self.key.nth_symmetric_key(0);
            let spending_key = SpendingKey::Symmetric(symmetric_key);
            // self.set_num_symmetric_keys(self.num_symmetric_keys() + 1)
            //     .await?;
            spending_key
        };

        let _ = crate::service::app::emit_event_to(
            "main",
            "send_state",
            "stmi: step 2. generate outputs.",
        );

        let (tx_inputs, db_ids, tip_msa, tip_height) = self
            .create_input(&outputs, fee, rule, must_include_utxos)
            .await?;

        let tx_outputs = self
            .generate_tx_outputs(
                outputs.clone(),
                owned_utxo_notification_medium,
                unowned_utxo_notification_medium,
                tip_height,
            )
            .await;

        let _ =
            crate::service::app::emit_event_to("main", "send_state", "stmi: step 3. create tx.");

        // NOTE: A change output will be added to tx_outputs if needed.
        let (transaction, transaction_details, maybe_change_output) = match self
            .create_transaction_with_prover_capability(
                tx_outputs.clone(),
                tx_inputs,
                change_key,
                owned_utxo_notification_medium,
                fee,
                now,
                tx_proving_capability,
                tip_msa,
                tip_height,
            )
            .await
        {
            Ok(tx) => tx,
            Err(e) => {
                tracing::error!("Could not create transaction: {}", e);
                return Err(e.into());
            }
        };

        let _ = crate::service::app::emit_event_to(
            "main",
            "send_state",
            "stmi: step 4. extract expected utxos.",
        );

        let mut full_outputs = tx_outputs;
        if let Some(change_output) = maybe_change_output {
            full_outputs.push(change_output);
        }

        let utxos_sent_to_self = self.extract_expected_utxos(&full_outputs, UtxoNotifier::Myself);

        let _ = crate::service::app::emit_event_to(
            "main",
            "send_state",
            "stmi: step 5. broadcast transaction.",
        );

        let txid = rpc_client::node_rpc_client()
            .broadcast_transaction(&transaction)
            .await?;

        let _ = crate::service::app::emit_event_to(
            "main",
            "send_state",
            "stmi: step 5. save to updater.",
        );

        let expected_utxo_data = utxos_sent_to_self
            .into_iter()
            .map(|expected_utxo| ExpectedUtxoData {
                id: 0,
                txid: txid.clone(),
                expected_utxo,
                timestamp: now,
            })
            .collect();
        self.add_expected_utxo(expected_utxo_data).await?;

        self.updater
            .add_transaction(txid.clone(), transaction_details, db_ids)
            .await?;

        Ok(transaction)
    }

    pub async fn generate_tx_outputs(
        &self,
        outputs: impl IntoIterator<Item = (ReceivingAddress, NativeCurrencyAmount)>,
        owned_utxo_notify_medium: UtxoNotificationMedium,
        unowned_utxo_notify_medium: UtxoNotificationMedium,
        block_height: BlockHeight,
    ) -> TxOutputList {
        // Convert outputs.  [address:amount] --> TxOutputList
        let tx_outputs: Vec<_> = outputs
            .into_iter()
            .map(|(address, amount)| {
                let sender_randomness = self
                    .key
                    .generate_sender_randomness(block_height, address.privacy_digest());

                // The UtxoNotifyMethod (Onchain or Offchain) is auto-detected
                // based on whether the address belongs to our wallet or not
                self.auto_outputs(
                    address,
                    amount,
                    sender_randomness,
                    owned_utxo_notify_medium,
                    unowned_utxo_notify_medium,
                )
            })
            .collect();

        tx_outputs.into()
    }

    pub fn can_unlock(&self, utxo: &Utxo) -> bool {
        self.get_known_spending_keys()
            .iter()
            .find(|k| k.lock_script_hash() == utxo.lock_script_hash())
            .is_some()
    }

    pub fn auto_outputs(
        &self,
        address: ReceivingAddress,
        amount: NativeCurrencyAmount,
        sender_randomness: Digest,
        owned_utxo_notify_medium: UtxoNotificationMedium,
        unowned_utxo_notify_medium: UtxoNotificationMedium,
    ) -> TxOutput {
        let utxo = Utxo::new_native_currency(address.lock_script_hash(), amount);

        let has_matching_spending_key = self.can_unlock(&utxo);

        let receiver_digest = address.privacy_digest();
        let notification_method = if has_matching_spending_key {
            match owned_utxo_notify_medium {
                UtxoNotificationMedium::OnChain => UtxoNotificationMethod::OnChain(address),
                UtxoNotificationMedium::OffChain => UtxoNotificationMethod::OffChain(address),
            }
        } else {
            match unowned_utxo_notify_medium {
                UtxoNotificationMedium::OnChain => UtxoNotificationMethod::OnChain(address),
                UtxoNotificationMedium::OffChain => UtxoNotificationMethod::OffChain(address),
            }
        };

        TxOutput::new(
            utxo,
            sender_randomness,
            receiver_digest,
            notification_method,
            has_matching_spending_key,
            false,
        )
    }

    /// Variant of [Self::create_transaction] that allows caller to specify
    /// prover capability. [Self::create_transaction] is the preferred interface
    /// for anything but tests.
    #[expect(clippy::too_many_arguments)]
    pub(crate) async fn create_transaction_with_prover_capability(
        &self,
        mut tx_outputs: TxOutputList,
        tx_inputs: Vec<UnlockedUtxo>,
        change_key: SpendingKey,
        change_utxo_notify_medium: UtxoNotificationMedium,
        fee: NativeCurrencyAmount,
        timestamp: Timestamp,
        prover_capability: TxProvingCapability,
        tip_msa: MutatorSetAccumulator,
        tip_height: BlockHeight,
    ) -> anyhow::Result<(Transaction, TransactionDetails, Option<TxOutput>)> {
        // 1. create/add change output if necessary.
        let total_spend = tx_outputs.total_native_coins() + fee;

        let total_spendable = tx_inputs
            .iter()
            .map(|x| x.utxo.get_native_currency_amount())
            .sum();

        // Add change, if required to balance tx.
        let mut maybe_change_output = None;
        if total_spend < total_spendable {
            let amount = total_spendable.checked_sub(&total_spend).ok_or_else(|| {
                anyhow::anyhow!("overflow subtracting total_spend from input_amount")
            })?;

            let change_utxo = self
                .create_change_output(amount, change_key, change_utxo_notify_medium, tip_height)
                .await?;
            tx_outputs.push(change_utxo.clone());
            maybe_change_output = Some(change_utxo);
        }

        let transaction_details = TransactionDetails::new_without_coinbase(
            tx_inputs,
            tx_outputs.to_owned(),
            fee,
            timestamp,
            tip_msa,
            self.network,
        );

        // note: if this task is cancelled, the proving job will continue
        // because TritonVmJobOptions::cancel_job_rx is None.
        // see how compose_task handles cancellation in mine_loop.

        // 2. Create the transaction
        let transaction = self
            .create_raw_transaction(&transaction_details, prover_capability)
            .await?;

        Ok((transaction, transaction_details, maybe_change_output))
    }

    /// Generate a change UTXO to ensure that the difference in input amount
    /// and output amount goes back to us. Return the UTXO in a format compatible
    /// with claiming it later on.
    //
    // "Later on" meaning: as an [ExpectedUtxo].
    pub async fn create_change_output(
        &self,
        change_amount: NativeCurrencyAmount,
        change_key: SpendingKey,
        change_utxo_notify_method: UtxoNotificationMedium,
        tip_height: BlockHeight,
    ) -> anyhow::Result<TxOutput> {
        let own_receiving_address = change_key.to_address();

        let receiver_digest = own_receiving_address.privacy_digest();
        let change_sender_randomness = {
            self.key
                .generate_sender_randomness(tip_height, receiver_digest)
        };

        let owned = true;
        let change_output = match change_utxo_notify_method {
            UtxoNotificationMedium::OnChain => TxOutput::onchain_native_currency(
                change_amount,
                change_sender_randomness,
                own_receiving_address,
                owned,
            ),
            UtxoNotificationMedium::OffChain => TxOutput::offchain_native_currency(
                change_amount,
                change_sender_randomness,
                own_receiving_address,
                owned,
            ),
        };

        Ok(change_output)
    }

    /// creates a Transaction.
    ///
    /// This API provides the caller complete control over selection of inputs
    /// and outputs.  When fine grained control is not required,
    /// [Self::create_transaction()] is easier to use and should be preferred.
    ///
    /// It is the caller's responsibility to provide inputs and outputs such
    /// that sum(inputs) == sum(outputs) + fee.  Else an error will result.
    ///
    /// Note that this means the caller must calculate the `change` amount if any
    /// and provide an output for the change.
    ///
    /// The `tx_outputs` parameter should normally be generated with
    /// [Self::generate_tx_outputs()] which determines which outputs should be
    /// notified `OnChain` or `OffChain`.
    ///
    /// After this call returns, it is the caller's responsibility to inform the
    /// wallet of any returned [ExpectedUtxo] for utxos that match wallet keys.
    /// Failure to do so can result in loss of funds!
    ///
    /// Note that `create_raw_transaction()` does not modify any state and does
    /// not require acquiring write lock.  This is important because internally
    /// it calls prove() which is a very lengthy operation.
    ///
    /// Example:
    ///
    /// See the implementation of [Self::create_transaction()].
    pub(crate) async fn create_raw_transaction(
        &self,
        transaction_details: &TransactionDetails,
        proving_power: TxProvingCapability,
    ) -> anyhow::Result<Transaction> {
        // note: this executes the prover which can take a very
        //       long time, perhaps minutes.  The `await` here, should avoid
        //       block the tokio executor and other async tasks.
        Self::create_transaction_from_data_worker(transaction_details, proving_power).await
    }

    // note: this executes the prover which can take a very
    //       long time, perhaps minutes. It should never be
    //       called directly.
    //       Use create_transaction_from_data() instead.
    //
    async fn create_transaction_from_data_worker(
        transaction_details: &TransactionDetails,
        proving_power: TxProvingCapability,
    ) -> anyhow::Result<Transaction> {
        let primitive_witness = PrimitiveWitness::from_transaction_details(transaction_details);

        debug!("primitive witness for transaction: {}", primitive_witness);

        info!(
            "Start: generate proof for {}-in {}-out transaction",
            primitive_witness.input_utxos.utxos.len(),
            primitive_witness.output_utxos.utxos.len()
        );
        let kernel = primitive_witness.kernel.clone();
        let proof = match proving_power {
            TxProvingCapability::PrimitiveWitness => TransactionProof::Witness(primitive_witness),
            TxProvingCapability::LockScript => todo!(),
            TxProvingCapability::ProofCollection => {
                let collection = tokio::task::spawn_blocking(move || {
                    ProofBuilder::produce_proof_collection(&primitive_witness)
                })
                .await??;

                TransactionProof::ProofCollection(collection)
            }
            TxProvingCapability::SingleProof => todo!(),
        };

        Ok(Transaction { kernel, proof })
    }

    /// Extract `ExpectedUtxo`s from the `TxOutputList` that require off-chain
    /// notifications and that are destined for this wallet.
    pub fn extract_expected_utxos(
        &self,
        tx_outputs: &TxOutputList,
        notifier: UtxoNotifier,
    ) -> Vec<ExpectedUtxo> {
        tx_outputs
            .iter()
            .filter(|txo| txo.is_offchain())
            .filter_map(|txo| {
                self.find_spending_key_for_utxo(&txo.utxo())
                    .map(|sk| (txo, sk))
            })
            .map(|(tx_output, spending_key)| {
                ExpectedUtxo::new(
                    tx_output.utxo(),
                    tx_output.sender_randomness(),
                    spending_key.privacy_preimage(),
                    notifier,
                )
            })
            .collect_vec()
    }
}

#[derive(Debug, Error)]
pub enum SendError {
    #[error(transparent)]
    Proof(#[from] anyhow::Error),
    #[error(transparent)]
    Broadcast(#[from] BroadcastError),
}
