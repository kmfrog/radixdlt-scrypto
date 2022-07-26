use clap::Parser;
use radix_engine::engine::{CallFrame, SystemApi};
use radix_engine::fee::{
    FeeTable, SystemLoanCostUnitCounter, DEFAULT_MAX_TRANSACTION_COST, DEFAULT_SYSTEM_LOAN_AMOUNT,
};
use radix_engine::state_manager::Track;
use scrypto::core::{SNodeRef, SystemSetEpochInput};
use scrypto::values::ScryptoValue;
use transaction::validation::{IdAllocator, IdSpace};

use crate::resim::*;

/// Set the current epoch
#[derive(Parser, Debug)]
pub struct SetCurrentEpoch {
    /// The new epoch number
    epoch: u64,
}

impl SetCurrentEpoch {
    pub fn run<O: std::io::Write>(&self, _out: &mut O) -> Result<(), Error> {
        let hash = Hash([0; Hash::LENGTH]);

        let mut id_allocator = IdAllocator::new(IdSpace::Application);
        let mut substate_store = RadixEngineDB::with_bootstrap(get_data_dir()?);
        let mut wasm_engine = DefaultWasmEngine::new();
        let mut wasm_instrumenter = WasmInstrumenter::new();
        let mut track = Track::new(&mut substate_store);
        let mut cost_unit_counter = SystemLoanCostUnitCounter::new(
            DEFAULT_MAX_TRANSACTION_COST,
            DEFAULT_SYSTEM_LOAN_AMOUNT,
        );
        let fee_table = FeeTable::new();

        // Create root call frame.
        let mut root_frame = CallFrame::new_root(
            false,
            hash,
            vec![],
            true,
            &mut id_allocator,
            &mut track,
            &mut wasm_engine,
            &mut wasm_instrumenter,
            &mut cost_unit_counter,
            &fee_table,
        );

        root_frame
            .invoke_snode(
                SNodeRef::SystemRef,
                "set_epoch".to_string(),
                ScryptoValue::from_typed(&SystemSetEpochInput { epoch: self.epoch }),
            )
            .map(|_| ())
            .map_err(Error::TransactionExecutionError)
    }
}
