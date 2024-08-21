use crate::{entity::Entity, symbol::SymbolSource, transformer::Transformer, Analyzer};
use oxc::{
  ast::ast::{BindingPattern, BindingPatternKind, BindingRestElement},
  semantic::SymbolId,
};
use rustc_hash::FxHashSet;

use super::binding_pattern::BindingPatternSource;

#[derive(Debug, Default, Clone)]
pub struct Data {
  referred_symbols: FxHashSet<SymbolId>,
}

impl<'a> Analyzer<'a> {
  pub(crate) fn exec_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    source: BindingPatternSource<'a>,
  ) {
  }

  pub(crate) fn calc_binding_rest_element(
    &self,
    node: &'a BindingRestElement<'a>,
    symbol: SymbolId,
  ) -> Entity {
    todo!()
  }

  pub(crate) fn refer_binding_rest_element(
    &mut self,
    node: &'a BindingRestElement<'a>,
    symbol: SymbolId,
    init_val: Entity,
  ) -> Entity {
    todo!()
  }
}

impl<'a> Transformer<'a> {
  pub(crate) fn transform_binding_rest_element(
    &self,
    node: BindingRestElement<'a>,
  ) -> Option<BindingRestElement<'a>> {
    let data = self.get_data::<Data>(&node);

    todo!()
  }
}
