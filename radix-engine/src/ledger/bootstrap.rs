use crate::blueprints::clock::ClockNativePackage;
use crate::blueprints::epoch_manager::EpochManagerNativePackage;
use crate::kernel::interpreters::ScryptoInterpreter;
use crate::ledger::{ReadableSubstateStore, WriteableSubstateStore};
use crate::transaction::{
    execute_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt,
};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::node_modules::auth::{AccessRulesAbi, AuthAddresses};
use radix_engine_interface::api::node_modules::metadata::MetadataAbi;
use radix_engine_interface::api::node_modules::royalty::RoyaltyAbi;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::package::{
    PackageLoaderPublishNativeInput, PackageLoaderPublishWasmInput,
};
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::access_controller::AccessControllerAbi;
use radix_engine_interface::blueprints::account::AccountAbi;
use radix_engine_interface::blueprints::auth_zone::AuthZoneAbi;
use radix_engine_interface::blueprints::clock::{
    ClockAbi, ClockCreateInput, CLOCK_BLUEPRINT, CLOCK_CREATE_IDENT,
};
use radix_engine_interface::blueprints::epoch_manager::{
    EpochManagerAbi, ManifestValidatorInit, EPOCH_MANAGER_BLUEPRINT, EPOCH_MANAGER_CREATE_IDENT,
};
use radix_engine_interface::blueprints::identity::IdentityAbi;
use radix_engine_interface::blueprints::logger::LoggerAbi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_runtime::TransactionRuntimeAbi;
use radix_engine_interface::data::*;
use radix_engine_interface::rule;
use transaction::data::{manifest_args, manifest_encode};
use transaction::model::{Instruction, SystemTransaction};
use transaction::validation::ManifestIdAllocator;

const XRD_SYMBOL: &str = "XRD";
const XRD_NAME: &str = "Radix";
const XRD_DESCRIPTION: &str = "The Radix Public Network's native token, used to pay the network's required transaction fees and to secure the network through staking to its validator nodes.";
const XRD_URL: &str = "https://tokens.radixdlt.com";
const XRD_MAX_SUPPLY: i128 = 1_000_000_000_000i128;

pub struct GenesisReceipt {
    pub faucet_component: ComponentAddress,
}

