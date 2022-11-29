use crate::engine::*;
use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, ComponentOffset, GlobalAddress, NativeFunction, NativeMethod,
    PackageOffset, RENodeId, ResourceManagerOffset, SubstateOffset, VaultOffset,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AuthError {
    VisibilityError(RENodeId),
    Unauthorized {
        actor: REActor,
        authorization: MethodAuthorization,
        error: MethodAuthorizationError,
    },
}

pub struct AuthModule;

impl AuthModule {
    pub fn supervisor_id() -> NonFungibleId {
        NonFungibleId::NumberU32(0)
    }

    pub fn system_id() -> NonFungibleId {
        NonFungibleId::NumberU32(1)
    }

    pub fn on_call_frame_enter<Y: SystemApi>(
        call_frame_update: &mut CallFrameUpdate,
        actor: &REActor,
        system_api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let refed = system_api.get_visible_node_ids()?;
        let auth_zone_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::AuthZoneStack(..)))
            .unwrap();
        call_frame_update.node_refs_to_copy.insert(auth_zone_id);

        if !matches!(
            actor,
            REActor::Method(ResolvedMethod::Native(NativeMethod::AuthZoneStack(..)), ..)
        ) {
            let handle = system_api.lock_substate(
                auth_zone_id,
                SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
                LockFlags::MUTABLE,
            )?;
            let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
            let auth_zone_ref_mut = substate_ref_mut.auth_zone();

            // New auth zone frame managed by the AuthModule
            auth_zone_ref_mut.new_frame(actor);
            system_api.drop_lock(handle)?;
        }

        Ok(())
    }

    pub fn on_before_frame_start<Y, X>(
        actor: &REActor,
        executor: &X,
        system_api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: SystemApi,
        X: Executor,
    {
        if matches!(
            actor,
            REActor::Method(ResolvedMethod::Native(NativeMethod::AuthZoneStack(..)), ..)
        ) {
            return Ok(());
        }

        let method_auths = match actor.clone() {
            REActor::Function(function_ident) => match function_ident {
                ResolvedFunction::Native(NativeFunction::EpochManager(system_func)) => {
                    EpochManager::function_auth(&system_func)
                }
                _ => vec![],
            },
            REActor::Method(method, resolved_receiver) => {
                match (method, resolved_receiver) {
                    (ResolvedMethod::Native(NativeMethod::Metadata(MetadataMethod::Set)), ..) => {
                        if let Some((
                            RENodeId::Global(GlobalAddress::Package(package_address)),
                            ..,
                        )) = resolved_receiver.derefed_from
                        {
                            // TODO: Cleanup package address + NonFungibleId integration
                            let bytes = scrypto_encode(&package_address).unwrap();

                            let non_fungible_id = NonFungibleId::from_bytes(bytes);
                            let non_fungible_address =
                                NonFungibleAddress::new(ENTITY_OWNER_TOKEN, non_fungible_id);
                            vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
                                HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                                    non_fungible_address,
                                )),
                            ))]
                        } else {
                            vec![MethodAuthorization::DenyAll]
                        }
                    }
                    (
                        ResolvedMethod::Native(NativeMethod::ResourceManager(ref method)),
                        ResolvedReceiver {
                            receiver: RENodeId::ResourceManager(resource_id),
                            derefed_from,
                        },
                    ) => {
                        match (method, derefed_from) {
                            (
                                ResourceManagerMethod::Mint,
                                Some((
                                    RENodeId::Global(GlobalAddress::Resource(resource_address)),
                                    ..,
                                )),
                            ) if resource_address.eq(&ENTITY_OWNER_TOKEN) => {
                                let actor = system_api.get_actor();
                                match actor {
                                    // TODO: Use associated function badge instead
                                    REActor::Function(ResolvedFunction::Native(
                                        NativeFunction::Package(PackageFunction::PublishWithOwner),
                                    )) => {
                                        vec![MethodAuthorization::AllowAll]
                                    }
                                    _ => {
                                        vec![MethodAuthorization::DenyAll]
                                    }
                                }
                            }
                            _ => {
                                let node_id = RENodeId::ResourceManager(resource_id);
                                let offset = SubstateOffset::ResourceManager(
                                    ResourceManagerOffset::ResourceManager,
                                );
                                let handle = system_api.lock_substate(
                                    node_id,
                                    offset,
                                    LockFlags::read_only(),
                                )?;
                                let substate_ref = system_api.get_ref(handle)?;
                                let resource_manager = substate_ref.resource_manager();
                                let method_auth =
                                    resource_manager.get_auth(*method, executor.args()).clone();
                                system_api.drop_lock(handle)?;
                                vec![method_auth]
                            }
                        }
                    }
                    (
                        ResolvedMethod::Native(NativeMethod::EpochManager(ref method)),
                        ResolvedReceiver {
                            receiver: RENodeId::EpochManager(..),
                            ..
                        },
                    ) => EpochManager::method_auth(method),
                    (
                        ResolvedMethod::Scrypto {
                            package_address,
                            blueprint_name,
                            ident,
                            ..
                        },
                        ResolvedReceiver {
                            receiver: RENodeId::Component(component_id),
                            ..
                        },
                    ) => {
                        let node_id = RENodeId::Global(GlobalAddress::Package(package_address));
                        let offset = SubstateOffset::Package(PackageOffset::Info);
                        let handle =
                            system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

                        // Assume that package_address/blueprint is the original impl of Component for now
                        // TODO: Remove this assumption
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package_info();
                        let schema = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint not found for existing component")
                            .structure
                            .clone();
                        system_api.drop_lock(handle)?;

                        let component_node_id = RENodeId::Component(component_id);
                        let state = {
                            let offset = SubstateOffset::Component(ComponentOffset::State);
                            let handle = system_api.lock_substate(
                                component_node_id,
                                offset,
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let state = substate_ref.component_state().clone(); // TODO: Remove clone
                            system_api.drop_lock(handle)?;
                            state
                        };
                        {
                            let offset =
                                SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
                            let handle = system_api.lock_substate(
                                component_node_id,
                                offset,
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let access_rules = substate_ref.access_rules();
                            let auth = access_rules.method_authorization(&state, &schema, &ident);
                            system_api.drop_lock(handle)?;
                            auth
                        }
                    }
                    (
                        ResolvedMethod::Native(NativeMethod::Vault(ref vault_fn)),
                        ResolvedReceiver {
                            receiver: RENodeId::Vault(vault_id),
                            ..
                        },
                    ) => {
                        let vault_node_id = RENodeId::Vault(vault_id);
                        let visibility = system_api.get_visible_node_data(vault_node_id)?;

                        let resource_address = {
                            let offset = SubstateOffset::Vault(VaultOffset::Vault);
                            let handle = system_api.lock_substate(
                                vault_node_id,
                                offset,
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let resource_address = substate_ref.vault().resource_address();
                            system_api.drop_lock(handle)?;
                            resource_address
                        };
                        let node_id = RENodeId::Global(GlobalAddress::Resource(resource_address));
                        let offset =
                            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
                        let handle =
                            system_api.lock_substate(node_id, offset, LockFlags::read_only())?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let resource_manager = substate_ref.resource_manager();

                        // TODO: Revisit what the correct abstraction is for visibility in the auth module
                        let auth = match visibility {
                            RENodeVisibilityOrigin::Normal => {
                                // TODO: Do we want to allow recaller to be able to withdraw from
                                // TODO: any visible vault?
                                vec![resource_manager.get_vault_auth(*vault_fn).clone()]
                            }
                            RENodeVisibilityOrigin::DirectAccess => match vault_fn {
                                VaultMethod::TakeNonFungibles | VaultMethod::Take => {
                                    vec![resource_manager.get_recall_auth().clone()]
                                }
                                _ => {
                                    return Err(RuntimeError::ModuleError(ModuleError::AuthError(
                                        AuthError::VisibilityError(vault_node_id),
                                    )));
                                }
                            },
                        };

                        system_api.drop_lock(handle)?;

                        auth
                    }
                    _ => vec![],
                }
            }
        };

        let refed = system_api.get_visible_node_ids()?;
        let auth_zone_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::AuthZoneStack(..)))
            .unwrap();

        let handle = system_api.lock_substate(
            auth_zone_id,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::read_only(),
        )?;
        let substate_ref = system_api.get_ref(handle)?;
        let auth_zone_ref = substate_ref.auth_zone();

        // Authorization check
        auth_zone_ref
            .check_auth(actor, method_auths)
            .map_err(|(authorization, error)| {
                RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized {
                    actor: actor.clone(),
                    authorization,
                    error,
                }))
            })?;

        system_api.drop_lock(handle)?;

        Ok(())
    }

    pub fn on_call_frame_exit<Y>(system_api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: SystemApi,
    {
        if matches!(
            system_api.get_actor(),
            REActor::Method(ResolvedMethod::Native(NativeMethod::AuthZoneStack(..)), ..)
        ) {
            return Ok(());
        }

        let refed = system_api.get_visible_node_ids()?;
        let auth_zone_id = refed
            .into_iter()
            .find(|e| matches!(e, RENodeId::AuthZoneStack(..)))
            .unwrap();
        let handle = system_api.lock_substate(
            auth_zone_id,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;
        {
            let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
            let auth_zone = substate_ref_mut.auth_zone();
            auth_zone.pop_frame();
        }
        system_api.drop_lock(handle)?;

        Ok(())
    }
}
