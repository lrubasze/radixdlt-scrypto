use super::*;
use crate::rust::collections::BTreeMap;
use crate::rust::string::String;
use crate::rust::vec::Vec;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
#[sbor(generic_type_id_bounds = "L")]
pub enum TypeKind<X: CustomTypeId, C: CustomTypeKind<L, CustomTypeId = X>, L: SchemaTypeLink> {
    Any,

    // Simple Types
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,

    // Composite Types
    Array { element_type: L },

    Tuple { field_types: Vec<L> },

    Enum { variants: BTreeMap<String, Vec<L>> },

    // Custom Types
    Custom(C),
}