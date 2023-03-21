use crate::*;
use radix_engine_common::crypto::Hash;
use sbor::rust::fmt::Debug;

pub const TRANSACTION_RUNTIME_BLUEPRINT: &str = "TransactionRuntime";

pub const TRANSACTION_RUNTIME_GET_HASH_IDENT: &str = "get_hash";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionRuntimeGetHashInput {}

pub type TransactionRuntimeGetHashOutput = Hash;

pub const TRANSACTION_RUNTIME_GENERATE_UUID_IDENT: &str = "generate_uuid";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct TransactionRuntimeGenerateUuidInput {}

pub type TransactionRuntimeGenerateUuidInputOutput = u128;