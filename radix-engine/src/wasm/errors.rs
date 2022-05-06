use scrypto::rust::string::String;
use wasmi::*;

/// Error coming from WASMI module which maps to wasmi:Error but is cloneable
#[derive(Debug, PartialEq, Clone)]
pub enum WasmiError {
    /// Module validation error. Might occur only at load time.
    Validation(String),
    /// Error while instantiating a module. Might occur when provided
    /// with incorrect exports (i.e. linkage failure).
    Instantiation(String),
    /// Function-level error.
    Function(String),
    /// Table-level error.
    Table(String),
    /// Memory-level error.
    Memory(String),
    /// Global-level error.
    Global(String),
    /// Value-level error.
    Value(String),
    /// Trap.
    Trap,
    /// Custom embedder error.
    Host,
}

impl From<wasmi::Error> for WasmiError {
    fn from(e: Error) -> Self {
        match e {
            Error::Validation(e) => WasmiError::Validation(e),
            Error::Instantiation(e) => WasmiError::Instantiation(e),
            Error::Function(e) => WasmiError::Function(e),
            Error::Table(e) => WasmiError::Table(e),
            Error::Memory(e) => WasmiError::Memory(e),
            Error::Global(e) => WasmiError::Global(e),
            Error::Value(e) => WasmiError::Value(e),
            Error::Trap(_) => WasmiError::Trap,
            Error::Host(_) => WasmiError::Host,
        }
    }
}

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Clone)]
pub enum WasmValidationError {
    /// The wasm module is invalid.
    InvalidModule,
    /// The wasm module contains a start function.
    StartFunctionNotAllowed,
    /// The wasm module uses float points.
    FloatingPointNotAllowed,
    /// The wasm module does not have memory export.
    NoValidMemoryExport,
    /// package_init function does not exist in module
    NoPackageInitExport(WasmiError),
    /// package_init function is not the correct interface
    InvalidPackageInit,
}