pub fn create_genesis(
    validator_set_and_stake_owners: BTreeMap<EcdsaSecp256k1PublicKey, (Decimal, ComponentAddress)>,
    account_xrd_allocations: BTreeMap<EcdsaSecp256k1PublicKey, Decimal>,
    initial_epoch: u64,
    rounds_per_epoch: u64,
    num_unstake_epochs: u64,
) -> SystemTransaction {
    // NOTES
    // * Create resources before packages to avoid circular dependencies.

    let mut id_allocator = ManifestIdAllocator::new();
    let mut instructions = Vec::new();
    let mut pre_allocated_ids = BTreeSet::new();

    // Metadata Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(METADATA_PACKAGE));
        let package_address = METADATA_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: METADATA_CODE_ID,
                abi: MetadataAbi::blueprint_abis(),
                dependent_resources: vec![],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::DenyAll,
            })
            .unwrap(),
        });
    }

    // Royalty Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(ROYALTY_PACKAGE));
        let package_address = ROYALTY_PACKAGE.to_array_without_entity_id();

        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: ROYALTY_CODE_ID,
                abi: RoyaltyAbi::blueprint_abis(),
                dependent_resources: vec![],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::DenyAll,
            })
            .unwrap(),
        });
    }

    // Access Rules Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(ACCESS_RULES_PACKAGE));
        let package_address = ACCESS_RULES_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: ACCESS_RULES_CODE_ID,
                abi: AccessRulesAbi::blueprint_abis(),
                dependent_resources: vec![],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::DenyAll,
            })
            .unwrap(),
        });
    }

    // Resource Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(RESOURCE_MANAGER_PACKAGE));
        let package_address = RESOURCE_MANAGER_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                native_package_code_id: RESOURCE_MANAGER_PACKAGE_CODE_ID,
                abi: ResourceManagerAbi::blueprint_abis(),
                dependent_resources: vec![],
                dependent_components: vec![],
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            })
            .unwrap(),
        });
    }

    // XRD Token
    {
        let mut metadata = BTreeMap::new();
        metadata.insert("symbol".to_owned(), XRD_SYMBOL.to_owned());
        metadata.insert("name".to_owned(), XRD_NAME.to_owned());
        metadata.insert("description".to_owned(), XRD_DESCRIPTION.to_owned());
        metadata.insert("url".to_owned(), XRD_URL.to_owned());

        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let initial_supply: Decimal = XRD_MAX_SUPPLY.into();
        let resource_address = RADIX_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalResourceManager(RADIX_TOKEN));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT
                .to_string(),
            args: manifest_encode(
                &ResourceManagerCreateFungibleWithInitialSupplyAndAddressInput {
                    divisibility: 18,
                    metadata,
                    access_rules,
                    initial_supply,
                    resource_address,
                },
            )
            .unwrap(),
        });
    }

    // Package Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = PACKAGE_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalResourceManager(PACKAGE_TOKEN));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT.to_string(),
            args: manifest_encode(&ResourceManagerCreateNonFungibleWithAddressInput {
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
                resource_address,
            })
            .unwrap(),
        });
    }

    // Identity Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(IDENTITY_PACKAGE));
        let package_address = IDENTITY_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                abi: IdentityAbi::blueprint_abis(),
                dependent_resources: vec![],
                dependent_components: vec![],
                native_package_code_id: IDENTITY_PACKAGE_CODE_ID,
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            })
            .unwrap(),
        });
    }

    // EpochManager Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(EPOCH_MANAGER_PACKAGE));
        let package_address = EPOCH_MANAGER_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                abi: EpochManagerAbi::blueprint_abis(),
                native_package_code_id: EPOCH_MANAGER_PACKAGE_CODE_ID,
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                dependent_resources: vec![RADIX_TOKEN, PACKAGE_TOKEN],
                dependent_components: vec![],
                package_access_rules: EpochManagerNativePackage::package_access_rules(),
                default_package_access_rule: AccessRule::DenyAll,
            })
            .unwrap(),
        });
    }

    // Clock Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(CLOCK_PACKAGE));
        let package_address = CLOCK_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                abi: ClockAbi::blueprint_abis(),
                native_package_code_id: CLOCK_PACKAGE_CODE_ID,
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                dependent_resources: vec![],
                dependent_components: vec![],
                package_access_rules: ClockNativePackage::package_access_rules(),
                default_package_access_rule: AccessRule::DenyAll,
            })
            .unwrap(),
        });
    }

    // Account Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(ACCOUNT_PACKAGE));
        let package_address = ACCOUNT_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                abi: AccountAbi::blueprint_abis(),
                native_package_code_id: ACCOUNT_PACKAGE_CODE_ID,
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                dependent_resources: vec![],
                dependent_components: vec![],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            })
            .unwrap(),
        });
    }

    // AccessRules Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(ACCESS_CONTROLLER_PACKAGE));
        let package_address = ACCESS_CONTROLLER_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                abi: AccessControllerAbi::blueprint_abis(),
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                native_package_code_id: ACCESS_CONTROLLER_PACKAGE_CODE_ID,
                dependent_resources: vec![],
                dependent_components: vec![CLOCK],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            })
            .unwrap(),
        });
    }

    // Logger Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(LOGGER_PACKAGE));
        let package_address = LOGGER_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                abi: TransactionRuntimeAbi::blueprint_abis(),
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                native_package_code_id: LOGGER_CODE_ID,
                dependent_resources: vec![],
                dependent_components: vec![],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::AllowAll,
            })
            .unwrap(),
        });
    }

    // TransactionRuntime Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(TRANSACTION_RUNTIME_PACKAGE));
        let package_address = TRANSACTION_RUNTIME_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                abi: LoggerAbi::blueprint_abis(),
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                native_package_code_id: TRANSACTION_RUNTIME_CODE_ID,
                dependent_resources: vec![],
                dependent_components: vec![],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::DenyAll,
            })
            .unwrap(),
        });
    }

    // AuthZone Package
    {
        pre_allocated_ids.insert(RENodeId::GlobalPackage(AUTH_ZONE_PACKAGE));
        let package_address = AUTH_ZONE_PACKAGE.to_array_without_entity_id();
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_NATIVE_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishNativeInput {
                package_address: Some(package_address), // TODO: Clean this up
                abi: AuthZoneAbi::blueprint_abis(),
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new(),
                native_package_code_id: AUTH_ZONE_CODE_ID,
                dependent_resources: vec![],
                dependent_components: vec![],
                package_access_rules: BTreeMap::new(),
                default_package_access_rule: AccessRule::DenyAll,
            })
            .unwrap(),
        });
    }

    // ECDSA
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = ECDSA_SECP256K1_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalResourceManager(ECDSA_SECP256K1_TOKEN));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT.to_string(),
            args: manifest_encode(&ResourceManagerCreateNonFungibleWithAddressInput {
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
                resource_address,
            })
            .unwrap(),
        });
    }

    // TODO: Perhaps combine with ecdsa token?
    // EDDSA ED25519 Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = EDDSA_ED25519_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalResourceManager(EDDSA_ED25519_TOKEN));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT.to_string(),
            args: manifest_encode(&ResourceManagerCreateNonFungibleWithAddressInput {
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
                resource_address,
            })
            .unwrap(),
        });
    }

    // TODO: Perhaps combine with ecdsa token?
    // System Token
    {
        let metadata: BTreeMap<String, String> = BTreeMap::new();
        let mut access_rules = BTreeMap::new();
        access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));
        let resource_address = SYSTEM_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalResourceManager(SYSTEM_TOKEN));
        instructions.push(Instruction::CallFunction {
            package_address: RESOURCE_MANAGER_PACKAGE,
            blueprint_name: RESOURCE_MANAGER_BLUEPRINT.to_string(),
            function_name: RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT.to_string(),
            args: manifest_encode(&ResourceManagerCreateNonFungibleWithAddressInput {
                id_type: NonFungibleIdType::Bytes,
                metadata,
                access_rules,
                resource_address,
            })
            .unwrap(),
        });
    }

    {
        let faucet_code = include_bytes!("../../../assets/faucet.wasm").to_vec();
        let faucet_abi = include_bytes!("../../../assets/faucet.abi").to_vec();
        let package_address = FAUCET_PACKAGE.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalPackage(FAUCET_PACKAGE));
        instructions.push(Instruction::CallFunction {
            package_address: PACKAGE_LOADER,
            blueprint_name: PACKAGE_LOADER_BLUEPRINT.to_string(),
            function_name: PACKAGE_LOADER_PUBLISH_WASM_IDENT.to_string(),
            args: manifest_encode(&PackageLoaderPublishWasmInput {
                package_address: Some(package_address),
                code: faucet_code, // TODO: Use blob here instead?
                abi: scrypto_decode(&faucet_abi).unwrap(), // TODO: Use blob here instead?
                royalty_config: BTreeMap::new(),
                metadata: BTreeMap::new(),
                access_rules: AccessRules::new().default(AccessRule::DenyAll, AccessRule::DenyAll),
            })
            .unwrap(),
        });
    }

    {
        let component_address = CLOCK.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalComponent(CLOCK));
        instructions.push(Instruction::CallFunction {
            package_address: CLOCK_PACKAGE,
            blueprint_name: CLOCK_BLUEPRINT.to_string(),
            function_name: CLOCK_CREATE_IDENT.to_string(),
            args: manifest_encode(&ClockCreateInput { component_address }).unwrap(),
        });
    }

    {
        let mut validators = BTreeMap::new();
        for (key, (amount, stake_account_address)) in validator_set_and_stake_owners {
            let initial_stake = id_allocator.new_bucket_id().unwrap();
            instructions.push(
                Instruction::TakeFromWorktopByAmount {
                    resource_address: RADIX_TOKEN,
                    amount,
                }
                .into(),
            );
            let validator_account_address = ComponentAddress::virtual_account_from_public_key(&key);
            validators.insert(
                key,
                ManifestValidatorInit {
                    validator_account_address,
                    initial_stake,
                    stake_account_address,
                },
            );
        }

        let component_address = EPOCH_MANAGER.to_array_without_entity_id();
        let olympia_validator_token_address = OLYMPIA_VALIDATOR_TOKEN.to_array_without_entity_id();
        pre_allocated_ids.insert(RENodeId::GlobalResourceManager(OLYMPIA_VALIDATOR_TOKEN));
        pre_allocated_ids.insert(RENodeId::GlobalComponent(EPOCH_MANAGER));
        instructions.push(Instruction::CallFunction {
            package_address: EPOCH_MANAGER_PACKAGE,
            blueprint_name: EPOCH_MANAGER_BLUEPRINT.to_string(),
            function_name: EPOCH_MANAGER_CREATE_IDENT.to_string(),
            args: manifest_args!(
                olympia_validator_token_address,
                component_address,
                validators,
                initial_epoch,
                rounds_per_epoch,
                num_unstake_epochs
            ),
        });
    }

    for (public_key, amount) in account_xrd_allocations.into_iter() {
        let bucket_id = id_allocator.new_bucket_id().unwrap();
        instructions.push(
            Instruction::TakeFromWorktopByAmount {
                resource_address: RADIX_TOKEN,
                amount,
            }
            .into(),
        );
        let component_address = ComponentAddress::virtual_account_from_public_key(
            &PublicKey::EcdsaSecp256k1(public_key),
        );
        instructions.push(
            Instruction::CallMethod {
                component_address: component_address,
                method_name: "deposit".to_string(),
                args: manifest_args!(bucket_id),
            }
            .into(),
        );
    }

    // Faucet
    {
        instructions.push(
            Instruction::TakeFromWorktop {
                resource_address: RADIX_TOKEN,
            }
            .into(),
        );

        let bucket = id_allocator.new_bucket_id().unwrap();
        instructions.push(Instruction::CallFunction {
            package_address: FAUCET_PACKAGE,
            blueprint_name: FAUCET_BLUEPRINT.to_string(),
            function_name: "new".to_string(),
            args: manifest_args!(bucket),
        });
    };

    SystemTransaction {
        instructions,
        blobs: Vec::new(),
        pre_allocated_ids,
        nonce: 0,
    }
}

