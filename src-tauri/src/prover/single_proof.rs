use anyhow::anyhow;
use anyhow::bail;
use anyhow::ensure;
use anyhow::Context;
use anyhow::Result;
use neptune_privacy::api::export::Digest;
use neptune_privacy::api::export::NativeCurrencyAmount;
use neptune_privacy::api::export::NeptuneProof;
use neptune_privacy::api::export::Network;
use neptune_privacy::api::export::SpendingKey;
use neptune_privacy::api::export::Timestamp;
use neptune_privacy::api::export::Transaction;
use neptune_privacy::api::export::TransactionDetails;
use neptune_privacy::api::export::TransactionProof;
use neptune_privacy::prelude::twenty_first::util_types::mmr::mmr_successor_proof::MmrSuccessorProof;
use neptune_privacy::protocol::consensus::block::mutator_set_update::MutatorSetUpdate;
use neptune_privacy::protocol::consensus::block::Block;
use neptune_privacy::protocol::consensus::transaction::transaction_kernel::TransactionKernelModifier;
use neptune_privacy::protocol::consensus::transaction::validity::single_proof::SingleProof;
use neptune_privacy::protocol::consensus::transaction::validity::single_proof::SingleProofWitness;
use neptune_privacy::protocol::consensus::transaction::validity::tasm::single_proof::merge_branch::MergeWitness;
use neptune_privacy::protocol::consensus::transaction::validity::tasm::single_proof::update_branch::UpdateWitness;
use neptune_privacy::protocol::proof_abstractions::tasm::program::ConsensusProgram;
use neptune_privacy::protocol::proof_abstractions::SecretWitness;
use neptune_privacy::state::wallet::utxo_notification::UtxoNotificationMethod;
use neptune_privacy::state::wallet::wallet_entropy::WalletEntropy;
use neptune_privacy::util_types::mutator_set::mutator_set_accumulator::MutatorSetAccumulator;
use rand::rng;
use rand::Rng;
use tracing::info;

impl super::ProofBuilder {
    pub fn upgrade_proof(
        &self,
        transaction: Transaction,
        block: &Block,
        own_wallet_entropy: Option<WalletEntropy>,
    ) -> Result<Transaction> {
        let proof_collection = match &transaction.proof {
            TransactionProof::Witness(primitive_witness) => {
                let proof_collection = Self::produce_proof_collection(&primitive_witness)?;
                proof_collection
            }
            TransactionProof::SingleProof(_) => bail!("single proof do not need upgrade"),
            TransactionProof::ProofCollection(proof_collection) => proof_collection.clone(),
        };

        let single_proof_witness = SingleProofWitness::from_collection(proof_collection);
        let proof = Self::single_proof_from_witness(&single_proof_witness)?;

        let new_tx = Transaction {
            kernel: transaction.kernel.clone(),
            proof: TransactionProof::SingleProof(proof),
        };

        if let Some(fee) = self.gobble_fee {
            let gobble_tx = self.build_gobbler(
                fee,
                &own_wallet_entropy.context("gobbler requires wallet_entropy")?,
                block,
                &transaction,
                self.network,
            )?;

            return self.merge_single_proof(gobble_tx, new_tx);
        }

        Ok(new_tx)
    }

    pub fn update_single_proof(
        &self,
        tx: Transaction,
        previous_mutator_set_accumulator: &MutatorSetAccumulator,
        mutator_set_update: MutatorSetUpdate,
    ) -> Result<Transaction> {
        let old_transaction_kernel = tx.kernel;
        let old_single_proof = match tx.proof {
            TransactionProof::SingleProof(proof) => proof,
            _ => return Err(anyhow!("No single proof found")),
        };

        let new_timestamp: Option<Timestamp> = Option::None;

        ensure!(
            old_transaction_kernel.mutator_set_hash == previous_mutator_set_accumulator.hash(),
            "Old transaction kernel's mutator set hash does not agree \
                with supplied mutator set accumulator."
        );

        // apply mutator set update to get new mutator set accumulator
        let addition_records = mutator_set_update.additions.clone();
        let mut calculated_new_mutator_set = previous_mutator_set_accumulator.clone();
        let mut new_inputs = old_transaction_kernel.inputs.clone();
        mutator_set_update
            .apply_to_accumulator_and_records(
                &mut calculated_new_mutator_set,
                &mut new_inputs.iter_mut().collect::<Vec<_>>(),
                &mut [],
            )
            .unwrap_or_else(|_| panic!("Could not apply mutator set update."));

        let aocl_successor_proof = MmrSuccessorProof::new_from_batch_append(
            &previous_mutator_set_accumulator.aocl,
            &addition_records
                .iter()
                .map(|addition_record| addition_record.canonical_commitment)
                .collect::<Vec<_>>(),
        );

        // compute new kernel
        let mut modifier = TransactionKernelModifier::default()
            .inputs(new_inputs)
            .mutator_set_hash(calculated_new_mutator_set.hash());
        if let Some(new_timestamp) = new_timestamp {
            modifier = modifier.timestamp(new_timestamp);
        }
        let new_kernel = modifier.clone_modify(&old_transaction_kernel);

        // compute updated proof through recursion
        let update_witness = UpdateWitness::from_old_transaction(
            old_transaction_kernel,
            old_single_proof,
            previous_mutator_set_accumulator.clone(),
            new_kernel.clone(),
            calculated_new_mutator_set,
            aocl_successor_proof,
        );
        // let update_claim = update_witness.claim();
        // let update_nondeterminism = update_witness.nondeterminism();
        // info!("updating transaction; starting update proof ...");
        // let update_proof = Update
        //     .prove(
        //         update_claim,
        //         update_nondeterminism,
        //         triton_vm_job_queue,
        //         proof_job_options,
        //     )
        //     .await?;
        // info!("done.");

        let new_single_proof_witness = SingleProofWitness::from_update(update_witness);

        info!("starting single proof via update ...");
        let proof = Self::single_proof_from_witness(&new_single_proof_witness)?;
        info!("done.");

        Ok(Transaction {
            kernel: new_kernel,
            proof: TransactionProof::SingleProof(proof),
        })
    }

