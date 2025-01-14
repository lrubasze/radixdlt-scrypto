use radix_engine::engine::KernelError;
use radix_engine::engine::RejectionError;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;
use utils::ContextualDisplay;

#[test]
fn test_manifest_with_non_existent_resource() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();
    let non_existent_resource = ResourceAddress::Normal([0u8; 26]);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account, 10u32.into())
        .take_from_worktop(non_existent_resource, |builder, bucket_id| {
            builder.call_method(account, "deposit", args!(Bucket(bucket_id)))
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::KernelError(
                KernelError::RENodeNotFound(..)
            ))
        )
    });
}

#[test]
fn test_call_method_with_all_resources_doesnt_drop_auth_zone_proofs() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account, dec!("10"))
        .create_proof_from_account(account, RADIX_TOKEN)
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_transaction_can_end_with_proofs_remaining_in_auth_zone() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account, dec!("10"))
        .create_proof_from_account_by_amount(account, dec!("1"), RADIX_TOKEN)
        .create_proof_from_account_by_amount(account, dec!("1"), RADIX_TOKEN)
        .create_proof_from_account_by_amount(account, dec!("1"), RADIX_TOKEN)
        .create_proof_from_account_by_amount(account, dec!("1"), RADIX_TOKEN)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_non_existent_blob_hash() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account, dec!("10"))
        .add_instruction(Instruction::CallNativeFunction {
            function_ident: NativeFunctionIdent {
                blueprint_name: PACKAGE_BLUEPRINT.to_string(),
                function_name: PackageFunction::Publish.to_string(),
            },
            args: scrypto_encode(&PackagePublishInvocation {
                code: Blob(Hash([0; 32])),
                abi: Blob(Hash([0; 32])),
                royalty_config: HashMap::new(),
                metadata: HashMap::new(),
                access_rules: AccessRules::new()
                    .default(AccessRule::AllowAll, AccessRule::AllowAll),
            })
            .unwrap(),
        })
        .0
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::KernelError(KernelError::BlobNotFound(_)))
    });
}

#[test]
fn test_entire_auth_zone() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account, dec!("10"))
        .create_proof_from_account_by_amount(account, dec!("1"), RADIX_TOKEN)
        .call_function(
            package_address,
            "Receiver",
            "assert_first_proof",
            args!(Expression::entire_auth_zone(), dec!("1"), RADIX_TOKEN),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_faucet_drain_attempt_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account, dec!("10"))
        .call_method(FAUCET_COMPONENT, "free", args!())
        .call_method(FAUCET_COMPONENT, "free", args!())
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.display(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_failure();
}