pub fn genesis_result(receipt: &TransactionReceipt) -> GenesisReceipt {
    // TODO: Remove this when appropriate syscalls are implemented for Scrypto
    let faucet_component = receipt.new_component_addresses().last().unwrap().clone();
    GenesisReceipt { faucet_component }
}

pub fn bootstrap<S, W>(
    substate_store: &mut S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
) -> Option<TransactionReceipt>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine,
{
    bootstrap_with_validator_set(
        substate_store,
        scrypto_interpreter,
        BTreeMap::new(),
        BTreeMap::new(),
        1u64,
        1u64,
        1u64,
    )
}

pub fn bootstrap_with_validator_set<S, W>(
    substate_store: &mut S,
    scrypto_interpreter: &ScryptoInterpreter<W>,
    validator_set_and_stake_owners: BTreeMap<EcdsaSecp256k1PublicKey, (Decimal, ComponentAddress)>,
    account_xrd_allocations: BTreeMap<EcdsaSecp256k1PublicKey, Decimal>,
    initial_epoch: u64,
    rounds_per_epoch: u64,
    num_unstake_epochs: u64,
) -> Option<TransactionReceipt>
where
    S: ReadableSubstateStore + WriteableSubstateStore,
    W: WasmEngine,
{
    if substate_store
        .get_substate(&SubstateId(
            RENodeId::GlobalResourceManager(RADIX_TOKEN),
            NodeModuleId::TypeInfo,
            SubstateOffset::TypeInfo(TypeInfoOffset::TypeInfo),
        ))
        .is_none()
    {
        let genesis_transaction = create_genesis(
            validator_set_and_stake_owners,
            account_xrd_allocations,
            initial_epoch,
            rounds_per_epoch,
            num_unstake_epochs,
        );

        let transaction_receipt = execute_transaction(
            substate_store,
            scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );

        let commit_result = transaction_receipt.expect_commit();
        commit_result.outcome.expect_success();
        commit_result.state_updates.commit(substate_store);

        Some(transaction_receipt)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use transaction::data::{manifest_decode, manifest_encode};
    use transaction::{data::ManifestBucket, ecdsa_secp256k1::EcdsaSecp256k1PrivateKey};

    use super::*;
    use crate::{ledger::TypedInMemorySubstateStore, wasm::DefaultWasmEngine};

    #[test]
    fn bootstrap_receipt_should_match_constants() {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let substate_store = TypedInMemorySubstateStore::new();
        let mut initial_validator_set = BTreeMap::new();
        let public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
        let account_address = ComponentAddress::virtual_account_from_public_key(&public_key);
        initial_validator_set.insert(
            EcdsaSecp256k1PublicKey([0; 33]),
            (Decimal::one(), account_address),
        );
        let genesis_transaction =
            create_genesis(initial_validator_set, BTreeMap::new(), 1u64, 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::debug(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );
        #[cfg(not(feature = "alloc"))]
        println!("{:?}", transaction_receipt);

        transaction_receipt.expect_commit_success();

        let genesis_receipt = genesis_result(&transaction_receipt);
        assert_eq!(genesis_receipt.faucet_component, FAUCET_COMPONENT);
    }

    #[test]
    fn test_genesis_xrd_allocation_to_accounts() {
        let scrypto_interpreter = ScryptoInterpreter::<DefaultWasmEngine>::default();
        let mut substate_store = TypedInMemorySubstateStore::new();
        let account_public_key = EcdsaSecp256k1PrivateKey::from_u64(1).unwrap().public_key();
        let account_component_address = ComponentAddress::virtual_account_from_public_key(
            &PublicKey::EcdsaSecp256k1(account_public_key.clone()),
        );
        let allocation_amount = dec!("100");
        let mut account_xrd_allocations = BTreeMap::new();
        account_xrd_allocations.insert(account_public_key, allocation_amount);
        let genesis_transaction =
            create_genesis(BTreeMap::new(), account_xrd_allocations, 1u64, 1u64, 1u64);

        let transaction_receipt = execute_transaction(
            &substate_store,
            &scrypto_interpreter,
            &FeeReserveConfig::default(),
            &ExecutionConfig::default(),
            &genesis_transaction.get_executable(vec![AuthAddresses::system_role()]),
        );

        let commit_result = transaction_receipt.result.expect_commit();
        commit_result.state_updates.commit(&mut substate_store);

        assert!(commit_result
            .resource_changes
            .iter()
            .any(|rc| rc.amount == allocation_amount
                && rc.node_id == RENodeId::GlobalComponent(account_component_address)));
    }

    #[test]
    fn test_encode_and_decode_validator_init() {
        let t = ManifestValidatorInit {
            validator_account_address: ComponentAddress::AccessController([0u8; 26]),
            initial_stake: ManifestBucket(1),
            stake_account_address: ComponentAddress::AccessController([0u8; 26]),
        };

        let bytes = manifest_encode(&t).unwrap();
        let decoded: ManifestValidatorInit = manifest_decode(&bytes).unwrap();
        assert_eq!(decoded, t);
    }
}