    fn single_proof_from_witness(witness: &SingleProofWitness) -> Result<NeptuneProof> {
        let claim = witness.claim();

        let proof = Self::produce(SingleProof.program(), claim, witness.nondeterminism())?.into();

        Ok(proof)
    }

    fn build_gobbler(
        &self,
        gobbling_fee: NativeCurrencyAmount,
        own_wallet_entropy: &WalletEntropy,
        block: &Block,
        old_tx: &Transaction,
        network: Network,
    ) -> Result<Transaction> {
        let mutator_set = block.mutator_set_accumulator_after()?;
        let current_block_height = block.header().height;
        let old_tx_timestamp = old_tx.kernel.timestamp;

        info!("Producing gobbler-transaction for a value of {gobbling_fee}");
        let (utxo_notification_method, receiver_preimage) =
            Self::gobbler_notification_method_with_receiver_preimage(own_wallet_entropy);
        let receiver_digest = receiver_preimage.hash();
        let gobbler = TransactionDetails::fee_gobbler(
            gobbling_fee,
            own_wallet_entropy.generate_sender_randomness(current_block_height, receiver_digest),
            mutator_set,
            old_tx_timestamp,
            utxo_notification_method,
            network,
        );

        let gobbler_witness = gobbler.primitive_witness();

        let proof_collection = Self::produce_proof_collection(&gobbler_witness)?;
        let single_proof_witness = SingleProofWitness::from_collection(proof_collection);
        let proof = Self::single_proof_from_witness(&single_proof_witness)?;

        info!("Done producing gobbler-transaction for a value of {gobbling_fee}");
        let gobbler_tx = Transaction {
            kernel: gobbler_witness.kernel,
            proof: TransactionProof::SingleProof(proof),
        };

        Ok(gobbler_tx)
    }

    fn gobbler_notification_method_with_receiver_preimage(
        own_wallet_entropy: &WalletEntropy,
    ) -> (UtxoNotificationMethod, Digest) {
        let gobble_beneficiary_key =
            SpendingKey::from(own_wallet_entropy.nth_generation_spending_key(0));

        let receiver_preimage = gobble_beneficiary_key.privacy_preimage();
        let gobble_beneficiary_address = gobble_beneficiary_key.to_address();

        let fee_notification_method = UtxoNotificationMethod::OnChain(gobble_beneficiary_address);

        (fee_notification_method, receiver_preimage)
    }

    fn merge_single_proof(&self, tx1: Transaction, tx2: Transaction) -> Result<Transaction> {
        ensure!(
            tx1.kernel.mutator_set_hash == tx2.kernel.mutator_set_hash,
            "Mutator sets must be equal for transaction merger."
        );

        // let tx1_single_proof = match tx1.proof {
        //     TransactionProof::SingleProof(proof) => proof,
        //     _ => return Err(anyhow!("tx1 does not contain a SingleProof")),
        // };
        // let tx2_single_proof = match tx2.proof {
        //     TransactionProof::SingleProof(proof) => proof,
        //     _ => return Err(anyhow!("tx2 does not contain a SingleProof")),
        // };

        let gobble_shuffle_seed: [u8; 32] = rng().random();

        let merge_witness = MergeWitness::from_transactions(tx1, tx2, gobble_shuffle_seed);

        let new_kernel = merge_witness.new_kernel.clone();
        let new_single_proof_witness = SingleProofWitness::from_merge(merge_witness);

        let proof = Self::single_proof_from_witness(&new_single_proof_witness)?;

        Ok(Transaction {
            kernel: new_kernel,
            proof: TransactionProof::SingleProof(proof),
        })
    }
}
