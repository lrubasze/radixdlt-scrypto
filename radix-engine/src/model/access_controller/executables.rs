use super::state_machine::*;
use super::*;
use crate::engine::{deref_and_update, ApplicationError, Executor, LockFlags, RENodeInit};
use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::{AccessRulesChainSubstate, GlobalAddressSubstate};
use crate::wasm::WasmEngine;
use native_sdk::resource::{SysBucket, Vault};
use radix_engine_interface::api::types::*;
use radix_engine_interface::constants::{CLOCK, PACKAGE_TOKEN};
use radix_engine_interface::data::scrypto_encode;
use radix_engine_interface::*;
use radix_engine_interface::{api::*, rule};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum AccessControllerError {
    /// Occurs when some action requires that the primary role is unlocked to happen.
    OperationRequiresUnlockedPrimaryRole,

    /// Occurs when some action requires that the primary role is locked to happen.
    OperationRequiresLockedPrimaryRole,

    /// Occurs when adding time to an [`Instant`] results in an overflow
    TimeOverflow,

    /// Occurs when a proposer attempts to initiate another recovery when they already have a
    /// recovery underway.
    RecoveryForThisProposerAlreadyExists { proposer: Proposer },

    /// Occurs when no recovery can be found for a given proposer.
    RecoveryForThisProposerDoesNotExist { proposer: Proposer },

    /// Occurs when there is no timed recoveries on the controller - typically because it isn't in
    /// the state that allows for it.
    NoTimedRecoveriesFound,

    /// Occurs when performing a quick confirm recovery if the proposer and the confirmor are the
    /// same.
    ProposerAndConfirmorAreTheSame,

    /// Occurs when trying to perform a timed confirm recovery on a recovery proposal that could
    /// be time-confirmed but whose delay has not yet elapsed.
    TimedRecoveryDelayHasNotElapsed,

    /// Occurs when the expected recovery proposal doesn't match that which was found
    RecoveryProposalMismatch {
        expected: RecoveryProposal,
        found: RecoveryProposal,
    },
}

impl From<AccessControllerError> for RuntimeError {
    fn from(value: AccessControllerError) -> Self {
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(value))
    }
}

//=================================
// Access Controller Create Global
//=================================

impl ExecutableInvocation for AccessControllerCreateGlobalInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor =
            ResolvedActor::function(NativeFn::AccessController(AccessControllerFn::CreateGlobal));
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.controlled_asset));

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerCreateGlobalInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Creating a new vault and putting in it the controlled asset
        let vault = {
            let controlled_asset = Bucket(self.controlled_asset);

            let mut vault = controlled_asset
                .sys_resource_address(api)
                .and_then(|resource_address| Vault::sys_new(resource_address, api))?;
            vault.sys_put(controlled_asset, api)?;

            vault
        };

        // Constructing the Access Controller RENode and Substates
        let access_controller = RENodeInit::AccessController(
            AccessControllerSubstate::new(vault.0, self.timed_recovery_delay_in_minutes),
            AccessRulesChainSubstate {
                access_rules_chain: [access_rules_from_rule_set(self.rule_set)].into(),
            },
        );

        // Allocating an RENodeId and creating the access controller RENode
        let node_id = api.allocate_node_id(RENodeType::AccessController)?;
        api.create_node(node_id, access_controller)?;

        // Creating a global component address for the access controller RENode
        let global_node_id = api.allocate_node_id(RENodeType::GlobalAccessController)?;
        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::AccessController(node_id.into())),
        )?;

        Ok((global_node_id.into(), CallFrameUpdate::empty()))
    }
}

//================================
// Access Controller Create Proof
//================================

pub struct AccessControllerCreateProofExecutable {
    pub receiver: RENodeId,
}

impl ExecutableInvocation for AccessControllerCreateProofInvocation {
    type Exec = AccessControllerCreateProofExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CreateProof),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerCreateProofExecutable {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let proof = transition(
            self.receiver,
            api,
            AccessControllerCreateProofStateMachineInput,
        )?;
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Proof(proof.0));

        Ok((proof, call_frame_update))
    }
}

//=====================================
// Access Controller Initiate Recovery
//=====================================

pub struct AccessControllerInitiateRecoveryExecutable {
    pub receiver: RENodeId,
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerInitiateRecoveryAsPrimaryInvocation {
    type Exec = AccessControllerInitiateRecoveryExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(CLOCK)));

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsPrimary),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposer: Proposer::Primary,
            proposal: self.proposal,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl ExecutableInvocation for AccessControllerInitiateRecoveryAsRecoveryInvocation {
    type Exec = AccessControllerInitiateRecoveryExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(CLOCK)));

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposer: Proposer::Recovery,
            proposal: self.proposal,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerInitiateRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerInitiateRecoveryStateMachineInput {
                proposer: self.proposer,
                proposal: self.proposal,
            },
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//==========================================
// Access Controller Quick Confirm Recovery
//==========================================

