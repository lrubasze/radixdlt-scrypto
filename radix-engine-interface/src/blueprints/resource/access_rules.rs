use crate::blueprints::resource::*;
use crate::rule;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_interface::api::ObjectModuleId;
use sbor::rust::collections::BTreeMap;
use sbor::rust::str;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use utils::btreemap;

use super::AccessRule;

pub const SELF_ROLE: &'static str = "_self_";
pub const OWNER_ROLE: &'static str = "_owner_";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct MethodKey {
    pub ident: String,
}

impl MethodKey {
    pub fn new<S: ToString>(method_ident: S) -> Self {
        Self {
            ident: method_ident.to_string(),
        }
    }
}

impl From<&str> for MethodKey {
    fn from(value: &str) -> Self {
        MethodKey::new(value)
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct MethodEntry {
    pub permission: MethodAccessibility,
}

impl MethodEntry {
    pub fn new<P: Into<MethodAccessibility>>(permission: P) -> Self {
        Self {
            permission: permission.into(),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum MethodAccessibility {
    /// Method is accessible to all
    Public,
    /// Only outer objects have access to a given method. Currently used by Validator blueprint
    /// to only allow ConsensusManager to access some methods.
    OuterObjectOnly,
    /// Method is only accessible by any role in the role list
    RoleProtected(RoleList),
}

impl MethodAccessibility {
    pub fn nobody() -> Self {
        MethodAccessibility::RoleProtected(RoleList::none())
    }
}

impl<const N: usize> From<[&str; N]> for MethodAccessibility {
    fn from(value: [&str; N]) -> Self {
        MethodAccessibility::RoleProtected(value.into())
    }
}

impl From<RoleList> for MethodAccessibility {
    fn from(value: RoleList) -> Self {
        Self::RoleProtected(value)
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum AttachedModule {
    Metadata,
    Royalty,
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct ModuleRoleKey {
    pub module: ObjectModuleId,
    pub key: RoleKey,
}

impl ModuleRoleKey {
    pub fn new<K: Into<RoleKey>>(module: ObjectModuleId, key: K) -> Self {
        Self {
            module,
            key: key.into(),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct RoleKey {
    pub key: String,
}

impl From<String> for RoleKey {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for RoleKey {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl RoleKey {
    pub fn new<S: Into<String>>(key: S) -> Self {
        RoleKey { key: key.into() }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub struct RoleEntry {
    pub rule: AccessRule,
    pub updaters: RoleList,
}

impl RoleEntry {
    pub fn new<A: Into<AccessRule>, M: Into<RoleList>>(rule: A, updaters: M) -> Self {
        Self {
            rule: rule.into(),
            updaters: updaters.into(),
        }
    }

    pub fn immutable<A: Into<AccessRule>>(rule: A) -> Self {
        Self {
            rule: rule.into(),
            updaters: RoleList::none(),
        }
    }

    // TODO: Remove and replace with set immutable rule
    pub fn disabled() -> Self {
        Self::immutable(AccessRule::DenyAll)
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct RoleList {
    pub list: Vec<RoleKey>,
}

impl RoleList {
    pub fn none() -> Self {
        Self { list: vec![] }
    }

    pub fn insert<R: Into<RoleKey>>(&mut self, role: R) {
        self.list.push(role.into());
    }

    pub fn to_list(self) -> Vec<String> {
        self.list.into_iter().map(|k| k.key).collect()
    }
}

impl From<Vec<&str>> for RoleList {
    fn from(value: Vec<&str>) -> Self {
        Self {
            list: value.into_iter().map(|s| RoleKey::new(s)).collect(),
        }
    }
}

impl From<Vec<String>> for RoleList {
    fn from(value: Vec<String>) -> Self {
        Self {
            list: value.into_iter().map(|s| RoleKey::new(s)).collect(),
        }
    }
}

impl<const N: usize> From<[&str; N]> for RoleList {
    fn from(value: [&str; N]) -> Self {
        Self {
            list: value.into_iter().map(|s| RoleKey::new(s)).collect(),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
pub enum OwnerRole {
    None,
    Fixed(AccessRule),
    Updatable(AccessRule),
}

impl OwnerRole {
    // TODO: Remove
    pub fn to_role_entry(self, owner_role_name: &str) -> RoleEntry {
        match self {
            OwnerRole::Fixed(rule) => RoleEntry::immutable(rule),
            OwnerRole::Updatable(rule) => RoleEntry::new(rule, [owner_role_name]),
            OwnerRole::None => RoleEntry::immutable(AccessRule::DenyAll),
        }
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct Roles {
    pub roles: BTreeMap<RoleKey, (RoleEntry, bool)>,
}

impl Roles {
    pub fn new() -> Self {
        Self { roles: btreemap!() }
    }

    pub fn define_immutable_role<K: Into<RoleKey>>(&mut self, role: K, access_rule: AccessRule) {
        self.roles
            .insert(role.into(), (RoleEntry::immutable(access_rule), true));
    }

    pub fn define_mutable_role<K: Into<RoleKey>>(&mut self, role: K, entry: RoleEntry) {
        self.roles.insert(role.into(), (entry, false));
    }
}

// TODO: Remove?
pub fn resource_access_rules_from_owner_badge(
    owner_badge: &NonFungibleGlobalId,
) -> BTreeMap<ResourceAction, (AccessRule, AccessRule)> {
    let mut access_rules = BTreeMap::new();
    access_rules.insert(
        ResourceAction::Withdraw,
        (AccessRule::AllowAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        ResourceAction::Deposit,
        (AccessRule::AllowAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        ResourceAction::Recall,
        (AccessRule::DenyAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        Mint,
        (AccessRule::DenyAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        Burn,
        (AccessRule::DenyAll, rule!(require(owner_badge.clone()))),
    );
    access_rules.insert(
        UpdateNonFungibleData,
        (
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        ),
    );
    access_rules.insert(
        UpdateMetadata,
        (
            rule!(require(owner_badge.clone())),
            rule!(require(owner_badge.clone())),
        ),
    );
    access_rules
}
