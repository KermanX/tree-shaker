use oxc::semantic::SymbolId;
use std::mem;

// Object symbol ids
pub const IMPORT_META_OBJECT_ID: SymbolId = unsafe { mem::transmute(1u32) };