pub struct AccessControllerQuickConfirmRecoveryExecutable {
    pub receiver: RENodeId,
    pub confirmor: Role,
    pub proposer: Proposer,
    pub proposal_to_confirm: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryAsPrimaryInvocation {
    type Exec = AccessControllerQuickConfirmRecoveryExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::QuickConfirmRecoveryAsPrimary),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            confirmor: Role::Primary,
            proposer: self.proposer,
            proposal_to_confirm: self.proposal_to_confirm,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryAsRecoveryInvocation {
    type Exec = AccessControllerQuickConfirmRecoveryExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::QuickConfirmRecoveryAsRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            confirmor: Role::Recovery,
            proposer: self.proposer,
            proposal_to_confirm: self.proposal_to_confirm,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryAsConfirmationInvocation {
    type Exec = AccessControllerQuickConfirmRecoveryExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::QuickConfirmRecoveryAsConfirmation),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            confirmor: Role::Confirmation,
            proposer: self.proposer,
            proposal_to_confirm: self.proposal_to_confirm,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerQuickConfirmRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let recovery_proposal = transition_mut(
            self.receiver,
            api,
            AccessControllerQuickConfirmRecoveryStateMachineInput {
                confirmor: self.confirmor,
                proposer: self.proposer,
                proposal_to_confirm: self.proposal_to_confirm,
            },
        )?;

        // Update the access rules
        let new_access_rules = access_rules_from_rule_set(recovery_proposal.rule_set);
        for (group_name, access_rule) in new_access_rules.get_all_grouped_auth().iter() {
            api.invoke(AccessRulesSetGroupAccessRuleInvocation {
                receiver: self.receiver,
                index: 0,
                name: group_name.into(),
                rule: access_rule.clone(),
            })?;
        }
        for (method_key, entry) in new_access_rules.get_all_method_auth().iter() {
            match entry {
                AccessRuleEntry::AccessRule(access_rule) => {
                    api.invoke(AccessRulesSetMethodAccessRuleInvocation {
                        receiver: self.receiver,
                        index: 0,
                        key: method_key.clone(),
                        rule: access_rule.clone(),
                    })?;
                }
                AccessRuleEntry::Group(..) => {} // Already updated above
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

//==========================================
// Access Controller Timed Confirm Recovery
//==========================================

pub struct AccessControllerTimedConfirmRecoveryExecutable {
    pub receiver: RENodeId,
    pub proposal_to_confirm: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerTimedConfirmRecoveryInvocation {
    type Exec = AccessControllerTimedConfirmRecoveryExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(CLOCK)));
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::TimedConfirmRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposal_to_confirm: self.proposal_to_confirm,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerTimedConfirmRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let recovery_proposal = transition_mut(
            self.receiver,
            api,
            AccessControllerTimedConfirmRecoveryStateMachineInput {
                proposal_to_confirm: self.proposal_to_confirm,
            },
        )?;

        // Update the access rules
        let new_access_rules = access_rules_from_rule_set(recovery_proposal.rule_set);
        for (group_name, access_rule) in new_access_rules.get_all_grouped_auth().iter() {
            api.invoke(AccessRulesSetGroupAccessRuleInvocation {
                receiver: self.receiver,
                index: 0,
                name: group_name.into(),
                rule: access_rule.clone(),
            })?;
        }
        for (method_key, entry) in new_access_rules.get_all_method_auth().iter() {
            match entry {
                AccessRuleEntry::AccessRule(access_rule) => {
                    api.invoke(AccessRulesSetMethodAccessRuleInvocation {
                        receiver: self.receiver,
                        index: 0,
                        key: method_key.clone(),
                        rule: access_rule.clone(),
                    })?;
                }
                AccessRuleEntry::Group(..) => {} // Already updated above
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

//===========================================
// Access Controller Cancel Recovery Attempt
//===========================================

pub struct AccessControllerCancelRecoveryAttemptExecutable {
    pub receiver: RENodeId,
    pub proposer: Proposer,
}

impl ExecutableInvocation for AccessControllerCancelRecoveryAttemptAsPrimaryInvocation {
    type Exec = AccessControllerCancelRecoveryAttemptExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CancelRecoveryAttemptAsPrimary),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposer: Proposer::Primary,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl ExecutableInvocation for AccessControllerCancelRecoveryAttemptAsRecoveryInvocation {
    type Exec = AccessControllerCancelRecoveryAttemptExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CancelRecoveryAttemptAsRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposer: Proposer::Recovery,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerCancelRecoveryAttemptExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerCancelRecoveryAttemptStateMachineInput {
                proposer: self.proposer,
            },
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=====================================
// Access Controller Lock Primary Role
//=====================================

pub struct AccessControllerLockPrimaryRoleExecutable {
    pub receiver: RENodeId,
}

impl ExecutableInvocation for AccessControllerLockPrimaryRoleInvocation {
    type Exec = AccessControllerLockPrimaryRoleExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::LockPrimaryRole),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerLockPrimaryRoleExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerLockPrimaryRoleStateMachineInput,
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=======================================
// Access Controller Unlock Primary Role
//=======================================

pub struct AccessControllerUnlockPrimaryRoleExecutable {
    pub receiver: RENodeId,
}

impl ExecutableInvocation for AccessControllerUnlockPrimaryRoleInvocation {
    type Exec = AccessControllerUnlockPrimaryRoleExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::UnlockPrimaryRole),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerUnlockPrimaryRoleExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerUnlockPrimaryRoleStateMachineInput,
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=======================================
// Access Controller Stop Timed Recovery
//=======================================

pub struct AccessControllerStopTimedRecoveryExecutable {
    pub receiver: RENodeId,
    pub proposal: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerStopTimedRecoveryInvocation {
    type Exec = AccessControllerStopTimedRecoveryExecutable;

    fn resolve<D: ResolverApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::StopTimedRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposal: self.proposal,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerStopTimedRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerStopTimedRecoveryStateMachineInput {
                proposal: self.proposal,
            },
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

fn access_rule_or(access_rules: Vec<AccessRule>) -> AccessRule {
    let mut rule_nodes = Vec::new();
    for access_rule in access_rules.into_iter() {
        match access_rule {
            AccessRule::AllowAll => return AccessRule::AllowAll,
            AccessRule::DenyAll => {}
            AccessRule::Protected(rule_node) => rule_nodes.push(rule_node),
        }
    }
    AccessRule::Protected(AccessRuleNode::AnyOf(rule_nodes))
}

//=========
// Helpers
//=========

fn access_rules_from_rule_set(rule_set: RuleSet) -> AccessRules {
    let mut access_rules = AccessRules::new();

    // Primary Role Rules
    let primary_group = "primary";
    access_rules.set_group_access_rule(primary_group.into(), rule_set.primary_role.clone());
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(AccessControllerFn::CreateProof)),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::InitiateRecoveryAsPrimary,
        )),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::QuickConfirmRecoveryAsPrimary,
        )),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::CancelRecoveryAttemptAsPrimary,
        )),
        primary_group.into(),
    );

    // Recovery Role Rules
    let recovery_group = "recovery";
    access_rules.set_group_access_rule(recovery_group.into(), rule_set.recovery_role.clone());
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::InitiateRecoveryAsRecovery,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::QuickConfirmRecoveryAsRecovery,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::TimedConfirmRecovery,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::CancelRecoveryAttemptAsRecovery,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::LockPrimaryRole,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::UnlockPrimaryRole,
        )),
        recovery_group.into(),
    );

    // Confirmation Role Rules
    let confirmation_group = "confirmation";
    access_rules.set_group_access_rule(
        confirmation_group.into(),
        rule_set.confirmation_role.clone(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::QuickConfirmRecoveryAsConfirmation,
        )),
        confirmation_group.into(),
    );

    // Other methods
    access_rules.set_method_access_rule(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::StopTimedRecovery,
        )),
        access_rule_or(
            [
                rule_set.primary_role,
                rule_set.recovery_role,
                rule_set.confirmation_role,
            ]
            .into(),
        ),
    );

    let non_fungible_local_id = NonFungibleLocalId::Bytes(
        scrypto_encode(&PackageIdentifier::Native(NativePackage::AccessController)).unwrap(),
    );
    let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);

    access_rules.default(rule!(deny_all), rule!(require(non_fungible_global_id)))
}

fn transition<Y, I>(
    node_id: RENodeId,
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerSubstate as Transition<I>>::Output, RuntimeError>
where
    Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    AccessControllerSubstate: Transition<I>,
{
    let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
    let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;

    let access_controller_clone = {
        let substate = api.get_ref(handle)?;
        let access_controller = substate.access_controller();
        access_controller.clone()
    };

    let rtn = access_controller_clone.transition(api, input)?;

    api.drop_lock(handle)?;

    Ok(rtn)
}

fn transition_mut<Y, I>(
    node_id: RENodeId,
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerSubstate as TransitionMut<I>>::Output, RuntimeError>
where
    Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    AccessControllerSubstate: TransitionMut<I>,
{
    let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
    let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

    let mut access_controller_clone = {
        let substate = api.get_ref(handle)?;
        let access_controller = substate.access_controller();
        access_controller.clone()
    };

    let rtn = access_controller_clone.transition_mut(api, input)?;

    {
        let mut substate = api.get_ref_mut(handle)?;
        let access_controller = substate.access_controller();
        *access_controller = access_controller_clone
    }

    api.drop_lock(handle)?;

    Ok(rtn)
}
