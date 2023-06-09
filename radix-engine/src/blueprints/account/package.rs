use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::types::*;
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::node_modules::metadata::{
    METADATA_GET_IDENT, METADATA_REMOVE_IDENT, METADATA_SET_IDENT,
};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::package::{
    BlueprintSetup, BlueprintTemplate, FunctionSetup, PackageSetup, VirtualLazyLoadExport,
};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintKeyValueStoreSchema, BlueprintSchema, FeaturedSchema,
    FieldSchema, ReceiverInfo, SchemaMethodKey, SchemaMethodPermission, TypeRef,
};

use crate::blueprints::account::{AccountBlueprint, SECURIFY_ROLE};
use crate::method_auth_template;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use radix_engine_interface::types::ClientCostingReason;
use resources_tracker_macro::trace_resources;

use super::AccountSubstate;

const ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME: &str = "create_virtual_ecdsa_secp256k1";
const ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME: &str = "create_virtual_ecdsa_ed25519";

pub struct AccountNativePackage;

impl AccountNativePackage {
    pub fn definition() -> PackageSetup {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::normal(
            aggregator.add_child_type_and_descendents::<AccountSubstate>(),
        ));

        let mut collections = Vec::new();
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<ResourceAddress>(),
                ),
                value: TypeRef::Blueprint(aggregator.add_child_type_and_descendents::<Own>()),
                can_own: true,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<ResourceAddress>(),
                ),
                value: TypeRef::Blueprint(
                    aggregator.add_child_type_and_descendents::<ResourceDepositRule>(),
                ),
                can_own: false,
            },
        ));

        let mut functions = BTreeMap::new();

        functions.insert(
            ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            FunctionSetup {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<AccountCreateAdvancedInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateAdvancedOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_CREATE_ADVANCED_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_IDENT.to_string(),
            FunctionSetup {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<AccountCreateInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_CREATE_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_LOCAL_IDENT.to_string(),
            FunctionSetup {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<AccountCreateLocalInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateLocalOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_CREATE_LOCAL_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_SECURIFY_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountSecurifyInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountSecurifyOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_SECURIFY_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountLockFeeInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountLockFeeOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_LOCK_FEE_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountLockContingentFeeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountLockContingentFeeOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_LOCK_CONTINGENT_FEE_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountDepositInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountDepositOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_DEPOSIT_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountDepositBatchInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountDepositBatchOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_DEPOSIT_BATCH_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountWithdrawInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountWithdrawOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_WITHDRAW_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountWithdrawNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountWithdrawNonFungiblesOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawNonFungiblesOutput>(
                    ),
                export: FeaturedSchema::normal(ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator.add_child_type_and_descendents::<AccountCreateProofInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateProofOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_CREATE_PROOF_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofOfAmountInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofOfAmountOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofOfNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofOfNonFungiblesOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountChangeDefaultDepositRuleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountChangeDefaultDepositRuleOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountConfigureResourceDepositRuleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountConfigureResourceDepositRuleOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositOrRefundInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositOrRefundOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositBatchOrRefundInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositBatchOrRefundOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountTryDepositOrAbortInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositOrAbortOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT.to_string(),
            FunctionSetup {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositBatchOrAbortInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositBatchOrAbortOutput>(),
                export: FeaturedSchema::normal(ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT),
            },
        );

        let virtual_lazy_load_functions = btreemap!(
            ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_ID => VirtualLazyLoadExport {
                export_name: ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME.to_string(),
            },
            ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_ID => VirtualLazyLoadExport {
                export_name: ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME.to_string(),
            }
        );

        let method_auth_template = method_auth_template!(
            SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;
            SchemaMethodKey::metadata(METADATA_SET_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [OWNER_ROLE];

            SchemaMethodKey::main(ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_WITHDRAW_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_LOCK_FEE_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_LOCK_CONTINGENT_FEE_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_CREATE_PROOF_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_SECURIFY_IDENT) => [SECURIFY_ROLE];
            SchemaMethodKey::main(ACCOUNT_DEPOSIT_IDENT) => [OWNER_ROLE];
            SchemaMethodKey::main(ACCOUNT_DEPOSIT_BATCH_IDENT) => [OWNER_ROLE];

            SchemaMethodKey::main(ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT) => SchemaMethodPermission::Public;
            SchemaMethodKey::main(ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT) => SchemaMethodPermission::Public;
            SchemaMethodKey::main(ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT) => SchemaMethodPermission::Public;
            SchemaMethodKey::main(ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT) => SchemaMethodPermission::Public;
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            ACCOUNT_BLUEPRINT.to_string() => BlueprintSetup {
                outer_blueprint: None,
                dependencies: btreeset!(
                    ECDSA_SECP256K1_SIGNATURE_VIRTUAL_BADGE.into(),
                    EDDSA_ED25519_SIGNATURE_VIRTUAL_BADGE.into(),
                    ACCOUNT_OWNER_BADGE.into(),
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                ),
                features: btreeset!(),
                schema,
                blueprint: BlueprintSchema {
                    fields,
                    collections,
                },
                event_schema: [].into(),
                function_auth: btreemap!(
                    ACCOUNT_CREATE_IDENT.to_string() => rule!(allow_all),
                    ACCOUNT_CREATE_LOCAL_IDENT.to_string() => rule!(allow_all),
                    ACCOUNT_CREATE_ADVANCED_IDENT.to_string() => rule!(allow_all),
                ),
                royalty_config: RoyaltyConfig::default(),
                template: BlueprintTemplate {
                    method_auth_template,
                    outer_method_auth_template: btreemap!(),
                },
                virtual_lazy_load_functions,
                functions,
            }
        );

        PackageSetup { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_virtual_secp256k1(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::create_virtual_ed25519(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_ADVANCED_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: AccountCreateAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_advanced(input.owner_role, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let _input: AccountCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_LOCAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let _input: AccountCreateLocalInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_local(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_SECURIFY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: AccountSecurifyInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::securify(receiver, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::lock_fee(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockContingentFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::lock_contingent_fee(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_DEPOSIT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountDepositInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::deposit(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_DEPOSIT_BATCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountDepositBatchInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::deposit_batch(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositOrRefundInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::try_deposit_or_refund(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositBatchOrRefundInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::try_deposit_batch_or_refund(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositOrAbortInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::try_deposit_or_abort(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositBatchOrAbortInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::try_deposit_batch_or_abort(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_WITHDRAW_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountWithdrawInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::withdraw(input.resource_address, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountWithdrawNonFungiblesInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::withdraw_non_fungibles(
                    input.resource_address,
                    input.ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockFeeAndWithdrawInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::lock_fee_and_withdraw(
                    input.amount_to_lock,
                    input.resource_address,
                    input.amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockFeeAndWithdrawNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = AccountBlueprint::lock_fee_and_withdraw_non_fungibles(
                    input.amount_to_lock,
                    input.resource_address,
                    input.ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::create_proof(input.resource_address, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::create_proof_of_amount(
                    input.resource_address,
                    input.amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = AccountBlueprint::create_proof_of_non_fungibles(
                    input.resource_address,
                    input.ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let AccountChangeDefaultDepositRuleInput {
                    default_deposit_rule,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::change_account_default_deposit_rule(
                    default_deposit_rule,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let AccountConfigureResourceDepositRuleInput {
                    resource_address,
                    resource_deposit_configuration,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::configure_resource_deposit_rule(
                    resource_address,
                    resource_deposit_configuration,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
