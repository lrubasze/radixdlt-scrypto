use radix_engine_common::data::scrypto::model::*;
use radix_engine_interface::api::node_modules::metadata::MetadataEntry;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::{AccessRule, AccessRulesConfig, MethodKey};
use radix_engine_interface::data::manifest::model::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub enum Instruction {
    /// Takes resource from worktop.
    TakeFromWorktop {
        resource_address: ResourceAddress,
    },

    /// Takes resource from worktop by the given amount.
    TakeFromWorktopByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Takes resource from worktop by the given non-fungible IDs.
    TakeFromWorktopByIds {
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    },

    /// Returns a bucket of resource to worktop.
    ReturnToWorktop {
        bucket_id: ManifestBucket,
    },

    /// Asserts worktop contains resource.
    AssertWorktopContains {
        resource_address: ResourceAddress,
    },

    /// Asserts worktop contains resource by at least the given amount.
    AssertWorktopContainsByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Asserts worktop contains resource by at least the given non-fungible IDs.
    AssertWorktopContainsByIds {
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    },

    /// Takes the last proof from the auth zone.
    PopFromAuthZone,

    /// Adds a proof to the auth zone.
    PushToAuthZone {
        proof_id: ManifestProof,
    },

    /// Drops all proofs in the auth zone
    ClearAuthZone,

    // TODO: do we need `CreateProofFromWorktop`, to avoid taking resource out and then creating proof?
    /// Creates a proof from the auth zone
    CreateProofFromAuthZone {
        resource_address: ResourceAddress,
    },

    /// Creates a proof from the auth zone, by the given amount
    CreateProofFromAuthZoneByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Creates a proof from the auth zone, by the given non-fungible IDs.
    CreateProofFromAuthZoneByIds {
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    },

    /// Creates a proof from a bucket.
    CreateProofFromBucket {
        bucket_id: ManifestBucket,
    },

    /// Clones a proof.
    CloneProof {
        proof_id: ManifestProof,
    },

    /// Drops a proof.
    DropProof {
        proof_id: ManifestProof,
    },

    /// Drops all of the proofs in the transaction.
    DropAllProofs,

    /// Publish a package.
    PublishPackage {
        code: ManifestBlobRef,
        schema: ManifestBlobRef,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        access_rules: AccessRulesConfig,
    },

    BurnResource {
        bucket_id: ManifestBucket,
    },

    RecallResource {
        vault_id: ObjectId,
        amount: Decimal,
    },

    SetMetadata {
        entity_address: ManifestAddress,
        key: String,
        value: MetadataEntry,
    },

    RemoveMetadata {
        entity_address: ManifestAddress,
        key: String,
    },

    SetPackageRoyaltyConfig {
        package_address: PackageAddress,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
    },

    SetComponentRoyaltyConfig {
        component_address: ComponentAddress,
        royalty_config: RoyaltyConfig,
    },

    ClaimPackageRoyalty {
        package_address: PackageAddress,
    },

    ClaimComponentRoyalty {
        component_address: ComponentAddress,
    },

    SetMethodAccessRule {
        entity_address: ManifestAddress,
        key: MethodKey,
        rule: AccessRule,
    },

    MintFungible {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    MintNonFungible {
        resource_address: ResourceAddress,
        entries: BTreeMap<NonFungibleLocalId, Vec<u8>>,
    },

    MintUuidNonFungible {
        resource_address: ResourceAddress,
        entries: Vec<Vec<u8>>,
    },

    AssertAccessRule {
        access_rule: AccessRule,
    },

    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: Vec<u8>,
    },

    CallMethod {
        component_address: ComponentAddress,
        method_name: String,
        args: Vec<u8>,
    },
}
