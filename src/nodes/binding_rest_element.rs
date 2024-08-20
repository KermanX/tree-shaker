use crate::{entity::Entity, symbol::SymbolSource, TreeShaker};
use oxc::{
  ast::ast::{BindingPattern, BindingPatternKind, BindingRestElement},
  semantic::SymbolId,
};
use rustc_hash::FxHashSet;

#[derive(Debug, Default, Clone)]
pub struct Data {
  referred_symbols: FxHashSet<SymbolId>,
}

impl<'a> TreeShaker<'a> {
  pub(crate) fn exec_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    symbol_source: SymbolSource<'a>,
  ) {
  }

  pub(crate) fn refer_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    symbol: SymbolId,
    init_val: Entity,
  ) -> Entity {
    let data = self.load_data::<Data>(node);

    todo!()
  }
}
