use radix_engine::types::*;
use sbor::rust::prelude::*;

// Import and re-export these types so they are available easily with a single import
pub use radix_engine::blueprints::access_controller::*;
pub use radix_engine::blueprints::account::*;
pub use radix_engine::blueprints::clock::*;
pub use radix_engine::blueprints::epoch_manager::*;
pub use radix_engine::blueprints::package::*;
pub use radix_engine::blueprints::resource::*;
pub use radix_engine::system::node_modules::access_rules::*;
pub use radix_engine::system::node_modules::metadata::*;
pub use radix_engine::system::node_modules::royalty::*;
pub use radix_engine::system::node_modules::type_info::*;

//=========================================================================
// Please update REP-60 after updating types/configs defined in this file!
//
// The below defines well-known substate types which are used in the
// Core API of the node.
//
// Specifically:
// * Every (EntityType, PartitionNumber, SubstateKey) should be mappable into a `TypedSubstateKey`
// * Every (&TypedSubstateKey, Data) should be mappable into a `TypedSubstateValue`
//
// Please keep them these in-line with the well-known objects, and please don't
// remove these without talking to the Network team.
//=========================================================================

//=========================================================================
// A partition can be in one of four types:
//
// - Field
//   => Has key: TupleKey(u8) also known as an offset
//   => No iteration exposed to engine
//   => Is versioned / locked substate-by-substate
// - KeyValue ("ConcurrentMap")
//   => Has key: MapKey(Vec<u8>)
//   => No iteration exposed to engine
//   => Is versioned / locked substate-by-substate
// - Index
//   => Has key: MapKey(Vec<u8>)
//   => Iteration exposed to engine via the MapKey's database key (ie hash of the key)
//   => Is versioned / locked in its entirety
// - SortedU16Index
//   => Has key: SortedU16Key(U16, Vec<u8>)
//   => Iteration exposed to engine via the user-controlled U16 prefix and then the MapKey's database key (ie hash of the key)
//   => Is versioned / locked in its entirety
//
// But in this file, we just handle explicitly supported/possible combinations of things.
//
// An entirely generic capturing of a substate key for a given node partition would look something like this:
//
// pub enum GenericModuleSubstateKey {
//    Field(TupleKey),
//    KeyValue(MapKey),
//    Index(MapKey),
//    SortedU16Index(SortedU16Key),
// }
//=========================================================================

/// By node module (roughly SysModule)
#[derive(Debug, Clone)]
pub enum TypedSubstateKey {
    TypeInfoModuleField(TypeInfoField),
    AccessRulesModuleField(AccessRulesField),
    RoyaltyModuleField(RoyaltyField),
    MetadataModuleEntryKey(String),
    MainModule(TypedMainModuleSubstateKey),
}

impl TypedSubstateKey {
    /// This method should be used to filter out substates which we don't want to map in the Core API.
    /// (See `radix-engine-tests/tests/bootstrap.rs` for an example of how it should be used)
    /// Just a work around for now to filter out "transient" substates we shouldn't really be storing.
    pub fn value_is_mappable(&self) -> bool {
        match self {
            TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::NonFungibleVaultField(
                NonFungibleVaultField::LockedNonFungible,
            )) => false,
            TypedSubstateKey::MainModule(TypedMainModuleSubstateKey::FungibleVaultField(
                FungibleVaultField::LockedFungible,
            )) => false,
            _ => true,
        }
    }
}

/// Doesn't include non-object modules, nor transient nodes.
#[derive(Debug, Clone)]
pub enum TypedMainModuleSubstateKey {
    // Objects
    PackageField(PackageField),
    FungibleResourceField(FungibleResourceManagerField),
    NonFungibleResourceField(NonFungibleResourceManagerField),
    NonFungibleResourceData(NonFungibleLocalId),
    FungibleVaultField(FungibleVaultField),
    NonFungibleVaultField(NonFungibleVaultField),
    NonFungibleVaultContentsIndexKey(NonFungibleLocalId),
    EpochManagerField(EpochManagerField),
    EpochManagerRegisteredValidatorsByStakeIndexKey(ValidatorByStakeKey),
    ClockField(ClockField),
    ValidatorField(ValidatorField),
    AccountVaultIndexKey(ResourceAddress),
    AccessControllerField(AccessControllerField),
    // Generic Scrypto Components
    GenericScryptoComponentField(ComponentField),
    // Substates for Generic KV Stores
    GenericKeyValueStoreKey(MapKey),
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct ValidatorByStakeKey {
    pub divided_stake: u16,
    pub validator_address: ComponentAddress,
}

impl TryFrom<SortedU16Key> for ValidatorByStakeKey {
    type Error = DecodeError;

