/// Defines the custom Scrypto schema types.
mod custom_schema;
/// Defines the model of Scrypto custom values.
mod custom_value;
/// Defines the custom value kind model that scrypto uses.
mod custom_value_kind;
/// Defines the scrypto custom well known types.
mod custom_well_known_types;
/// Indexed Scrypto value.
mod indexed_value;
/// Matches a Scrypto schema type with a Scrypto value.
mod schema_matcher;
/// Defines a way to uniquely identify an element within a Scrypto schema type.
mod schema_path;
/// Format any Scrypto value using the Manifest syntax.
mod value_formatter;
#[cfg(feature = "serde")]
/// One-way serialize any Scrypto value.
mod value_serializer;

pub mod model;

pub use custom_schema::*;
pub use custom_value::*;
pub use custom_value_kind::*;
pub use custom_well_known_types::*;
pub use indexed_value::*;
use sbor::rust::vec::Vec;
use sbor::{
    Categorize, Decode, DecodeError, Decoder, Encode, EncodeError, Encoder, Value, ValueKind,
    VecDecoder, VecEncoder,
};
pub use schema_matcher::*;
pub use schema_path::*;
pub use value_formatter::*;
#[cfg(feature = "serde")]
pub use value_serializer::*;

pub const MAX_SCRYPTO_SBOR_DEPTH: u8 = 64;

pub type ScryptoEncoder<'a> = VecEncoder<'a, ScryptoCustomValueKind, MAX_SCRYPTO_SBOR_DEPTH>;
pub type ScryptoDecoder<'a> = VecDecoder<'a, ScryptoCustomValueKind, MAX_SCRYPTO_SBOR_DEPTH>;
pub type ScryptoValueKind = ValueKind<ScryptoCustomValueKind>;
pub type ScryptoValue = Value<ScryptoCustomValueKind, ScryptoCustomValue>;

// 0x5c for [5c]rypto - (91 in decimal)
pub const SCRYPTO_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5c;

// The following trait "aliases" are to be used in parameters.
//
// They are much nicer to read than the underlying traits, but because they are "new", and are defined
// via blanket impls, they can only be used for parameters, but cannot be used for implementations.
//
// Implementations should instead implement the underlying traits:
// * Categorize<ScryptoCustomValueKind>
// * Encode<ScryptoCustomValueKind, E> (impl over all E: Encoder<ScryptoCustomValueKind>)
// * Decode<ScryptoCustomValueKind, D> (impl over all D: Decoder<ScryptoCustomValueKind>)
//
// TODO: Change these to be Trait aliases once stable in rust: https://github.com/rust-lang/rust/issues/41517
pub trait ScryptoCategorize: Categorize<ScryptoCustomValueKind> {}
impl<T: Categorize<ScryptoCustomValueKind> + ?Sized> ScryptoCategorize for T {}

pub trait ScryptoDecode: for<'a> Decode<ScryptoCustomValueKind, ScryptoDecoder<'a>> {}
impl<T: for<'a> Decode<ScryptoCustomValueKind, ScryptoDecoder<'a>>> ScryptoDecode for T {}

pub trait ScryptoEncode: for<'a> Encode<ScryptoCustomValueKind, ScryptoEncoder<'a>> {}
impl<T: for<'a> Encode<ScryptoCustomValueKind, ScryptoEncoder<'a>> + ?Sized> ScryptoEncode for T {}

/// Encodes a data structure into byte array.
pub fn scrypto_encode<T: ScryptoEncode + ?Sized>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = ScryptoEncoder::new(&mut buf);
    encoder.encode_payload(value, SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

/// Decodes a data structure from a byte array.
pub fn scrypto_decode<T: ScryptoDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    ScryptoDecoder::new(buf).decode_payload(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
}

#[macro_export]
macro_rules! count {
    () => {0usize};
    ($a:expr) => {1usize};
    ($a:expr, $($rest:expr),*) => {1usize + $crate::count!($($rest),*)};
}

#[macro_export]
macro_rules! scrypto_args {
    ($($args: expr),*) => {{
        use ::sbor::Encoder;
        let mut buf = ::sbor::rust::vec::Vec::new();
        let mut encoder = $crate::data::ScryptoEncoder::new(&mut buf);
        encoder.write_payload_prefix($crate::data::SCRYPTO_SBOR_V1_PAYLOAD_PREFIX).unwrap();
        encoder.write_value_kind($crate::data::ScryptoValueKind::Tuple).unwrap();
        // Hack: stringify to skip ownership move semantics
        encoder.write_size($crate::count!($(stringify!($args)),*)).unwrap();
        $(
            let arg = $args;
            encoder.encode(&arg).unwrap();
        )*
        buf
    }};
}
