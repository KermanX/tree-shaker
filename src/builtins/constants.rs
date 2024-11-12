use oxc::semantic::SymbolId;
use std::mem;

// Builtin object ids
pub const IMPORT_META_OBJECT_ID: SymbolId = unsafe { mem::transmute(1u32) };
pub const REACT_NAMESPACE_OBJECT_ID: SymbolId = unsafe { mem::transmute(2u32) };
