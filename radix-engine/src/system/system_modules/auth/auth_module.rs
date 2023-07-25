use super::Authorization;
use crate::blueprints::package::PackageAuthNativeBlueprint;
use crate::errors::*;
use crate::kernel::actor::{Actor, AuthInfo, FunctionActor, MethodActor};
use crate::kernel::kernel_api::KernelApi;
use crate::system::module::KernelModule;
use crate::system::node_modules::role_assignment::RoleAssignmentNativePackage;
use crate::system::system::SystemService;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::package::{
    BlueprintVersionKey, MethodAuthTemplate, RoleSpecification,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;
use transaction::model::AuthZoneParams;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthError {
    NoFunction(FnIdentifier),
    NoMethodMapping(FnIdentifier),
    VisibilityError(NodeId),
    Unauthorized(Box<Unauthorized>),
    InnerBlueprintDoesNotExist(String),
    InvalidOuterObjectMapping,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FailedAccessRules {
    RoleList(Vec<(RoleKey, Vec<AccessRule>)>),
    AccessRule(Vec<AccessRule>),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct Unauthorized {
    pub failed_access_rules: FailedAccessRules,
    pub fn_identifier: FnIdentifier,
}

#[derive(Debug, Clone)]
pub struct AuthModule {
    pub params: AuthZoneParams,
}

pub enum AuthorizationCheckResult {
    Authorized,
    Failed(Vec<AccessRule>),
}

pub enum AuthorityListAuthorizationResult {
    Authorized,
    Failed(Vec<(RoleKey, Vec<AccessRule>)>),
}

pub enum ResolvedPermission {
    RoleList {
        role_assignment_of: NodeId,
        module_id: ObjectModuleId,
        role_list: RoleList,
    },
    AccessRule(AccessRule),
    AllowAll,
}

impl AuthModule {
    pub fn check_function_authorization<V, Y>(
        api: &mut Y,
        auth_info: AuthInfo,
        blueprint_id: &BlueprintId,
        ident: &str,
    ) -> Result<(), RuntimeError>
        where
            V: SystemCallbackObject,
            Y: KernelApi<SystemConfig<V>>,
    {
        let mut system = SystemService::new(api);

        // Step 1: Resolve method to permission
        let permission = PackageAuthNativeBlueprint::resolve_function_permission(
            blueprint_id.package_address.as_node_id(),
            &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
            ident,
            system.api,
        )?;

        // Step 2: Check permission
        let fn_identifier = FnIdentifier {
            blueprint_id: blueprint_id.clone(),
            ident: ident.to_string(),
        };
        Self::check_permission(auth_info, permission, fn_identifier, &mut system)?;

        Ok(())
    }

    pub fn check_method_authorization<V, Y>(
        api: &mut Y,
        receiver: &NodeId,
        module_id: ObjectModuleId,
        ident: &str,
        args: &IndexedScryptoValue,
        auth_info: AuthInfo,
    ) -> Result<(), RuntimeError>
        where
            V: SystemCallbackObject,
            Y: KernelApi<SystemConfig<V>>,
    {
        let mut system = SystemService::new(api);

        // Step 1: Resolve method to permission
        let blueprint_id = system
            .get_blueprint_info(receiver, module_id)?
            .blueprint_id;

        let permission = Self::resolve_method_permission(
            &mut system,
            &blueprint_id,
            receiver,
            &module_id,
            ident,
            args,
        )?;

        // Step 2: Check permission
        let fn_identifier = FnIdentifier {
            blueprint_id: blueprint_id.clone(),
            ident: ident.to_string(),
        };
        Self::check_permission(auth_info, permission, fn_identifier, &mut system)?;

        Ok(())
    }

    fn check_permission<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        auth_info: AuthInfo,
        resolved_permission: ResolvedPermission,
        fn_identifier: FnIdentifier,
        api: &mut SystemService<Y, V>,
    ) -> Result<(), RuntimeError> {
        match resolved_permission {
            ResolvedPermission::AllowAll => return Ok(()),
            ResolvedPermission::AccessRule(rule) => {
                let result =
                    Authorization::check_authorization_against_access_rule(&auth_info, &rule, api)?;

                match result {
                    AuthorizationCheckResult::Authorized => Ok(()),
                    AuthorizationCheckResult::Failed(access_rule_stack) => Err(
                        RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                            AuthError::Unauthorized(Box::new(Unauthorized {
                                failed_access_rules: FailedAccessRules::AccessRule(
                                    access_rule_stack,
                                ),
                                fn_identifier,
                            })),
                        )),
                    ),
                }
            }
            ResolvedPermission::RoleList {
                role_assignment_of,
                role_list,
                module_id,
            } => {
                let result = Authorization::check_authorization_against_role_list(
                    &auth_info,
                    &role_assignment_of,
                    module_id,
                    &role_list,
                    api,
                )?;

                match result {
                    AuthorityListAuthorizationResult::Authorized => Ok(()),
                    AuthorityListAuthorizationResult::Failed(auth_list_fail) => Err(
                        RuntimeError::SystemModuleError(SystemModuleError::AuthError(
                            AuthError::Unauthorized(Box::new(Unauthorized {
                                failed_access_rules: FailedAccessRules::RoleList(auth_list_fail),
                                fn_identifier,
                            })),
                        )),
                    ),
                }
            }
        }
    }

    fn resolve_method_permission<Y: KernelApi<SystemConfig<V>>, V: SystemCallbackObject>(
        api: &mut SystemService<Y, V>,
        blueprint_id: &BlueprintId,
        receiver: &NodeId,
        module_id: &ObjectModuleId,
        ident: &str,
        args: &IndexedScryptoValue,
    ) -> Result<ResolvedPermission, RuntimeError> {
        let method_key = MethodKey::new(ident);

        if let ObjectModuleId::RoleAssignment = module_id {
            return RoleAssignmentNativePackage::authorization(
                receiver,
                ident,
                args,
                api,
            );
        }

        let auth_template = PackageAuthNativeBlueprint::get_bp_auth_template(
            blueprint_id.package_address.as_node_id(),
            &BlueprintVersionKey::new_default(blueprint_id.blueprint_name.as_str()),
            api.api,
        )?
        .method_auth;

        let receiver_object_info = api.get_object_info(&receiver)?;

        let (role_assignment_of, method_permissions) = match auth_template {
            MethodAuthTemplate::StaticRoles(static_roles) => {
                let role_assignment_of = match static_roles.roles {
                    RoleSpecification::Normal(..) => {
                        // Non-globalized objects do not have access rules module
                        if !receiver_object_info.global {
                            return Ok(ResolvedPermission::AllowAll);
                        }

                        receiver.clone()
                    }
                    RoleSpecification::UseOuter => {
                        let role_assignment_of = receiver_object_info.get_outer_object();
                        role_assignment_of.into_node_id()
                    }
                };

                (role_assignment_of, static_roles.methods)
            }
            MethodAuthTemplate::AllowAll => return Ok(ResolvedPermission::AllowAll),
        };

        match method_permissions.get(&method_key) {
            Some(MethodAccessibility::Public) => Ok(ResolvedPermission::AllowAll),
            Some(MethodAccessibility::OwnPackageOnly) => {
                let package = blueprint_id.package_address;
                Ok(ResolvedPermission::AccessRule(rule!(require(
                    package_of_direct_caller(package)
                ))))
            }
            Some(MethodAccessibility::OuterObjectOnly) => match module_id {
                ObjectModuleId::Main => {
                    let outer_object_info = &receiver_object_info.blueprint_info.outer_obj_info;
                    match outer_object_info {
                        OuterObjectInfo::Some { outer_object } => {
                            Ok(ResolvedPermission::AccessRule(rule!(require(
                                global_caller(*outer_object)
                            ))))
                        }
                        OuterObjectInfo::None { .. } => Err(RuntimeError::SystemModuleError(
                            SystemModuleError::AuthError(AuthError::InvalidOuterObjectMapping),
                        )),
                    }
                }
                _ => Err(RuntimeError::SystemModuleError(
                    SystemModuleError::AuthError(AuthError::InvalidOuterObjectMapping),
                )),
            },
            Some(MethodAccessibility::RoleProtected(role_list)) => {
                Ok(ResolvedPermission::RoleList {
                    role_assignment_of,
                    role_list: role_list.clone(),
                    module_id: module_id.clone(),
                })
            }
            None => {
                let fn_identifier = FnIdentifier {
                    blueprint_id: blueprint_id.clone(),
                    ident: ident.to_string(),
                };
                Err(RuntimeError::SystemModuleError(
                    SystemModuleError::AuthError(AuthError::NoMethodMapping(fn_identifier)),
                ))
            },
        }
    }
}

impl<V: SystemCallbackObject> KernelModule<SystemConfig<V>> for AuthModule {
}
