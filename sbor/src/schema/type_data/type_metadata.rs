use crate::rust::prelude::*;
use crate::*;

/// This is the struct used in the Schema
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct NovelTypeMetadata {
    pub type_hash: TypeHash,
    pub type_metadata: TypeMetadata,
}

/// This enables the type to be represented as eg JSON
/// Also used to facilitate type reconstruction
#[derive(Debug, Clone, PartialEq, Eq, Default, Sbor)]
pub struct TypeMetadata {
    pub type_name: Cow<'static, str>,
    pub children: Children,
}

impl TypeMetadata {
    pub fn with_type_hash(self, type_hash: TypeHash) -> NovelTypeMetadata {
        NovelTypeMetadata {
            type_hash,
            type_metadata: self,
        }
    }

    pub fn no_child_names(name: &'static str) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            children: Children::None,
        }
    }

    pub fn unnamed_fields(name: &'static str) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            children: Children::UnnamedFields,
        }
    }

    pub fn named_fields(name: &'static str, field_names: &[&'static str]) -> Self {
        let field_names = field_names
            .iter()
            .map(|field_name| FieldMetadata {
                field_name: Cow::Borrowed(*field_name),
            })
            .collect();
        Self {
            type_name: Cow::Borrowed(name),
            children: Children::NamedFields(field_names),
        }
    }

    pub fn enum_variants(name: &'static str, variant_naming: BTreeMap<u8, TypeMetadata>) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            children: Children::EnumVariants(variant_naming),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Sbor)]
pub enum Children {
    #[default]
    None,
    UnnamedFields,
    NamedFields(Vec<FieldMetadata>),
    EnumVariants(BTreeMap<u8, TypeMetadata>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Sbor)]
pub struct FieldMetadata {
    pub field_name: Cow<'static, str>,
}
