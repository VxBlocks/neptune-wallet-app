use anyhow::Result;
use itertools::Itertools;
use neptune_privacy::api::export::Tip5;
use neptune_privacy::prelude::tasm_lib;
use neptune_privacy::prelude::triton_vm::vm::PublicInput;
use neptune_privacy::protocol::consensus::transaction::primitive_witness::PrimitiveWitness;
use neptune_privacy::protocol::consensus::transaction::transaction_kernel::TransactionKernelField;
use neptune_privacy::protocol::consensus::transaction::validity::collect_lock_scripts::CollectLockScriptsWitness;
use neptune_privacy::protocol::consensus::transaction::validity::collect_type_scripts::CollectTypeScriptsWitness;
use neptune_privacy::protocol::consensus::transaction::validity::kernel_to_outputs::KernelToOutputsWitness;
use neptune_privacy::protocol::consensus::transaction::validity::proof_collection::ProofCollection;
use neptune_privacy::protocol::consensus::transaction::validity::removal_records_integrity::RemovalRecordsIntegrityWitness;
use neptune_privacy::protocol::proof_abstractions::mast_hash::MastHash;
use neptune_privacy::protocol::proof_abstractions::SecretWitness;
use tasm_lib::triton_vm::proof::Claim;
use tracing::debug;
use tracing::info;

impl super::ProofBuilder {
    pub fn produce_proof_collection(
        primitive_witness: &PrimitiveWitness,
    ) -> Result<ProofCollection> {
        let (
            removal_records_integrity_witness,
            collect_lock_scripts_witness,
            kernel_to_outputs_witness,
            collect_type_scripts_witness,
        ) = Self::extract_specific_witnesses(primitive_witness);

        let txk_mast_hash = primitive_witness.kernel.mast_hash();
        let txk_mast_hash_as_input = PublicInput::new(txk_mast_hash.reversed().values().to_vec());
        let salted_inputs_hash = Tip5::hash(&primitive_witness.input_utxos);
        let salted_outputs_hash = Tip5::hash(&primitive_witness.output_utxos);
        debug!("proving, txk hash: {}", txk_mast_hash);
        debug!("proving, salted inputs hash: {}", salted_inputs_hash);
        debug!("proving, salted outputs hash: {}", salted_outputs_hash);

        // prove
        debug!("proving RemovalRecordsIntegrity");
        let removal_records_integrity = Self::produce(
            removal_records_integrity_witness.program(),
            removal_records_integrity_witness.claim(),
            removal_records_integrity_witness.nondeterminism(),
        )?
        .into();

        debug!("proving CollectLockScripts");
        let collect_lock_scripts = Self::produce(
            collect_lock_scripts_witness.program(),
            collect_lock_scripts_witness.claim(),
            collect_lock_scripts_witness.nondeterminism(),
        )?
        .into();

        debug!("proving KernelToOutputs");
        let kernel_to_outputs = Self::produce(
            kernel_to_outputs_witness.program(),
            kernel_to_outputs_witness.claim(),
            kernel_to_outputs_witness.nondeterminism(),
        )?
        .into();

        debug!("proving CollectTypeScripts");
        let collect_type_scripts = Self::produce(
            collect_type_scripts_witness.program(),
            collect_type_scripts_witness.claim(),
            collect_type_scripts_witness.nondeterminism(),
        )?
        .into();

        debug!("proving lock scripts");
        let mut lock_scripts_halt = vec![];
        for lock_script_and_witness in &primitive_witness.lock_scripts_and_witnesses {
            let claim = Claim::new(lock_script_and_witness.program.hash())
                .with_input(txk_mast_hash_as_input.clone().individual_tokens);
            let lock_script_and_witness = Self::produce(
                lock_script_and_witness.program.clone(),
                claim,
                lock_script_and_witness.nondeterminism(),
            )?
            .into();
            lock_scripts_halt.push(lock_script_and_witness);
        }

        debug!("proving type scripts");
        let mut type_scripts_halt = vec![];
        for (i, tsaw) in primitive_witness
            .type_scripts_and_witnesses
            .iter()
            .enumerate()
        {
            debug!("proving type script number {i}: {}", tsaw.program.hash());
            let input = [txk_mast_hash, salted_inputs_hash, salted_outputs_hash]
                .into_iter()
                .flat_map(|d| d.reversed().values())
                .collect_vec();
            let claim = Claim::new(tsaw.program.hash()).with_input(input);

            let type_script_halt =
                Self::produce(tsaw.program.clone(), claim, tsaw.nondeterminism())?.into();

            type_scripts_halt.push(type_script_halt);
        }
        info!("done proving proof collection");

        // collect hashes
        let lock_script_hashes = primitive_witness
            .lock_scripts_and_witnesses
            .iter()
            .map(|lsaw| lsaw.program.hash())
            .collect_vec();
        let type_script_hashes = primitive_witness
            .type_scripts_and_witnesses
            .iter()
            .map(|tsaw| tsaw.program.hash())
            .collect_vec();

        let merge_bit_mast_path = primitive_witness
            .kernel
            .mast_path(TransactionKernelField::MergeBit);

        Ok(ProofCollection {
            removal_records_integrity,
            collect_lock_scripts,
            lock_scripts_halt,
            kernel_to_outputs,
            collect_type_scripts,
            type_scripts_halt,
            lock_script_hashes,
            type_script_hashes,
            kernel_mast_hash: txk_mast_hash,
            salted_inputs_hash,
            salted_outputs_hash,
            merge_bit_mast_path,
        })
    }

    fn extract_specific_witnesses(
        primitive_witness: &PrimitiveWitness,
    ) -> (
        RemovalRecordsIntegrityWitness,
        CollectLockScriptsWitness,
        KernelToOutputsWitness,
        CollectTypeScriptsWitness,
    ) {
        // collect witnesses
        let removal_records_integrity_witness =
            RemovalRecordsIntegrityWitness::from(primitive_witness);
        let collect_lock_scripts_witness = CollectLockScriptsWitness::from(primitive_witness);
        let kernel_to_outputs_witness = KernelToOutputsWitness::from(primitive_witness);
        let collect_type_scripts_witness = CollectTypeScriptsWitness::from(primitive_witness);

        (
            removal_records_integrity_witness,
            collect_lock_scripts_witness,
            kernel_to_outputs_witness,
            collect_type_scripts_witness,
        )
    }
}
