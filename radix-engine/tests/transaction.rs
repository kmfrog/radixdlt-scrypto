use radix_engine::constants::*;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::transaction::TransactionExecutor;
use radix_engine::wasm::DefaultWasmEngine;
use radix_engine::wasm::WasmInstrumenter;
use scrypto::core::Network;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;
use transaction::builder::TransactionBuilder;
use transaction::model::TransactionHeader;
use transaction::signing::EcdsaPrivateKey;
use transaction::validation::ValidationParameters;
use transaction::validation::{TestIntentHashStore, TransactionValidator};

#[test]
fn test_normal_transaction_flow() {
    let mut substate_store = InMemorySubstateStore::with_bootstrap();
    let mut wasm_engine = DefaultWasmEngine::new();
    let mut wasm_instrumenter = WasmInstrumenter::new();
    let intent_hash_store = TestIntentHashStore::new();
    let parameters: ValidationParameters = ValidationParameters {
        network: Network::LocalSimulator,
        current_epoch: 1,
        max_cost_unit_limit: DEFAULT_COST_UNIT_LIMIT,
        min_tip_bps: 0,
    };
    let cost_unit_price = DEFAULT_COST_UNIT_PRICE.parse().unwrap();
    let max_call_depth = DEFAULT_MAX_CALL_DEPTH;
    let system_loan = DEFAULT_SYSTEM_LOAN;
    let is_system = false;
    let trace = false;

    let raw_transaction = create_transaction();
    let validated_transaction = TransactionValidator::validate_from_slice(
        &raw_transaction,
        &intent_hash_store,
        &parameters,
    )
    .expect("Invalid transaction");

    let mut executor = TransactionExecutor::new(
        &mut substate_store,
        &mut wasm_engine,
        &mut wasm_instrumenter,
        cost_unit_price,
        max_call_depth,
        system_loan,
        is_system,
        trace,
    );
    let receipt = executor.execute_and_commit(&validated_transaction);

    receipt.expect_success();
}

fn create_transaction() -> Vec<u8> {
    // create key pairs
    let sk1 = EcdsaPrivateKey::from_u64(1).unwrap();
    let sk2 = EcdsaPrivateKey::from_u64(2).unwrap();
    let sk_notary = EcdsaPrivateKey::from_u64(3).unwrap();

    let transaction = TransactionBuilder::new()
        .header(TransactionHeader {
            version: 1,
            network: Network::LocalSimulator,
            start_epoch_inclusive: 0,
            end_epoch_exclusive: 100,
            nonce: 5,
            notary_public_key: sk_notary.public_key(),
            notary_as_signatory: false,
            cost_unit_limit: 1_000_000,
            tip_bps: 5,
        })
        .manifest(
            ManifestBuilder::new(Network::LocalSimulator)
                .clear_auth_zone()
                .build(),
        )
        .sign(&sk1)
        .sign(&sk2)
        .notarize(&sk_notary)
        .build();

    transaction.to_bytes()
}
