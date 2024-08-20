use crate::entity::Entity;
use oxc::semantic::SymbolId;
use rustc_hash::FxHashMap;
use std::rc::Rc;

pub struct Context {
  pub(crate) this: Rc<Entity>,
  pub(crate) vars: FxHashMap<SymbolId, Rc<Entity>>,
}
