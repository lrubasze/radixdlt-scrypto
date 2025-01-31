use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::api::api::*;
use crate::api::types::AuthZoneStackId;
use crate::math::Decimal;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZonePopInvocation {
    pub receiver: AuthZoneStackId,
}

impl Invocation for AuthZonePopInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AuthZonePopInvocation {
    type ScryptoOutput = Proof;
}

impl Into<SerializedInvocation> for AuthZonePopInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::Pop(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZonePushInvocation {
    pub receiver: AuthZoneStackId,
    pub proof: Proof,
}

impl Invocation for AuthZonePushInvocation {
    type Output = ();
}

impl SerializableInvocation for AuthZonePushInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for AuthZonePushInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::Push(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInvocation {
    pub receiver: AuthZoneStackId,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AuthZoneCreateProofInvocation {
    type ScryptoOutput = Proof;
}

impl Into<SerializedInvocation> for AuthZoneCreateProofInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::CreateProof(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInvocation {
    pub receiver: AuthZoneStackId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofByAmountInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AuthZoneCreateProofByAmountInvocation {
    type ScryptoOutput = Proof;
}

impl Into<SerializedInvocation> for AuthZoneCreateProofByAmountInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::CreateProofByAmount(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInvocation {
    pub receiver: AuthZoneStackId,
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofByIdsInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AuthZoneCreateProofByIdsInvocation {
    type ScryptoOutput = Proof;
}

impl Into<SerializedInvocation> for AuthZoneCreateProofByIdsInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::CreateProofByIds(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneClearInvocation {
    pub receiver: AuthZoneStackId,
}

impl Invocation for AuthZoneClearInvocation {
    type Output = ();
}

impl SerializableInvocation for AuthZoneClearInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for AuthZoneClearInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::Clear(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneDrainInvocation {
    pub receiver: AuthZoneStackId,
}

impl Invocation for AuthZoneDrainInvocation {
    type Output = Vec<Proof>;
}

impl SerializableInvocation for AuthZoneDrainInvocation {
    type ScryptoOutput = Vec<Proof>;
}

impl Into<SerializedInvocation> for AuthZoneDrainInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::Drain(self),
        ))
        .into()
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneAssertAccessRuleInvocation {
    pub receiver: AuthZoneStackId,
    pub access_rule: AccessRule,
}

impl Invocation for AuthZoneAssertAccessRuleInvocation {
    type Output = ();
}

impl SerializableInvocation for AuthZoneAssertAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for AuthZoneAssertAccessRuleInvocation {
    fn into(self) -> SerializedInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::AssertAuthRule(self),
        ))
        .into()
    }
}