    fn try_from(value: SortedU16Key) -> Result<Self, Self::Error> {
        // See to_sorted_key in validator.rs
        Ok(Self {
            divided_stake: u16::MAX - value.0,
            validator_address: scrypto_decode(&value.1)?,
        })
    }
}

fn error(descriptor: &'static str) -> String {
    format!("Could not convert {} to TypedSubstateKey", descriptor)
}

pub fn to_typed_substate_key(
    entity_type: EntityType,
    partition_num: PartitionNumber,
    substate_key: &SubstateKey,
) -> Result<TypedSubstateKey, String> {
    let substate_type = match partition_num {
        TYPE_INFO_FIELD_PARTITION => TypedSubstateKey::TypeInfoModuleField(
            TypeInfoField::try_from(substate_key).map_err(|_| error("TypeInfoOffset"))?,
        ),
        METADATA_KV_STORE_PARTITION => TypedSubstateKey::MetadataModuleEntryKey(
            scrypto_decode(
                substate_key
                    .for_map()
                    .ok_or_else(|| error("Metadata key"))?,
            )
            .map_err(|_| error("string Metadata key"))?,
        ),
        ROYALTY_FIELD_PARTITION => TypedSubstateKey::RoyaltyModuleField(
            RoyaltyField::try_from(substate_key).map_err(|_| error("RoyaltyOffset"))?,
        ),
        ACCESS_RULES_FIELD_PARTITION => TypedSubstateKey::AccessRulesModuleField(
            AccessRulesField::try_from(substate_key).map_err(|_| error("AccessRulesOffset"))?,
        ),
        partition_num @ _ if partition_num >= OBJECT_BASE_PARTITION => {
            TypedSubstateKey::MainModule(to_typed_object_module_substate_key(
                entity_type,
                partition_num.0 - OBJECT_BASE_PARTITION.0,
                substate_key,
            )?)
        }
        _ => return Err(format!("Unknown partition {:?}", partition_num)),
    };
    Ok(substate_type)
}

pub fn to_typed_object_module_substate_key(
    entity_type: EntityType,
    partition_offset: u8,
    substate_key: &SubstateKey,
) -> Result<TypedMainModuleSubstateKey, String> {
    return to_typed_object_substate_key_internal(entity_type, partition_offset, substate_key)
        .map_err(|_| {
            format!(
                "Could not convert {:?} {:?} key to TypedObjectSubstateKey",
                entity_type, substate_key
            )
        });
}

fn to_typed_object_substate_key_internal(
    entity_type: EntityType,
    partition_offset: u8,
    substate_key: &SubstateKey,
) -> Result<TypedMainModuleSubstateKey, ()> {
    let substate_type = match entity_type {
        EntityType::InternalGenericComponent | EntityType::GlobalGenericComponent => {
            TypedMainModuleSubstateKey::GenericScryptoComponentField(ComponentField::try_from(
                substate_key,
            )?)
        }
        EntityType::GlobalPackage => {
            TypedMainModuleSubstateKey::PackageField(PackageField::try_from(substate_key)?)
        }
        EntityType::GlobalFungibleResource => TypedMainModuleSubstateKey::FungibleResourceField(
            FungibleResourceManagerField::try_from(substate_key)?,
        ),
        EntityType::GlobalNonFungibleResource => {
            let partition_offset =
                NonFungibleResourceManagerPartitionOffset::try_from(partition_offset)?;
            match partition_offset {
                NonFungibleResourceManagerPartitionOffset::ResourceManager => {
                    TypedMainModuleSubstateKey::NonFungibleResourceField(
                        NonFungibleResourceManagerField::try_from(substate_key)?,
                    )
                }
                NonFungibleResourceManagerPartitionOffset::NonFungibleData => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::NonFungibleResourceData(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
            }
        }
        EntityType::GlobalEpochManager => {
            let partition_offset = EpochManagerPartitionOffset::try_from(partition_offset)?;
            match partition_offset {
                EpochManagerPartitionOffset::EpochManager => {
                    TypedMainModuleSubstateKey::EpochManagerField(EpochManagerField::try_from(
                        substate_key,
                    )?)
                }
                EpochManagerPartitionOffset::RegisteredValidatorsByStakeIndex => {
                    let key = substate_key.for_sorted().ok_or(())?;
                    TypedMainModuleSubstateKey::EpochManagerRegisteredValidatorsByStakeIndexKey(
                        key.clone().try_into().map_err(|_| ())?,
                    )
                }
            }
        }
        EntityType::GlobalValidator => {
            TypedMainModuleSubstateKey::ValidatorField(ValidatorField::try_from(substate_key)?)
        }
        EntityType::GlobalClock => {
            TypedMainModuleSubstateKey::ClockField(ClockField::try_from(substate_key)?)
        }
        EntityType::GlobalAccessController => TypedMainModuleSubstateKey::AccessControllerField(
            AccessControllerField::try_from(substate_key)?,
        ),
        EntityType::GlobalVirtualSecp256k1Account
        | EntityType::GlobalVirtualEd25519Account
        | EntityType::InternalAccount
        | EntityType::GlobalAccount => {
            let key = substate_key.for_map().ok_or(())?;
            TypedMainModuleSubstateKey::AccountVaultIndexKey(scrypto_decode(&key).map_err(|_| ())?)
        }
        EntityType::GlobalVirtualSecp256k1Identity
        | EntityType::GlobalVirtualEd25519Identity
        | EntityType::GlobalIdentity => Err(())?, // Identity doesn't have any substates
        EntityType::InternalFungibleVault => TypedMainModuleSubstateKey::FungibleVaultField(
            FungibleVaultField::try_from(substate_key)?,
        ),
        EntityType::InternalNonFungibleVault => {
            let partition_offset = NonFungibleVaultPartitionOffset::try_from(partition_offset)?;

            match partition_offset {
                NonFungibleVaultPartitionOffset::Balance => {
                    TypedMainModuleSubstateKey::NonFungibleVaultField(
                        NonFungibleVaultField::try_from(substate_key)?,
                    )
                }
                NonFungibleVaultPartitionOffset::NonFungibles => {
                    let key = substate_key.for_map().ok_or(())?;
                    TypedMainModuleSubstateKey::NonFungibleVaultContentsIndexKey(
                        scrypto_decode(&key).map_err(|_| ())?,
                    )
                }
            }
        }
        // These seem to be spread between Object and Virtualized SysModules
        EntityType::InternalKeyValueStore => {
            let key = substate_key.for_map().ok_or(())?;
            TypedMainModuleSubstateKey::GenericKeyValueStoreKey(key.clone())
        }
    };
    Ok(substate_type)
}

#[derive(Debug, Clone)]
pub enum TypedSubstateValue {
    TypeInfoModuleFieldValue(TypedTypeInfoModuleFieldValue),
    AccessRulesModuleFieldValue(TypedAccessRulesModuleFieldValue),
    RoyaltyModuleFieldValue(TypedRoyaltyModuleFieldValue),
    MetadataModuleEntryValue(MetadataValueSubstate),
    MainModule(TypedMainModuleSubstateValue),
}

#[derive(Debug, Clone)]
pub enum TypedTypeInfoModuleFieldValue {
    TypeInfo(TypeInfoSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccessRulesModuleFieldValue {
    MethodAccessRules(MethodAccessRulesSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedRoyaltyModuleFieldValue {
    ComponentRoyaltyConfig(ComponentRoyaltyConfigSubstate),
    ComponentRoyaltyAccumulator(ComponentRoyaltyAccumulatorSubstate),
}

/// Contains all the main module substate values, by each known partition layout
#[derive(Debug, Clone)]
pub enum TypedMainModuleSubstateValue {
    // Objects
    Package(TypedPackageFieldValue),
    FungibleResource(TypedFungibleResourceManagerFieldValue),
    NonFungibleResource(TypedNonFungibleResourceManagerFieldValue),
    NonFungibleResourceData(Option<ScryptoOwnedRawValue>),
    FungibleVault(TypedFungibleVaultFieldValue),
    NonFungibleVaultField(TypedNonFungibleVaultFieldValue),
    NonFungibleVaultContentsIndexEntry(NonFungibleVaultContentsEntry),
    EpochManagerField(TypedEpochManagerFieldValue),
    EpochManagerRegisteredValidatorsByStakeIndexEntry(EpochRegisteredValidatorByStakeEntry),
    Clock(TypedClockFieldValue),
    Validator(TypedValidatorFieldValue),
    AccountVaultIndex(AccountVaultIndexEntry), // (We don't yet have account fields yet)
    AccessController(TypedAccessControllerFieldValue),
    // Generic Scrypto Components and KV Stores
    GenericScryptoComponent(GenericScryptoComponentFieldValue),
    GenericKeyValueStore(Option<ScryptoOwnedRawValue>),
}

#[derive(Debug, Clone)]
pub enum TypedPackageFieldValue {
    Info(PackageInfoSubstate),
    CodeType(PackageCodeTypeSubstate),
    Code(PackageCodeSubstate),
    Royalty(PackageRoyaltySubstate),
    FunctionAccessRules(PackageFunctionAccessRulesSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedFungibleResourceManagerFieldValue {
    Divisibility(FungibleResourceManagerDivisibilitySubstate),
    TotalSupply(FungibleResourceManagerTotalSupplySubstate),
}

#[derive(Debug, Clone)]
pub enum TypedNonFungibleResourceManagerFieldValue {
    IdType(NonFungibleResourceManagerIdTypeSubstate),
    MutableFields(NonFungibleResourceManagerMutableFieldsSubstate),
    TotalSupply(NonFungibleResourceManagerTotalSupplySubstate),
}

#[derive(Debug, Clone)]
pub enum TypedFungibleVaultFieldValue {
    Balance(FungibleVaultBalanceSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedNonFungibleVaultFieldValue {
    Balance(NonFungibleVaultBalanceSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedEpochManagerFieldValue {
    Config(EpochManagerConfigSubstate),
    EpochManager(EpochManagerSubstate),
    CurrentValidatorSet(CurrentValidatorSetSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedClockFieldValue {
    CurrentTimeRoundedToMinutes(ClockSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedValidatorFieldValue {
    Validator(ValidatorSubstate),
}

#[derive(Debug, Clone)]
pub enum TypedAccessControllerFieldValue {
    AccessController(AccessControllerSubstate),
}

#[derive(Debug, Clone)]
pub enum GenericScryptoComponentFieldValue {
    State(GenericScryptoSborPayload),
}

#[derive(Debug, Clone)]
pub struct GenericScryptoSborPayload {
    pub data: Vec<u8>,
}

pub fn to_typed_substate_value(
    substate_key: &TypedSubstateKey,
    data: &[u8],
) -> Result<TypedSubstateValue, String> {
    to_typed_substate_value_internal(substate_key, data).map_err(|err| {
        format!(
            "Error decoding substate data for key {:?} - {:?}",
            substate_key, err
        )
    })
}

fn to_typed_substate_value_internal(
    substate_key: &TypedSubstateKey,
    data: &[u8],
) -> Result<TypedSubstateValue, DecodeError> {
    let substate_value = match substate_key {
        TypedSubstateKey::TypeInfoModuleField(type_info_offset) => {
            TypedSubstateValue::TypeInfoModuleFieldValue(match type_info_offset {
                TypeInfoField::TypeInfo => {
                    TypedTypeInfoModuleFieldValue::TypeInfo(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::AccessRulesModuleField(access_rules_offset) => {
            TypedSubstateValue::AccessRulesModuleFieldValue(match access_rules_offset {
                AccessRulesField::AccessRules => {
                    TypedAccessRulesModuleFieldValue::MethodAccessRules(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::RoyaltyModuleField(royalty_offset) => {
            TypedSubstateValue::RoyaltyModuleFieldValue(match royalty_offset {
                RoyaltyField::RoyaltyConfig => {
                    TypedRoyaltyModuleFieldValue::ComponentRoyaltyConfig(scrypto_decode(data)?)
                }
                RoyaltyField::RoyaltyAccumulator => {
                    TypedRoyaltyModuleFieldValue::ComponentRoyaltyAccumulator(scrypto_decode(data)?)
                }
            })
        }
        TypedSubstateKey::MetadataModuleEntryKey(_) => {
            TypedSubstateValue::MetadataModuleEntryValue(scrypto_decode(data)?)
        }
        TypedSubstateKey::MainModule(object_substate_key) => TypedSubstateValue::MainModule(
            to_typed_object_substate_value(object_substate_key, data)?,
        ),
    };
    Ok(substate_value)
}

fn to_typed_object_substate_value(
    substate_key: &TypedMainModuleSubstateKey,
    data: &[u8],
) -> Result<TypedMainModuleSubstateValue, DecodeError> {
    let substate_value = match substate_key {
        TypedMainModuleSubstateKey::PackageField(offset) => {
            TypedMainModuleSubstateValue::Package(match offset {
                PackageField::Info => TypedPackageFieldValue::Info(scrypto_decode(data)?),
                PackageField::CodeType => TypedPackageFieldValue::CodeType(scrypto_decode(data)?),
                PackageField::Code => TypedPackageFieldValue::Code(scrypto_decode(data)?),
                PackageField::Royalty => TypedPackageFieldValue::Royalty(scrypto_decode(data)?),
                PackageField::FunctionAccessRules => {
                    TypedPackageFieldValue::FunctionAccessRules(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::FungibleResourceField(offset) => {
            TypedMainModuleSubstateValue::FungibleResource(match offset {
                FungibleResourceManagerField::Divisibility => {
                    TypedFungibleResourceManagerFieldValue::Divisibility(scrypto_decode(data)?)
                }
                FungibleResourceManagerField::TotalSupply => {
                    TypedFungibleResourceManagerFieldValue::TotalSupply(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::NonFungibleResourceField(offset) => {
            TypedMainModuleSubstateValue::NonFungibleResource(match offset {
                NonFungibleResourceManagerField::IdType => {
                    TypedNonFungibleResourceManagerFieldValue::IdType(scrypto_decode(data)?)
                }
                NonFungibleResourceManagerField::MutableFields => {
                    TypedNonFungibleResourceManagerFieldValue::MutableFields(scrypto_decode(data)?)
                }
                NonFungibleResourceManagerField::TotalSupply => {
                    TypedNonFungibleResourceManagerFieldValue::TotalSupply(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::NonFungibleResourceData(_) => {
            TypedMainModuleSubstateValue::NonFungibleResourceData(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::FungibleVaultField(offset) => {
            TypedMainModuleSubstateValue::FungibleVault(match offset {
                FungibleVaultField::LiquidFungible => {
                    TypedFungibleVaultFieldValue::Balance(scrypto_decode(data)?)
                }
                // This shouldn't be persistable - so use a bizarre (but temporary!) placeholder error code here!
                FungibleVaultField::LockedFungible => Err(DecodeError::InvalidCustomValue)?,
            })
        }
        TypedMainModuleSubstateKey::NonFungibleVaultField(offset) => {
            TypedMainModuleSubstateValue::NonFungibleVaultField(match offset {
                NonFungibleVaultField::LiquidNonFungible => {
                    TypedNonFungibleVaultFieldValue::Balance(scrypto_decode(data)?)
                }
                // This shouldn't be persistable - so use a bizarre (but temporary!) placeholder error code here!
                NonFungibleVaultField::LockedNonFungible => Err(DecodeError::InvalidCustomValue)?,
            })
        }
        TypedMainModuleSubstateKey::NonFungibleVaultContentsIndexKey(_) => {
            TypedMainModuleSubstateValue::NonFungibleVaultContentsIndexEntry(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::EpochManagerField(offset) => {
            TypedMainModuleSubstateValue::EpochManagerField(match offset {
                EpochManagerField::Config => {
                    TypedEpochManagerFieldValue::Config(scrypto_decode(data)?)
                }
                EpochManagerField::EpochManager => {
                    TypedEpochManagerFieldValue::EpochManager(scrypto_decode(data)?)
                }
                EpochManagerField::CurrentValidatorSet => {
                    TypedEpochManagerFieldValue::CurrentValidatorSet(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::EpochManagerRegisteredValidatorsByStakeIndexKey(_) => {
            TypedMainModuleSubstateValue::EpochManagerRegisteredValidatorsByStakeIndexEntry(
                scrypto_decode(data)?,
            )
        }
        TypedMainModuleSubstateKey::ClockField(offset) => {
            TypedMainModuleSubstateValue::Clock(match offset {
                ClockField::CurrentTimeRoundedToMinutes => {
                    TypedClockFieldValue::CurrentTimeRoundedToMinutes(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::ValidatorField(offset) => {
            TypedMainModuleSubstateValue::Validator(match offset {
                ValidatorField::Validator => {
                    TypedValidatorFieldValue::Validator(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::AccountVaultIndexKey(_) => {
            TypedMainModuleSubstateValue::AccountVaultIndex(scrypto_decode(data)?)
        }
        TypedMainModuleSubstateKey::AccessControllerField(offset) => {
            TypedMainModuleSubstateValue::AccessController(match offset {
                AccessControllerField::AccessController => {
                    TypedAccessControllerFieldValue::AccessController(scrypto_decode(data)?)
                }
            })
        }
        TypedMainModuleSubstateKey::GenericScryptoComponentField(offset) => {
            TypedMainModuleSubstateValue::GenericScryptoComponent(match offset {
                ComponentField::State0 => {
                    GenericScryptoComponentFieldValue::State(GenericScryptoSborPayload {
                        data: data.to_vec(),
                    })
                }
            })
        }
        TypedMainModuleSubstateKey::GenericKeyValueStoreKey(_) => {
            TypedMainModuleSubstateValue::GenericKeyValueStore(scrypto_decode(data)?)
        }
    };
    Ok(substate_value)
}
