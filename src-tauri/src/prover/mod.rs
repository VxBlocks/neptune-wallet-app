use neptune_privacy::api::export::NativeCurrencyAmount;
use neptune_privacy::api::export::Network;
use neptune_privacy::prelude::tasm_lib;
use neptune_privacy::prelude::triton_vm::proof::Proof;
use tasm_lib::triton_vm::prelude::Program;
use tasm_lib::triton_vm::proof::Claim;
use tasm_lib::triton_vm::prove;
use tasm_lib::triton_vm::stark::Stark;
use tasm_lib::triton_vm::vm::NonDeterminism;
use tracing::*;

mod proof_collection;
mod single_proof;

pub struct ProofBuilder {
    gobble_fee: Option<NativeCurrencyAmount>,
    network: Network,
}

impl ProofBuilder {
    pub fn new(gobble_fee: Option<NativeCurrencyAmount>, network: Network) -> Self {
        Self {
            gobble_fee,
            network,
        }
    }
    fn produce(
        program: Program,
        claim: Claim,
        non_determinism: NonDeterminism,
    ) -> anyhow::Result<Proof> {
        let default_stark: Stark = Stark::default();

        let proof = prove(default_stark, &claim, program, non_determinism)?;
        info!("triton-vm: completed proof");

        Ok(proof.into())
    }
}
