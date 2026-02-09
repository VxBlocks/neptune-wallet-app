use std::sync::OnceLock;

use neptune_cash::api::export::AdditionRecord;
use neptune_cash::application::json_rpc::core::model::wallet::block::RpcWalletBlock;
use neptune_cash::prelude::tasm_lib::prelude::Digest;
use neptune_cash::prelude::tasm_lib::prelude::Tip5;
use neptune_cash::prelude::triton_vm::prelude::BFieldCodec;
use neptune_cash::prelude::twenty_first::prelude::MerkleTree;
use neptune_cash::protocol::consensus::block::block_kernel::BlockKernel;
use neptune_cash::protocol::consensus::block::mutator_set_update::MutatorSetUpdate;
use neptune_cash::protocol::proof_abstractions::mast_hash::MastHash;
use neptune_cash::util_types::mutator_set::mutator_set_accumulator::MutatorSetAccumulator;
use neptune_cash::util_types::mutator_set::removal_record::removal_record_list::RemovalRecordList;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBlock {
    pub kernel: BlockKernel,
    pub proof_leaf: Digest,

    // this is only here as an optimization for Block::hash()
    // so that we lazily compute the hash at most once.
    #[serde(skip)]
    digest: OnceLock<Digest>,
}

impl From<&RpcWalletBlock> for WalletBlock {
    fn from(block: &RpcWalletBlock) -> Self {
        WalletBlock {
            kernel: BlockKernel::new(
                block.kernel.header.clone().into(),
                block.kernel.body.clone().into(),
                block.kernel.appendix.clone().into(),
            ),
            proof_leaf: block.proof_leaf,
            digest: OnceLock::default(),
        }
    }
}

impl WalletBlock {
    /// Calculate the block hash without assuming that the proof is valid.
    fn mast_hash(&self) -> Digest {
        let kernel_mast_squences = self.kernel.mast_sequences();
        let kernel_leafs = [
            Tip5::hash_varlen(&kernel_mast_squences[0]),
            Tip5::hash_varlen(&kernel_mast_squences[1]),
            Tip5::hash_varlen(&kernel_mast_squences[2]),
            Digest::default(),
        ];
        let kernel_hash = MerkleTree::sequential_frugal_root(&kernel_leafs).unwrap();
        let block_leafs = [Tip5::hash_varlen(&kernel_hash.encode()), self.proof_leaf];

        MerkleTree::sequential_frugal_root(&block_leafs).unwrap()
    }

    /// Calculate the block hash without reading the proof, meaning that the
    /// block hash can be calculated without the exported block containing a
    /// valid proof.
    #[inline]
    pub fn hash(&self) -> Digest {
        *self.digest.get_or_init(|| self.mast_hash())
    }

    /// Return the addition records of the guesser reward of this block.
    fn guesser_fee_addition_records(&self) -> Vec<AdditionRecord> {
        let block_hash = self.hash();
        self.kernel
            .guesser_fee_addition_records(block_hash)
            .expect("Exported blocks are assumed valid")
    }

    /// Return the mutator set as it looks after the application of this block.
    ///
    /// Includes the guesser-fee UTXOs which are not included by the
    /// `mutator_set_accumulator` field on the block body.
    pub fn mutator_set_accumulator_after(&self) -> MutatorSetAccumulator {
        let guesser_fee_addition_records = self.guesser_fee_addition_records();
        let msa = self
            .kernel
            .body
            .mutator_set_accumulator_after(guesser_fee_addition_records);

        msa
    }

    /// Return the mutator set update representing the change to the mutator set
    /// caused by this block.
    pub fn mutator_set_update(&self) -> MutatorSetUpdate {
        let inputs =
            RemovalRecordList::try_unpack(self.kernel.body.transaction_kernel.inputs.clone())
                .expect(
                    "Exported blocks are assumed valid, so removal record list unpacking must work",
                );

        let mut mutator_set_update =
            MutatorSetUpdate::new(inputs, self.kernel.body.transaction_kernel.outputs.clone());

        let guesser_addition_records = self.guesser_fee_addition_records();
        mutator_set_update
            .additions
            .extend(guesser_addition_records);

        mutator_set_update
    }
}
