use anyhow::{anyhow, ensure, Result};
use neptune_cash::{
    api::export::{NeptuneProof, Timestamp, Transaction, TransactionProof},
    models::{
        blockchain::{
            block::Block,
            transaction::{
                transaction_kernel::TransactionKernelModifier,
                validity::{
                    proof_collection::ProofCollection,
                    single_proof::{SingleProof, SingleProofWitness},
                    tasm::single_proof::update_branch::UpdateWitness,
                },
            },
        },
        proof_abstractions::{tasm::program::ConsensusProgram, SecretWitness},
    },
    prelude::twenty_first::util_types::mmr::mmr_successor_proof::MmrSuccessorProof,
};
use tracing::info;

impl super::ProofBuilder {
    pub fn upgrade_proof_collection(
        &self,
        proof_collection: ProofCollection,
    ) -> Result<NeptuneProof> {
        let single_proof_witness = SingleProofWitness::from_collection(proof_collection);
        Self::single_proof_from_witness(&single_proof_witness)
    }

    pub fn update_single_proof(
        &self,
        tx: Transaction,
        old_block: Block,
        block: Block,
    ) -> Result<Transaction> {
        let old_transaction_kernel = tx.kernel;
        let old_single_proof = match tx.proof {
            TransactionProof::SingleProof(proof) => proof,
            _ => return Err(anyhow!("No single proof found")),
        };

        let new_timestamp: Option<Timestamp> = Option::None;

        let previous_mutator_set_accumulator = old_block.mutator_set_accumulator_after()?;
        let mutator_set_update = block.mutator_set_update()?;

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

        let proof = Self::produce(SingleProof.program(), claim, witness.nondeterminism())?;

        Ok(proof)
    }
}
