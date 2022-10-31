use crate::engine::{LockFlags, RENode, SystemApi};
use crate::fee::FeeReserve;
use crate::model::{InvokeError, ResourceOperationError};
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ProofError {
    /// Error produced by a resource container.
    ResourceOperationError(ResourceOperationError),
    /// Can't generate zero-amount or empty non-fungible set proofs.
    EmptyProofNotAllowed,
    /// The base proofs are not enough to cover the requested amount or non-fungible ids.
    InsufficientBaseProofs,
    /// Can't apply a non-fungible operation on fungible proofs.
    NonFungibleOperationNotAllowed,
    /// Can't apply a fungible operation on non-fungible proofs.
    FungibleOperationNotAllowed,
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
}

pub struct Proof;

impl Proof {
    pub fn main<'s, Y, R>(
        proof_id: ProofId,
        method: ProofMethod,
        args: ScryptoValue,
        system_api: &mut Y,
    ) -> Result<ScryptoValue, InvokeError<ProofError>>
    where
        Y: SystemApi<'s, R>,
        R: FeeReserve,
    {
        let node_id = RENodeId::Proof(proof_id);
        let offset = SubstateOffset::Proof(ProofOffset::Proof);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
        let substate_ref = system_api.get_ref(handle)?;
        let proof = substate_ref.proof();

        let rtn = match method {
            ProofMethod::GetAmount => {
                let _: ProofGetAmountInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                ScryptoValue::from_typed(&proof.total_amount())
            }
            ProofMethod::GetNonFungibleIds => {
                let _: ProofGetNonFungibleIdsInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                ScryptoValue::from_typed(&proof.total_ids()?)
            }
            ProofMethod::GetResourceAddress => {
                let _: ProofGetResourceAddressInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                ScryptoValue::from_typed(&proof.resource_address)
            }
            ProofMethod::Clone => {
                let _: ProofCloneInput = scrypto_decode(&args.raw)
                    .map_err(|e| InvokeError::Error(ProofError::InvalidRequestData(e)))?;
                let cloned_proof = proof.clone();
                let proof_id = system_api.create_node(RENode::Proof(cloned_proof))?.into();
                ScryptoValue::from_typed(&scrypto::resource::Proof(proof_id))
            }
        };

        Ok(rtn)
    }
}