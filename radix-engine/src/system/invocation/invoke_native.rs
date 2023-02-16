use crate::kernel::kernel_api::KernelInvokeApi;
use crate::{blueprints::transaction_processor::NativeOutput, types::*};
use radix_engine_interface::api::types::{
    AccessRulesChainInvocation, ComponentInvocation, MetadataInvocation, NativeInvocation,
    PackageInvocation,
};

pub fn invoke_native_fn<Y, E>(
    invocation: NativeInvocation,
    api: &mut Y,
) -> Result<Box<dyn NativeOutput>, E>
where
    Y: KernelInvokeApi<E>,
{
    match invocation {
        NativeInvocation::Component(component_invocation) => match component_invocation {
            ComponentInvocation::Globalize(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentInvocation::GlobalizeWithOwner(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentInvocation::SetRoyaltyConfig(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            ComponentInvocation::ClaimRoyalty(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::Package(package_invocation) => match package_invocation {
            PackageInvocation::Publish(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            PackageInvocation::PublishNative(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            PackageInvocation::SetRoyaltyConfig(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            PackageInvocation::ClaimRoyalty(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
        NativeInvocation::AccessRulesChain(access_rules_invocation) => {
            match access_rules_invocation {
                AccessRulesChainInvocation::AddAccessCheck(invocation) => {
                    let rtn = api.kernel_invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::SetMethodAccessRule(invocation) => {
                    let rtn = api.kernel_invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::SetMethodMutability(invocation) => {
                    let rtn = api.kernel_invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::SetGroupAccessRule(invocation) => {
                    let rtn = api.kernel_invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::SetGroupMutability(invocation) => {
                    let rtn = api.kernel_invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
                AccessRulesChainInvocation::GetLength(invocation) => {
                    let rtn = api.kernel_invoke(invocation)?;
                    Ok(Box::new(rtn))
                }
            }
        }
        NativeInvocation::Metadata(metadata_invocation) => match metadata_invocation {
            MetadataInvocation::Set(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
            MetadataInvocation::Get(invocation) => {
                let rtn = api.kernel_invoke(invocation)?;
                Ok(Box::new(rtn))
            }
        },
    }
}
