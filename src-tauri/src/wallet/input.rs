use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use neptune_cash::api::export::NativeCurrencyAmount;
use neptune_cash::api::export::ReceivingAddress;
use neptune_cash::api::export::SpendingKey;
use neptune_cash::api::export::Timestamp;
use neptune_cash::api::export::Tip5;
use neptune_cash::api::export::Utxo;
use neptune_cash::prelude::tasm_lib::prelude::Digest;
use neptune_cash::state::wallet::unlocked_utxo::UnlockedUtxo;
use neptune_cash::util_types::mutator_set::archival_mutator_set::RequestMsMembershipProofEx;
use neptune_cash::util_types::mutator_set::ms_membership_proof::MsMembershipProof;
use neptune_cash::util_types::mutator_set::removal_record::absolute_index_set::AbsoluteIndexSet;
use rand::seq::SliceRandom;

use super::wallet_state_table::UtxoDbData;
use super::UtxoRecoveryData;
use crate::rpc_client;

pub enum InputSelectionRule {
    Minimum,
    Maximum,
    Oldest,
    Newest,
    Random,
}

impl Default for InputSelectionRule {
    fn default() -> Self {
        InputSelectionRule::Oldest
    }
}

impl InputSelectionRule {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "minimum" => Some(InputSelectionRule::Minimum),
            "maximum" => Some(InputSelectionRule::Maximum),
            "oldest" => Some(InputSelectionRule::Oldest),
            "newest" => Some(InputSelectionRule::Newest),
            "random" => Some(InputSelectionRule::Random),
            _ => None,
        }
    }
    pub fn apply(&self, mut utxos: Vec<UtxoDbData>) -> Vec<UtxoDbData> {
        match self {
            InputSelectionRule::Minimum => utxos.sort_by(|a, b| {
                a.recovery_data
                    .utxo
                    .get_native_currency_amount()
                    .cmp(&b.recovery_data.utxo.get_native_currency_amount())
            }),
            InputSelectionRule::Maximum => utxos.sort_by(|a, b| {
                b.recovery_data
                    .utxo
                    .get_native_currency_amount()
                    .cmp(&a.recovery_data.utxo.get_native_currency_amount())
            }),
            InputSelectionRule::Oldest => {
                utxos.sort_by(|a, b| a.confirm_height.cmp(&b.confirm_height))
            }
            InputSelectionRule::Newest => {
                utxos.sort_by(|a, b| b.confirm_height.cmp(&a.confirm_height))
            }
            InputSelectionRule::Random => utxos.shuffle(&mut rand::rng()),
        };
        utxos
    }
}

impl super::WalletState {
    pub async fn create_input(
        &self,
        outputs: &[(ReceivingAddress, NativeCurrencyAmount)],
        fee: NativeCurrencyAmount,
        rule: InputSelectionRule,
        must_include_inputs: Vec<i64>,
    ) -> anyhow::Result<(Vec<UnlockedUtxo>, Vec<i64>, Digest)> {
        let mut utxos = self.get_unspent_utxos().await?;

        let pending_utxos = self.updater.get_pending_spent_utxos().await?;
        utxos.retain(|utxo| !pending_utxos.contains(&utxo.id));

        let utxos = rule.apply(utxos);
        let unspent = utxos
            .into_iter()
            .filter(|utxo| !must_include_inputs.contains(&utxo.id));

        let total_amount = outputs
            .iter()
            .map(|(_, amount)| amount.to_nau())
            .sum::<i128>()
            + fee.to_nau();

        let inputs = self
            .get_unspent_inputs_with_ids(&must_include_inputs)
            .await?;

        let mut inputs = inputs
            .into_iter()
            .map(|input| input.recovery_data)
            .collect::<Vec<_>>();

        let mut total_input_amount = inputs
            .iter()
            .map(|input| input.utxo.get_native_currency_amount().to_nau())
            .sum::<i128>();

        let now = Timestamp::now();
        let mut db_idxs = must_include_inputs.clone();
        for utxo in unspent {
            let recovery_data = utxo.recovery_data;
            if total_input_amount >= total_amount {
                break;
            }

            if let Some(release) = recovery_data.utxo.release_date() {
                if release > now {
                    continue;
                }
            }

            total_input_amount += recovery_data.utxo.get_native_currency_amount().to_nau();
            inputs.push(recovery_data);
            db_idxs.push(utxo.id);
        }

        let (inputs, tip_digest) = self.unlock_utxos(inputs).await?;

        ensure!(
            inputs.len() == db_idxs.len(),
            "Inputs and db_idxs must have the same length"
        );

        Ok((inputs, db_idxs, tip_digest))
    }

    pub async fn unlock_utxos(
        &self,
        utxos: Vec<UtxoRecoveryData>,
    ) -> anyhow::Result<(Vec<UnlockedUtxo>, Digest)> {
        let mut rpc_params = Vec::with_capacity(utxos.len());

        for utxo in &utxos {
            let item = Tip5::hash(&utxo.utxo);
            let swbf_indices = AbsoluteIndexSet::compute(
                item,
                utxo.sender_randomness,
                utxo.receiver_preimage,
                utxo.aocl_index,
            );
            let aocl_leaf_index = utxo.aocl_index;

            let param = RequestMsMembershipProofEx {
                swbf_indices: swbf_indices.to_vec(),
                aocl_leaf_index,
            };
            rpc_params.push(param);
        }

        let proofs = rpc_client::node_rpc_client()
            .restore_msmp(rpc_params)
            .await?;

        let mut unlocked = Vec::with_capacity(utxos.len());
        for (proof, utxo) in proofs.proofs.into_iter().zip(utxos) {
            let spending_key = self
                .find_spending_key_for_utxo(&utxo.utxo)
                .context("No spending key found for utxo")?;

            let membership_proof = MsMembershipProof {
                sender_randomness: utxo.sender_randomness,
                receiver_preimage: utxo.receiver_preimage,
                auth_path_aocl: proof.auth_path_aocl,
                aocl_leaf_index: utxo.aocl_index,
                target_chunks: proof.target_chunks,
            };

            unlocked.push(UnlockedUtxo::unlock(
                utxo.utxo,
                spending_key.lock_script_and_witness(),
                membership_proof,
            ));
        }

        Ok((unlocked, proofs.block_id))
    }

    // returns Some(SpendingKey) if the utxo can be unlocked by one of the known
    // wallet keys.
    pub fn find_spending_key_for_utxo(&self, utxo: &Utxo) -> Option<SpendingKey> {
        self.get_known_spending_keys()
            .into_iter()
            .find(|k| k.lock_script_hash() == utxo.lock_script_hash())
    }

    pub async fn get_recovery_data_from_utxo(&self, utxo: &Utxo) -> Result<UtxoRecoveryData> {
        let digest = Tip5::hash(utxo);
        let db_data = self.get_utxo_db_data(&digest).await?;
        match db_data {
            Some(db_data) => Ok(db_data.recovery_data),
            None => Err(anyhow::anyhow!("UTXO not found")),
        }
    }
}
