use crate::errors::RuntimeError;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::blueprints::resource::Resource;
use sbor::rust::collections::BTreeMap;

use super::actor::ResolvedActor;
use super::call_frame::CallFrameUpdate;
use super::kernel_api::{KernelModuleApi, LockFlags};

pub trait KernelModule {
    //======================
    // Kernel module setup
    //======================
    #[inline(always)]
    fn on_init<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Invocation events
    //
    // -> BeforeInvoke
    // -> BeforePushFrame
    //        -> ExecutionStart
    //        -> ExecutionFinish
    // -> AfterPopFrame
    // -> AfterInvoke
    //======================

    #[inline(always)]
    fn before_invoke<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _fn_identifier: &FnIdentifier,
        _input_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &ResolvedActor,
        _down_movement: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_execution_start<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _caller: &ResolvedActor,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _caller: &ResolvedActor,
        _up_movement: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_pop_frame<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_invoke<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _output_size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // RENode events
    //======================

    #[inline(always)]
    fn on_allocate_node_id<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_type: &RENodeType,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_create_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn before_drop_node<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_drop_node<Y: KernelModuleApi<RuntimeError>>(_api: &mut Y) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Substate events
    //======================

    #[inline(always)]
    fn before_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _node_id: &RENodeId,
        _module_id: &NodeModuleId,
        _offset: &SubstateOffset,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn after_lock_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_read_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_write_substate<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_drop_lock<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    //======================
    // Other events
    //======================

    #[inline(always)]
    fn on_consume_cost_units<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _units: u32,
        _reason: ClientCostingReason,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_credit_cost_units<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _vault_id: VaultId,
        locked_fee: Resource,
        _contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        Ok(locked_fee)
    }

    #[inline(always)]
    fn on_update_instruction_index<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _new_index: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }

    #[inline(always)]
    fn on_update_wasm_memory_usage<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _consumed_memory: usize,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}